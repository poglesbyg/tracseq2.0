import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import axios from 'axios';
import {
  DocumentArrowUpIcon,
  EyeIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  MagnifyingGlassIcon,
  SparklesIcon,
} from '@heroicons/react/24/outline';

interface RagExtractionResult {
  success: boolean;
  samples: Array<{
    name: string;
    barcode: string;
    location: string;
    metadata: any;
  }>;
  confidence_score: number;
  warnings: string[];
  processing_time: number;
  source_document: string;
  message: string;
}

export default function RagSubmissions() {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [dragActive, setDragActive] = useState(false);
  const [extractionResult, setExtractionResult] = useState<RagExtractionResult | null>(null);
  const [showPreview, setShowPreview] = useState(false);
  const [autoCreate, setAutoCreate] = useState(false);
  const [confidenceThreshold, setConfidenceThreshold] = useState(0.7);
  const [query, setQuery] = useState('');
  const [queryResult, setQueryResult] = useState<string | null>(null);

  const queryClient = useQueryClient();

  // Process document mutation
  const processDocumentMutation = useMutation({
    mutationFn: async (formData: FormData) => {
      const response = await axios.post('/api/samples/rag/process-document', formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      });
      return response.data;
    },
    onSuccess: (data: RagExtractionResult) => {
      setExtractionResult(data);
      queryClient.invalidateQueries({ queryKey: ['samples'] });
    },
  });

  // Preview document mutation
  const previewDocumentMutation = useMutation({
    mutationFn: async (formData: FormData) => {
      const response = await axios.post('/api/samples/rag/preview', formData, {
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

  // Query mutation
  const queryMutation = useMutation({
    mutationFn: async (queryText: string) => {
      const response = await axios.post('/api/samples/rag/query', {
        query: queryText,
        context: 'recent_submissions',
        limit: 10,
      });
      return response.data;
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
          <div className="flex items-center">
            <SparklesIcon className="h-8 w-8 text-indigo-600 mr-3" />
            <div>
              <h1 className="text-xl font-semibold text-gray-900">AI-Powered Document Submissions</h1>
              <p className="mt-2 text-sm text-gray-700">
                Upload laboratory documents (PDF, DOCX, TXT) to automatically extract sample data using AI
              </p>
            </div>
          </div>
        </div>
      </div>

      <div className="mt-8 grid grid-cols-1 lg:grid-cols-2 gap-8">
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
                  className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                />
                <label htmlFor="auto-create" className="ml-2 text-sm text-gray-700">
                  Automatically create samples after extraction
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

                {extractionResult.warnings.length > 0 && (
                  <div>
                    <span className="text-sm font-medium text-gray-700 block mb-2">Warnings</span>
                    <div className="space-y-1">
                      {extractionResult.warnings.map((warning, index) => (
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
  );
} 
