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
  rows: Record<string, any>[];
  row_count: number;
  execution_time_ms: number;
  query: string;
}

interface ReportTemplate {
  id: string;
  name: string;
  description: string;
  sql: string;
  category: string;
}

interface DatabaseSchema {
  tables: TableInfo[];
}

interface TableInfo {
  name: string;
  columns: ColumnInfo[];
}

interface ColumnInfo {
  name: string;
  data_type: string;
  is_nullable: boolean;
  is_primary_key: boolean;
}

interface ReportConfig {
  id: string;
  name: string;
  type: 'sample' | 'template' | 'system' | 'custom';
  parameters: ReportParameter[];
  schedule?: ReportSchedule;
  metadata?: Record<string, unknown>;
}

interface ReportParameter {
  name: string;
  type: 'string' | 'number' | 'date' | 'boolean' | 'select';
  required: boolean;
  defaultValue?: string | number | boolean | null;
  options?: string[];
}

interface ReportSchedule {
  frequency: 'daily' | 'weekly' | 'monthly';
  time: string;
  enabled: boolean;
}

interface GeneratedReport {
  id: string;
  configId: string;
  generatedAt: string;
  status: 'generating' | 'completed' | 'failed';
  data: ReportData[];
  fileUrl?: string;
  errors?: string[];
}

interface ReportData {
  [key: string]: string | number | boolean | null;
}

export default function Reports() {
  const [sqlQuery, setSqlQuery] = useState('SELECT * FROM samples LIMIT 10;');
  const [activeTab, setActiveTab] = useState<'editor' | 'templates' | 'schema'>('editor');
  const [reportResult, setReportResult] = useState<ReportResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Fetch report templates
  const { data: templates } = useQuery<ReportTemplate[]>({
    queryKey: ['reportTemplates'],
    queryFn: async () => {
      const response = await axios.get('/api/reports/templates');
      return response.data;
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
    onError: (error: any) => {
      setError(error.response?.data || 'An error occurred while executing the query');
      setReportResult(null);
    },
  });

  const handleExecuteQuery = () => {
    if (!sqlQuery.trim()) return;
    executeQuery.mutate(sqlQuery);
  };

  const handleUseTemplate = (template: ReportTemplate) => {
    setSqlQuery(template.sql);
    setActiveTab('editor');
  };

  const groupedTemplates = templates?.reduce((acc, template) => {
    if (!acc[template.category]) {
      acc[template.category] = [];
    }
    acc[template.category].push(template);
    return acc;
  }, {} as Record<string, ReportTemplate[]>);

  const exportToCSV = () => {
    if (!reportResult) return;

    const csvContent = [
      reportResult.columns.join(','),
      ...reportResult.rows.map(row =>
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
                  </div>
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
                        <span>{reportResult.row_count} rows</span>
                        <span className="flex items-center">
                          <ClockIcon className="w-4 h-4 mr-1" />
                          {reportResult.execution_time_ms}ms
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
                      {reportResult.rows.map((row, index) => (
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
            {groupedTemplates && Object.entries(groupedTemplates).map(([category, categoryTemplates]) => (
              <div key={category} className="bg-white rounded-lg shadow">
                <div className="px-6 py-4 border-b border-gray-200">
                  <h2 className="text-lg font-medium text-gray-900">{category}</h2>
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
                        <div className="mt-3">
                          <code className="text-xs bg-gray-100 px-2 py-1 rounded">
                            {template.sql.length > 60 ? `${template.sql.substring(0, 60)}...` : template.sql}
                          </code>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {activeTab === 'schema' && (
          <div className="bg-white rounded-lg shadow">
            <div className="px-6 py-4 border-b border-gray-200">
              <h2 className="text-lg font-medium text-gray-900">Database Schema</h2>
              <p className="mt-1 text-sm text-gray-500">
                Available tables and columns for your queries
              </p>
            </div>
            <div className="p-6">
              {schema?.tables.map((table) => (
                <div key={table.name} className="mb-6 last:mb-0">
                  <h3 className="text-lg font-semibold text-gray-900 mb-3">{table.name}</h3>
                  <div className="overflow-x-auto">
                    <table className="min-w-full border border-gray-200 rounded-lg">
                      <thead className="bg-gray-50">
                        <tr>
                          <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Column</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Type</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Nullable</th>
                          <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Primary Key</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-gray-200">
                        {table.columns.map((column) => (
                          <tr key={column.name}>
                            <td className="px-4 py-2 text-sm font-medium text-gray-900">{column.name}</td>
                            <td className="px-4 py-2 text-sm text-gray-600">{column.data_type}</td>
                            <td className="px-4 py-2 text-sm text-gray-600">
                              {column.is_nullable ? 'Yes' : 'No'}
                            </td>
                            <td className="px-4 py-2 text-sm text-gray-600">
                              {column.is_primary_key ? (
                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                                  PK
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
      </div>
    </div>
  );
}

function formatCellValue(value: any): string {
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
