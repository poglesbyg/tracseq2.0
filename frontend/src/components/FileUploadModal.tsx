import { useState, useRef, useCallback } from 'react';
import { useMutation } from '@tanstack/react-query';
import axios from 'axios';
import {
  CloudArrowUpIcon,
  DocumentTextIcon,
  TableCellsIcon,
  XMarkIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
} from '@heroicons/react/24/outline';

interface FileUploadModalProps {
  onClose: () => void;
  onSuccess: () => void;
}

interface UploadResponse {
  success: boolean;
  dataset?: any;
  message: string;
}

export default function FileUploadModal({ onClose, onSuccess }: FileUploadModalProps) {
  const [dragActive, setDragActive] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [uploadedBy, setUploadedBy] = useState('');
  const [sheetName, setSheetName] = useState('');
  const [uploadStatus, setUploadStatus] = useState<'idle' | 'uploading' | 'success' | 'error'>('idle');
  const [uploadMessage, setUploadMessage] = useState('');
  const fileInputRef = useRef<HTMLInputElement>(null);

  const uploadMutation = useMutation({
    mutationFn: async (formData: FormData) => {
      const response = await axios.post<UploadResponse>('/api/spreadsheets/upload', formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      });
      return response.data;
    },
    onSuccess: (data) => {
      setUploadStatus('success');
      setUploadMessage(data.message);
      setTimeout(() => {
        onSuccess();
      }, 2000);
    },
    onError: (error: any) => {
      setUploadStatus('error');
      setUploadMessage(error.response?.data?.message || 'Upload failed. Please try again.');
    },
  });

  const handleFiles = useCallback((files: FileList) => {
    const file = files[0];
    if (file && isValidFileType(file)) {
      setSelectedFile(file);
      setUploadStatus('idle');
      setUploadMessage('');
    } else {
      alert('Please select a CSV, XLSX, or XLS file.');
    }
  }, []);

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

  const handleUpload = async () => {
    if (!selectedFile) return;

    const formData = new FormData();
    formData.append('file', selectedFile);
    
    if (uploadedBy.trim()) {
      formData.append('uploaded_by', uploadedBy.trim());
    }
    
    if (sheetName.trim()) {
      formData.append('sheet_name', sheetName.trim());
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

  const isExcelFile = (filename: string) => {
    const extension = filename.split('.').pop()?.toLowerCase();
    return extension === 'xlsx' || extension === 'xls';
  };

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

              {isExcelFile(selectedFile.name) && (
                <div>
                  <label htmlFor="sheet-name" className="block text-sm font-medium text-gray-700">
                    Sheet Name (Optional)
                  </label>
                  <input
                    type="text"
                    id="sheet-name"
                    value={sheetName}
                    onChange={(e) => setSheetName(e.target.value)}
                    placeholder="Leave blank to use first sheet"
                    className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                    disabled={uploadStatus === 'uploading'}
                  />
                  <p className="mt-1 text-sm text-gray-500">
                    Specify which Excel sheet to process. If left blank, the first sheet will be used.
                  </p>
                </div>
              )}
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
                disabled={uploadStatus === 'uploading'}
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
                    Upload File
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
