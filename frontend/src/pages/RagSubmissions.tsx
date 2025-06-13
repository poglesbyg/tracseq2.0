import { useState, useEffect } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useSearchParams, Link } from 'react-router-dom';
import axios from 'axios';
import API_CONFIG from '../utils/api';
import {
  DocumentArrowUpIcon,
  EyeIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  MagnifyingGlassIcon,
  SparklesIcon,
  ArrowLeftIcon,
} from '@heroicons/react/24/outline';

interface RagSubmission {
  id: string;
  submission_id: string;
  submitter_name: string;
  submitter_email: string;
  sample_type: string;
  sample_name: string;
  confidence_score: number;
  created_at: string;
  status: string;
}

interface RagExtractionResult {
  success: boolean;
  samples: Array<{
    name: string;
    barcode: string;
    location: string;
    metadata: any;
  }>;
  confidence_score: number;
  validation_warnings: string[];
  processing_time: number;
  source_document?: string;
  extraction_result?: {
    success: boolean;
    confidence_score: number;
    warnings: string[];
    source_document: string;
    submission?: any;
  };
}

export default function RagSubmissions() {
  const [searchParams] = useSearchParams();
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [dragActive, setDragActive] = useState(false);
  const [extractionResult, setExtractionResult] = useState<RagExtractionResult | null>(null);
  const [showPreview, setShowPreview] = useState(false);
  const [autoCreate, setAutoCreate] = useState(false);
  const [confidenceThreshold, setConfidenceThreshold] = useState(0.7);
  const [query, setQuery] = useState('');
  const [queryResult, setQueryResult] = useState<string | null>(null);

  const queryClient = useQueryClient();
  
  // Handle URL parameters for Dashboard integration
  const mode = searchParams.get('mode');
  const isPreviewMode = mode === 'preview';
  
  useEffect(() => {
    if (isPreviewMode) {
      setShowPreview(true);
      setAutoCreate(false);
    }
  }, [isPreviewMode]);

  // Fetch existing RAG submissions
  const { data: ragSubmissions, isLoading: isLoadingSubmissions } = useQuery<RagSubmission[]>({
    queryKey: ['rag-submissions'],
    queryFn: async () => {
      const url = `${API_CONFIG.rag.baseUrl}/api/rag/submissions`;
      const response = await axios.get(url);
      return response.data;
    },
  });

  // Process document mutation
  const processDocumentMutation = useMutation({
    mutationFn: async (formData: FormData) => {
      const url = `${API_CONFIG.rag.baseUrl}/api/rag/process`;
      const response = await axios.post(url, formData, {
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
      const url = `${API_CONFIG.rag.baseUrl}/api/rag/process`;
      const response = await axios.post(url, formData, {
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
      // For now, just return a placeholder response
      return {
        answer: `Query "${queryText}" processed. Check the submissions list for actual data.`
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

  const getConfidenceColor = (score: number) => {
    if (score >= 0.8) return 'text-green-600 bg-green-100';
    if (score >= 0.6) return 'text-yellow-600 bg-yellow-100';
    return 'text-red-600 bg-red-100';
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
              <h1 className="text-2xl font-bold text-gray-900">AI-Powered Document Submissions</h1>
              <p className="mt-2 text-sm text-gray-700">
                {isPreviewMode 
                  ? "Preview mode: Upload a document to see extracted data without creating samples"
                  : "Upload laboratory documents (PDF, DOCX, TXT) to automatically extract sample data using AI"
                }
              </p>
              {isPreviewMode && (
                <div className="mt-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                  Preview Mode Active
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      <div className="mt-8 space-y-8">
        {/* Existing RAG Submissions */}
        <div className="bg-white shadow rounded-lg p-6">
          <h2 className="text-lg font-medium text-gray-900 mb-4">Recent RAG Submissions</h2>
          {isLoadingSubmissions ? (
            <div className="text-center py-4">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600 mx-auto"></div>
              <p className="mt-2 text-sm text-gray-500">Loading submissions...</p>
            </div>
          ) : ragSubmissions && ragSubmissions.length > 0 ? (
            <div className="space-y-3">
              {ragSubmissions.map((submission) => (
                <div key={submission.id} className="border border-gray-200 rounded-md p-4 hover:bg-gray-50">
                  <div className="flex justify-between items-start">
                    <div className="flex-1">
                      <h3 className="text-sm font-medium text-gray-900">{submission.sample_name}</h3>
                      <p className="text-xs text-gray-500 mt-1">
                        Submitted by: {submission.submitter_name} ({submission.submitter_email})
                      </p>
                      <p className="text-xs text-gray-500">
                        Sample Type: {submission.sample_type}
                      </p>
                      <p className="text-xs text-gray-500">
                        Created: {new Date(submission.created_at).toLocaleDateString()}
                      </p>
                    </div>
                    <div className="flex items-center space-x-2">
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getConfidenceColor(submission.confidence_score)}`}>
                        {(submission.confidence_score * 100).toFixed(1)}%
                      </span>
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        submission.status === 'completed' ? 'bg-green-100 text-green-800' : 
                        submission.status === 'failed' ? 'bg-red-100 text-red-800' : 
                        'bg-yellow-100 text-yellow-800'
                      }`}>
                        {submission.status}
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
                  <span className="text-sm text-gray-900">{extractionResult.samples.length}</span>
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

                {extractionResult.samples.length > 0 && (
                  <div>
                    <span className="text-sm font-medium text-gray-700 block mb-2">Extracted Samples</span>
                    <div className="space-y-2">
                      {extractionResult.samples.map((sample, index) => (
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
    </div>
  );
} 
