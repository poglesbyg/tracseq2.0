import { useState } from 'react';
import { Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import {
  BeakerIcon,
  DocumentTextIcon,
  MagnifyingGlassIcon,
  FunnelIcon,
  SparklesIcon,
  EyeIcon,
  ChevronRightIcon,
  XMarkIcon,
  ExclamationTriangleIcon,
  CheckCircleIcon,
  ChartBarIcon,
  ArrowLeftIcon,
  DocumentIcon,
  UserIcon,
  CalendarIcon,
  InformationCircleIcon,
} from '@heroicons/react/24/outline';

// Type definitions
interface ExtractedDataSection {
  [key: string]: string | number | boolean | null | ExtractedDataSection | ExtractedDataSection[];
}

interface RagSample {
  id: string;
  name: string;
  barcode: string;
  location: string;
  status: string;
  created_at: string;
  metadata?: {
    confidence_score?: number;
    processing_time?: number;
    source_document?: string;
    submitter_name?: string;
    submitter_email?: string;
    rag_submission_id?: string;
    extraction_method?: string;
    sample_type?: string;
    validation_warnings?: string[];
    extraction_warnings?: string[];
    [key: string]: unknown;
  };
}

interface RagSubmissionDetail {
  id: string;
  submission_id: string;
  source_document: string;
  submitter_name: string;
  submitter_email: string;
  confidence_score: number;
  processing_time: number;
  created_at: string;
  status: string;
  samples_created: number;
  extracted_data?: {
    administrative_info?: ExtractedDataSection;
    source_material?: ExtractedDataSection;
    pooling_info?: ExtractedDataSection;
    sequence_generation?: ExtractedDataSection;
    container_info?: ExtractedDataSection;
    informatics_info?: ExtractedDataSection;
    sample_details?: ExtractedDataSection;
    [key: string]: ExtractedDataSection | undefined;
  };
}

export default function RagSamples() {
  const [searchTerm, setSearchTerm] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('');
  const [confidenceFilter, setConfidenceFilter] = useState<string>('');
  const [selectedSample, setSelectedSample] = useState<RagSample | null>(null);
  const [showDetailModal, setShowDetailModal] = useState(false);

  // Fetch RAG samples
  const { data: ragSamples, isLoading: isLoadingSamples, error: samplesError, refetch } = useQuery<RagSample[]>({
    queryKey: ['rag-samples', searchTerm, statusFilter, confidenceFilter],
    queryFn: async () => {
      try {
        // Use the dedicated RAG samples endpoint first
        console.log('🔍 Fetching RAG samples...');
        const response = await axios.get('/api/rag/samples');
        console.log('📊 RAG samples response:', response.data);
        
        let samples = response.data || [];
        
        // If no data from RAG endpoint, try the filtered samples endpoint
        if (!samples || samples.length === 0) {
          console.log('🔄 Trying filtered samples endpoint...');
          const params = new URLSearchParams();
          params.append('extraction_method', 'ai_rag');
          const fallbackResponse = await axios.get(`/api/samples?${params}`);
          console.log('📊 Filtered samples response:', fallbackResponse.data);
          samples = Array.isArray(fallbackResponse.data) ? fallbackResponse.data : [];
        }
        
        // Apply client-side filtering
        if (searchTerm || statusFilter || confidenceFilter) {
          samples = samples.filter((sample: RagSample) => {
            let matches = true;
            
            if (searchTerm) {
              const searchLower = searchTerm.toLowerCase();
              matches = matches && (
                sample.name.toLowerCase().includes(searchLower) ||
                sample.barcode.toLowerCase().includes(searchLower) ||
                (sample.metadata?.submitter_name && typeof sample.metadata.submitter_name === 'string' && sample.metadata.submitter_name.toLowerCase().includes(searchLower))
              );
            }
            
            if (statusFilter) {
              matches = matches && sample.status === statusFilter;
            }
            
            if (confidenceFilter) {
              const confidence = sample.metadata?.confidence_score || 0;
              switch (confidenceFilter) {
                case 'high':
                  matches = matches && confidence >= 0.8;
                  break;
                case 'medium':
                  matches = matches && confidence >= 0.6 && confidence < 0.8;
                  break;
                case 'low':
                  matches = matches && confidence < 0.6;
                  break;
              }
            }
            
            return matches;
          });
        }
        
        console.log(`✅ Final RAG samples count: ${samples.length}`);
        return samples;
      } catch (error) {
        console.error('❌ Failed to fetch RAG samples:', error);
        return [];
      }
    },
    retry: 2,
    retryDelay: 1000,
  });

  // Fetch RAG submission details
  const { data: ragSubmissionDetail, isLoading: isLoadingDetail } = useQuery<RagSubmissionDetail>({
    queryKey: ['rag-submission-detail', selectedSample?.metadata?.rag_submission_id],
    queryFn: async () => {
      if (!selectedSample?.metadata?.rag_submission_id) return null;
      
      try {
        const response = await axios.get(`/api/rag/submissions/${selectedSample.metadata.rag_submission_id}`);
        return response.data;
      } catch (error) {
        console.error('Failed to fetch RAG submission details:', error);
        return null;
      }
    },
    enabled: !!selectedSample?.metadata?.rag_submission_id,
  });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Completed':
        return 'bg-green-100 text-green-800';
      case 'Pending':
        return 'bg-yellow-100 text-yellow-800';
      case 'Validated':
        return 'bg-blue-100 text-blue-800';
      case 'InStorage':
        return 'bg-purple-100 text-purple-800';
      case 'InSequencing':
        return 'bg-indigo-100 text-indigo-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getConfidenceColor = (score: number) => {
    if (score >= 0.8) return 'text-green-600 bg-green-100';
    if (score >= 0.6) return 'text-yellow-600 bg-yellow-100';
    return 'text-red-600 bg-red-100';
  };

  const getConfidenceLabel = (score: number) => {
    if (score >= 0.8) return 'High';
    if (score >= 0.6) return 'Medium';
    return 'Low';
  };

  const handleViewDetails = (sample: RagSample) => {
    setSelectedSample(sample);
    setShowDetailModal(true);
  };

  const handleClearFilters = () => {
    setSearchTerm('');
    setStatusFilter('');
    setConfidenceFilter('');
  };

  const renderExtractedDataSection = (title: string, data: ExtractedDataSection | undefined) => {
    if (!data || typeof data !== 'object') return null;

    return (
      <div className="mb-6">
        <h5 className="text-sm font-semibold text-gray-900 mb-3 border-b border-gray-200 pb-2">
          {title}
        </h5>
        <dl className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          {Object.entries(data).map(([key, value]) => {
            if (value === null || value === undefined || value === '') return null;
            
            const displayValue = Array.isArray(value) 
              ? value.join(', ') 
              : typeof value === 'object' 
                ? JSON.stringify(value, null, 2)
                : String(value);

            return (
              <div key={key} className="border-l-2 border-blue-100 pl-3">
                <dt className="text-xs font-medium text-gray-500 uppercase tracking-wide">
                  {key.replace(/_/g, ' ')}
                </dt>
                <dd className="text-sm text-gray-900 mt-1 break-words">
                  {displayValue.length > 100 ? (
                    <details className="cursor-pointer">
                      <summary className="text-blue-600 hover:text-blue-800">
                        {displayValue.substring(0, 100)}... (click to expand)
                      </summary>
                      <div className="mt-2 p-2 bg-gray-50 rounded text-xs font-mono whitespace-pre-wrap">
                        {displayValue}
                      </div>
                    </details>
                  ) : (
                    displayValue
                  )}
                </dd>
              </div>
            );
          })}
        </dl>
      </div>
    );
  };

  const statusOptions = [
    { value: '', label: 'All Statuses' },
    { value: 'Pending', label: 'Pending' },
    { value: 'Validated', label: 'Validated' },
    { value: 'InStorage', label: 'In Storage' },
    { value: 'InSequencing', label: 'In Sequencing' },
    { value: 'Completed', label: 'Completed' },
  ];

  const confidenceOptions = [
    { value: '', label: 'All Confidence' },
    { value: 'high', label: 'High (≥80%)' },
    { value: 'medium', label: 'Medium (60-79%)' },
    { value: 'low', label: 'Low (<60%)' },
  ];

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      {/* Header */}
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <div className="flex items-start mb-4">
            <Link
              to="/rag-submissions"
              className="inline-flex items-center text-sm text-gray-500 hover:text-gray-700"
            >
              <ArrowLeftIcon className="h-4 w-4 mr-1" />
              Back to AI Submissions
            </Link>
          </div>
          
          <div className="flex items-center">
            <div className="h-10 w-10 bg-gradient-to-br from-emerald-500 to-teal-600 rounded-lg flex items-center justify-center mr-4">
              <BeakerIcon className="h-6 w-6 text-white" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900">AI-Generated Sample Records</h1>
              <p className="mt-2 text-sm text-gray-700">
                Digital sample records created from AI document processing with extraction details and confidence scores.
              </p>
            </div>
          </div>
        </div>
        
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
          <Link
            to="/rag-submissions"
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
          >
            <SparklesIcon className="h-4 w-4 mr-2" />
            Process New Documents
          </Link>
        </div>
      </div>

      {/* Statistics Cards */}
      <div className="mt-8 grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <BeakerIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">
                    Total AI Samples
                  </dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {ragSamples?.length || 0}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <CheckCircleIcon className="h-6 w-6 text-green-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">
                    High Confidence
                  </dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {ragSamples?.filter(s => (s.metadata?.confidence_score || 0) >= 0.8).length || 0}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <DocumentTextIcon className="h-6 w-6 text-blue-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">
                    Documents Processed
                  </dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {new Set(ragSamples?.map(s => s.metadata?.rag_submission_id).filter(Boolean)).size || 0}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ChartBarIcon className="h-6 w-6 text-purple-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">
                    Avg. Confidence
                  </dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {ragSamples?.length ? 
                      Math.round((ragSamples.reduce((sum, s) => sum + (s.metadata?.confidence_score || 0), 0) / ragSamples.length) * 100) : 0}%
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Filters */}
      <div className="mt-8 bg-white shadow rounded-lg p-6">
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-4">
          <div>
            <label htmlFor="search" className="block text-sm font-medium text-gray-700">
              Search Samples
            </label>
            <div className="mt-1 relative rounded-md shadow-sm">
              <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
              </div>
              <input
                type="text"
                name="search"
                id="search"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                placeholder="Search by name, barcode..."
              />
            </div>
          </div>

          <div>
            <label htmlFor="status-filter" className="block text-sm font-medium text-gray-700">
              Status
            </label>
            <select
              id="status-filter"
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value)}
              className="mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md"
            >
              {statusOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label htmlFor="confidence-filter" className="block text-sm font-medium text-gray-700">
              Confidence Level
            </label>
            <select
              id="confidence-filter"
              value={confidenceFilter}
              onChange={(e) => setConfidenceFilter(e.target.value)}
              className="mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md"
            >
              {confidenceOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          <div className="flex items-end">
            <button
              type="button"
              onClick={handleClearFilters}
              className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            >
              <FunnelIcon className="h-4 w-4 mr-2" />
              Clear Filters
            </button>
          </div>
        </div>
      </div>

      {/* Samples List */}
      <div className="mt-8 bg-white shadow overflow-hidden sm:rounded-md">
        {isLoadingSamples ? (
          <div className="text-center py-12">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto"></div>
            <p className="mt-2 text-sm text-gray-500">Loading AI-generated samples...</p>
          </div>
        ) : samplesError ? (
          <div className="text-center py-12">
            <ExclamationTriangleIcon className="h-12 w-12 text-red-400 mx-auto" />
            <h3 className="mt-2 text-sm font-medium text-gray-900">Error Loading Samples</h3>
            <p className="mt-1 text-sm text-gray-500">Unable to load AI-generated samples.</p>
            <button
              onClick={() => refetch()}
              className="mt-4 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
            >
              Try Again
            </button>
          </div>
        ) : ragSamples?.length === 0 ? (
          <div className="text-center py-12">
            <BeakerIcon className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900">No AI-Generated Samples Found</h3>
            <p className="mt-1 text-sm text-gray-500">
              No samples have been created from AI document processing yet.
            </p>
            <div className="mt-6">
              <Link
                to="/rag-submissions"
                className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
              >
                <SparklesIcon className="h-4 w-4 mr-2" />
                Process Your First Document
              </Link>
            </div>
          </div>
        ) : (
          <ul className="divide-y divide-gray-200">
            {ragSamples?.map((sample) => (
              <li key={sample.id}>
                <div className="px-4 py-4 sm:px-6">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center">
                      <div className="flex-shrink-0">
                        <div className="h-10 w-10 bg-gradient-to-br from-emerald-500 to-teal-600 rounded-lg flex items-center justify-center">
                          <SparklesIcon className="h-5 w-5 text-white" />
                        </div>
                      </div>
                      <div className="ml-4">
                        <div className="flex items-center">
                          <p className="text-sm font-medium text-gray-900">
                            {sample.name}
                          </p>
                          <span className={`ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(sample.status)}`}>
                            {sample.status}
                          </span>
                          {sample.metadata?.confidence_score && (
                            <span className={`ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getConfidenceColor(sample.metadata.confidence_score)}`}>
                              {getConfidenceLabel(sample.metadata.confidence_score)} ({(sample.metadata.confidence_score * 100).toFixed(1)}%)
                            </span>
                          )}
                        </div>
                        <div className="mt-1 flex items-center text-sm text-gray-500">
                          <p>Barcode: {sample.barcode}</p>
                          <span className="mx-2">•</span>
                          <p>Location: {sample.location}</p>
                          {sample.metadata?.sample_type && (
                            <>
                              <span className="mx-2">•</span>
                              <p>Type: {sample.metadata.sample_type}</p>
                            </>
                          )}
                        </div>
                        <div className="mt-1 flex items-center text-xs text-gray-400">
                          {sample.metadata?.source_document && (
                            <>
                              <DocumentIcon className="h-3 w-3 mr-1" />
                              <p>Source: {sample.metadata.source_document}</p>
                            </>
                          )}
                          {sample.metadata?.submitter_name && (
                            <>
                              <span className="mx-2">•</span>
                              <UserIcon className="h-3 w-3 mr-1" />
                              <p>By: {sample.metadata.submitter_name}</p>
                            </>
                          )}
                          <span className="mx-2">•</span>
                          <CalendarIcon className="h-3 w-3 mr-1" />
                          <p>Created: {new Date(sample.created_at).toLocaleDateString()}</p>
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      <button
                        type="button"
                        onClick={() => handleViewDetails(sample)}
                        className="inline-flex items-center p-2 border border-transparent rounded-full shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                      >
                        <EyeIcon className="h-4 w-4" />
                      </button>
                      <ChevronRightIcon className="h-5 w-5 text-gray-400" />
                    </div>
                  </div>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* Sample Detail Modal */}
      {showDetailModal && selectedSample && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-y-auto">
            <div className="px-6 py-4 border-b border-gray-200">
              <div className="flex items-center justify-between">
                <div className="flex items-center">
                  <div className="h-8 w-8 bg-gradient-to-br from-emerald-500 to-teal-600 rounded-lg flex items-center justify-center mr-3">
                    <SparklesIcon className="h-4 w-4 text-white" />
                  </div>
                  <h3 className="text-lg font-medium text-gray-900">
                    AI Sample Details: {selectedSample.name}
                  </h3>
                </div>
                <button
                  type="button"
                  onClick={() => setShowDetailModal(false)}
                  className="text-gray-400 hover:text-gray-500"
                >
                  <XMarkIcon className="h-6 w-6" />
                </button>
              </div>
            </div>

            <div className="px-6 py-4">
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* Basic Sample Information */}
                <div className="space-y-4">
                  <div>
                    <h4 className="text-sm font-medium text-gray-900 mb-3">Sample Information</h4>
                    <dl className="space-y-2">
                      <div className="flex justify-between">
                        <dt className="text-sm text-gray-500">Name:</dt>
                        <dd className="text-sm font-medium text-gray-900">{selectedSample.name}</dd>
                      </div>
                      <div className="flex justify-between">
                        <dt className="text-sm text-gray-500">Barcode:</dt>
                        <dd className="text-sm font-medium text-gray-900">{selectedSample.barcode}</dd>
                      </div>
                      <div className="flex justify-between">
                        <dt className="text-sm text-gray-500">Location:</dt>
                        <dd className="text-sm font-medium text-gray-900">{selectedSample.location}</dd>
                      </div>
                      <div className="flex justify-between">
                        <dt className="text-sm text-gray-500">Status:</dt>
                        <dd>
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(selectedSample.status)}`}>
                            {selectedSample.status}
                          </span>
                        </dd>
                      </div>
                      <div className="flex justify-between">
                        <dt className="text-sm text-gray-500">Created:</dt>
                        <dd className="text-sm font-medium text-gray-900">
                          {new Date(selectedSample.created_at).toLocaleString()}
                        </dd>
                      </div>
                    </dl>
                  </div>

                  {/* AI Extraction Details */}
                  <div>
                    <h4 className="text-sm font-medium text-gray-900 mb-3">AI Extraction Details</h4>
                    <dl className="space-y-2">
                      {selectedSample.metadata?.confidence_score && (
                        <div className="flex justify-between">
                          <dt className="text-sm text-gray-500">Confidence Score:</dt>
                          <dd>
                            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getConfidenceColor(selectedSample.metadata.confidence_score)}`}>
                              {(selectedSample.metadata.confidence_score * 100).toFixed(1)}% ({getConfidenceLabel(selectedSample.metadata.confidence_score)})
                            </span>
                          </dd>
                        </div>
                      )}
                      {selectedSample.metadata?.processing_time && (
                        <div className="flex justify-between">
                          <dt className="text-sm text-gray-500">Processing Time:</dt>
                          <dd className="text-sm font-medium text-gray-900">
                            {selectedSample.metadata.processing_time.toFixed(2)}s
                          </dd>
                        </div>
                      )}
                      {selectedSample.metadata?.source_document && (
                        <div className="flex justify-between">
                          <dt className="text-sm text-gray-500">Source Document:</dt>
                          <dd className="text-sm font-medium text-gray-900 truncate" title={selectedSample.metadata.source_document}>
                            {selectedSample.metadata.source_document}
                          </dd>
                        </div>
                      )}
                      {selectedSample.metadata?.submitter_name && (
                        <div className="flex justify-between">
                          <dt className="text-sm text-gray-500">Submitted By:</dt>
                          <dd className="text-sm font-medium text-gray-900">
                            {selectedSample.metadata.submitter_name}
                            {selectedSample.metadata.submitter_email && (
                              <span className="text-gray-500 ml-1">({selectedSample.metadata.submitter_email})</span>
                            )}
                          </dd>
                        </div>
                      )}
                    </dl>
                  </div>
                </div>

                {/* Warnings and Additional Information */}
                <div className="space-y-4">
                  {/* Extraction Warnings */}
                  {(selectedSample.metadata?.validation_warnings?.length || selectedSample.metadata?.extraction_warnings?.length) && (
                    <div>
                      <h4 className="text-sm font-medium text-gray-900 mb-3">Warnings</h4>
                      <div className="space-y-2">
                        {selectedSample.metadata?.validation_warnings?.map((warning, index) => (
                          <div key={`validation-${index}`} className="flex items-start">
                            <ExclamationTriangleIcon className="h-4 w-4 text-yellow-500 mt-0.5 mr-2 flex-shrink-0" />
                            <span className="text-sm text-yellow-700">{warning}</span>
                          </div>
                        ))}
                        {selectedSample.metadata?.extraction_warnings?.map((warning, index) => (
                          <div key={`extraction-${index}`} className="flex items-start">
                            <InformationCircleIcon className="h-4 w-4 text-blue-500 mt-0.5 mr-2 flex-shrink-0" />
                            <span className="text-sm text-blue-700">{warning}</span>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                                  {/* Extracted Data from Document */}
                {selectedSample.metadata?.rag_submission_id && (
                  <div>
                    <h4 className="text-sm font-medium text-gray-900 mb-3 flex items-center">
                      <DocumentTextIcon className="h-4 w-4 mr-2" />
                      Scraped Document Data
                    </h4>
                    {isLoadingDetail ? (
                      <div className="bg-gray-50 rounded-md p-6 text-center">
                        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-600 mx-auto"></div>
                        <p className="mt-2 text-xs text-gray-500">Loading extracted data...</p>
                      </div>
                    ) : ragSubmissionDetail?.extracted_data ? (
                      <div className="bg-gray-50 rounded-md p-4 max-h-80 overflow-y-auto">
                        {renderExtractedDataSection('Administrative Information', ragSubmissionDetail.extracted_data.administrative_info)}
                        {renderExtractedDataSection('Source Material', ragSubmissionDetail.extracted_data.source_material)}
                        {renderExtractedDataSection('Pooling Information', ragSubmissionDetail.extracted_data.pooling_info)}
                        {renderExtractedDataSection('Sequence Generation', ragSubmissionDetail.extracted_data.sequence_generation)}
                        {renderExtractedDataSection('Container Information', ragSubmissionDetail.extracted_data.container_info)}
                        {renderExtractedDataSection('Informatics Requirements', ragSubmissionDetail.extracted_data.informatics_info)}
                        {renderExtractedDataSection('Sample Details', ragSubmissionDetail.extracted_data.sample_details)}
                        
                        {/* Show any other extracted data sections */}
                        {Object.entries(ragSubmissionDetail.extracted_data)
                          .filter(([key]) => ![
                            'administrative_info', 'source_material', 'pooling_info',
                            'sequence_generation', 'container_info', 'informatics_info', 'sample_details'
                          ].includes(key))
                          .map(([key, value]) => renderExtractedDataSection(
                            key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase()), 
                            value
                          ))
                        }
                      </div>
                    ) : (
                      <div className="bg-gray-50 rounded-md p-4 text-center">
                        <ExclamationTriangleIcon className="h-6 w-6 text-gray-400 mx-auto" />
                        <p className="mt-2 text-xs text-gray-500">No extracted data available</p>
                      </div>
                    )}
                  </div>
                )}

                {/* Additional Sample Metadata */}
                {selectedSample.metadata && Object.keys(selectedSample.metadata).length > 0 && (
                  <div>
                    <h4 className="text-sm font-medium text-gray-900 mb-3">Sample Metadata</h4>
                    <div className="bg-gray-50 rounded-md p-3">
                      <dl className="space-y-1">
                        {Object.entries(selectedSample.metadata)
                          .filter(([key]) => ![
                            'confidence_score', 'processing_time', 'source_document', 
                            'submitter_name', 'submitter_email', 'rag_submission_id',
                            'extraction_method', 'validation_warnings', 'extraction_warnings'
                          ].includes(key))
                          .map(([key, value]) => (
                            <div key={key} className="flex justify-between">
                              <dt className="text-xs text-gray-500 capitalize">
                                {key.replace(/_/g, ' ')}:
                              </dt>
                              <dd className="text-xs font-medium text-gray-900 max-w-xs truncate" title={String(value)}>
                                {String(value)}
                              </dd>
                            </div>
                          ))}
                      </dl>
                    </div>
                  </div>
                )}

                  {/* RAG Submission Link */}
                  {selectedSample.metadata?.rag_submission_id && (
                    <div>
                      <h4 className="text-sm font-medium text-gray-900 mb-3">Related Submission</h4>
                      <Link
                        to={`/rag-submissions?submission=${selectedSample.metadata.rag_submission_id}`}
                        className="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-indigo-700 bg-indigo-100 hover:bg-indigo-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                        onClick={() => setShowDetailModal(false)}
                      >
                        <DocumentTextIcon className="h-4 w-4 mr-2" />
                        View Original RAG Submission
                      </Link>
                    </div>
                  )}
                </div>
              </div>
            </div>

            <div className="px-6 py-4 border-t border-gray-200 bg-gray-50">
              <div className="flex justify-end space-x-3">
                <button
                  type="button"
                  onClick={() => setShowDetailModal(false)}
                  className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                >
                  Close
                </button>
                <Link
                  to={`/samples/${selectedSample.id}/edit`}
                  className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                  onClick={() => setShowDetailModal(false)}
                >
                  Edit Sample
                </Link>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
} 
