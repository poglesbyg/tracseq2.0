import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import {
  XMarkIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  FunnelIcon,
  DocumentTextIcon,
  TableCellsIcon,
  MagnifyingGlassIcon,
} from '@heroicons/react/24/outline';

interface SpreadsheetDataViewerProps {
  dataset: {
    id: string;
    original_filename: string;
    file_type: string;
    total_rows: number;
    total_columns: number;
    column_headers: string[];
    created_at: string;
    uploaded_by?: string;
    sheet_name?: string;
  };
  onClose: () => void;
}

interface DataRecord {
  id: string;
  row_number: number;
  row_data: Record<string, any>;
  created_at: string;
}

interface DataResponse {
  records: DataRecord[];
  total_count: number;
}

export default function SpreadsheetDataViewer({ dataset, onClose }: SpreadsheetDataViewerProps) {
  const [currentPage, setCurrentPage] = useState(0);
  const [searchTerm, setSearchTerm] = useState('');
  const [columnFilters, setColumnFilters] = useState<Record<string, string>>({});
  const [showFilters, setShowFilters] = useState(false);
  const rowsPerPage = 50;

  // Build search query
  const buildSearchQuery = () => {
    const params = new URLSearchParams();
    params.append('dataset_id', dataset.id);
    params.append('limit', rowsPerPage.toString());
    params.append('offset', (currentPage * rowsPerPage).toString());

    if (searchTerm.trim()) {
      params.append('search_term', searchTerm.trim());
    }

    // Add column filters
    Object.entries(columnFilters).forEach(([column, value]) => {
      if (value.trim()) {
        params.append(`filter_${column}`, value.trim());
      }
    });

    return params.toString();
  };

  // Fetch data
  const { data: dataResponse, isLoading, error } = useQuery<DataResponse>({
    queryKey: ['spreadsheet-data', dataset.id, currentPage, searchTerm, columnFilters],
    queryFn: async () => {
      const query = buildSearchQuery();
      const response = await axios.get(`/api/spreadsheets/search?${query}`);
      return response.data.data;
    },
  });

  const totalPages = dataResponse ? Math.ceil(dataResponse.total_count / rowsPerPage) : 0;
  const startRow = currentPage * rowsPerPage + 1;
  const endRow = Math.min((currentPage + 1) * rowsPerPage, dataResponse?.total_count || 0);

  const goToNextPage = () => {
    if (currentPage < totalPages - 1) {
      setCurrentPage(currentPage + 1);
    }
  };

  const goToPrevPage = () => {
    if (currentPage > 0) {
      setCurrentPage(currentPage - 1);
    }
  };

  const handleColumnFilterChange = (column: string, value: string) => {
    setColumnFilters(prev => ({
      ...prev,
      [column]: value
    }));
    setCurrentPage(0); // Reset to first page when filtering
  };

  const clearFilters = () => {
    setColumnFilters({});
    setSearchTerm('');
    setCurrentPage(0);
  };

  const hasActiveFilters = searchTerm.trim() || Object.values(columnFilters).some(v => v.trim());

  const getFileIcon = (fileType: string) => {
    switch (fileType.toLowerCase()) {
      case 'csv':
        return <DocumentTextIcon className="h-6 w-6 text-green-500" />;
      case 'xlsx':
      case 'xls':
        return <TableCellsIcon className="h-6 w-6 text-blue-500" />;
      default:
        return <DocumentTextIcon className="h-6 w-6 text-gray-500" />;
    }
  };



  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50 flex items-start justify-center p-4 pt-4">
      <div className="bg-white rounded-lg shadow-xl max-w-7xl w-full max-h-[95vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-gray-200 bg-gray-50">
          <div className="flex items-center">
            {getFileIcon(dataset.file_type)}
            <div className="ml-4">
              <h2 className="text-xl font-semibold text-gray-900">{dataset.original_filename}</h2>
              <div className="flex items-center space-x-4 text-sm text-gray-500 mt-1">
                <span className="flex items-center">
                  <span className="uppercase font-mono text-xs bg-gray-200 px-2 py-1 rounded mr-2">
                    {dataset.file_type}
                  </span>
                  {dataset.sheet_name && `Sheet: ${dataset.sheet_name}`}
                </span>
                <span>{dataset.total_rows.toLocaleString()} rows</span>
                <span>{dataset.total_columns} columns</span>
                <span>{new Date(dataset.created_at).toLocaleDateString()}</span>
                {dataset.uploaded_by && <span>by {dataset.uploaded_by}</span>}
              </div>
            </div>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <XMarkIcon className="h-6 w-6" />
          </button>
        </div>

        {/* Search and Filter Controls */}
        <div className="p-4 border-b border-gray-200 bg-gray-50">
          <div className="flex space-x-4 items-center">
            {/* Search */}
            <div className="flex-1 relative">
              <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
              </div>
              <input
                type="text"
                value={searchTerm}
                onChange={(e) => {
                  setSearchTerm(e.target.value);
                  setCurrentPage(0);
                }}
                placeholder="Search within this dataset..."
                className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
              />
            </div>

            {/* Filter Toggle */}
            <button
              type="button"
              onClick={() => setShowFilters(!showFilters)}
              className={`inline-flex items-center px-3 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium ${
                showFilters || Object.keys(columnFilters).length > 0
                  ? 'text-indigo-700 bg-indigo-50 border-indigo-300'
                  : 'text-gray-700 bg-white hover:bg-gray-50'
              } focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500`}
            >
              <FunnelIcon className="h-4 w-4 mr-1" />
              Filters
              {Object.keys(columnFilters).length > 0 && (
                <span className="ml-1 inline-flex items-center px-1.5 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800">
                  {Object.keys(columnFilters).length}
                </span>
              )}
            </button>

            {hasActiveFilters && (
              <button
                type="button"
                onClick={clearFilters}
                className="text-sm text-indigo-600 hover:text-indigo-800"
              >
                Clear all
              </button>
            )}
          </div>

          {/* Column Filters */}
          {showFilters && (
            <div className="mt-4 bg-white border border-gray-200 rounded-md p-4">
              <h3 className="text-sm font-medium text-gray-900 mb-3">Filter by columns</h3>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                {dataset.column_headers.slice(0, 8).map((header) => (
                  <div key={header}>
                    <label htmlFor={`filter-${header}`} className="block text-xs font-medium text-gray-700 mb-1">
                      {header}
                    </label>
                    <input
                      type="text"
                      id={`filter-${header}`}
                      value={columnFilters[header] || ''}
                      onChange={(e) => handleColumnFilterChange(header, e.target.value)}
                      placeholder={`Filter ${header}...`}
                      className="block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 text-sm"
                    />
                  </div>
                ))}
              </div>
              {dataset.column_headers.length > 8 && (
                <p className="text-xs text-gray-500 mt-2">
                  Showing first 8 columns. Use search above to filter across all columns.
                </p>
              )}
            </div>
          )}
        </div>

        {/* Data Display */}
        <div className="flex-1 overflow-auto">
          {error && (
            <div className="p-6 text-center">
              <div className="text-red-600">
                <h3 className="text-lg font-medium">Error Loading Data</h3>
                <p className="mt-2">Failed to load spreadsheet data. Please try again.</p>
              </div>
            </div>
          )}

          {isLoading && (
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
              <span className="ml-3 text-gray-600">Loading data...</span>
            </div>
          )}

          {dataResponse && (
            <>
              {/* Results Info */}
              <div className="px-6 py-3 bg-gray-50 border-b border-gray-200">
                <div className="flex justify-between items-center text-sm text-gray-700">
                  <span>
                    Showing {startRow}-{endRow} of{' '}
                    <span className="font-medium">{dataResponse.total_count.toLocaleString()}</span> rows
                    {hasActiveFilters && ' (filtered)'}
                  </span>
                  {totalPages > 1 && (
                    <span>Page {currentPage + 1} of {totalPages}</span>
                  )}
                </div>
              </div>

              {/* Data Table */}
              {dataResponse.records.length > 0 ? (
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-300">
                    <thead className="bg-gray-50 sticky top-0">
                      <tr>
                        <th className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-r border-gray-200">
                          Row #
                        </th>
                        {dataset.column_headers.map((header) => (
                          <th
                            key={header}
                            className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-r border-gray-200 min-w-32"
                          >
                            <div className="truncate" title={header}>
                              {header}
                            </div>
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {dataResponse.records.map((record, index) => (
                        <tr key={record.id} className={index % 2 === 0 ? 'bg-white' : 'bg-gray-50'}>
                          <td className="px-3 py-2 whitespace-nowrap text-sm font-medium text-gray-900 border-r border-gray-200">
                            {record.row_number}
                          </td>
                          {dataset.column_headers.map((header) => (
                            <td
                              key={header}
                              className="px-3 py-2 text-sm text-gray-900 border-r border-gray-200 max-w-48"
                            >
                              <div className="truncate" title={String(record.row_data[header] || '')}>
                                {record.row_data[header] || '-'}
                              </div>
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : (
                <div className="p-8 text-center text-gray-500">
                  <DocumentTextIcon className="mx-auto h-12 w-12 text-gray-400" />
                  <h3 className="mt-2 text-sm font-medium text-gray-900">No data found</h3>
                  <p className="mt-1 text-sm text-gray-500">
                    {hasActiveFilters
                      ? 'No rows match your current filters. Try adjusting your search criteria.'
                      : 'This dataset appears to be empty.'}
                  </p>
                </div>
              )}
            </>
          )}
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-between border-t border-gray-200 bg-white px-6 py-3">
            <div className="flex flex-1 justify-between sm:hidden">
              <button
                onClick={goToPrevPage}
                disabled={currentPage === 0}
                className="relative inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Previous
              </button>
              <button
                onClick={goToNextPage}
                disabled={currentPage === totalPages - 1}
                className="relative ml-3 inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Next
              </button>
            </div>
            <div className="hidden sm:flex sm:flex-1 sm:items-center sm:justify-between">
              <div>
                <p className="text-sm text-gray-700">
                  Showing <span className="font-medium">{startRow}</span> to{' '}
                  <span className="font-medium">{endRow}</span> of{' '}
                  <span className="font-medium">{dataResponse?.total_count.toLocaleString()}</span> results
                </p>
              </div>
              <div>
                <nav className="isolate inline-flex -space-x-px rounded-md shadow-sm" aria-label="Pagination">
                  <button
                    onClick={goToPrevPage}
                    disabled={currentPage === 0}
                    className="relative inline-flex items-center rounded-l-md px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <ChevronLeftIcon className="h-5 w-5" />
                  </button>
                  <span className="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-900 ring-1 ring-inset ring-gray-300">
                    Page {currentPage + 1} of {totalPages}
                  </span>
                  <button
                    onClick={goToNextPage}
                    disabled={currentPage === totalPages - 1}
                    className="relative inline-flex items-center rounded-r-md px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <ChevronRightIcon className="h-5 w-5" />
                  </button>
                </nav>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
} 
