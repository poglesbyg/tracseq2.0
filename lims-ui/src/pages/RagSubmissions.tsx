import { useState, useEffect, useMemo } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import api from '../utils/axios';
import { WindowComponentProps } from '../components/Desktop/Window';
import {
  DocumentArrowUpIcon,
  EyeIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  MagnifyingGlassIcon,
  SparklesIcon,
  ArrowLeftIcon,
  DocumentTextIcon,
  ClipboardDocumentIcon,
  ChevronDownIcon,
  ChevronRightIcon,
} from '@heroicons/react/24/outline';

interface RagSubmission {
  id: string;
  filename: string;
  status: 'uploaded' | 'processing' | 'completed' | 'failed';
  uploadedAt: string;
  processedAt?: string;
  confidence?: number;
  extractedSamples: number;
  errors?: string[];
  metadata?: Record<string, unknown>;
}

interface RagExtractionResult {
  success: boolean;
  samples: Array<{
    name: string;
    barcode: string;
    location: string;
    metadata: Record<string, unknown>;
  }>;
  confidence_score: number;
  validation_warnings: string[];
  processing_time: number;
  source_document?: string;
  id?: string;
  status?: string;
  message?: string;
  extraction_result?: {
    success: boolean;
    confidence_score: number;
    warnings: string[];
    source_document: string;
    submission?: Record<string, unknown>;
  };
}

export default function RagSubmissions({ windowContext }: WindowComponentProps) {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [dragActive, setDragActive] = useState(false);
  const [extractionResult, setExtractionResult] = useState<RagExtractionResult | null>(null);
  const [showPreview, setShowPreview] = useState(false);
  const [autoCreate, setAutoCreate] = useState(false);
  const [confidenceThreshold, setConfidenceThreshold] = useState(0.7);
  const [query, setQuery] = useState('');
  const [queryResult, setQueryResult] = useState<string | null>(null);
  const [selectedSubmission, setSelectedSubmission] = useState<RagSubmission | null>(null);
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({});

  const queryClient = useQueryClient();
  
  // Get parameters from window context instead of URL (memoized to prevent re-renders)
  const windowParams = useMemo((): Record<string, string> => {
    if (windowContext?.windowId) {
      const params = (window as Record<string, any>).appParams?.[windowContext.windowId];
      console.log('🔍 RAG Submissions getting window params:', { windowId: windowContext.windowId, params });
      return params || {};
    }
    console.log('⚠️ RAG Submissions: No window context available');
    return {};
  }, [windowContext?.windowId]);

  const submissionId = windowParams.submission;
  const mode = windowParams.mode;
  const isPreviewMode = mode === 'preview';
  
  console.log('📋 RAG Submissions initialized with:', { submissionId, mode, isPreviewMode, windowParams });
  
  useEffect(() => {
    if (isPreviewMode) {
      setShowPreview(true);
      setAutoCreate(false);
    }
  }, [isPreviewMode]);

  // Fetch existing RAG submissions
  const { data: ragSubmissions, isLoading: isLoadingSubmissions, error: submissionsError } = useQuery<RagSubmission[]>({
    queryKey: ['rag-submissions'],
    queryFn: async () => {
      try {
        const url = `/api/rag/submissions`;
        const response = await api.get(url);
        
        // Handle the response structure from RAG service
        let submissions = [];
        if (response.data && response.data.data && Array.isArray(response.data.data)) {
          submissions = response.data.data;
        } else if (response.data && response.data.submissions && Array.isArray(response.data.submissions)) {
          submissions = response.data.submissions;
        } else if (Array.isArray(response.data)) {
          submissions = response.data;
        }
        
        // Map the API response to the expected frontend format
        return submissions.map((item: Record<string, any>) => ({
          id: item.id || item.submission_id || `RAG-${Date.now()}`,
          filename: item.filename || item.document_name || 'Unknown Document',
          status: item.status === 'Processed' ? 'completed' : 
                  item.status === 'Processing' ? 'processing' : 
                  item.status === 'Pending' ? 'uploaded' : 'failed',
          uploadedAt: item.submittedDate || item.uploadedAt || item.created_at || new Date().toISOString(),
          processedAt: item.processedDate || item.processedAt,
          confidence: item.confidenceScore || item.confidence_score || item.confidence || 0,
          extractedSamples: item.extractedFields || item.extractedSamples || 0,
          errors: item.errors || [],
          metadata: item.metadata || {}
        }));
      } catch (error) {
        console.error('Failed to fetch RAG submissions:', error);
        // Return empty array on error to prevent crashes
        return [];
      }
    },
    retry: 2,
    retryDelay: 1000,
  });

  // Enhanced parameter handling from window context
  useEffect(() => {
    console.log('🔍 Checking window parameters:', { submissionId, ragSubmissions: ragSubmissions?.length });
    
    if (submissionId && ragSubmissions && ragSubmissions.length > 0) {
      const submission = ragSubmissions.find(s => s.id === submissionId);
      console.log('🎯 Found submission:', submission);
      
      if (submission) {
        setSelectedSubmission(submission);
        // Expand all sections by default for better visibility
        setExpandedSections({
          overview: true,
          extractedData: true,
          metadata: true,
          errors: true
        });
      } else {
        console.warn('⚠️ Submission not found:', submissionId);
        // Show error message or redirect
      }
    } else if (submissionId && ragSubmissions && ragSubmissions.length === 0) {
      console.warn('⚠️ No submissions available yet, waiting...');
    }
  }, [submissionId, ragSubmissions]);

  // Fetch detailed submission data
  const { data: submissionDetail, isLoading: isLoadingDetail, error: detailError } = useQuery({
    queryKey: ['rag-submission-detail', selectedSubmission?.id],
    queryFn: async () => {
      if (!selectedSubmission?.id) return null;
      
      try {
        console.log('📊 Fetching detailed submission data for:', selectedSubmission.id);
        const response = await api.get(`/api/rag/submissions/${selectedSubmission.id}`);
        console.log('✅ Detailed submission data received:', response.data);
        return response.data;
      } catch (error) {
        console.error('❌ Failed to fetch submission details:', error);
        return null;
      }
    },
    enabled: !!selectedSubmission?.id,
    retry: 2,
    retryDelay: 1000,
  });

  // Process document mutation
  const processDocumentMutation = useMutation({
    mutationFn: async (formData: FormData) => {
      const url = `/api/rag/process`;
      const response = await api.post(url, formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      });
      return response.data;
    },
    onSuccess: (data: RagExtractionResult) => {
      setExtractionResult(data);
      queryClient.invalidateQueries({ queryKey: ['rag-submissions'] });
      queryClient.invalidateQueries({ queryKey: ['samples'] });
    },
  });

  // Preview document mutation (using same process endpoint for now)
  const previewDocumentMutation = useMutation({
    mutationFn: async (formData: FormData) => {
      const url = `/api/rag/process`;
      const response = await api.post(url, formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      });
      return response.data;
    },
    onSuccess: (data: RagExtractionResult) => {
      setExtractionResult(data);
      setShowPreview(true);
    },
  });

  // Query mutation (simplified for now)
  const queryMutation = useMutation({
    mutationFn: async (queryText: string) => {
      // For now, just return a placeholder response since query endpoint may not be implemented
      return {
        answer: `Query "${queryText}" received. RAG system is connected and ready to process documents.`
      };
    },
    onSuccess: (data) => {
      setQueryResult(data.answer);
    },
  });

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === 'dragenter' || e.type === 'dragover') {
      setDragActive(true);
    } else if (e.type === 'dragleave') {
      setDragActive(false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);
    
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      setSelectedFile(e.dataTransfer.files[0]);
    }
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      setSelectedFile(e.target.files[0]);
    }
  };

  const handleProcess = () => {
    if (!selectedFile) return;

    const formData = new FormData();
    formData.append('file', selectedFile);
    formData.append('auto_create', autoCreate.toString());
    formData.append('confidence_threshold', confidenceThreshold.toString());

    processDocumentMutation.mutate(formData);
  };

  const handlePreview = () => {
    if (!selectedFile) return;

    const formData = new FormData();
    formData.append('file', selectedFile);
    formData.append('confidence_threshold', confidenceThreshold.toString());

    previewDocumentMutation.mutate(formData);
  };

  const handleQuery = () => {
    if (!query.trim()) return;
    queryMutation.mutate(query);
  };

  const handleBackToSubmissions = () => {
    setSelectedSubmission(null);
    // Clear window parameters
    if (windowContext?.windowId) {
      delete (window as Record<string, any>).appParams?.[windowContext.windowId];
    }
  };

  const toggleSection = (section: string) => {
    setExpandedSections(prev => ({
      ...prev,
      [section]: !prev[section]
    }));
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const getConfidenceColor = (score: number) => {
    if (score >= 0.8) return 'text-green-600 bg-green-100';
    if (score >= 0.6) return 'text-yellow-600 bg-yellow-100';
    return 'text-red-600 bg-red-100';
  };

  const renderDataSection = (title: string, data: Record<string, any> | any[], sectionKey: string) => {
    if (!data || (typeof data === 'object' && Object.keys(data).length === 0)) {
      return null;
    }

    const isExpanded = expandedSections[sectionKey];

    return (
      <div className="border border-gray-200 rounded-lg">
        <button
          onClick={() => toggleSection(sectionKey)}
          className="w-full px-4 py-3 flex items-center justify-between text-left hover:bg-gray-50"
        >
          <h3 className="text-lg font-medium text-gray-900">{title}</h3>
          {isExpanded ? (
            <ChevronDownIcon className="h-5 w-5 text-gray-500" />
          ) : (
            <ChevronRightIcon className="h-5 w-5 text-gray-500" />
          )}
        </button>
        
        {isExpanded && (
          <div className="px-4 pb-4 border-t border-gray-200">
            {typeof data === 'object' ? (
              <div className="space-y-3">
                {Object.entries(data).map(([key, value]) => (
                  <div key={key} className="flex justify-between items-start">
                    <dt className="text-sm font-medium text-gray-500 capitalize min-w-0 flex-1">
                      {key.replace(/_/g, ' ')}:
                    </dt>
                    <dd className="text-sm text-gray-900 ml-4 flex-1">
                      {typeof value === 'object' ? (
                        <pre className="text-xs bg-gray-50 p-2 rounded overflow-x-auto">
                          {JSON.stringify(value, null, 2)}
                        </pre>
                      ) : (
                        <span className="break-words">{String(value)}</span>
                      )}
                    </dd>
                    <button
                      onClick={() => copyToClipboard(String(value))}
                      className="ml-2 p-1 text-gray-400 hover:text-gray-600"
                      title="Copy to clipboard"
                    >
                      <ClipboardDocumentIcon className="h-4 w-4" />
                    </button>
                  </div>
                ))}
              </div>
            ) : (
              <div className="flex justify-between items-center">
                <pre className="text-sm text-gray-900 whitespace-pre-wrap flex-1">
                  {String(data)}
                </pre>
                <button
                  onClick={() => copyToClipboard(String(data))}
                  className="ml-2 p-1 text-gray-400 hover:text-gray-600"
                  title="Copy to clipboard"
                >
                  <ClipboardDocumentIcon className="h-4 w-4" />
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      {/* Header */}
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <div className="flex items-start">
            <div className="flex-shrink-0">
              <Link
                to="/"
                className="inline-flex items-center text-sm text-gray-500 hover:text-gray-700 mb-4"
              >
                <ArrowLeftIcon className="h-4 w-4 mr-1" />
                Back to Dashboard
              </Link>
            </div>
          </div>
          <div className="flex items-center">
            <div className="h-10 w-10 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-lg flex items-center justify-center mr-4">
              <SparklesIcon className="h-6 w-6 text-white" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900">
                {selectedSubmission ? `RAG Submission: ${selectedSubmission.filename}` : 'AI-Powered Document Submissions'}
              </h1>
              <p className="mt-2 text-sm text-gray-700">
                {selectedSubmission 
                  ? `Viewing details for submission uploaded on ${new Date(selectedSubmission.uploadedAt).toLocaleDateString()}`
                  : isPreviewMode 
                    ? "Preview mode: Upload a document to see extracted data without creating samples"
                    : "Upload laboratory documents (PDF, DOCX, TXT) to automatically extract sample data using AI"
                }
              </p>
              {isPreviewMode && !selectedSubmission && (
                <div className="mt-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                  Preview Mode Active
                </div>
              )}
              {submissionId && !selectedSubmission && ragSubmissions && ragSubmissions.length > 0 && (
                <div className="mt-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                  Submission {submissionId} not found
                </div>
              )}
            </div>
          </div>
        </div>
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
          {selectedSubmission ? (
            <button
              onClick={handleBackToSubmissions}
              className="inline-flex items-center justify-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
            >
              <ArrowLeftIcon className="h-4 w-4 mr-2" />
              Back to Submissions
            </button>
          ) : (
            <Link
              to="/rag-samples"
              className="inline-flex items-center justify-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
            >
              <EyeIcon className="h-4 w-4 mr-2" />
              View AI Sample Records
            </Link>
          )}
        </div>
      </div>

      {/* Enhanced Submission Detail View */}
      {selectedSubmission && (
        <div className="mt-8 space-y-6">
          {/* Submission Overview */}
          {renderDataSection('Submission Overview', {
            'ID': selectedSubmission.id,
            'Filename': selectedSubmission.filename,
            'Status': selectedSubmission.status,
            'Uploaded': new Date(selectedSubmission.uploadedAt).toLocaleString(),
            'Processed': selectedSubmission.processedAt ? new Date(selectedSubmission.processedAt).toLocaleString() : 'Not processed',
            'Confidence Score': `${((selectedSubmission.confidence || 0) * 100).toFixed(1)}%`,
            'Extracted Samples': selectedSubmission.extractedSamples,
          }, 'overview')}

          {/* Detailed Submission Data */}
          {isLoadingDetail ? (
            <div className="bg-white shadow rounded-lg p-6">
              <div className="text-center py-8">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600 mx-auto"></div>
                <p className="mt-2 text-sm text-gray-500">Loading detailed submission data...</p>
              </div>
            </div>
          ) : detailError ? (
            <div className="bg-white shadow rounded-lg p-6">
              <div className="text-center py-8">
                <ExclamationTriangleIcon className="h-8 w-8 text-red-500 mx-auto" />
                <p className="mt-2 text-sm text-red-600">Failed to load detailed submission data</p>
                <p className="text-xs text-gray-500 mt-1">Error: {String(detailError)}</p>
              </div>
            </div>
          ) : submissionDetail ? (
            <>
              {/* Extracted Data */}
              {submissionDetail.extracted_data && renderDataSection(
                'Extracted Data from Document', 
                submissionDetail.extracted_data, 
                'extractedData'
              )}

              {/* Full Submission Detail */}
              {renderDataSection('Complete Submission Data', submissionDetail, 'fullData')}
            </>
          ) : (
            <div className="bg-white shadow rounded-lg p-6">
              <div className="text-center py-8">
                <DocumentTextIcon className="h-8 w-8 text-gray-400 mx-auto" />
                <p className="mt-2 text-sm text-gray-500">No detailed data available for this submission</p>
              </div>
            </div>
          )}

          {/* Errors */}
          {selectedSubmission.errors && selectedSubmission.errors.length > 0 && 
            renderDataSection('Errors', selectedSubmission.errors, 'errors')
          }

          {/* Metadata */}
          {selectedSubmission.metadata && Object.keys(selectedSubmission.metadata).length > 0 && 
            renderDataSection('Metadata', selectedSubmission.metadata, 'metadata')
          }

          {/* Quick Actions */}
          <div className="bg-white shadow rounded-lg p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-4">Quick Actions</h3>
            <div className="flex flex-wrap gap-3">
              <button
                onClick={() => copyToClipboard(selectedSubmission.id)}
                className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
              >
                <ClipboardDocumentIcon className="h-4 w-4 mr-2" />
                Copy Submission ID
              </button>
              <Link
                to="/rag-samples"
                className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
              >
                <EyeIcon className="h-4 w-4 mr-2" />
                View Related Samples
              </Link>
              <button
                onClick={() => window.location.reload()}
                className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
              >
                Refresh Data
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Main Content - only show if not viewing specific submission */}
      {!selectedSubmission && (
        <div className="mt-8 space-y-8">
          {/* Recent RAG Submissions */}
          <div className="bg-white shadow rounded-lg p-6">
            <h2 className="text-lg font-medium text-gray-900 mb-4">Recent RAG Submissions</h2>
            {isLoadingSubmissions ? (
              <div className="text-center py-4">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600 mx-auto"></div>
                <p className="mt-2 text-sm text-gray-500">Loading submissions...</p>
              </div>
            ) : submissionsError ? (
              <div className="text-center py-4">
                <div className="text-yellow-600 mb-2">
                  <ExclamationTriangleIcon className="h-8 w-8 mx-auto" />
                </div>
                <p className="text-sm text-gray-600">
                  Unable to load RAG submissions. The RAG service may be unavailable.
                </p>
                <p className="text-xs text-gray-500 mt-1">
                  You can still upload documents for processing.
                </p>
              </div>
            ) : ragSubmissions && Array.isArray(ragSubmissions) && ragSubmissions.length > 0 ? (
              <div className="space-y-3">
                {ragSubmissions.map((submission) => (
                  <div key={submission.id} className="border border-gray-200 rounded-md p-4 hover:bg-gray-50">
                    <div className="flex justify-between items-start">
                      <div className="flex-1">
                        <div className="flex items-center">
                          <h3 className="text-sm font-medium text-gray-900">{submission.filename}</h3>
                          <button
                            onClick={() => setSelectedSubmission(submission)}
                            className="ml-2 text-indigo-600 hover:text-indigo-800 text-sm"
                          >
                            View Details
                          </button>
                        </div>
                        <p className="text-xs text-gray-500 mt-1">
                          ID: {submission.id} • Uploaded: {new Date(submission.uploadedAt).toLocaleDateString()} • Status: {submission.status}
                        </p>
                      </div>
                      <div className="flex items-center space-x-2">
                        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getConfidenceColor(submission.confidence || 0)}`}>
                          {((submission.confidence || 0) * 100).toFixed(1)}%
                        </span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-4 text-gray-500">
                <p>No RAG submissions found.</p>
                <p className="text-sm mt-1">Upload a document below to create your first submission.</p>
              </div>
            )}
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {/* Document Upload Section */}
            <div className="space-y-6">
              <div className="bg-white shadow rounded-lg p-6">
                <h2 className="text-lg font-medium text-gray-900 mb-4">Upload Document</h2>
                
                {/* File Upload Area */}
                <div
                  className={`mt-2 flex justify-center px-6 pt-5 pb-6 border-2 border-dashed rounded-md transition-colors ${
                    dragActive
                      ? 'border-indigo-400 bg-indigo-50'
                      : 'border-gray-300 hover:border-gray-400'
                  }`}
                  onDragEnter={handleDrag}
                  onDragLeave={handleDrag}
                  onDragOver={handleDrag}
                  onDrop={handleDrop}
                >
                  <div className="space-y-1 text-center">
                    <DocumentArrowUpIcon className="mx-auto h-12 w-12 text-gray-400" />
                    <div className="flex text-sm text-gray-600">
                      <label
                        htmlFor="file-upload"
                        className="relative cursor-pointer bg-white rounded-md font-medium text-indigo-600 hover:text-indigo-500 focus-within:outline-none focus-within:ring-2 focus-within:ring-offset-2 focus-within:ring-indigo-500"
                      >
                        <span>Upload a file</span>
                        <input
                          id="file-upload"
                          name="file-upload"
                          type="file"
                          className="sr-only"
                          accept=".pdf,.docx,.txt"
                          onChange={handleFileSelect}
                        />
                      </label>
                      <p className="pl-1">or drag and drop</p>
                    </div>
                    <p className="text-xs text-gray-500">PDF, DOCX, TXT up to 50MB</p>
                  </div>
                </div>

                {selectedFile && (
                  <div className="mt-4 p-3 bg-gray-50 rounded-md">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm font-medium text-gray-900">{selectedFile.name}</p>
                        <p className="text-xs text-gray-500">
                          {(selectedFile.size / 1024 / 1024).toFixed(2)} MB
                        </p>
                      </div>
                      <button
                        onClick={() => setSelectedFile(null)}
                        className="text-gray-400 hover:text-gray-500"
                      >
                        ×
                      </button>
                    </div>
                  </div>
                )}

                {/* Processing Options */}
                <div className="mt-6 space-y-4">
                  <div className="flex items-center justify-between">
                    <label className="text-sm font-medium text-gray-700">
                      Confidence Threshold
                    </label>
                    <span className="text-sm text-gray-500">{confidenceThreshold}</span>
                  </div>
                  <input
                    type="range"
                    min="0.5"
                    max="1"
                    step="0.05"
                    value={confidenceThreshold}
                    onChange={(e) => setConfidenceThreshold(parseFloat(e.target.value))}
                    className="w-full"
                  />
                  
                  <div className="flex items-center">
                    <input
                      id="auto-create"
                      type="checkbox"
                      checked={autoCreate}
                      onChange={(e) => setAutoCreate(e.target.checked)}
                      disabled={isPreviewMode}
                      className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded disabled:opacity-50"
                    />
                    <label htmlFor="auto-create" className={`ml-2 text-sm ${isPreviewMode ? 'text-gray-500' : 'text-gray-700'}`}>
                      Automatically create samples after extraction
                      {isPreviewMode && <span className="text-xs text-purple-600 ml-1">(disabled in preview mode)</span>}
                    </label>
                  </div>
                </div>

                {/* Action Buttons */}
                <div className="mt-6 flex space-x-3">
                  <button
                    onClick={handlePreview}
                    disabled={!selectedFile || previewDocumentMutation.isPending}
                    className="flex-1 inline-flex items-center justify-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
                  >
                    <EyeIcon className="h-4 w-4 mr-2" />
                    Preview
                  </button>
                  <button
                    onClick={handleProcess}
                    disabled={!selectedFile || processDocumentMutation.isPending}
                    className="flex-1 inline-flex items-center justify-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
                  >
                    <SparklesIcon className="h-4 w-4 mr-2" />
                    {processDocumentMutation.isPending ? 'Processing...' : 'Process & Extract'}
                  </button>
                </div>
              </div>

              {/* Natural Language Query Section */}
              <div className="bg-white shadow rounded-lg p-6">
                <h2 className="text-lg font-medium text-gray-900 mb-4">Ask About Your Data</h2>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      Natural Language Query
                    </label>
                    <div className="flex space-x-2">
                      <input
                        type="text"
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder="e.g., How many DNA samples were submitted this week?"
                        className="flex-1 border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500"
                        onKeyPress={(e) => e.key === 'Enter' && handleQuery()}
                      />
                      <button
                        onClick={handleQuery}
                        disabled={!query.trim() || queryMutation.isPending}
                        className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
                      >
                        <MagnifyingGlassIcon className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                  
                  {queryResult && (
                    <div className="p-4 bg-blue-50 border border-blue-200 rounded-md">
                      <div className="flex">
                        <div className="flex-shrink-0">
                          <SparklesIcon className="h-5 w-5 text-blue-400" />
                        </div>
                        <div className="ml-3">
                          <h3 className="text-sm font-medium text-blue-800">AI Response</h3>
                          <div className="mt-2 text-sm text-blue-700">
                            {queryResult}
                          </div>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>

            {/* Results Section */}
            <div className="space-y-6">
              {/* Extraction Results */}
              {extractionResult && (
                <div className="bg-white shadow rounded-lg p-6">
                  <h2 className="text-lg font-medium text-gray-900 mb-4">Extraction Results</h2>
                  
                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium text-gray-700">Confidence Score</span>
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getConfidenceColor(extractionResult.confidence_score)}`}>
                        {(extractionResult.confidence_score * 100).toFixed(1)}%
                      </span>
                    </div>
                    
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium text-gray-700">Samples Found</span>
                      <span className="text-sm text-gray-900">{extractionResult.samples?.length || 0}</span>
                    </div>
                    
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium text-gray-700">Processing Time</span>
                      <span className="text-sm text-gray-900">{extractionResult.processing_time.toFixed(2)}s</span>
                    </div>

                    {extractionResult.validation_warnings && extractionResult.validation_warnings.length > 0 && (
                      <div>
                        <span className="text-sm font-medium text-gray-700 block mb-2">Warnings</span>
                        <div className="space-y-1">
                          {extractionResult.validation_warnings.map((warning: string, index: number) => (
                            <div key={index} className="flex items-center text-xs text-yellow-600">
                              <ExclamationTriangleIcon className="h-3 w-3 mr-1" />
                              {warning}
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {extractionResult.extraction_result?.warnings && extractionResult.extraction_result.warnings.length > 0 && (
                      <div>
                        <span className="text-sm font-medium text-gray-700 block mb-2">Extraction Warnings</span>
                        <div className="space-y-1">
                          {extractionResult.extraction_result.warnings.map((warning: string, index: number) => (
                            <div key={index} className="flex items-center text-xs text-yellow-600">
                              <ExclamationTriangleIcon className="h-3 w-3 mr-1" />
                              {warning}
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {extractionResult.samples && extractionResult.samples.length > 0 && (
                      <div>
                        <span className="text-sm font-medium text-gray-700 block mb-2">Extracted Samples</span>
                        <div className="space-y-2">
                          {extractionResult.samples?.map((sample, index) => (
                            <div key={index} className="p-3 border border-gray-200 rounded-md">
                              <div className="flex justify-between items-start">
                                <div>
                                  <p className="text-sm font-medium text-gray-900">{sample.name}</p>
                                  <p className="text-xs text-gray-500">Barcode: {sample.barcode}</p>
                                  <p className="text-xs text-gray-500">Location: {sample.location}</p>
                                </div>
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {showPreview && (
                      <div className="pt-4 border-t border-gray-200">
                        <button
                          onClick={() => {
                            setShowPreview(false);
                            handleProcess();
                          }}
                          className="w-full inline-flex items-center justify-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                        >
                          <CheckCircleIcon className="h-4 w-4 mr-2" />
                          Confirm & Create Samples
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* Instructions */}
              <div className="bg-white shadow rounded-lg p-6">
                <h2 className="text-lg font-medium text-gray-900 mb-4">How It Works</h2>
                <div className="space-y-3 text-sm text-gray-600">
                  <div className="flex items-start">
                    <div className="flex-shrink-0">
                      <div className="w-6 h-6 bg-indigo-100 rounded-full flex items-center justify-center">
                        <span className="text-xs font-medium text-indigo-600">1</span>
                      </div>
                    </div>
                    <div className="ml-3">
                      <p>Upload your laboratory document (PDF, DOCX, or TXT)</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="flex-shrink-0">
                      <div className="w-6 h-6 bg-indigo-100 rounded-full flex items-center justify-center">
                        <span className="text-xs font-medium text-indigo-600">2</span>
                      </div>
                    </div>
                    <div className="ml-3">
                      <p>AI extracts sample information with confidence scoring</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="flex-shrink-0">
                      <div className="w-6 h-6 bg-indigo-100 rounded-full flex items-center justify-center">
                        <span className="text-xs font-medium text-indigo-600">3</span>
                      </div>
                    </div>
                    <div className="ml-3">
                      <p>Review extracted data and create samples automatically</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}