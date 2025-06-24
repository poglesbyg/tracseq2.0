import { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import {
  MagnifyingGlassIcon,
  XMarkIcon,
  FunnelIcon,
  DocumentTextIcon,
} from '@heroicons/react/24/outline';

interface SearchResult {
  id: string;
  [key: string]: string | number | boolean | null;
}

interface SearchResponse {
  records: SearchResult[];
  total_count: number;
  dataset_info?: {
    id: string;
    original_filename: string;
    file_type: string;
  };
}

interface SearchFilters {
  [key: string]: string;
}

interface AvailableFilters {
  pools: string[];
  samples: string[];
  projects: string[];
  all_columns: string[];
  column_values: Record<string, string[]>;
}

interface SpreadsheetSearchModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectResult: (result: SearchResult) => void;
}

export default function SpreadsheetSearchModal({ onClose }: SpreadsheetSearchModalProps) {
  const [searchTerm, setSearchTerm] = useState('');
  const [filters, setFilters] = useState<SearchFilters>({});
  const [poolFilter, setPoolFilter] = useState('');
  const [sampleFilter, setSampleFilter] = useState('');
  const [projectFilter, setProjectFilter] = useState('');
  const [showFilters, setShowFilters] = useState(false);
  const [limit, setLimit] = useState(25);
  const [offset, setOffset] = useState(0);

  // Fetch available filters
  const { data: availableFilters } = useQuery<AvailableFilters>({
    queryKey: ['available-filters'],
    queryFn: async () => {
      const response = await axios.get('/api/spreadsheets/filters');
      return response.data.data;
    },
  });

  // Debounced search
  const [debouncedSearchTerm, setDebouncedSearchTerm] = useState(searchTerm);
  
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedSearchTerm(searchTerm);
      setOffset(0); // Reset pagination when search changes
    }, 300);
    return () => clearTimeout(timer);
  }, [searchTerm]);

  // Reset pagination when filters change
  useEffect(() => {
    setOffset(0);
  }, [poolFilter, sampleFilter, projectFilter]);

  // Build search query
  const buildSearchQuery = () => {
    const params = new URLSearchParams();
    
    if (debouncedSearchTerm.trim()) {
      params.append('search_term', debouncedSearchTerm.trim());
    }
    
    // Add pool/sample/project filters
    if (poolFilter.trim()) {
      params.append('pool_filter', poolFilter.trim());
    }
    if (sampleFilter.trim()) {
      params.append('sample_filter', sampleFilter.trim());
    }
    if (projectFilter.trim()) {
      params.append('project_filter', projectFilter.trim());
    }
    
    // Add column filters
    Object.entries(filters).forEach(([column, value]) => {
      if (value.trim()) {
        params.append(`filter_${column}`, value.trim());
      }
    });
    
    params.append('limit', limit.toString());
    params.append('offset', offset.toString());
    
    return params.toString();
  };

  // Search query
  const { data: searchResults, isLoading, error } = useQuery<SearchResponse>({
    queryKey: ['spreadsheet-search', debouncedSearchTerm, filters, limit, offset],
    queryFn: async () => {
      const query = buildSearchQuery();
      const response = await axios.get(`/api/spreadsheets/search?${query}`);
      return response.data.data;
    },
    enabled: debouncedSearchTerm.trim().length > 0 || 
             Object.values(filters).some(v => v.trim().length > 0) ||
             poolFilter.trim().length > 0 ||
             sampleFilter.trim().length > 0 ||
             projectFilter.trim().length > 0,
  });

  const handleFilterChange = (column: string, value: string) => {
    setFilters(prev => ({
      ...prev,
      [column]: value
    }));
    setOffset(0); // Reset pagination when filters change
  };

  const removeFilter = (column: string) => {
    setFilters(prev => {
      const newFilters = { ...prev };
      delete newFilters[column];
      return newFilters;
    });
  };

  const clearAllFilters = () => {
    setFilters({});
    setSearchTerm('');
    setPoolFilter('');
    setSampleFilter('');
    setProjectFilter('');
    setOffset(0);
  };

  const hasActiveSearch = debouncedSearchTerm.trim().length > 0 || 
                         Object.values(filters).some(v => v.trim().length > 0) ||
                         poolFilter.trim().length > 0 ||
                         sampleFilter.trim().length > 0 ||
                         projectFilter.trim().length > 0;
  const totalPages = searchResults ? Math.ceil(searchResults.total_count / limit) : 0;
  const currentPage = Math.floor(offset / limit) + 1;

  const goToNextPage = () => {
    if (offset + limit < (searchResults?.total_count || 0)) {
      setOffset(offset + limit);
    }
  };

  const goToPrevPage = () => {
    if (offset > 0) {
      setOffset(Math.max(0, offset - limit));
    }
  };

  const highlightSearchTerm = (text: string) => {
    if (!debouncedSearchTerm.trim()) return text;
    
    const regex = new RegExp(`(${debouncedSearchTerm.trim()})`, 'gi');
    const parts = text.split(regex);
    
    return parts.map((part, index) => 
      regex.test(part) ? (
        <mark key={index} className="bg-yellow-200 text-yellow-900 font-medium">
          {part}
        </mark>
      ) : part
    );
  };

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50 flex items-start justify-center p-4 pt-8">
      <div className="bg-white rounded-lg shadow-xl max-w-6xl w-full max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-gray-200">
          <div>
            <h2 className="text-xl font-semibold text-gray-900">Search Spreadsheet Data</h2>
            <p className="text-sm text-gray-500 mt-1">
              Search across all uploaded CSV and Excel files
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <XMarkIcon className="h-6 w-6" />
          </button>
        </div>

        {/* Search Controls */}
        <div className="p-6 border-b border-gray-200 bg-gray-50">
          {/* Main Search */}
          <div className="flex space-x-4 mb-4">
            <div className="flex-1 relative">
              <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
              </div>
              <input
                type="text"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                placeholder="Search across all data... (e.g., LAB001, Oncology, high priority)"
                className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500"
              />
            </div>
            <button
              type="button"
              onClick={() => setShowFilters(!showFilters)}
              className={`inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium ${
                showFilters || Object.keys(filters).length > 0 || poolFilter || sampleFilter || projectFilter
                  ? 'text-indigo-700 bg-indigo-50 border-indigo-300'
                  : 'text-gray-700 bg-white hover:bg-gray-50'
              } focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500`}
            >
              <FunnelIcon className="h-4 w-4 mr-2" />
              Filters
              {(Object.keys(filters).length > 0 || poolFilter || sampleFilter || projectFilter) && (
                <span className="ml-1 inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800">
                  {Object.keys(filters).length + (poolFilter ? 1 : 0) + (sampleFilter ? 1 : 0) + (projectFilter ? 1 : 0)}
                </span>
              )}
            </button>
          </div>

          {/* Pool/Sample/Project Quick Filters */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
            <div>
              <label htmlFor="pool-filter" className="block text-sm font-medium text-gray-700 mb-1">
                Pool
              </label>
              <select
                id="pool-filter"
                value={poolFilter}
                onChange={(e) => setPoolFilter(e.target.value)}
                className="block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
              >
                <option value="">All pools</option>
                {availableFilters?.pools.map((pool) => (
                  <option key={pool} value={pool}>
                    {pool}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label htmlFor="sample-filter" className="block text-sm font-medium text-gray-700 mb-1">
                Sample
              </label>
              <select
                id="sample-filter"
                value={sampleFilter}
                onChange={(e) => setSampleFilter(e.target.value)}
                className="block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
              >
                <option value="">All samples</option>
                {availableFilters?.samples.map((sample) => (
                  <option key={sample} value={sample}>
                    {sample}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label htmlFor="project-filter" className="block text-sm font-medium text-gray-700 mb-1">
                Project
              </label>
              <select
                id="project-filter"
                value={projectFilter}
                onChange={(e) => setProjectFilter(e.target.value)}
                className="block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
              >
                <option value="">All projects</option>
                {availableFilters?.projects.map((project) => (
                  <option key={project} value={project}>
                    {project}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* Column Filters */}
          {showFilters && (
            <div className="bg-white border border-gray-200 rounded-md p-4">
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-medium text-gray-900">Column Filters</h3>
                {Object.keys(filters).length > 0 && (
                  <button
                    type="button"
                    onClick={clearAllFilters}
                    className="text-sm text-indigo-600 hover:text-indigo-800"
                  >
                    Clear all
                  </button>
                )}
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {['Sample_ID', 'Patient_ID', 'Department', 'Priority', 'Sample_Type', 'Submitter'].map((column) => (
                  <div key={column}>
                    <label htmlFor={`filter-${column}`} className="block text-sm font-medium text-gray-700 mb-1">
                      {column.replace('_', ' ')}
                    </label>
                    <input
                      type="text"
                      id={`filter-${column}`}
                      value={filters[column] || ''}
                      onChange={(e) => handleFilterChange(column, e.target.value)}
                      placeholder={`Filter by ${column.replace('_', ' ').toLowerCase()}`}
                      className="block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                    />
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Active Filters */}
          {Object.keys(filters).length > 0 && (
            <div className="flex flex-wrap gap-2 mt-3">
              {Object.entries(filters).map(([column, value]) => 
                value.trim() && (
                  <span
                    key={column}
                    className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800"
                  >
                    {column}: {value}
                    <button
                      type="button"
                      onClick={() => removeFilter(column)}
                      className="flex-shrink-0 ml-1 h-4 w-4 rounded-full inline-flex items-center justify-center text-indigo-400 hover:bg-indigo-200 hover:text-indigo-500 focus:outline-none focus:bg-indigo-500 focus:text-white"
                    >
                      <XMarkIcon className="h-3 w-3" />
                    </button>
                  </span>
                )
              )}
            </div>
          )}
        </div>

        {/* Results */}
        <div className="flex-1 overflow-y-auto">
          {isLoading && hasActiveSearch && (
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
              <span className="ml-3 text-gray-600">Searching...</span>
            </div>
          )}

          {error && (
            <div className="p-6 text-center">
              <div className="text-red-600">
                <h3 className="text-lg font-medium">Search Error</h3>
                <p className="mt-2">Failed to search data. Please try again.</p>
              </div>
            </div>
          )}

          {!hasActiveSearch && (
            <div className="p-8 text-center text-gray-500">
              <MagnifyingGlassIcon className="mx-auto h-12 w-12 text-gray-400" />
              <h3 className="mt-2 text-sm font-medium text-gray-900">Start your search</h3>
              <p className="mt-1 text-sm text-gray-500">
                Enter a search term or use column filters to find data across all uploaded files.
              </p>
            </div>
          )}

          {hasActiveSearch && searchResults && searchResults.records.length === 0 && !isLoading && (
            <div className="p-8 text-center text-gray-500">
              <DocumentTextIcon className="mx-auto h-12 w-12 text-gray-400" />
              <h3 className="mt-2 text-sm font-medium text-gray-900">No results found</h3>
              <p className="mt-1 text-sm text-gray-500">
                Try adjusting your search terms or filters.
              </p>
            </div>
          )}

          {searchResults && searchResults.records.length > 0 && (
            <div className="px-6 py-4">
              {/* Results Info */}
              <div className="flex items-center justify-between mb-4">
                <p className="text-sm text-gray-700">
                  Showing {offset + 1}-{Math.min(offset + limit, searchResults.total_count)} of{' '}
                  <span className="font-medium">{searchResults.total_count.toLocaleString()}</span> results
                </p>
                <div className="flex items-center space-x-2">
                  <label htmlFor="results-per-page" className="text-sm text-gray-700">
                    Results per page:
                  </label>
                  <select
                    id="results-per-page"
                    value={limit}
                    onChange={(e) => {
                      setLimit(Number(e.target.value));
                      setOffset(0);
                    }}
                    className="border-gray-300 rounded-md text-sm focus:ring-indigo-500 focus:border-indigo-500"
                  >
                    <option value={10}>10</option>
                    <option value={25}>25</option>
                    <option value={50}>50</option>
                    <option value={100}>100</option>
                  </select>
                </div>
              </div>

              {/* Results Table */}
              <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 rounded-lg">
                <table className="min-w-full divide-y divide-gray-300">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Row #
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Data
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Source File
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {searchResults.records.map((record, index) => (
                      <tr key={record.id} className={index % 2 === 0 ? 'bg-white' : 'bg-gray-50'}>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <div className="font-medium">#{record.row_number}</div>
                          <div className="text-xs text-gray-500">
                            {new Date(record.created_at).toLocaleDateString()}
                          </div>
                        </td>
                        <td className="px-6 py-4 text-sm text-gray-900">
                          <div className="space-y-1">
                            {Object.entries(record).slice(0, 4).map(([key, value]) => (
                              <div key={key} className="flex">
                                <span className="font-medium text-gray-600 w-24 flex-shrink-0">
                                  {key}:
                                </span>
                                <span className="text-gray-900">
                                  {highlightSearchTerm(String(value || '-'))}
                                </span>
                              </div>
                            ))}
                            {Object.keys(record).length > 4 && (
                              <div className="text-xs text-gray-500">
                                +{Object.keys(record).length - 4} more fields
                              </div>
                            )}
                          </div>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          <div className="text-gray-900 font-medium">
                            {record.original_filename}
                          </div>
                          <div className="text-xs text-gray-500 uppercase">
                            {record.file_type}
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              {/* Pagination */}
              {totalPages > 1 && (
                <div className="flex items-center justify-between mt-4">
                  <div className="text-sm text-gray-700">
                    Page {currentPage} of {totalPages}
                  </div>
                  <div className="flex space-x-2">
                    <button
                      onClick={goToPrevPage}
                      disabled={offset === 0}
                      className="px-3 py-1 text-sm border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      Previous
                    </button>
                    <button
                      onClick={goToNextPage}
                      disabled={offset + limit >= searchResults.total_count}
                      className="px-3 py-1 text-sm border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      Next
                    </button>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
} 
