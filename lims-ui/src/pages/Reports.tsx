import { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import axios from 'axios';
import { 
  PlayIcon, 
  TableCellsIcon, 
  DocumentArrowDownIcon,
  ClockIcon,
  ExclamationTriangleIcon,
  InformationCircleIcon,
  BookOpenIcon
} from '@heroicons/react/24/outline';

interface ReportResult {
  columns: string[];
  rows?: Record<string, unknown>[];
  data?: Record<string, unknown>[];
  row_count?: number;
  rowCount?: number;
  execution_time_ms?: number;
  executionTime?: number;
  query?: string;
}

interface ReportTemplate {
  id: string;
  name: string;
  description: string;
  sql: string;
  category: string;
  tags?: string[];
}

interface DatabaseSchema {
  tables: TableInfo[];
  total_tables?: number;
  database_name?: string;
  last_updated?: string;
}

interface TableInfo {
  name: string;
  columns: ColumnInfo[];
  row_count?: number;
  type?: string;
  comment?: string;
}

interface ColumnInfo {
  name: string;
  data_type?: string;
  type?: string;
  is_nullable?: boolean;
  nullable?: boolean;
  is_primary_key?: boolean;
  foreign_key?: {
    references_table: string;
    references_column: string;
  };
  default?: string;
  comment?: string;
  max_length?: number;
  precision?: number;
  scale?: number;
}



export default function Reports() {
  const [sqlQuery, setSqlQuery] = useState('SELECT * FROM samples LIMIT 10;');
  const [activeTab, setActiveTab] = useState<'editor' | 'templates' | 'schema' | 'history'>('editor');
  const [reportResult, setReportResult] = useState<ReportResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [searchTerm, setSearchTerm] = useState<string>('');
  const [queryHistory, setQueryHistory] = useState<Array<{id: string, query: string, timestamp: string, name?: string}>>([]);
  const [favorites, setFavorites] = useState<Array<{id: string, query: string, name: string}>>([]);

  // Fetch report templates
  const { data: templates } = useQuery<ReportTemplate[]>({
    queryKey: ['reportTemplates'],
    queryFn: async () => {
      const response = await axios.get('/api/reports/templates');
      // Handle both response formats - direct array or nested in data/templates
      return Array.isArray(response.data) 
        ? response.data 
        : (response.data.data || response.data.templates || []);
    },
  });

  // Fetch database schema
  const { data: schema } = useQuery<DatabaseSchema>({
    queryKey: ['databaseSchema'],
    queryFn: async () => {
      const response = await axios.get('/api/reports/schema');
      return response.data;
    },
  });

  // Execute SQL query mutation
  const executeQuery = useMutation({
    mutationFn: async (sql: string) => {
      const response = await axios.post('/api/reports/execute', { sql });
      return response.data;
    },
    onSuccess: (data: ReportResult) => {
      setReportResult(data);
      setError(null);
    },
    onError: (error: unknown) => {
      let errorMessage = 'An error occurred while executing the query';
      
      if (error && typeof error === 'object' && 'response' in error && 
          error.response && typeof error.response === 'object' && 'data' in error.response) {
        const responseData = error.response.data;
        
        // Handle different response data formats
        if (typeof responseData === 'string') {
          errorMessage = responseData;
        } else if (responseData && typeof responseData === 'object') {
          // Handle FastAPI error response format
          if ('detail' in responseData && typeof responseData.detail === 'string') {
            errorMessage = responseData.detail;
          } else if ('message' in responseData && typeof responseData.message === 'string') {
            errorMessage = responseData.message;
          } else {
            errorMessage = JSON.stringify(responseData);
          }
        }
      }
      
      setError(errorMessage);
      setReportResult(null);
    },
  });

  const handleExecuteQuery = () => {
    if (!sqlQuery.trim()) return;
    
    // Add to query history
    const newHistoryItem = {
      id: Date.now().toString(),
      query: sqlQuery,
      timestamp: new Date().toISOString(),
      name: `Query ${queryHistory.length + 1}`
    };
    setQueryHistory(prev => [newHistoryItem, ...prev.slice(0, 19)]); // Keep last 20 queries
    
    executeQuery.mutate(sqlQuery);
  };

  const saveToFavorites = () => {
    if (!sqlQuery.trim()) return;
    
    const name = prompt('Enter a name for this query:');
    if (name) {
      const newFavorite = {
        id: Date.now().toString(),
        query: sqlQuery,
        name: name
      };
      setFavorites(prev => [...prev, newFavorite]);
    }
  };

  const loadFromHistory = (query: string) => {
    setSqlQuery(query);
    setActiveTab('editor');
  };

  const handleUseTemplate = (template: ReportTemplate) => {
    setSqlQuery(template.sql);
    setActiveTab('editor');
  };

  // Filter templates based on category and search
  const filteredTemplates = templates?.filter(template => {
    const matchesCategory = selectedCategory === 'all' || template.category === selectedCategory;
    const matchesSearch = searchTerm === '' || 
      template.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      template.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
      (template.tags && template.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase())));
    return matchesCategory && matchesSearch;
  });

  const groupedTemplates = filteredTemplates?.reduce((acc, template) => {
    if (!acc[template.category]) {
      acc[template.category] = [];
    }
    acc[template.category].push(template);
    return acc;
  }, {} as Record<string, ReportTemplate[]>);

  // Get unique categories for filter
  const categories = templates ? ['all', ...new Set(templates.map(t => t.category))] : ['all'];

  const exportToCSV = () => {
    if (!reportResult) return;

    const rows = reportResult.rows || reportResult.data || [];
    const csvContent = [
      reportResult.columns.join(','),
      ...rows.map(row =>
        reportResult.columns.map(col => {
          const value = row[col];
          // Escape quotes and wrap in quotes if contains comma
          if (typeof value === 'string' && (value.includes(',') || value.includes('"'))) {
            return `"${value.replace(/"/g, '""')}"`;
          }
          return value?.toString() || '';
        }).join(',')
      )
    ].join('\n');

    const blob = new Blob([csvContent], { type: 'text/csv' });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `report_${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    window.URL.revokeObjectURL(url);
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">SQL Reports</h1>
          <p className="mt-2 text-sm text-gray-600">
            Write SQL queries to generate custom reports from your lab data
          </p>
        </div>

        {/* Tabs */}
        <div className="border-b border-gray-200 mb-6">
          <nav className="-mb-px flex space-x-8">
            <button
              onClick={() => setActiveTab('editor')}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'editor'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <PlayIcon className="w-5 h-5 inline-block mr-2" />
              Query Editor
            </button>
            <button
              onClick={() => setActiveTab('templates')}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'templates'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <BookOpenIcon className="w-5 h-5 inline-block mr-2" />
              Report Templates
            </button>
            <button
              onClick={() => setActiveTab('schema')}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'schema'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <TableCellsIcon className="w-5 h-5 inline-block mr-2" />
              Database Schema
            </button>
            <button
              onClick={() => setActiveTab('history')}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'history'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <ClockIcon className="w-5 h-5 inline-block mr-2" />
              Query History
            </button>
          </nav>
        </div>

        {/* Content */}
        {activeTab === 'editor' && (
          <div className="space-y-6">
            {/* SQL Editor */}
            <div className="bg-white rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200">
                <h2 className="text-lg font-medium text-gray-900">SQL Query Editor</h2>
                <p className="mt-1 text-sm text-gray-500">
                  Only SELECT queries are allowed for security reasons
                </p>
              </div>
              <div className="p-6">
                <textarea
                  value={sqlQuery}
                  onChange={(e) => setSqlQuery(e.target.value)}
                  className="w-full h-40 font-mono text-sm border border-gray-300 rounded-lg p-4 focus:ring-indigo-500 focus:border-indigo-500"
                  placeholder="SELECT * FROM samples WHERE status = 'pending';"
                />
                <div className="mt-4 flex justify-between items-center">
                  <div className="flex items-center space-x-4">
                    <div className="flex items-center text-sm text-gray-500">
                      <InformationCircleIcon className="w-4 h-4 mr-1" />
                      Read-only queries only
                    </div>
                    <div className="text-sm text-gray-500">
                      Lines: {sqlQuery.split('\n').length} | Characters: {sqlQuery.length}
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <button
                      onClick={saveToFavorites}
                      disabled={!sqlQuery.trim()}
                      className="inline-flex items-center px-3 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      ⭐ Save
                    </button>
                    <button
                      onClick={handleExecuteQuery}
                      disabled={executeQuery.isPending || !sqlQuery.trim()}
                      className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {executeQuery.isPending ? (
                        <>
                          <svg className="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                          </svg>
                          Executing...
                        </>
                      ) : (
                        <>
                          <PlayIcon className="w-4 h-4 mr-2" />
                          Execute Query
                        </>
                      )}
                    </button>
                  </div>
                </div>
              </div>
            </div>

            {/* Error Display */}
            {error && (
              <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                <div className="flex">
                  <ExclamationTriangleIcon className="h-5 w-5 text-red-400" />
                  <div className="ml-3">
                    <h3 className="text-sm font-medium text-red-800">Query Error</h3>
                    <div className="mt-2 text-sm text-red-700">
                      <pre className="whitespace-pre-wrap">{error}</pre>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Results */}
            {reportResult && (
              <div className="bg-white rounded-lg shadow">
                <div className="px-6 py-4 border-b border-gray-200">
                  <div className="flex justify-between items-center">
                    <div>
                      <h2 className="text-lg font-medium text-gray-900">Query Results</h2>
                      <div className="mt-1 flex items-center space-x-4 text-sm text-gray-500">
                                              <span>{reportResult.row_count || reportResult.rowCount || 0} rows</span>
                      <span className="flex items-center">
                        <ClockIcon className="w-4 h-4 mr-1" />
                        {reportResult.execution_time_ms || reportResult.executionTime || 0}ms
                        </span>
                      </div>
                    </div>
                    <button
                      onClick={exportToCSV}
                      className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                    >
                      <DocumentArrowDownIcon className="w-4 h-4 mr-2" />
                      Export CSV
                    </button>
                  </div>
                </div>
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        {reportResult.columns.map((column) => (
                          <th
                            key={column}
                            className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                          >
                            {column}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {(reportResult.rows || reportResult.data || []).map((row, index) => (
                        <tr key={index} className="hover:bg-gray-50">
                          {reportResult.columns.map((column) => (
                            <td key={column} className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                              {formatCellValue(row[column])}
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            )}
          </div>
        )}

        {activeTab === 'templates' && (
          <div className="space-y-6">
            {/* Template Filters */}
            <div className="bg-white rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200">
                <h2 className="text-lg font-medium text-gray-900">Query Templates</h2>
                <p className="mt-1 text-sm text-gray-500">
                  Pre-built queries for common database operations - {filteredTemplates?.length || 0} templates available
                </p>
              </div>
              <div className="p-6">
                <div className="flex flex-col sm:flex-row gap-4 mb-6">
                  <div className="flex-1">
                    <label htmlFor="search" className="block text-sm font-medium text-gray-700 mb-1">
                      Search Templates
                    </label>
                    <input
                      type="text"
                      id="search"
                      value={searchTerm}
                      onChange={(e) => setSearchTerm(e.target.value)}
                      placeholder="Search by name, description, or tags..."
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500"
                    />
                  </div>
                  <div className="sm:w-48">
                    <label htmlFor="category" className="block text-sm font-medium text-gray-700 mb-1">
                      Category
                    </label>
                    <select
                      id="category"
                      value={selectedCategory}
                      onChange={(e) => setSelectedCategory(e.target.value)}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500"
                    >
                      {categories.map(category => (
                        <option key={category} value={category}>
                          {category === 'all' ? 'All Categories' : category.charAt(0).toUpperCase() + category.slice(1)}
                        </option>
                      ))}
                    </select>
                  </div>
                </div>
              </div>
            </div>

            {/* Template Results */}
            {groupedTemplates && Object.entries(groupedTemplates).map(([category, categoryTemplates]) => (
              <div key={category} className="bg-white rounded-lg shadow">
                <div className="px-6 py-4 border-b border-gray-200">
                  <h2 className="text-lg font-medium text-gray-900 capitalize">{category} Templates ({categoryTemplates.length})</h2>
                </div>
                <div className="p-6">
                  <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {categoryTemplates.map((template) => (
                      <div
                        key={template.id}
                        className="border border-gray-200 rounded-lg p-4 hover:border-indigo-300 hover:shadow-sm transition-all cursor-pointer"
                        onClick={() => handleUseTemplate(template)}
                      >
                        <h3 className="font-medium text-gray-900">{template.name}</h3>
                        <p className="mt-1 text-sm text-gray-500">{template.description}</p>
                        {template.tags && (
                          <div className="mt-2 flex flex-wrap gap-1">
                            {template.tags.slice(0, 3).map(tag => (
                              <span key={tag} className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                                {tag}
                              </span>
                            ))}
                            {template.tags.length > 3 && (
                              <span className="text-xs text-gray-500">+{template.tags.length - 3} more</span>
                            )}
                          </div>
                        )}
                        <div className="mt-3">
                          <code className="text-xs bg-gray-100 px-2 py-1 rounded block overflow-hidden">
                            {template.sql.length > 80 ? `${template.sql.substring(0, 80)}...` : template.sql}
                          </code>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            ))}

            {filteredTemplates?.length === 0 && (
              <div className="bg-white rounded-lg shadow p-6 text-center">
                <p className="text-gray-500">No templates found matching your criteria.</p>
                <button
                  onClick={() => {
                    setSearchTerm('');
                    setSelectedCategory('all');
                  }}
                  className="mt-2 text-indigo-600 hover:text-indigo-500"
                >
                  Clear filters
                </button>
              </div>
            )}
          </div>
        )}

        {activeTab === 'schema' && (
          <div className="bg-white rounded-lg shadow">
            <div className="px-6 py-4 border-b border-gray-200">
              <h2 className="text-lg font-medium text-gray-900">Database Schema</h2>
              <p className="mt-1 text-sm text-gray-500">
                Available tables and columns for your queries - {schema?.total_tables || schema?.tables?.length || 0} tables
              </p>
            </div>
            <div className="p-6">
              {schema?.tables.map((table) => (
                <div key={table.name} className="mb-6 last:mb-0">
                  <div className="flex items-center justify-between mb-3">
                    <h3 className="text-lg font-semibold text-gray-900">{table.name}</h3>
                    {table.row_count !== undefined && (
                      <span className="text-sm text-gray-500">{table.row_count} rows</span>
                    )}
                  </div>
                  <div className="overflow-x-auto">
                    <table className="min-w-full border border-gray-200 rounded-lg">
                      <thead className="bg-gray-50">
                        <tr>
                          <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Column</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Type</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Nullable</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Key</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-gray-200">
                        {table.columns.map((column) => (
                          <tr key={column.name}>
                            <td className="px-4 py-2 text-sm font-medium text-gray-900">{column.name}</td>
                            <td className="px-4 py-2 text-sm text-gray-600">{column.data_type || column.type || 'unknown'}</td>
                            <td className="px-4 py-2 text-sm text-gray-600">
                              {(column.is_nullable ?? column.nullable) ? 'Yes' : 'No'}
                            </td>
                            <td className="px-4 py-2 text-sm text-gray-600">
                              {column.is_primary_key ? (
                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                                  PK
                                </span>
                              ) : column.foreign_key ? (
                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                                  FK
                                </span>
                              ) : (
                                '-'
                              )}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'history' && (
          <div className="space-y-6">
            {/* Query History */}
            <div className="bg-white rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200">
                <h2 className="text-lg font-medium text-gray-900">Query History</h2>
                <p className="mt-1 text-sm text-gray-500">
                  Recently executed queries - {queryHistory.length} queries saved
                </p>
              </div>
              <div className="p-6">
                {queryHistory.length > 0 ? (
                  <div className="space-y-3">
                    {queryHistory.map((item) => (
                      <div key={item.id} className="border border-gray-200 rounded-lg p-4 hover:border-indigo-300 transition-all">
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="flex items-center space-x-2 mb-2">
                              <h4 className="font-medium text-gray-900">{item.name}</h4>
                              <span className="text-xs text-gray-500">
                                {new Date(item.timestamp).toLocaleString()}
                              </span>
                            </div>
                            <code className="text-sm bg-gray-100 px-3 py-2 rounded block overflow-hidden">
                              {item.query.length > 120 ? `${item.query.substring(0, 120)}...` : item.query}
                            </code>
                          </div>
                          <button
                            onClick={() => loadFromHistory(item.query)}
                            className="ml-4 px-3 py-1 text-sm bg-indigo-100 text-indigo-700 rounded hover:bg-indigo-200"
                          >
                            Use Query
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="text-center py-8">
                    <p className="text-gray-500">No query history yet. Execute some queries to see them here.</p>
                  </div>
                )}
              </div>
            </div>

            {/* Favorites */}
            <div className="bg-white rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200">
                <h2 className="text-lg font-medium text-gray-900">Saved Queries</h2>
                <p className="mt-1 text-sm text-gray-500">
                  Your favorite queries - {favorites.length} saved
                </p>
              </div>
              <div className="p-6">
                {favorites.length > 0 ? (
                  <div className="space-y-3">
                    {favorites.map((item) => (
                      <div key={item.id} className="border border-gray-200 rounded-lg p-4 hover:border-indigo-300 transition-all">
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <h4 className="font-medium text-gray-900 mb-2">{item.name}</h4>
                            <code className="text-sm bg-gray-100 px-3 py-2 rounded block overflow-hidden">
                              {item.query.length > 120 ? `${item.query.substring(0, 120)}...` : item.query}
                            </code>
                          </div>
                          <div className="ml-4 flex space-x-2">
                            <button
                              onClick={() => loadFromHistory(item.query)}
                              className="px-3 py-1 text-sm bg-indigo-100 text-indigo-700 rounded hover:bg-indigo-200"
                            >
                              Use Query
                            </button>
                            <button
                              onClick={() => setFavorites(prev => prev.filter(f => f.id !== item.id))}
                              className="px-3 py-1 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200"
                            >
                              Remove
                            </button>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="text-center py-8">
                    <p className="text-gray-500">No saved queries yet. Use the ⭐ Save button in the editor to save your favorite queries.</p>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function formatCellValue(value: unknown): string {
  if (value === null || value === undefined) {
    return '';
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  if (typeof value === 'boolean') {
    return value ? 'true' : 'false';
  }
  return String(value);
}
