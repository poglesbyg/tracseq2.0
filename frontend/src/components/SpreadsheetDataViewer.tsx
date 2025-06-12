import { useState, useMemo, useCallback } from 'react';
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
  ArrowsPointingOutIcon,
  ArrowDownTrayIcon,
  ChartBarIcon,
  CalendarIcon,
  HashtagIcon,
  ArrowUpIcon,
  ArrowDownIcon,
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

// Data type detection
type DataType = 'text' | 'number' | 'date' | 'boolean' | 'email' | 'url';

interface ColumnStats {
  type: DataType;
  nonEmptyCount: number;
  uniqueCount: number;
  min?: string | number;
  max?: string | number;
  avg?: number;
  nullCount: number;
}

interface SortConfig {
  column: string;
  direction: 'asc' | 'desc';
}

export default function SpreadsheetDataViewer({ dataset, onClose }: SpreadsheetDataViewerProps) {
  const [currentPage, setCurrentPage] = useState(0);
  const [searchTerm, setSearchTerm] = useState('');
  const [columnFilters, setColumnFilters] = useState<Record<string, string>>({});
  const [showFilters, setShowFilters] = useState(false);
  const [showStats, setShowStats] = useState(false);
  const [fullScreen, setFullScreen] = useState(false);
  const [sortConfig, setSortConfig] = useState<SortConfig | null>(null);
  const [selectedRows, setSelectedRows] = useState<Set<string>>(new Set());
  const [rowsPerPage, setRowsPerPage] = useState(50);

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
    queryKey: ['spreadsheet-data', dataset.id, currentPage, searchTerm, columnFilters, rowsPerPage],
    queryFn: async () => {
      const query = buildSearchQuery();
      const response = await axios.get(`/api/spreadsheets/search?${query}`);
      return response.data.data;
    },
  });

  // Extract column headers from data if not available in dataset metadata
  const getColumnHeaders = () => {
    // Check if dataset has valid column headers
    if (dataset.column_headers && Array.isArray(dataset.column_headers) && dataset.column_headers.length > 0) {
      return dataset.column_headers;
    }
    
    // Extract from first row of data if available
    if (dataResponse && dataResponse.records && dataResponse.records.length > 0) {
      const firstRecord = dataResponse.records[0];
      if (firstRecord.row_data && typeof firstRecord.row_data === 'object') {
        return Object.keys(firstRecord.row_data);
      }
    }
    
    return [];
  };

  const columnHeaders = getColumnHeaders();

  // Data type detection utilities
  const detectDataType = useCallback((value: any): DataType => {
    if (value === null || value === undefined || value === '') return 'text';
    
    const strValue = String(value).trim();
    
    // Email detection
    if (/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(strValue)) return 'email';
    
    // URL detection
    if (/^https?:\/\//.test(strValue)) return 'url';
    
    // Number detection
    if (!isNaN(Number(strValue)) && strValue !== '') return 'number';
    
    // Date detection (various formats)
    if (Date.parse(strValue) && /\d/.test(strValue)) return 'date';
    
    // Boolean detection
    if (['true', 'false', 'yes', 'no', '1', '0'].includes(strValue.toLowerCase())) return 'boolean';
    
    return 'text';
  }, []);

  // Calculate column statistics
  const columnStats = useMemo((): Record<string, ColumnStats> => {
    if (!dataResponse || !dataResponse.records || columnHeaders.length === 0) return {};
    
    const stats: Record<string, ColumnStats> = {};
    
    columnHeaders.forEach(header => {
      const values = dataResponse.records.map(record => record.row_data[header]);
      const nonEmptyValues = values.filter(v => v !== null && v !== undefined && v !== '');
      const uniqueValues = new Set(nonEmptyValues);
      
      // Detect predominant data type
      const typeCounts: Record<DataType, number> = {
        text: 0, number: 0, date: 0, boolean: 0, email: 0, url: 0
      };
      
      nonEmptyValues.forEach(value => {
        const type = detectDataType(value);
        typeCounts[type]++;
      });
      
      const predominantType = Object.entries(typeCounts).reduce((a, b) => 
        typeCounts[a[0] as DataType] > typeCounts[b[0] as DataType] ? a : b
      )[0] as DataType;
      
      // Calculate min, max, avg for numeric columns
      let min, max, avg;
      if (predominantType === 'number') {
        const numericValues = nonEmptyValues
          .map(v => Number(v))
          .filter(v => !isNaN(v));
        
        if (numericValues.length > 0) {
          min = Math.min(...numericValues);
          max = Math.max(...numericValues);
          avg = numericValues.reduce((sum, val) => sum + val, 0) / numericValues.length;
        }
      }
      
      stats[header] = {
        type: predominantType,
        nonEmptyCount: nonEmptyValues.length,
        uniqueCount: uniqueValues.size,
        nullCount: values.length - nonEmptyValues.length,
        min,
        max,
        avg
      };
    });
    
    return stats;
  }, [dataResponse, columnHeaders, detectDataType]);

  // Sorting logic
  const sortedRecords = useMemo(() => {
    if (!dataResponse || !sortConfig) return dataResponse?.records || [];
    
    return [...dataResponse.records].sort((a, b) => {
      const aVal = a.row_data[sortConfig.column];
      const bVal = b.row_data[sortConfig.column];
      
      // Handle null/undefined values
      if (aVal === null || aVal === undefined || aVal === '') {
        if (bVal === null || bVal === undefined || bVal === '') return 0;
        return sortConfig.direction === 'asc' ? 1 : -1;
      }
      if (bVal === null || bVal === undefined || bVal === '') {
        return sortConfig.direction === 'asc' ? -1 : 1;
      }
      
      // Type-specific sorting
      const columnType = columnStats[sortConfig.column]?.type || 'text';
      let comparison = 0;
      
      switch (columnType) {
        case 'number':
          comparison = Number(aVal) - Number(bVal);
          break;
        case 'date':
          comparison = new Date(aVal).getTime() - new Date(bVal).getTime();
          break;
        default:
          comparison = String(aVal).localeCompare(String(bVal));
      }
      
      return sortConfig.direction === 'asc' ? comparison : -comparison;
    });
  }, [dataResponse, sortConfig, columnStats]);

  const totalPages = dataResponse ? Math.ceil(dataResponse.total_count / rowsPerPage) : 0;
  const startRow = currentPage * rowsPerPage + 1;
  const endRow = Math.min(startRow + (dataResponse?.records.length || 0) - 1, dataResponse?.total_count || 0);

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

  // Utility functions
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

  const getDataTypeIcon = (type: DataType) => {
    switch (type) {
      case 'number':
        return <HashtagIcon className="h-3 w-3 text-blue-500" />;
      case 'date':
        return <CalendarIcon className="h-3 w-3 text-green-500" />;
      case 'email':
        return <span className="text-purple-500 text-xs">@</span>;
      case 'url':
        return <span className="text-blue-500 text-xs">🔗</span>;
      case 'boolean':
        return <span className="text-orange-500 text-xs">✓</span>;
      default:
        return <DocumentTextIcon className="h-3 w-3 text-gray-500" />;
    }
  };

  const handleSort = (column: string) => {
    setSortConfig(prev => {
      if (prev?.column === column) {
        return prev.direction === 'asc' 
          ? { column, direction: 'desc' }
          : null; // Remove sort on third click
      }
      return { column, direction: 'asc' };
    });
  };

  const handleRowSelect = (rowId: string) => {
    setSelectedRows(prev => {
      const newSet = new Set(prev);
      if (newSet.has(rowId)) {
        newSet.delete(rowId);
      } else {
        newSet.add(rowId);
      }
      return newSet;
    });
  };

  const handleSelectAll = () => {
    if (!dataResponse) return;
    
    if (selectedRows.size === dataResponse.records.length) {
      setSelectedRows(new Set());
    } else {
      setSelectedRows(new Set(dataResponse.records.map(r => r.id)));
    }
  };

  const exportData = async (format: 'csv' | 'json') => {
    if (!dataResponse || !columnHeaders.length) return;
    
    const records = sortedRecords.slice(currentPage * rowsPerPage, (currentPage + 1) * rowsPerPage);
    
    if (format === 'csv') {
      const csvContent = [
        columnHeaders.join(','),
        ...records.map(record => 
          columnHeaders.map(header => {
            const value = record.row_data[header] || '';
            const stringValue = String(value);
            // Escape quotes and wrap in quotes if contains comma
            return stringValue.includes(',') || stringValue.includes('"') 
              ? `"${stringValue.replace(/"/g, '""')}"` 
              : stringValue;
          }).join(',')
        )
      ].join('\n');
      
      const blob = new Blob([csvContent], { type: 'text/csv' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${dataset.original_filename}_page_${currentPage + 1}.csv`;
      a.click();
      URL.revokeObjectURL(url);
    } else if (format === 'json') {
      const jsonData = records.map(record => record.row_data);
      const blob = new Blob([JSON.stringify(jsonData, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${dataset.original_filename}_page_${currentPage + 1}.json`;
      a.click();
      URL.revokeObjectURL(url);
    }
  };

  const formatCellValue = (value: any, type: DataType) => {
    if (value === null || value === undefined || value === '') {
      return <span className="text-gray-400 italic">—</span>;
    }
    
    const stringValue = String(value);
    
    switch (type) {
      case 'number':
        const num = Number(value);
        return isNaN(num) ? stringValue : num.toLocaleString();
      case 'date':
        try {
          const date = new Date(value);
          return isNaN(date.getTime()) ? stringValue : date.toLocaleDateString();
        } catch {
          return stringValue;
        }
      case 'email':
        return <a href={`mailto:${stringValue}`} className="text-blue-600 hover:underline">{stringValue}</a>;
      case 'url':
        return <a href={stringValue} target="_blank" rel="noopener noreferrer" className="text-blue-600 hover:underline">{stringValue}</a>;
      case 'boolean':
        const lowerVal = stringValue.toLowerCase();
        const isTrue = ['true', 'yes', '1'].includes(lowerVal);
        return (
          <span className={`px-2 py-1 rounded-full text-xs ${isTrue ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
            {isTrue ? 'Yes' : 'No'}
          </span>
        );
      default:
        return stringValue;
    }
  };

  return (
    <div className={`fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50 flex items-start justify-center ${fullScreen ? 'p-0' : 'p-4 pt-4'}`}>
      <div className={`bg-white shadow-xl w-full overflow-hidden flex flex-col ${
        fullScreen 
          ? 'h-full max-w-none' 
          : 'rounded-lg max-w-7xl max-h-[95vh]'
      }`}>
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-gray-200 bg-gray-50">
          <div className="flex items-center flex-1">
            {getFileIcon(dataset.file_type)}
            <div className="ml-4 flex-1">
              <h2 className="text-xl font-semibold text-gray-900">{dataset.original_filename}</h2>
              <div className="flex items-center space-x-4 text-sm text-gray-500 mt-1">
                <span className="flex items-center">
                  <span className="uppercase font-mono text-xs bg-gray-200 px-2 py-1 rounded mr-2">
                    {dataset.file_type}
                  </span>
                  {dataset.sheet_name && `Sheet: ${dataset.sheet_name}`}
                </span>
                <span>{dataResponse?.total_count.toLocaleString() || dataset.total_rows.toLocaleString()} rows</span>
                <span>{columnHeaders.length || dataset.total_columns} columns</span>
                <span>{new Date(dataset.created_at).toLocaleDateString()}</span>
                {dataset.uploaded_by && <span>by {dataset.uploaded_by}</span>}
                {selectedRows.size > 0 && (
                  <span className="bg-blue-100 text-blue-800 px-2 py-1 rounded-full text-xs font-medium">
                    {selectedRows.size} selected
                  </span>
                )}
              </div>
            </div>
          </div>
          
          {/* Header Actions */}
          <div className="flex items-center space-x-2 ml-4">
            <button
              onClick={() => setShowStats(!showStats)}
              className={`p-2 rounded-md transition-colors ${
                showStats ? 'bg-blue-100 text-blue-700' : 'text-gray-400 hover:text-gray-600'
              }`}
              title="Toggle Statistics"
            >
              <ChartBarIcon className="h-5 w-5" />
            </button>
            
            <div className="relative group">
              <button
                className="p-2 rounded-md text-gray-400 hover:text-gray-600 transition-colors"
                title="Export Data"
              >
                <ArrowDownTrayIcon className="h-5 w-5" />
              </button>
              <div className="absolute right-0 top-full mt-1 bg-white border border-gray-200 rounded-md shadow-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-10">
                <div className="p-2 min-w-32">
                  <button
                    onClick={() => exportData('csv')}
                    className="block w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded"
                  >
                    Export CSV
                  </button>
                  <button
                    onClick={() => exportData('json')}
                    className="block w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded"
                  >
                    Export JSON
                  </button>
                </div>
              </div>
            </div>
            
            <button
              onClick={() => setFullScreen(!fullScreen)}
              className="p-2 rounded-md text-gray-400 hover:text-gray-600 transition-colors"
              title={fullScreen ? "Exit Fullscreen" : "Enter Fullscreen"}
            >
              <ArrowsPointingOutIcon className="h-5 w-5" />
            </button>
            
            <button
              onClick={onClose}
              className="p-2 rounded-md text-gray-400 hover:text-gray-600 transition-colors"
              title="Close"
            >
              <XMarkIcon className="h-5 w-5" />
            </button>
          </div>
        </div>

        {/* Statistics Panel */}
        {showStats && columnHeaders.length > 0 && (
          <div className="border-b border-gray-200 bg-gray-50 p-4">
            <h3 className="text-sm font-medium text-gray-900 mb-3">Column Statistics</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              {columnHeaders.slice(0, 8).map(header => {
                const stats = columnStats[header];
                if (!stats) return null;
                
                return (
                  <div key={header} className="bg-white border border-gray-200 rounded-lg p-3">
                    <div className="flex items-center justify-between mb-2">
                      <h4 className="text-xs font-medium text-gray-900 truncate" title={header}>
                        {header}
                      </h4>
                      <div className="flex items-center">
                        {getDataTypeIcon(stats.type)}
                        <span className="ml-1 text-xs text-gray-500 capitalize">{stats.type}</span>
                      </div>
                    </div>
                    <div className="space-y-1 text-xs text-gray-600">
                      <div className="flex justify-between">
                        <span>Non-empty:</span>
                        <span className="font-medium">{stats.nonEmptyCount.toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Unique:</span>
                        <span className="font-medium">{stats.uniqueCount.toLocaleString()}</span>
                      </div>
                      {stats.nullCount > 0 && (
                        <div className="flex justify-between">
                          <span>Empty:</span>
                          <span className="font-medium text-red-600">{stats.nullCount.toLocaleString()}</span>
                        </div>
                      )}
                      {stats.type === 'number' && (
                        <>
                          {stats.min !== undefined && (
                            <div className="flex justify-between">
                              <span>Min:</span>
                              <span className="font-medium">{Number(stats.min).toLocaleString()}</span>
                            </div>
                          )}
                          {stats.max !== undefined && (
                            <div className="flex justify-between">
                              <span>Max:</span>
                              <span className="font-medium">{Number(stats.max).toLocaleString()}</span>
                            </div>
                          )}
                          {stats.avg !== undefined && (
                            <div className="flex justify-between">
                              <span>Avg:</span>
                              <span className="font-medium">{stats.avg.toFixed(2)}</span>
                            </div>
                          )}
                        </>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
            {columnHeaders.length > 8 && (
              <p className="text-xs text-gray-500 mt-3">
                Showing statistics for first 8 columns
              </p>
            )}
          </div>
        )}

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

            {/* Rows per page selector */}
            <div className="flex items-center space-x-2">
              <label htmlFor="rows-per-page" className="text-sm text-gray-700">Show:</label>
              <select
                id="rows-per-page"
                value={rowsPerPage}
                onChange={(e) => {
                  setRowsPerPage(Number(e.target.value));
                  setCurrentPage(0);
                }}
                className="border border-gray-300 rounded-md text-sm py-2 px-3 bg-white focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500"
              >
                <option value={25}>25</option>
                <option value={50}>50</option>
                <option value={100}>100</option>
                <option value={200}>200</option>
              </select>
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
              {columnHeaders.length > 0 ? (
                <>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                    {columnHeaders.slice(0, 8).map((header) => (
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
                  {columnHeaders.length > 8 && (
                    <p className="text-xs text-gray-500 mt-2">
                      Showing first 8 columns. Use search above to filter across all columns.
                    </p>
                  )}
                </>
              ) : (
                <p className="text-sm text-gray-500">No columns available for filtering. Data may still be loading.</p>
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
              {dataResponse.records.length > 0 && columnHeaders.length > 0 ? (
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-300">
                    <thead className="bg-gray-50 sticky top-0">
                      <tr>
                        <th className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-r border-gray-200 w-20">
                          <input
                            type="checkbox"
                            checked={selectedRows.size === dataResponse.records.length && dataResponse.records.length > 0}
                            onChange={handleSelectAll}
                            className="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                          />
                        </th>
                        <th className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-r border-gray-200">
                          Row #
                        </th>
                        {columnHeaders.map((header) => {
                          const stats = columnStats[header];
                          return (
                            <th
                              key={header}
                              className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider border-r border-gray-200 min-w-32 cursor-pointer hover:bg-gray-100"
                              onClick={() => handleSort(header)}
                            >
                              <div className="flex items-center justify-between group">
                                <div className="flex items-center space-x-1">
                                  {stats && getDataTypeIcon(stats.type)}
                                  <span className="truncate" title={header}>
                                    {header}
                                  </span>
                                </div>
                                <div className="flex items-center">
                                  {sortConfig?.column === header && (
                                    sortConfig.direction === 'asc' ? (
                                      <ArrowUpIcon className="h-3 w-3 text-indigo-600" />
                                    ) : (
                                      <ArrowDownIcon className="h-3 w-3 text-indigo-600" />
                                    )
                                  )}
                                </div>
                              </div>
                            </th>
                          );
                        })}
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {sortedRecords.map((record, index) => (
                        <tr 
                          key={record.id} 
                          className={`${index % 2 === 0 ? 'bg-white' : 'bg-gray-50'} ${
                            selectedRows.has(record.id) ? 'bg-blue-50' : ''
                          } hover:bg-gray-100 transition-colors`}
                        >
                          <td className="px-3 py-2 whitespace-nowrap text-sm border-r border-gray-200">
                            <input
                              type="checkbox"
                              checked={selectedRows.has(record.id)}
                              onChange={() => handleRowSelect(record.id)}
                              className="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                            />
                          </td>
                          <td className="px-3 py-2 whitespace-nowrap text-sm font-medium text-gray-900 border-r border-gray-200">
                            {record.row_number}
                          </td>
                          {columnHeaders.map((header) => {
                            const value = record.row_data[header];
                            const stats = columnStats[header];
                            const formattedValue = formatCellValue(value, stats?.type || 'text');
                            
                            return (
                              <td
                                key={header}
                                className="px-3 py-2 text-sm border-r border-gray-200 max-w-48"
                              >
                                <div className="truncate" title={String(value || '')}>
                                  {formattedValue}
                                </div>
                              </td>
                            );
                          })}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : (
                <div className="p-8 text-center text-gray-500">
                  <DocumentTextIcon className="mx-auto h-12 w-12 text-gray-400" />
                  {dataResponse && dataResponse.records.length > 0 && columnHeaders.length === 0 ? (
                    <>
                      <h3 className="mt-2 text-sm font-medium text-gray-900">Unable to display data</h3>
                      <p className="mt-1 text-sm text-gray-500">
                        This spreadsheet contains data but the column structure could not be determined. 
                        The file may need to be re-uploaded or processed differently.
                      </p>
                    </>
                  ) : (
                    <>
                      <h3 className="mt-2 text-sm font-medium text-gray-900">No data found</h3>
                      <p className="mt-1 text-sm text-gray-500">
                        {hasActiveFilters
                          ? 'No rows match your current filters. Try adjusting your search criteria.'
                          : 'This dataset appears to be empty.'}
                      </p>
                    </>
                  )}
                </div>
              )}
            </>
          )}
        </div>

        {/* Pagination */}
        {dataResponse && dataResponse.total_count > 0 && (
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
