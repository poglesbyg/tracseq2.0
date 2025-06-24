import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import axios from 'axios';
import {
  DocumentTextIcon,
  MagnifyingGlassIcon,
  TrashIcon,
  CloudArrowUpIcon,
  TableCellsIcon,
  CalendarDaysIcon,
  UserIcon,
  EyeIcon,
} from '@heroicons/react/24/outline';
import FileUploadModal from '../components/FileUploadModal';
import SpreadsheetSearchModal from '../components/SpreadsheetSearchModal';
import SpreadsheetDataViewer from '../components/SpreadsheetDataViewer';

  interface SpreadsheetDataset {
    id: string;
    filename: string;
    original_filename: string;
    file_type: string;
    file_size: number;
    sheet_name?: string;
    total_rows: number;
    total_columns: number;
    column_headers: string[];
    upload_status: 'processing' | 'completed' | 'failed';
    error_message?: string;
    uploaded_by?: string;
    created_at: string;
    updated_at: string;
    metadata: Record<string, unknown>;
  }

export default function Spreadsheets() {
  const [showUploadModal, setShowUploadModal] = useState(false);
  const [showSearchModal, setShowSearchModal] = useState(false);
  const [viewingDataset, setViewingDataset] = useState<SpreadsheetDataset | null>(null);
  const queryClient = useQueryClient();

  // Fetch datasets
  const { data: datasets, isLoading, error } = useQuery<SpreadsheetDataset[]>({
    queryKey: ['spreadsheet-datasets'],
    queryFn: async () => {
      const response = await axios.get('/api/spreadsheets/datasets');
      return response.data.data || [];
    },
  });

  // Delete dataset mutation
  const deleteDatasetMutation = useMutation({
    mutationFn: async (datasetId: string) => {
      await axios.delete(`/api/spreadsheets/datasets/${datasetId}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['spreadsheet-datasets'] });
    },
  });

  const handleDelete = async (dataset: SpreadsheetDataset) => {
          if (window.confirm(`Are you sure you want to delete "${dataset.original_filename}"? This action cannot be undone.`)) {
      try {
        await deleteDatasetMutation.mutateAsync(dataset.id);
      } catch (error) {
        console.error('Failed to delete dataset:', error);
        alert('Failed to delete dataset. Please try again.');
      }
    }
  };

  const getStatusColor = (status: string) => {
    switch (status?.toLowerCase()) {
      case 'completed':
        return 'bg-green-100 text-green-800';
      case 'processing':
        return 'bg-yellow-100 text-yellow-800';
      case 'failed':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getFileTypeIcon = (fileType: string) => {
    switch (fileType.toLowerCase()) {
      case 'csv':
        return <DocumentTextIcon className="h-5 w-5 text-green-500" />;
      case 'xlsx':
      case 'xls':
        return <TableCellsIcon className="h-5 w-5 text-blue-500" />;
      default:
        return <DocumentTextIcon className="h-5 w-5 text-gray-500" />;
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  if (error) {
    return (
      <div className="px-4 sm:px-6 lg:px-8">
        <div className="bg-red-50 border border-red-200 rounded-md p-4">
          <div className="text-red-800">
            <h3 className="text-lg font-medium">Error Loading Datasets</h3>
            <p className="mt-2">Failed to load spreadsheet datasets. Please try refreshing the page.</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      {/* Header */}
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-xl font-semibold text-gray-900">Spreadsheet Data</h1>
          <p className="mt-2 text-sm text-gray-700">
            Upload and manage CSV/Excel files containing laboratory data. Search across all uploaded datasets.
          </p>
        </div>
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none space-x-3">
          <button
            type="button"
            onClick={() => setShowSearchModal(true)}
            className="inline-flex items-center justify-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
          >
            <MagnifyingGlassIcon className="h-4 w-4 mr-2" />
            Search Data
          </button>
          <button
            type="button"
            onClick={() => setShowUploadModal(true)}
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
          >
            <CloudArrowUpIcon className="h-4 w-4 mr-2" />
            Upload File
          </button>
        </div>
      </div>

      {/* Statistics Cards */}
      {datasets && datasets.length > 0 && (
        <div className="mt-8 grid grid-cols-1 gap-5 sm:grid-cols-3">
          <div className="bg-white overflow-hidden shadow rounded-lg">
            <div className="p-5">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <DocumentTextIcon className="h-6 w-6 text-gray-400" />
                </div>
                <div className="ml-5 w-0 flex-1">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 truncate">Total Datasets</dt>
                    <dd className="text-lg font-medium text-gray-900">{datasets.length}</dd>
                  </dl>
                </div>
              </div>
            </div>
          </div>
          <div className="bg-white overflow-hidden shadow rounded-lg">
            <div className="p-5">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <TableCellsIcon className="h-6 w-6 text-gray-400" />
                </div>
                <div className="ml-5 w-0 flex-1">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 truncate">Total Records</dt>
                    <dd className="text-lg font-medium text-gray-900">
                      {datasets.reduce((sum, dataset) => sum + dataset.total_rows, 0).toLocaleString()}
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
                  <CloudArrowUpIcon className="h-6 w-6 text-gray-400" />
                </div>
                <div className="ml-5 w-0 flex-1">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 truncate">Completed Uploads</dt>
                    <dd className="text-lg font-medium text-gray-900">
                      {datasets.filter(d => d.upload_status?.toLowerCase() === 'completed').length}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Datasets Table */}
      <div className="mt-8 flex flex-col">
        <div className="-my-2 -mx-4 overflow-x-auto sm:-mx-6 lg:-mx-8">
          <div className="inline-block min-w-full py-2 align-middle md:px-6 lg:px-8">
            <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
              <table className="min-w-full divide-y divide-gray-300">
                <thead className="bg-gray-50">
                  <tr>
                    <th scope="col" className="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-6">
                      File
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Type
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Size
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Rows
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Status
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Uploaded
                    </th>
                    <th scope="col" className="relative py-3.5 pl-3 pr-4 sm:pr-6">
                      <span className="sr-only">Actions</span>
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-200 bg-white">
                  {isLoading ? (
                    <tr>
                      <td colSpan={7} className="px-3 py-8 text-sm text-gray-500 text-center">
                        <div className="flex items-center justify-center">
                          <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-600"></div>
                          <span className="ml-2">Loading datasets...</span>
                        </div>
                      </td>
                    </tr>
                  ) : !datasets || datasets.length === 0 ? (
                    <tr>
                      <td colSpan={7} className="px-3 py-8 text-sm text-gray-500 text-center">
                        <div className="text-center">
                          <DocumentTextIcon className="mx-auto h-12 w-12 text-gray-400" />
                          <h3 className="mt-2 text-sm font-medium text-gray-900">No datasets</h3>
                          <p className="mt-1 text-sm text-gray-500">
                            Get started by uploading a CSV or Excel file.
                          </p>
                          <div className="mt-6">
                            <button
                              type="button"
                              onClick={() => setShowUploadModal(true)}
                              className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                            >
                              <CloudArrowUpIcon className="h-4 w-4 mr-2" />
                              Upload your first file
                            </button>
                          </div>
                        </div>
                      </td>
                    </tr>
                  ) : (
                    datasets.map((dataset) => (
                      <tr 
                        key={dataset.id}
                        className={`${
                          dataset.upload_status?.toLowerCase() === 'completed' 
                            ? 'hover:bg-gray-50 cursor-pointer' 
                            : ''
                        }`}
                        title={
                          dataset.upload_status?.toLowerCase() === 'completed'
                            ? 'Click to view spreadsheet data'
                            : dataset.upload_status?.toLowerCase() === 'failed'
                            ? 'Upload failed - cannot view data'
                            : 'Processing - data not yet available'
                        }
                        onClick={() => {
                          if (dataset.upload_status?.toLowerCase() === 'completed') {
                            setViewingDataset(dataset);
                          }
                        }}
                      >
                        <td className="whitespace-nowrap py-4 pl-4 pr-3 sm:pl-6">
                          <div className="flex items-center">
                            <div className="flex-shrink-0">
                              {getFileTypeIcon(dataset.file_type)}
                            </div>
                            <div className="ml-4">
                              <div className="text-sm font-medium text-gray-900">
                                {dataset.original_filename}
                              </div>
                              {dataset.sheet_name && (
                                <div className="text-sm text-gray-500">
                                  Sheet: {dataset.sheet_name}
                                </div>
                              )}
                              {dataset.uploaded_by && (
                                <div className="text-xs text-gray-400 flex items-center mt-1">
                                  <UserIcon className="h-3 w-3 mr-1" />
                                  {dataset.uploaded_by}
                                </div>
                              )}
                            </div>
                          </div>
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          <span className="uppercase font-mono text-xs bg-gray-100 px-2 py-1 rounded">
                            {dataset.file_type}
                          </span>
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          {formatFileSize(dataset.file_size)}
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          <div>
                            <div className="font-medium">{dataset.total_rows.toLocaleString()}</div>
                            <div className="text-xs text-gray-400">{dataset.total_columns} cols</div>
                          </div>
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm">
                          <span className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${getStatusColor(dataset.upload_status)}`}>
                            {dataset.upload_status}
                          </span>
                          {dataset.error_message && (
                            <div className="text-xs text-red-600 mt-1" title={dataset.error_message}>
                              Error occurred
                            </div>
                          )}
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          <div className="flex items-center">
                            <CalendarDaysIcon className="h-4 w-4 mr-1 text-gray-400" />
                            {new Date(dataset.created_at).toLocaleDateString()}
                          </div>
                        </td>
                        <td className="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-6">
                          <div className="flex items-center space-x-2">
                            {dataset.upload_status?.toLowerCase() === 'completed' && (
                              <button
                                onClick={(e) => {
                                  e.stopPropagation();
                                  setViewingDataset(dataset);
                                }}
                                className="text-indigo-600 hover:text-indigo-900 flex items-center"
                              >
                                <EyeIcon className="h-4 w-4 mr-1" />
                                View Data
                              </button>
                            )}
                            <button
                              onClick={(e) => {
                                e.stopPropagation();
                                handleDelete(dataset);
                              }}
                              className="text-red-600 hover:text-red-900"
                              disabled={deleteDatasetMutation.isPending}
                              title="Delete dataset"
                            >
                              <TrashIcon className="h-4 w-4" />
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>

      {/* Modals */}
      {showUploadModal && (
        <FileUploadModal
          onClose={() => setShowUploadModal(false)}
          onSuccess={() => {
            setShowUploadModal(false);
            queryClient.invalidateQueries({ queryKey: ['spreadsheet-datasets'] });
          }}
        />
      )}

      {showSearchModal && (
        <SpreadsheetSearchModal
          onClose={() => setShowSearchModal(false)}
        />
      )}

      {viewingDataset && (
        <SpreadsheetDataViewer
          dataset={viewingDataset}
          onClose={() => setViewingDataset(null)}
        />
      )}
    </div>
  );
} 
