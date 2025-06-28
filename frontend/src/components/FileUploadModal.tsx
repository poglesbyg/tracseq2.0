import React, { useState, useRef, useCallback } from 'react';
import { useMutation } from '@tanstack/react-query';
import axios from 'axios';
import {
  CloudArrowUpIcon,
  DocumentTextIcon,
  TableCellsIcon,
  XMarkIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
  ArrowPathIcon,
} from '@heroicons/react/24/outline';

interface FileUploadModalProps {
  onClose: () => void;
  onSuccess: () => void;
}

interface UploadResponse {
  success: boolean;
  data?: unknown[];
  message: string;
}

interface SheetNamesResponse {
  success: boolean;
  data?: string[];
  message: string;
}

export default function FileUploadModal({ onClose, onSuccess }: FileUploadModalProps) {
  const [dragActive, setDragActive] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [uploadedBy, setUploadedBy] = useState('');
  const [availableSheets, setAvailableSheets] = useState<string[]>([]);
  const [selectedSheets, setSelectedSheets] = useState<string[]>([]);
  const [isLoadingSheets, setIsLoadingSheets] = useState(false);
  const [uploadStatus, setUploadStatus] = useState<'idle' | 'uploading' | 'success' | 'error'>('idle');
  const [uploadMessage, setUploadMessage] = useState('');
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Mutation for getting sheet names
  const sheetNamesMutation = useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData();
      formData.append('file', file);
      
      const response = await axios.post<SheetNamesResponse>('/api/spreadsheets/preview-sheets', formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      });
      return response.data;
    },
    onSuccess: (data) => {
      if (data.success && data.data) {
        setAvailableSheets(data.data);
        setSelectedSheets(data.data); // Select all sheets by default
        setIsLoadingSheets(false);
      }
    },
    onError: (error: unknown) => {
      console.error('Failed to get sheet names:', error);
      setIsLoadingSheets(false);
      setUploadMessage((error as { response?: { data?: { message?: string } } })?.response?.data?.message || 'Failed to read file sheets');
      setUploadStatus('error');
    },
  });

  // Mutation for uploading file
  const uploadMutation = useMutation({
    mutationFn: async (formData: FormData) => {
      const response = await axios.post<UploadResponse>('/api/spreadsheets/upload-multiple', formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      });
      return response.data;
    },
    onSuccess: (data) => {
      setUploadStatus('success');
      const datasetsCount = data.data?.length || 1;
      setUploadMessage(`Successfully processed ${datasetsCount} dataset(s): ${data.message}`);
      setTimeout(() => {
        onSuccess();
      }, 2000);
    },
    onError: (error: unknown) => {
      setUploadStatus('error');
      setUploadMessage((error as { response?: { data?: { message?: string } } })?.response?.data?.message || 'Upload failed. Please try again.');
    },
  });

  const handleFiles = useCallback((files: FileList) => {
    const file = files[0];
    if (file && isValidFileType(file)) {
      setSelectedFile(file);
      setUploadStatus('idle');
      setUploadMessage('');
      setAvailableSheets([]);
      setSelectedSheets([]);
      
      // For Excel files, automatically get sheet names
      if (isExcelFile(file.name)) {
        setIsLoadingSheets(true);
        sheetNamesMutation.mutate(file);
      }
    } else {
      alert('Please select a CSV, XLSX, or XLS file.');
    }
  }, [sheetNamesMutation]);

  const handleDrag = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === 'dragenter' || e.type === 'dragover') {
      setDragActive(true);
    } else if (e.type === 'dragleave') {
      setDragActive(false);
    }
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);
    
    const files = e.dataTransfer.files;
    if (files && files.length > 0) {
      handleFiles(files);
    }
  }, [handleFiles]);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files && files.length > 0) {
      handleFiles(files);
    }
  };

  const isValidFileType = (file: File) => {
    const validTypes = ['.csv', '.xlsx', '.xls'];
    const fileExtension = '.' + file.name.split('.').pop()?.toLowerCase();
    return validTypes.includes(fileExtension);
  };

  const getFileIcon = (filename: string) => {
    const extension = filename.split('.').pop()?.toLowerCase();
    switch (extension) {
      case 'csv':
        return <DocumentTextIcon className="h-8 w-8 text-green-500" />;
      case 'xlsx':
      case 'xls':
        return <TableCellsIcon className="h-8 w-8 text-blue-500" />;
      default:
        return <DocumentTextIcon className="h-8 w-8 text-gray-500" />;
    }
  };

  const isExcelFile = (filename: string) => {
    const extension = filename.split('.').pop()?.toLowerCase();
    return extension === 'xlsx' || extension === 'xls';
  };

  const handleSheetSelection = (sheetName: string) => {
    setSelectedSheets(prev => 
      prev.includes(sheetName) 
        ? prev.filter(s => s !== sheetName)
        : [...prev, sheetName]
    );
  };

  const selectAllSheets = () => {
    setSelectedSheets(availableSheets);
  };

  const selectNoSheets = () => {
    setSelectedSheets([]);
  };

  const handleUpload = async () => {
    if (!selectedFile) return;

    const formData = new FormData();
    formData.append('file', selectedFile);
    
    if (uploadedBy.trim()) {
      formData.append('uploaded_by', uploadedBy.trim());
    }
    
    // For Excel files with sheet selection, send selected sheets
    if (isExcelFile(selectedFile.name) && selectedSheets.length > 0) {
      formData.append('selected_sheets', JSON.stringify(selectedSheets));
    }

    setUploadStatus('uploading');
    uploadMutation.mutate(formData);
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  const canUpload = selectedFile && (
    !isExcelFile(selectedFile.name) || 
    (availableSheets.length > 0 && selectedSheets.length > 0)
  );

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50 flex items-center justify-center p-4">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-gray-200">
          <h2 className="text-xl font-semibold text-gray-900">Upload Spreadsheet</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
            disabled={uploadStatus === 'uploading'}
          >
            <XMarkIcon className="h-6 w-6" />
          </button>
        </div>

        <div className="p-6">
          {/* Upload Status Messages */}
          {uploadStatus === 'success' && (
            <div className="mb-6 bg-green-50 border border-green-200 rounded-md p-4">
              <div className="flex items-center">
                <CheckCircleIcon className="h-5 w-5 text-green-500 mr-2" />
                <span className="text-green-800 font-medium">Upload Successful!</span>
              </div>
              <p className="text-green-700 mt-1">{uploadMessage}</p>
            </div>
          )}

          {uploadStatus === 'error' && (
            <div className="mb-6 bg-red-50 border border-red-200 rounded-md p-4">
              <div className="flex items-center">
                <ExclamationCircleIcon className="h-5 w-5 text-red-500 mr-2" />
                <span className="text-red-800 font-medium">Upload Failed</span>
              </div>
              <p className="text-red-700 mt-1">{uploadMessage}</p>
            </div>
          )}

          {/* File Drop Zone */}
          {!selectedFile && uploadStatus !== 'success' && (
            <div
              className={`relative border-2 border-dashed rounded-lg p-8 text-center hover:bg-gray-50 transition-colors ${
                dragActive ? 'border-indigo-500 bg-indigo-50' : 'border-gray-300'
              }`}
              onDragEnter={handleDrag}
              onDragLeave={handleDrag}
              onDragOver={handleDrag}
              onDrop={handleDrop}
            >
              <CloudArrowUpIcon className="mx-auto h-12 w-12 text-gray-400" />
              <div className="mt-4">
                <p className="text-lg font-medium text-gray-900">
                  Drop your file here, or{' '}
                  <button
                    type="button"
                    onClick={() => fileInputRef.current?.click()}
                    className="text-indigo-600 hover:text-indigo-500 font-medium"
                  >
                    browse
                  </button>
                </p>
                <p className="text-sm text-gray-500 mt-2">
                  Supports CSV, XLSX, and XLS files up to 100MB
                </p>
              </div>
              <input
                ref={fileInputRef}
                type="file"
                accept=".csv,.xlsx,.xls"
                onChange={handleFileChange}
                className="hidden"
              />
            </div>
          )}

          {/* Selected File Display */}
          {selectedFile && uploadStatus !== 'success' && (
            <div className="border border-gray-200 rounded-lg p-4 mb-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center">
                  {getFileIcon(selectedFile.name)}
                  <div className="ml-4">
                    <p className="text-sm font-medium text-gray-900">{selectedFile.name}</p>
                    <p className="text-sm text-gray-500">{formatFileSize(selectedFile.size)}</p>
                  </div>
                </div>
                <button
                  onClick={() => setSelectedFile(null)}
                  className="text-gray-400 hover:text-gray-600"
                  disabled={uploadStatus === 'uploading'}
                >
                  <XMarkIcon className="h-5 w-5" />
                </button>
              </div>
            </div>
          )}

          {/* Upload Options */}
          {selectedFile && uploadStatus !== 'success' && (
            <div className="space-y-4 mb-6">
              <div>
                <label htmlFor="uploaded-by" className="block text-sm font-medium text-gray-700">
                  Uploaded By (Optional)
                </label>
                <input
                  type="text"
                  id="uploaded-by"
                  value={uploadedBy}
                  onChange={(e) => setUploadedBy(e.target.value)}
                  placeholder="Enter your email or name"
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                  disabled={uploadStatus === 'uploading'}
                />
              </div>

              {/* Sheet Selection for Excel files */}
              {isExcelFile(selectedFile.name) && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Available Sheets
                  </label>
                  
                  {isLoadingSheets ? (
                    <div className="flex items-center justify-center py-4">
                      <ArrowPathIcon className="h-5 w-5 animate-spin text-indigo-600 mr-2" />
                      <span className="text-sm text-gray-600">Loading sheets...</span>
                    </div>
                  ) : availableSheets.length > 0 ? (
                    <div className="border border-gray-200 rounded-md p-3">
                      <div className="flex justify-between items-center mb-3">
                        <span className="text-sm text-gray-600">
                          Select sheets to process ({selectedSheets.length} of {availableSheets.length} selected)
                        </span>
                        <div className="space-x-2">
                          <button
                            type="button"
                            onClick={selectAllSheets}
                            className="text-xs text-indigo-600 hover:text-indigo-800"
                            disabled={uploadStatus === 'uploading'}
                          >
                            Select All
                          </button>
                          <button
                            type="button"
                            onClick={selectNoSheets}
                            className="text-xs text-gray-600 hover:text-gray-800"
                            disabled={uploadStatus === 'uploading'}
                          >
                            Select None
                          </button>
                        </div>
                      </div>
                      <div className="space-y-2 max-h-32 overflow-y-auto">
                        {availableSheets.map((sheetName) => (
                          <label key={sheetName} className="flex items-center">
                            <input
                              type="checkbox"
                              checked={selectedSheets.includes(sheetName)}
                              onChange={() => handleSheetSelection(sheetName)}
                              className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
                              disabled={uploadStatus === 'uploading'}
                            />
                            <span className="ml-2 text-sm text-gray-700">{sheetName}</span>
                          </label>
                        ))}
                      </div>
                      {selectedSheets.length === 0 && (
                        <p className="text-sm text-red-600 mt-2">
                          Please select at least one sheet to process.
                        </p>
                      )}
                    </div>
                  ) : null}
                </div>
              )}
            </div>
          )}

          {/* Upload Status */}
          {uploadMessage && (
            <div className={`rounded-md p-4 mb-4 ${
              uploadStatus === 'success' ? 'bg-green-50 text-green-800' : 
              uploadStatus === 'error' ? 'bg-red-50 text-red-800' : 
              'bg-blue-50 text-blue-800'
            }`}>
              {uploadMessage}
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex justify-end space-x-3">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              disabled={uploadStatus === 'uploading'}
            >
              {uploadStatus === 'success' ? 'Close' : 'Cancel'}
            </button>
            
            {selectedFile && uploadStatus !== 'success' && (
              <button
                type="button"
                onClick={handleUpload}
                disabled={uploadStatus === 'uploading' || !canUpload}
                className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
              >
                {uploadStatus === 'uploading' ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                    Uploading...
                  </>
                ) : (
                  <>
                    <CloudArrowUpIcon className="h-4 w-4 mr-2" />
                    Upload {isExcelFile(selectedFile.name) && selectedSheets.length > 1 
                      ? `${selectedSheets.length} Sheets` 
                      : 'File'}
                  </>
                )}
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
} 
