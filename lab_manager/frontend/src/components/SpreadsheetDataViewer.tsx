import { useState, useMemo, useCallback, useEffect, useRef } from 'react';
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
  EyeIcon,
  EyeSlashIcon,
  AdjustmentsHorizontalIcon,
  ClipboardDocumentCheckIcon,
  ExclamationTriangleIcon,
  CheckCircleIcon,
  InformationCircleIcon,
  ArrowPathIcon,
  CommandLineIcon,
  BookmarkIcon,
  PrinterIcon,
  Squares2X2Icon,
  CogIcon,
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
  row_data: Record<string, unknown>;
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
  qualityScore: number;
}

interface SortConfig {
  column: string;
  direction: 'asc' | 'desc';
}

interface AdvancedFilter {
  column: string;
  operator: 'contains' | 'equals' | 'starts_with' | 'ends_with' | 'greater_than' | 'less_than' | 'between' | 'is_empty' | 'is_not_empty';
  value: string;
  value2?: string; // For 'between' operator
}

interface SavedView {
  id: string;
  name: string;
  filters: Record<string, string>;
  advancedFilters: AdvancedFilter[];
  searchTerm: string;
  sortConfig: SortConfig | null;
  hiddenColumns: Set<string>;
  rowsPerPage: number;
}

interface SpreadsheetRecord {
  [key: string]: string | number | boolean | null;
}

interface SpreadsheetDataViewerProps {
  data: SpreadsheetRecord[];
  columns: string[];
  onCellEdit?: (rowIndex: number, columnName: string, value: string | number | boolean | null) => void;
}

export default function SpreadsheetDataViewer({ dataset, onClose }: SpreadsheetDataViewerProps) {
  const [currentPage, setCurrentPage] = useState(0);
  const [searchTerm, setSearchTerm] = useState('');
  const [columnFilters, setColumnFilters] = useState<Record<string, string>>({});
  const [advancedFilters, setAdvancedFilters] = useState<AdvancedFilter[]>([]);
  const [showFilters, setShowFilters] = useState(false);
  const [showAdvancedFilters, setShowAdvancedFilters] = useState(false);
  const [showStats, setShowStats] = useState(false);
  const [showColumnManager, setShowColumnManager] = useState(false);
  const [fullScreen, setFullScreen] = useState(false);
  const [sortConfig, setSortConfig] = useState<SortConfig | null>(null);
  const [selectedRows, setSelectedRows] = useState<Set<string>>(new Set());
  const [rowsPerPage, setRowsPerPage] = useState(50);
  const [hiddenColumns, setHiddenColumns] = useState<Set<string>>(new Set());
  const [pinnedColumns, setPinnedColumns] = useState<Set<string>>(new Set());
  // const [columnWidths, setColumnWidths] = useState<Record<string, number>>({});
  const [viewMode, setViewMode] = useState<'table' | 'cards'>('table');
  const [showKeyboardShortcuts, setShowKeyboardShortcuts] = useState(false);
  const [savedViews, setSavedViews] = useState<SavedView[]>([]);
  const [showSaveViewDialog, setShowSaveViewDialog] = useState(false);
  const [newViewName, setNewViewName] = useState('');
  const [dataQualityCheck, setDataQualityCheck] = useState(true);
  const [refreshInterval, setRefreshInterval] = useState<number | null>(null);
  
  const searchInputRef = useRef<HTMLInputElement>(null);
  const tableRef = useRef<HTMLDivElement>(null);

  // Debounced search
  const [debouncedSearchTerm, setDebouncedSearchTerm] = useState(searchTerm);
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedSearchTerm(searchTerm);
    }, 300);
    return () => clearTimeout(timer);
  }, [searchTerm]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyboard = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey) {
        switch (e.key) {
          case 'f':
            e.preventDefault();
            searchInputRef.current?.focus();
            break;
          case 'r':
            e.preventDefault();
            window.location.reload();
            break;
          case 'p':
            e.preventDefault();
            handlePrint();
            break;
          case 's':
            e.preventDefault();
            setShowSaveViewDialog(true);
            break;
          case 'h':
            e.preventDefault();
            setShowKeyboardShortcuts(true);
            break;
        }
      }
      if (e.key === 'Escape') {
        if (showKeyboardShortcuts) setShowKeyboardShortcuts(false);
        else if (showSaveViewDialog) setShowSaveViewDialog(false);
        else if (fullScreen) setFullScreen(false);
        else onClose();
      }
    };

    document.addEventListener('keydown', handleKeyboard);
    return () => document.removeEventListener('keydown', handleKeyboard);
  }, [showKeyboardShortcuts, showSaveViewDialog, fullScreen, onClose]);

  // Auto-refresh
  useEffect(() => {
    if (!refreshInterval) return;
    
    const interval = setInterval(() => {
      // Trigger refetch - this would need to be implemented with query client
      console.log('Auto-refreshing data...');
    }, refreshInterval * 1000);

    return () => clearInterval(interval);
  }, [refreshInterval]);

  // Build search query
  const buildSearchQuery = () => {
    const params = new URLSearchParams();
    params.append('dataset_id', dataset.id);
    params.append('limit', rowsPerPage.toString());
    params.append('offset', (currentPage * rowsPerPage).toString());

    if (debouncedSearchTerm.trim()) {
      params.append('search_term', debouncedSearchTerm.trim());
    }

    // Add basic column filters
    Object.entries(columnFilters).forEach(([column, value]) => {
      if (value.trim()) {
        params.append(`filter_${column}`, value.trim());
      }
    });

    // Convert supported advanced filters to basic column filters
    // The backend currently only supports basic ILIKE pattern matching
    advancedFilters.forEach((filter) => {
      if (filter.value.trim()) {
        const columnName = filter.column;
        const filterValue = filter.value.trim();
        
        // Convert advanced filter operators to basic patterns where possible
        switch (filter.operator) {
          case 'contains':
            // Default behavior - backend uses ILIKE %value%
            break;
          case 'equals':
            // For exact match, we'll just use the value as-is
            // Note: Backend still uses ILIKE so it won't be truly exact
            break;
          case 'starts_with':
            // Backend will still wrap in %, but we can optimize by not adding leading %
            break;
          case 'ends_with':
            // Backend will still wrap in %, but we can optimize by not adding trailing %
            break;
          case 'is_empty':
          case 'is_not_empty':
            // Skip these for now as backend doesn't support them
            console.warn(`Advanced filter operator '${filter.operator}' not supported by backend`);
            return;
          case 'greater_than':
          case 'less_than':
          case 'between':
            // Skip numeric comparisons for now as backend doesn't support them
            console.warn(`Advanced filter operator '${filter.operator}' not supported by backend`);
            return;
          default:
            break;
        }
        
        params.append(`filter_${columnName}`, filterValue);
      }
    });

    // Note: Sorting is currently handled client-side in sortedRecords
    // The backend search doesn't support sort parameters yet
    // if (sortConfig) {
    //   params.append('sort_column', sortConfig.column);
    //   params.append('sort_direction', sortConfig.direction);
    // }

    return params.toString();
  };

  // Fetch data
  const { data: dataResponse, isLoading, error, refetch } = useQuery<DataResponse>({
    queryKey: ['spreadsheet-data', dataset.id, currentPage, debouncedSearchTerm, columnFilters, advancedFilters, rowsPerPage, sortConfig],
    queryFn: async () => {
      const query = buildSearchQuery();
      const response = await axios.get(`/api/spreadsheets/search?${query}`);
      
      // The API returns data wrapped in ApiResponse structure
      if (response.data.success && response.data.data) {
        return {
          records: response.data.data.records || [],
          total_count: response.data.data.total_count || 0
        };
      } else {
        throw new Error(response.data.message || 'Failed to fetch data');
      }
    },
    refetchInterval: refreshInterval ? refreshInterval * 1000 : false,
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
  const visibleColumns = columnHeaders.filter(header => !hiddenColumns.has(header));
  const pinnedColumnsArray = visibleColumns.filter(header => pinnedColumns.has(header));
  const unpinnedColumns = visibleColumns.filter(header => !pinnedColumns.has(header));
  const orderedColumns = [...pinnedColumnsArray, ...unpinnedColumns];

  // Data type detection utilities
  const detectDataType = useCallback((value: unknown): DataType => {
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

  // Calculate column statistics with data quality
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

      // Calculate data quality score
      const completenessScore = (nonEmptyValues.length / values.length) * 40;
      const uniquenessScore = (uniqueValues.size / Math.max(nonEmptyValues.length, 1)) * 30;
      const consistencyScore = (typeCounts[predominantType] / Math.max(nonEmptyValues.length, 1)) * 30;
      const qualityScore = Math.round(completenessScore + uniquenessScore + consistencyScore);
      
      stats[header] = {
        type: predominantType,
        nonEmptyCount: nonEmptyValues.length,
        uniqueCount: uniqueValues.size,
        nullCount: values.length - nonEmptyValues.length,
        qualityScore,
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
          comparison = new Date(aVal as string | number | Date).getTime() - new Date(bVal as string | number | Date).getTime();
          break;
        default:
          comparison = String(aVal).localeCompare(String(bVal));
      }
      
      return sortConfig.direction === 'asc' ? comparison : -comparison;
    });
  }, [dataResponse, sortConfig, columnStats]);

  const totalPages = dataResponse && dataResponse.total_count > 0 ? Math.ceil(dataResponse.total_count / rowsPerPage) : 0;
  const startRow = dataResponse && dataResponse.records.length > 0 ? currentPage * rowsPerPage + 1 : 0;
  const endRow = dataResponse ? Math.min(startRow + dataResponse.records.length - 1, dataResponse.total_count) : 0;

  // Reset current page if it becomes invalid after data changes
  useEffect(() => {
    if (totalPages > 0 && currentPage >= totalPages) {
      setCurrentPage(Math.max(0, totalPages - 1));
    }
  }, [totalPages, currentPage]);

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
    setAdvancedFilters([]);
    setSearchTerm('');
    setCurrentPage(0);
  };

  const hasActiveFilters = searchTerm.trim() || Object.values(columnFilters).some(v => v.trim()) || advancedFilters.length > 0;

  // Advanced filter management
  const addAdvancedFilter = () => {
    setAdvancedFilters(prev => [...prev, {
      column: columnHeaders[0] || '',
      operator: 'contains',
      value: ''
    }]);
  };

  const updateAdvancedFilter = (index: number, updates: Partial<AdvancedFilter>) => {
    setAdvancedFilters(prev => prev.map((filter, i) => 
      i === index ? { ...filter, ...updates } : filter
    ));
  };

  const removeAdvancedFilter = (index: number) => {
    setAdvancedFilters(prev => prev.filter((_, i) => i !== index));
  };

  // Column management
  const toggleColumnVisibility = (column: string) => {
    setHiddenColumns(prev => {
      const newSet = new Set(prev);
      if (newSet.has(column)) {
        newSet.delete(column);
      } else {
        newSet.add(column);
      }
      return newSet;
    });
  };

  const toggleColumnPin = (column: string) => {
    setPinnedColumns(prev => {
      const newSet = new Set(prev);
      if (newSet.has(column)) {
        newSet.delete(column);
      } else {
        newSet.add(column);
      }
      return newSet;
    });
  };

  // Bulk operations
  const handleBulkOperation = (operation: 'export' | 'copy' | 'delete') => {
    const selectedData = sortedRecords.filter(record => selectedRows.has(record.id));
    
    switch (operation) {
      case 'export':
        exportSelectedData(selectedData);
        break;
      case 'copy':
        copySelectedData(selectedData);
        break;
      case 'delete':
        console.log('Delete operation would be implemented here');
        break;
    }
  };

  const exportSelectedData = (records: DataRecord[]) => {
    const csvContent = [
      visibleColumns.join(','),
      ...records.map(record => 
        visibleColumns.map(header => {
          const value = record.row_data[header] || '';
          const stringValue = String(value);
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
    a.download = `${dataset.original_filename}_selected_rows.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const copySelectedData = (records: DataRecord[]) => {
    const textContent = [
      visibleColumns.join('\t'),
      ...records.map(record => 
        visibleColumns.map(header => String(record.row_data[header] || '')).join('\t')
      )
    ].join('\n');
    
    navigator.clipboard.writeText(textContent);
  };

  // Save and load views
  const saveCurrentView = () => {
    if (!newViewName.trim()) return;
    
    const newView: SavedView = {
      id: Date.now().toString(),
      name: newViewName.trim(),
      filters: columnFilters,
      advancedFilters,
      searchTerm,
      sortConfig,
      hiddenColumns,
      rowsPerPage
    };
    
    setSavedViews(prev => [...prev, newView]);
    setNewViewName('');
    setShowSaveViewDialog(false);
  };

  const loadView = (view: SavedView) => {
    setColumnFilters(view.filters);
    setAdvancedFilters(view.advancedFilters);
    setSearchTerm(view.searchTerm);
    setSortConfig(view.sortConfig);
    setHiddenColumns(view.hiddenColumns);
    setRowsPerPage(view.rowsPerPage);
    setCurrentPage(0);
  };

  const deleteView = (viewId: string) => {
    setSavedViews(prev => prev.filter(view => view.id !== viewId));
  };

  // Print functionality
  const handlePrint = () => {
    if (!dataResponse) return;
    
    const printContent = `
      <html>
        <head>
          <title>${dataset.original_filename}</title>
          <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            table { border-collapse: collapse; width: 100%; margin-top: 20px; }
            th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
            th { background-color: #f2f2f2; font-weight: bold; }
            .header { margin-bottom: 20px; }
            .info { color: #666; font-size: 14px; }
          </style>
        </head>
        <body>
          <div class="header">
            <h1>${dataset.original_filename}</h1>
            <div class="info">
              Generated on ${new Date().toLocaleString()} | 
              ${dataResponse.total_count.toLocaleString()} total rows | 
              ${visibleColumns.length} visible columns
            </div>
          </div>
          <table>
            <thead>
              <tr>
                ${visibleColumns.map(header => `<th>${header}</th>`).join('')}
              </tr>
            </thead>
            <tbody>
              ${sortedRecords.slice(0, 100).map(record => `
                <tr>
                  ${visibleColumns.map(header => `<td>${String(record.row_data[header] || '')}</td>`).join('')}
                </tr>
              `).join('')}
            </tbody>
          </table>
          ${sortedRecords.length > 100 ? '<p><em>Showing first 100 rows only</em></p>' : ''}
        </body>
      </html>
    `;
    
    const printWindow = window.open('', '_blank');
    if (printWindow) {
      printWindow.document.write(printContent);
      printWindow.document.close();
      printWindow.print();
    }
  };

  // Utility functions
  const getFileIcon = (fileType: string) => {
    switch (fileType.toLowerCase()) {
      case 'csv':
        return <DocumentTextIcon className="h-6 w-6 text-emerald-500" />;
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

  const getQualityIndicator = (score: number) => {
    if (score >= 80) return { icon: CheckCircleIcon, color: 'text-green-500', bg: 'bg-green-50' };
    if (score >= 60) return { icon: InformationCircleIcon, color: 'text-yellow-500', bg: 'bg-yellow-50' };
    return { icon: ExclamationTriangleIcon, color: 'text-red-500', bg: 'bg-red-50' };
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
    if (!dataResponse || !visibleColumns.length) return;
    
    const records = sortedRecords.slice(currentPage * rowsPerPage, (currentPage + 1) * rowsPerPage);
    
    if (format === 'csv') {
      const csvContent = [
        visibleColumns.join(','),
        ...records.map(record => 
          visibleColumns.map(header => {
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
      const jsonData = records.map(record => 
        Object.fromEntries(visibleColumns.map(header => [header, record.row_data[header]]))
      );
      const blob = new Blob([JSON.stringify(jsonData, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${dataset.original_filename}_page_${currentPage + 1}.json`;
      a.click();
      URL.revokeObjectURL(url);
    }
  };

  const formatCellValue = (value: unknown, type: DataType) => {
    if (value === null || value === undefined || value === '') {
      return <span className="text-gray-400 italic">—</span>;
    }
    
    const stringValue = String(value);
    
    switch (type) {
      case 'number': {
        const num = Number(value);
        return isNaN(num) ? stringValue : (
          <span className="font-mono text-right">{num.toLocaleString()}</span>
        );
      }
      case 'date':
        try {
          const date = new Date(value as string | number | Date);
          return isNaN(date.getTime()) ? stringValue : (
            <span className="text-gray-700">{date.toLocaleDateString()}</span>
          );
        } catch {
          return stringValue;
        }
      case 'email':
        return (
          <a href={`mailto:${stringValue}`} className="text-blue-600 hover:text-blue-800 hover:underline">
            {stringValue}
          </a>
        );
      case 'url':
        return (
          <a href={stringValue} target="_blank" rel="noopener noreferrer" className="text-blue-600 hover:text-blue-800 hover:underline flex items-center">
            <span className="truncate">{stringValue}</span>
            <svg className="h-3 w-3 ml-1 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
            </svg>
          </a>
        );
      case 'boolean': {
        const lowerVal = stringValue.toLowerCase();
        const isTrue = ['true', 'yes', '1'].includes(lowerVal);
        return (
          <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
            isTrue ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
          }`}>
            {isTrue ? '✓ Yes' : '✗ No'}
          </span>
        );
      }
      default:
        return <span className="text-gray-900">{stringValue}</span>;
    }
  };

  return (
    <>
      <div className={`fixed inset-0 bg-black/20 backdrop-blur-sm overflow-y-auto h-full w-full z-50 flex items-start justify-center ${fullScreen ? 'p-0' : 'p-4 pt-2'}`}>
        <div className={`bg-white shadow-2xl w-full overflow-hidden flex flex-col border border-gray-200 ${
          fullScreen 
            ? 'h-full max-w-none' 
            : 'rounded-xl max-w-7xl max-h-[96vh]'
        }`}>
          {/* Enhanced Header */}
          <div className="flex justify-between items-center p-6 border-b border-gray-200 bg-gradient-to-r from-gray-50 to-gray-100">
            <div className="flex items-center flex-1">
              <div className="flex items-center justify-center w-12 h-12 rounded-lg bg-white shadow-sm border border-gray-200">
                {getFileIcon(dataset.file_type)}
              </div>
              <div className="ml-4 flex-1">
                <h2 className="text-xl font-bold text-gray-900">{dataset.original_filename}</h2>
                <div className="flex items-center flex-wrap gap-3 text-sm text-gray-600 mt-1">
                  <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                    {dataset.file_type.toUpperCase()}
                  </span>
                  {dataset.sheet_name && (
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                      Sheet: {dataset.sheet_name}
                    </span>
                  )}
                  <span className="flex items-center">
                    <TableCellsIcon className="h-4 w-4 mr-1" />
                    {dataResponse?.total_count.toLocaleString() || dataset.total_rows.toLocaleString()} rows
                  </span>
                  <span className="flex items-center">
                    <Squares2X2Icon className="h-4 w-4 mr-1" />
                    {visibleColumns.length}/{columnHeaders.length} columns
                  </span>
                  <span className="flex items-center">
                    <CalendarIcon className="h-4 w-4 mr-1" />
                    {new Date(dataset.created_at).toLocaleDateString()}
                  </span>
                  {dataset.uploaded_by && (
                    <span className="text-gray-500">by {dataset.uploaded_by}</span>
                  )}
                  {selectedRows.size > 0 && (
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                      <CheckCircleIcon className="h-3 w-3 mr-1" />
                      {selectedRows.size} selected
                    </span>
                  )}
                </div>
              </div>
            </div>
            
            {/* Enhanced Header Actions */}
            <div className="flex items-center space-x-1 ml-4">
              {/* Refresh Button */}
              <button
                onClick={() => refetch()}
                className="p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-white/80 transition-all duration-200"
                title="Refresh Data"
              >
                <ArrowPathIcon className="h-5 w-5" />
              </button>

              {/* View Mode Toggle */}
              <div className="flex items-center bg-white rounded-lg border border-gray-200 p-1">
                <button
                  onClick={() => setViewMode('table')}
                  className={`p-1.5 rounded-md transition-all duration-200 ${
                    viewMode === 'table' ? 'bg-blue-100 text-blue-700' : 'text-gray-500 hover:text-gray-700'
                  }`}
                  title="Table View"
                >
                  <TableCellsIcon className="h-4 w-4" />
                </button>
                <button
                  onClick={() => setViewMode('cards')}
                  className={`p-1.5 rounded-md transition-all duration-200 ${
                    viewMode === 'cards' ? 'bg-blue-100 text-blue-700' : 'text-gray-500 hover:text-gray-700'
                  }`}
                  title="Card View"
                >
                  <Squares2X2Icon className="h-4 w-4" />
                </button>
              </div>

              {/* Column Manager */}
              <button
                onClick={() => setShowColumnManager(!showColumnManager)}
                className={`p-2 rounded-lg transition-all duration-200 ${
                  showColumnManager ? 'bg-blue-100 text-blue-700' : 'text-gray-500 hover:text-gray-700 hover:bg-white/80'
                }`}
                title="Manage Columns"
              >
                <AdjustmentsHorizontalIcon className="h-5 w-5" />
              </button>

              {/* Statistics Toggle */}
              <button
                onClick={() => setShowStats(!showStats)}
                className={`p-2 rounded-lg transition-all duration-200 ${
                  showStats ? 'bg-blue-100 text-blue-700' : 'text-gray-500 hover:text-gray-700 hover:bg-white/80'
                }`}
                title="Toggle Statistics"
              >
                <ChartBarIcon className="h-5 w-5" />
              </button>

              {/* Saved Views */}
              <div className="relative group">
                <button
                  className="p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-white/80 transition-all duration-200"
                  title="Saved Views"
                >
                  <BookmarkIcon className="h-5 w-5" />
                </button>
                <div className="absolute right-0 top-full mt-2 bg-white border border-gray-200 rounded-lg shadow-xl opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-20 min-w-64">
                  <div className="p-3">
                    <div className="flex items-center justify-between mb-2">
                      <h3 className="text-sm font-medium text-gray-900">Saved Views</h3>
                      <button
                        onClick={() => setShowSaveViewDialog(true)}
                        className="text-xs text-blue-600 hover:text-blue-800"
                      >
                        Save Current
                      </button>
                    </div>
                    {savedViews.length > 0 ? (
                      <div className="space-y-1 max-h-40 overflow-y-auto">
                        {savedViews.map(view => (
                          <div key={view.id} className="flex items-center justify-between p-2 rounded-md hover:bg-gray-50">
                            <button
                              onClick={() => loadView(view)}
                              className="flex-1 text-left text-sm text-gray-700 hover:text-gray-900"
                            >
                              {view.name}
                            </button>
                            <button
                              onClick={() => deleteView(view.id)}
                              className="text-red-500 hover:text-red-700 ml-2"
                            >
                              <XMarkIcon className="h-3 w-3" />
                            </button>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <p className="text-xs text-gray-500 py-2">No saved views yet</p>
                    )}
                  </div>
                </div>
              </div>
              
              {/* Export Menu */}
              <div className="relative group">
                <button
                  className="p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-white/80 transition-all duration-200"
                  title="Export Options"
                >
                  <ArrowDownTrayIcon className="h-5 w-5" />
                </button>
                <div className="absolute right-0 top-full mt-2 bg-white border border-gray-200 rounded-lg shadow-xl opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-20">
                  <div className="p-2 min-w-48">
                    <button
                      onClick={() => exportData('csv')}
                      className="block w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-md"
                    >
                      Export Current Page (CSV)
                    </button>
                    <button
                      onClick={() => exportData('json')}
                      className="block w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-md"
                    >
                      Export Current Page (JSON)
                    </button>
                    {selectedRows.size > 0 && (
                      <>
                        <hr className="my-2" />
                        <button
                          onClick={() => handleBulkOperation('export')}
                          className="block w-full text-left px-3 py-2 text-sm text-blue-700 hover:bg-blue-50 rounded-md"
                        >
                          Export Selected Rows
                        </button>
                        <button
                          onClick={() => handleBulkOperation('copy')}
                          className="block w-full text-left px-3 py-2 text-sm text-blue-700 hover:bg-blue-50 rounded-md"
                        >
                          Copy Selected Rows
                        </button>
                      </>
                    )}
                    <hr className="my-2" />
                    <button
                      onClick={handlePrint}
                      className="block w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-md"
                    >
                      <div className="flex items-center">
                        <PrinterIcon className="h-4 w-4 mr-2" />
                        Print View
                      </div>
                    </button>
                  </div>
                </div>
              </div>

              {/* Settings Menu */}
              <div className="relative group">
                <button
                  className="p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-white/80 transition-all duration-200"
                  title="Settings"
                >
                  <CogIcon className="h-5 w-5" />
                </button>
                <div className="absolute right-0 top-full mt-2 bg-white border border-gray-200 rounded-lg shadow-xl opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-20">
                  <div className="p-3 min-w-56">
                    <h3 className="text-sm font-medium text-gray-900 mb-2">Settings</h3>
                    <div className="space-y-3">
                      <div>
                        <label className="flex items-center">
                          <input
                            type="checkbox"
                            checked={dataQualityCheck}
                            onChange={(e) => setDataQualityCheck(e.target.checked)}
                            className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                          />
                          <span className="ml-2 text-sm text-gray-700">Show data quality indicators</span>
                        </label>
                      </div>
                      <div>
                        <label className="block text-sm text-gray-700 mb-1">Auto-refresh (seconds)</label>
                        <select
                          value={refreshInterval || ''}
                          onChange={(e) => setRefreshInterval(e.target.value ? Number(e.target.value) : null)}
                          className="block w-full text-xs border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
                        >
                          <option value="">Off</option>
                          <option value="30">30s</option>
                          <option value="60">1min</option>
                          <option value="300">5min</option>
                        </select>
                      </div>
                    </div>
                    <hr className="my-3" />
                    <button
                      onClick={() => setShowKeyboardShortcuts(true)}
                      className="flex items-center w-full px-2 py-1 text-sm text-gray-700 hover:bg-gray-100 rounded-md"
                    >
                      <CommandLineIcon className="h-4 w-4 mr-2" />
                      Keyboard Shortcuts
                    </button>
                  </div>
                </div>
              </div>
              
              <button
                onClick={() => setFullScreen(!fullScreen)}
                className="p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-white/80 transition-all duration-200"
                title={fullScreen ? "Exit Fullscreen" : "Enter Fullscreen"}
              >
                <ArrowsPointingOutIcon className="h-5 w-5" />
              </button>
              
              <button
                onClick={onClose}
                className="p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-white/80 transition-all duration-200"
                title="Close (Esc)"
              >
                <XMarkIcon className="h-5 w-5" />
              </button>
            </div>
          </div>

          {/* Column Manager Panel */}
          {showColumnManager && (
            <div className="border-b border-gray-200 bg-gray-50 p-4">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-sm font-medium text-gray-900">Column Management</h3>
                <div className="flex items-center space-x-2">
                  <span className="text-xs text-gray-500">
                    {visibleColumns.length} of {columnHeaders.length} visible
                  </span>
                </div>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {columnHeaders.map((header, index) => {
                  const stats = columnStats[header];
                  const isVisible = !hiddenColumns.has(header);
                  const isPinned = pinnedColumns.has(header);
                  
                  return (
                    <div key={`column-${header}-${index}`} className={`p-3 border rounded-lg transition-all duration-200 ${
                      isVisible ? 'bg-white border-gray-200 shadow-sm' : 'bg-gray-100 border-gray-300'
                    }`}>
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center flex-1 min-w-0">
                          {stats && getDataTypeIcon(stats.type)}
                          <span className="ml-2 text-xs font-medium text-gray-900 truncate" title={header}>
                            {header}
                          </span>
                        </div>
                        <div className="flex items-center space-x-1 ml-2">
                          {dataQualityCheck && stats && (
                            <div className={`w-2 h-2 rounded-full ${getQualityIndicator(stats.qualityScore).bg}`} 
                                 title={`Quality Score: ${stats.qualityScore}%`} />
                          )}
                          <button
                            onClick={() => toggleColumnPin(header)}
                            className={`p-1 rounded transition-colors ${
                              isPinned ? 'text-blue-600 hover:text-blue-800' : 'text-gray-400 hover:text-gray-600'
                            }`}
                            title={isPinned ? 'Unpin column' : 'Pin column'}
                          >
                            <svg className="h-3 w-3" fill="currentColor" viewBox="0 0 20 20">
                              <path fillRule="evenodd" d="M10 2a8 8 0 100 16 8 8 0 000-16zm3.857 6.857a.5.5 0 01-.708 0L10 5.707 6.851 8.857a.5.5 0 01-.708-.708L9.293 5 6.143 1.85a.5.5 0 01.708-.708L10 4.293l3.149-3.151a.5.5 0 01.708.708L10.707 5l3.15 3.149a.5.5 0 010 .708z" clipRule="evenodd" />
                            </svg>
                          </button>
                          <button
                            onClick={() => toggleColumnVisibility(header)}
                            className={`p-1 rounded transition-colors ${
                              isVisible ? 'text-green-600 hover:text-green-800' : 'text-red-600 hover:text-red-800'
                            }`}
                            title={isVisible ? 'Hide column' : 'Show column'}
                          >
                            {isVisible ? <EyeIcon className="h-3 w-3" /> : <EyeSlashIcon className="h-3 w-3" />}
                          </button>
                        </div>
                      </div>
                      {stats && (
                        <div className="text-xs text-gray-500 space-y-1">
                          <div className="flex justify-between">
                            <span>Type:</span>
                            <span className="capitalize font-medium">{stats.type}</span>
                          </div>
                          {dataQualityCheck && (
                            <div className="flex justify-between">
                              <span>Quality:</span>
                              <span className={`font-medium ${
                                stats.qualityScore >= 80 ? 'text-green-600' : 
                                stats.qualityScore >= 60 ? 'text-yellow-600' : 'text-red-600'
                              }`}>
                                {stats.qualityScore}%
                              </span>
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Enhanced Statistics Panel */}
          {showStats && columnHeaders.length > 0 && (
            <div className="border-b border-gray-200 bg-gradient-to-r from-blue-50 to-indigo-50 p-4">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold text-gray-900">Data Insights & Statistics</h3>
                <div className="flex items-center space-x-4 text-sm text-gray-600">
                  <span className="flex items-center">
                    <CheckCircleIcon className="h-4 w-4 text-green-500 mr-1" />
                    High Quality: {Object.values(columnStats).filter(s => s.qualityScore >= 80).length}
                  </span>
                  <span className="flex items-center">
                    <InformationCircleIcon className="h-4 w-4 text-yellow-500 mr-1" />
                    Medium Quality: {Object.values(columnStats).filter(s => s.qualityScore >= 60 && s.qualityScore < 80).length}
                  </span>
                  <span className="flex items-center">
                    <ExclamationTriangleIcon className="h-4 w-4 text-red-500 mr-1" />
                    Low Quality: {Object.values(columnStats).filter(s => s.qualityScore < 60).length}
                  </span>
                </div>
              </div>
              
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                {columnHeaders.slice(0, 12).map((header, index) => {
                  const stats = columnStats[header];
                  if (!stats) return null;
                  
                  const qualityInfo = getQualityIndicator(stats.qualityScore);
                  
                  return (
                    <div key={`stats-${header}-${index}`} className="bg-white border border-gray-200 rounded-xl p-4 shadow-sm hover:shadow-md transition-all duration-200">
                      <div className="flex items-center justify-between mb-3">
                        <div className="flex items-center min-w-0 flex-1">
                          <div className="flex-shrink-0">
                            {getDataTypeIcon(stats.type)}
                          </div>
                          <h4 className="ml-2 text-sm font-semibold text-gray-900 truncate" title={header}>
                            {header}
                          </h4>
                        </div>
                        {dataQualityCheck && (
                          <div className={`flex items-center px-2 py-1 rounded-full ${qualityInfo.bg}`}>
                            <qualityInfo.icon className={`h-3 w-3 ${qualityInfo.color}`} />
                            <span className={`ml-1 text-xs font-medium ${qualityInfo.color}`}>
                              {stats.qualityScore}%
                            </span>
                          </div>
                        )}
                      </div>
                      
                      <div className="space-y-2">
                        <div className="grid grid-cols-2 gap-2 text-xs">
                          <div className="flex flex-col">
                            <span className="text-gray-500">Non-empty</span>
                            <span className="font-semibold text-gray-900">{stats.nonEmptyCount.toLocaleString()}</span>
                          </div>
                          <div className="flex flex-col">
                            <span className="text-gray-500">Unique</span>
                            <span className="font-semibold text-gray-900">{stats.uniqueCount.toLocaleString()}</span>
                          </div>
                        </div>
                        
                        {stats.nullCount > 0 && (
                          <div className="flex justify-between items-center p-2 bg-red-50 rounded-md">
                            <span className="text-xs text-red-700">Missing values</span>
                            <span className="text-xs font-semibold text-red-800">{stats.nullCount.toLocaleString()}</span>
                          </div>
                        )}
                        
                        {stats.type === 'number' && (
                          <div className="space-y-1 p-2 bg-blue-50 rounded-md">
                            {stats.min !== undefined && (
                              <div className="flex justify-between text-xs">
                                <span className="text-blue-700">Min:</span>
                                <span className="font-semibold text-blue-900">{Number(stats.min).toLocaleString()}</span>
                              </div>
                            )}
                            {stats.max !== undefined && (
                              <div className="flex justify-between text-xs">
                                <span className="text-blue-700">Max:</span>
                                <span className="font-semibold text-blue-900">{Number(stats.max).toLocaleString()}</span>
                              </div>
                            )}
                            {stats.avg !== undefined && (
                              <div className="flex justify-between text-xs">
                                <span className="text-blue-700">Avg:</span>
                                <span className="font-semibold text-blue-900">{stats.avg.toFixed(2)}</span>
                              </div>
                            )}
                          </div>
                        )}
                        
                        <div className="text-xs text-gray-500 capitalize bg-gray-50 px-2 py-1 rounded">
                          Type: {stats.type}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
              
              {columnHeaders.length > 12 && (
                <div className="mt-4 text-center">
                  <p className="text-sm text-gray-600">
                    Showing statistics for first 12 columns
                  </p>
                  <button
                    onClick={() => setShowColumnManager(true)}
                    className="mt-2 text-sm text-blue-600 hover:text-blue-800 font-medium"
                  >
                    View all columns in Column Manager
                  </button>
                </div>
              )}
            </div>
          )}

          {/* Enhanced Search and Filter Controls */}
          <div className="p-4 border-b border-gray-200 bg-gradient-to-r from-gray-50 to-gray-100">
            <div className="flex flex-col lg:flex-row lg:items-center space-y-4 lg:space-y-0 lg:space-x-4">
              {/* Search */}
              <div className="flex-1 relative">
                <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                  <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
                </div>
                <input
                  ref={searchInputRef}
                  type="text"
                  value={searchTerm}
                  onChange={(e) => {
                    setSearchTerm(e.target.value);
                    setCurrentPage(0);
                  }}
                  placeholder="Search within this dataset... (Ctrl+F)"
                  className="block w-full pl-10 pr-4 py-3 border border-gray-300 rounded-lg bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm shadow-sm transition-all duration-200"
                />
                {searchTerm && (
                  <button
                    onClick={() => {
                      setSearchTerm('');
                      setCurrentPage(0);
                    }}
                    className="absolute inset-y-0 right-0 pr-3 flex items-center"
                  >
                    <XMarkIcon className="h-4 w-4 text-gray-400 hover:text-gray-600" />
                  </button>
                )}
              </div>

              {/* Controls */}
              <div className="flex items-center space-x-3">
                {/* Rows per page selector */}
                <div className="flex items-center space-x-2">
                  <label htmlFor="rows-per-page" className="text-sm text-gray-700 whitespace-nowrap">Show:</label>
                  <select
                    id="rows-per-page"
                    value={rowsPerPage}
                    onChange={(e) => {
                      setRowsPerPage(Number(e.target.value));
                      setCurrentPage(0);
                    }}
                    className="border border-gray-300 rounded-lg text-sm py-2 px-3 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 shadow-sm"
                  >
                    <option value={25}>25</option>
                    <option value={50}>50</option>
                    <option value={100}>100</option>
                    <option value={200}>200</option>
                  </select>
                </div>

                {/* Basic Filter Toggle */}
                <button
                  type="button"
                  onClick={() => setShowFilters(!showFilters)}
                  className={`inline-flex items-center px-4 py-2 border rounded-lg shadow-sm text-sm font-medium transition-all duration-200 ${
                    showFilters || Object.keys(columnFilters).length > 0
                      ? 'text-blue-700 bg-blue-50 border-blue-300 hover:bg-blue-100'
                      : 'text-gray-700 bg-white border-gray-300 hover:bg-gray-50'
                  } focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500`}
                >
                  <FunnelIcon className="h-4 w-4 mr-2" />
                  Basic Filters
                  {Object.keys(columnFilters).length > 0 && (
                    <span className="ml-2 inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                      {Object.keys(columnFilters).length}
                    </span>
                  )}
                </button>

                {/* Advanced Filter Toggle */}
                <button
                  type="button"
                  onClick={() => setShowAdvancedFilters(!showAdvancedFilters)}
                  className={`inline-flex items-center px-4 py-2 border rounded-lg shadow-sm text-sm font-medium transition-all duration-200 ${
                    showAdvancedFilters || advancedFilters.length > 0
                      ? 'text-purple-700 bg-purple-50 border-purple-300 hover:bg-purple-100'
                      : 'text-gray-700 bg-white border-gray-300 hover:bg-gray-50'
                  } focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500`}
                >
                  <AdjustmentsHorizontalIcon className="h-4 w-4 mr-2" />
                  Advanced
                  {advancedFilters.length > 0 && (
                    <span className="ml-2 inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                      {advancedFilters.length}
                    </span>
                  )}
                </button>

                {hasActiveFilters && (
                  <button
                    type="button"
                    onClick={clearFilters}
                    className="inline-flex items-center px-3 py-2 text-sm text-red-600 hover:text-red-800 hover:bg-red-50 rounded-lg transition-all duration-200"
                  >
                    <XMarkIcon className="h-4 w-4 mr-1" />
                    Clear all
                  </button>
                )}
              </div>
            </div>

            {/* Basic Column Filters */}
            {showFilters && (
              <div className="mt-6 bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-sm font-semibold text-gray-900">Basic Column Filters</h3>
                  <span className="text-xs text-gray-500">Filter by text matching</span>
                </div>
                {visibleColumns.length > 0 ? (
                  <>
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                      {visibleColumns.slice(0, 8).map((header, index) => {
                        const stats = columnStats[header];
                        return (
                          <div key={`filter-${header}-${index}`} className="space-y-2">
                            <div className="flex items-center justify-between">
                              <label htmlFor={`filter-${header}`} className="block text-xs font-medium text-gray-700">
                                {header}
                              </label>
                              {stats && (
                                <div className="flex items-center space-x-1">
                                  {getDataTypeIcon(stats.type)}
                                  <span className="text-xs text-gray-500 capitalize">{stats.type}</span>
                                </div>
                              )}
                            </div>
                            <input
                              type="text"
                              id={`filter-${header}`}
                              value={columnFilters[header] || ''}
                              onChange={(e) => handleColumnFilterChange(header, e.target.value)}
                              placeholder={`Filter ${header}...`}
                              className="block w-full border-gray-300 rounded-lg shadow-sm focus:ring-blue-500 focus:border-blue-500 text-sm"
                            />
                          </div>
                        );
                      })}
                    </div>
                    {visibleColumns.length > 8 && (
                      <p className="text-xs text-gray-500 mt-4 bg-gray-50 p-2 rounded">
                        Showing first 8 visible columns. Use global search above to filter across all columns.
                      </p>
                    )}
                  </>
                ) : (
                  <p className="text-sm text-gray-500 text-center py-4">No visible columns available for filtering.</p>
                )}
              </div>
            )}

            {/* Advanced Filters */}
            {showAdvancedFilters && (
              <div className="mt-6 bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-sm font-semibold text-gray-900">Advanced Filters</h3>
                  <button
                    onClick={addAdvancedFilter}
                    className="inline-flex items-center px-3 py-1.5 text-xs font-medium text-purple-700 bg-purple-100 hover:bg-purple-200 rounded-md transition-colors"
                  >
                    + Add Filter
                  </button>
                </div>

                {advancedFilters.length > 0 ? (
                  <div className="space-y-4">
                    {advancedFilters.map((filter, index) => (
                      <div key={index} className="flex items-center space-x-3 p-3 bg-gray-50 rounded-lg">
                        <select
                          value={filter.column}
                          onChange={(e) => updateAdvancedFilter(index, { column: e.target.value })}
                          className="flex-1 border-gray-300 rounded-md text-sm focus:ring-purple-500 focus:border-purple-500"
                        >
                          {visibleColumns.map(header => (
                            <option key={header} value={header}>{header}</option>
                          ))}
                        </select>

                        <select
                          value={filter.operator}
                          onChange={(e) => updateAdvancedFilter(index, { operator: e.target.value as any })}
                          className="border-gray-300 rounded-md text-sm focus:ring-purple-500 focus:border-purple-500"
                        >
                          <option value="contains">Contains</option>
                          <option value="equals">Equals</option>
                          <option value="starts_with">Starts with</option>
                          <option value="ends_with">Ends with</option>
                          <option value="greater_than">Greater than</option>
                          <option value="less_than">Less than</option>
                          <option value="between">Between</option>
                          <option value="is_empty">Is empty</option>
                          <option value="is_not_empty">Is not empty</option>
                        </select>

                        {filter.operator !== 'is_empty' && filter.operator !== 'is_not_empty' && (
                          <input
                            type="text"
                            value={filter.value}
                            onChange={(e) => updateAdvancedFilter(index, { value: e.target.value })}
                            placeholder="Value..."
                            className="flex-1 border-gray-300 rounded-md text-sm focus:ring-purple-500 focus:border-purple-500"
                          />
                        )}

                        {filter.operator === 'between' && (
                          <input
                            type="text"
                            value={filter.value2 || ''}
                            onChange={(e) => updateAdvancedFilter(index, { value2: e.target.value })}
                            placeholder="and..."
                            className="flex-1 border-gray-300 rounded-md text-sm focus:ring-purple-500 focus:border-purple-500"
                          />
                        )}

                        <button
                          onClick={() => removeAdvancedFilter(index)}
                          className="p-2 text-red-500 hover:text-red-700 hover:bg-red-50 rounded-md transition-colors"
                        >
                          <XMarkIcon className="h-4 w-4" />
                        </button>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-gray-500 text-center py-4">
                    No advanced filters yet. Click "Add Filter" to create complex filtering rules.
                  </p>
                )}
              </div>
            )}
          </div>

          {/* Enhanced Data Display */}
          <div className="flex-1 overflow-auto" ref={tableRef}>
            {error && (
              <div className="p-8 text-center">
                <div className="max-w-md mx-auto">
                  <div className="w-16 h-16 mx-auto mb-4 bg-red-100 rounded-full flex items-center justify-center">
                    <ExclamationTriangleIcon className="h-8 w-8 text-red-600" />
                  </div>
                  <h3 className="text-lg font-semibold text-gray-900 mb-2">Error Loading Data</h3>
                  <p className="text-gray-600 mb-4">Failed to load spreadsheet data. Please try again.</p>
                  <button
                    onClick={() => refetch()}
                    className="inline-flex items-center px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                  >
                    <ArrowPathIcon className="h-4 w-4 mr-2" />
                    Retry
                  </button>
                </div>
              </div>
            )}

            {isLoading && (
              <div className="flex flex-col items-center justify-center py-12">
                <div className="relative">
                  <div className="animate-spin rounded-full h-12 w-12 border-4 border-gray-200 border-t-blue-600"></div>
                  <div className="absolute inset-0 flex items-center justify-center">
                    <TableCellsIcon className="h-6 w-6 text-blue-600" />
                  </div>
                </div>
                <p className="mt-4 text-gray-600 font-medium">Loading data...</p>
                <p className="text-sm text-gray-500">Processing {dataset.total_rows.toLocaleString()} rows</p>
              </div>
            )}

            {dataResponse && (
              <>
                {/* Enhanced Results Info */}
                <div className="px-6 py-4 bg-gradient-to-r from-gray-50 to-gray-100 border-b border-gray-200">
                  <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between space-y-2 sm:space-y-0">
                    <div className="flex items-center space-x-4">
                      <span className="text-sm text-gray-700">
                        Showing <span className="font-semibold text-gray-900">{startRow}-{endRow}</span> of{' '}
                        <span className="font-semibold text-blue-600">{dataResponse.total_count.toLocaleString()}</span> rows
                        {hasActiveFilters && (
                          <span className="ml-2 inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                            Filtered
                          </span>
                        )}
                      </span>
                      {selectedRows.size > 0 && (
                        <div className="flex items-center space-x-2">
                          <span className="text-sm text-blue-700 font-medium">
                            {selectedRows.size} selected
                          </span>
                          <div className="flex items-center space-x-1">
                            <button
                              onClick={() => handleBulkOperation('copy')}
                              className="p-1 text-blue-600 hover:text-blue-800 hover:bg-blue-50 rounded"
                              title="Copy selected rows"
                            >
                              <ClipboardDocumentCheckIcon className="h-4 w-4" />
                            </button>
                            <button
                              onClick={() => handleBulkOperation('export')}
                              className="p-1 text-blue-600 hover:text-blue-800 hover:bg-blue-50 rounded"
                              title="Export selected rows"
                            >
                              <ArrowDownTrayIcon className="h-4 w-4" />
                            </button>
                          </div>
                        </div>
                      )}
                    </div>
                    {totalPages > 1 && (
                      <div className="flex items-center space-x-2 text-sm text-gray-600">
                        <span>Page {currentPage + 1} of {totalPages}</span>
                        <div className="w-32 bg-gray-200 rounded-full h-1.5">
                          <div 
                            className="bg-blue-600 h-1.5 rounded-full transition-all duration-300" 
                            style={{ width: `${((currentPage + 1) / totalPages) * 100}%` }}
                          />
                        </div>
                      </div>
                    )}
                  </div>
                </div>

                {/* Data Display */}
                {dataResponse.records.length > 0 && visibleColumns.length > 0 ? (
                  viewMode === 'table' ? (
                    <div className="overflow-x-auto">
                      <table className="min-w-full">
                        <thead className="bg-gradient-to-r from-gray-50 to-gray-100 sticky top-0 z-10">
                          <tr>
                            <th className="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider border-r border-gray-300 w-16 bg-gray-50">
                              <input
                                type="checkbox"
                                checked={selectedRows.size === dataResponse.records.length && dataResponse.records.length > 0}
                                onChange={handleSelectAll}
                                className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                              />
                            </th>
                            <th className="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider border-r border-gray-300 bg-gray-50 sticky left-0 z-20">
                              Row #
                            </th>
                            {orderedColumns.map((header, index) => {
                              const stats = columnStats[header];
                              const isPinned = pinnedColumns.has(header);
                              return (
                                <th
                                  key={`header-${header}-${index}`}
                                  className={`px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider border-r border-gray-300 min-w-32 cursor-pointer hover:bg-gray-200 transition-all duration-200 ${
                                    isPinned ? 'bg-blue-50 sticky z-10' : ''
                                  }`}
                                  style={isPinned ? { left: `${80 + (pinnedColumnsArray.indexOf(header) * 128)}px` } : {}}
                                  onClick={() => handleSort(header)}
                                >
                                  <div className="flex items-center justify-between group">
                                    <div className="flex items-center space-x-2 min-w-0">
                                      {stats && (
                                        <div className="flex items-center space-x-1">
                                          {getDataTypeIcon(stats.type)}
                                          {dataQualityCheck && (
                                            <div className={`w-1.5 h-1.5 rounded-full ${getQualityIndicator(stats.qualityScore).bg}`} />
                                          )}
                                        </div>
                                      )}
                                      <span className="truncate font-medium" title={header}>
                                        {header}
                                      </span>
                                      {isPinned && (
                                        <div className="w-1 h-1 bg-blue-500 rounded-full" />
                                      )}
                                    </div>
                                    <div className="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                      {sortConfig?.column === header && (
                                        sortConfig.direction === 'asc' ? (
                                          <ArrowUpIcon className="h-3 w-3 text-blue-600" />
                                        ) : (
                                          <ArrowDownIcon className="h-3 w-3 text-blue-600" />
                                        )
                                      )}
                                      <button
                                        onClick={(e) => {
                                          e.stopPropagation();
                                          toggleColumnPin(header);
                                        }}
                                        className={`p-0.5 rounded ${isPinned ? 'text-blue-600' : 'text-gray-400'} hover:text-blue-600`}
                                        title={isPinned ? 'Unpin column' : 'Pin column'}
                                      >
                                        <svg className="h-3 w-3" fill="currentColor" viewBox="0 0 20 20">
                                          <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
                                        </svg>
                                      </button>
                                    </div>
                                  </div>
                                </th>
                              );
                            })}
                          </tr>
                        </thead>
                        <tbody className="bg-white">
                          {sortedRecords.map((record, index) => (
                            <tr 
                              key={record.id} 
                              className={`transition-all duration-200 border-b border-gray-100 ${
                                index % 2 === 0 ? 'bg-white' : 'bg-gray-50/50'
                              } ${
                                selectedRows.has(record.id) ? 'bg-blue-50 border-blue-200' : ''
                              } hover:bg-blue-50/80 hover:border-blue-200`}
                            >
                              <td className="px-4 py-3 whitespace-nowrap border-r border-gray-200">
                                <input
                                  type="checkbox"
                                  checked={selectedRows.has(record.id)}
                                  onChange={() => handleRowSelect(record.id)}
                                  className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                />
                              </td>
                              <td className="px-4 py-3 whitespace-nowrap text-sm font-medium text-gray-900 border-r border-gray-200 bg-gray-50 sticky left-0 z-10">
                                {record.row_number}
                              </td>
                              {orderedColumns.map((header, colIndex) => {
                                const value = record.row_data[header];
                                const stats = columnStats[header];
                                const formattedValue = formatCellValue(value, stats?.type || 'text');
                                const isPinned = pinnedColumns.has(header);
                                
                                return (
                                  <td
                                    key={`cell-${header}-${colIndex}`}
                                    className={`px-4 py-3 text-sm border-r border-gray-200 max-w-64 ${
                                      isPinned ? 'bg-blue-50/30 sticky z-10' : ''
                                    }`}
                                    style={isPinned ? { left: `${80 + (pinnedColumnsArray.indexOf(header) * 128)}px` } : {}}
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
                    /* Card View */
                    <div className="p-6">
                      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                        {sortedRecords.map((record, _recordIndex) => (
                          <div 
                            key={record.id}
                            className={`bg-white border rounded-xl p-4 shadow-sm hover:shadow-md transition-all duration-200 ${
                              selectedRows.has(record.id) ? 'border-blue-300 bg-blue-50' : 'border-gray-200'
                            }`}
                          >
                            <div className="flex items-center justify-between mb-3">
                              <span className="text-sm font-medium text-gray-500">Row {record.row_number}</span>
                              <input
                                type="checkbox"
                                checked={selectedRows.has(record.id)}
                                onChange={() => handleRowSelect(record.id)}
                                className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                              />
                            </div>
                            <div className="space-y-2">
                              {visibleColumns.slice(0, 6).map((header, _index) => {
                                const value = record.row_data[header];
                                const stats = columnStats[header];
                                const formattedValue = formatCellValue(value, stats?.type || 'text');
                                
                                return (
                                  <div key={`card-${header}-${_index}`} className="flex items-start justify-between">
                                    <div className="flex items-center space-x-1 min-w-0">
                                      {stats && getDataTypeIcon(stats.type)}
                                      <span className="text-xs font-medium text-gray-600 truncate">
                                        {header}:
                                      </span>
                                    </div>
                                    <div className="text-xs text-gray-900 ml-2 truncate max-w-32" title={String(value || '')}>
                                      {formattedValue}
                                    </div>
                                  </div>
                                );
                              })}
                              {visibleColumns.length > 6 && (
                                <div className="text-xs text-gray-500 pt-1 border-t border-gray-100">
                                  +{visibleColumns.length - 6} more columns
                                </div>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )
                ) : (
                  <div className="p-12 text-center">
                    <div className="max-w-md mx-auto">
                      <div className="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center">
                        <DocumentTextIcon className="h-8 w-8 text-gray-400" />
                      </div>
                      {dataResponse && dataResponse.records.length > 0 && visibleColumns.length === 0 ? (
                        <>
                          <h3 className="text-lg font-semibold text-gray-900 mb-2">No visible columns</h3>
                          <p className="text-gray-600 mb-4">
                            All columns are currently hidden. Use the Column Manager to show columns.
                          </p>
                          <button
                            onClick={() => setShowColumnManager(true)}
                            className="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                          >
                            <AdjustmentsHorizontalIcon className="h-4 w-4 mr-2" />
                            Manage Columns
                          </button>
                        </>
                      ) : (
                        <>
                          <h3 className="text-lg font-semibold text-gray-900 mb-2">No data found</h3>
                          <p className="text-gray-600 mb-4">
                            {hasActiveFilters
                              ? 'No rows match your current filters. Try adjusting your search criteria.'
                              : 'This dataset appears to be empty.'}
                          </p>
                          {hasActiveFilters && (
                            <button
                              onClick={clearFilters}
                              className="inline-flex items-center px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                            >
                              <XMarkIcon className="h-4 w-4 mr-2" />
                              Clear Filters
                            </button>
                          )}
                        </>
                      )}
                    </div>
                  </div>
                )}
              </>
            )}
          </div>

          {/* Enhanced Pagination */}
          {dataResponse && dataResponse.total_count > 0 && (
            <div className="flex items-center justify-between border-t border-gray-200 bg-white px-6 py-4">
              <div className="flex flex-1 justify-between sm:hidden">
                <button
                  onClick={goToPrevPage}
                  disabled={currentPage === 0}
                  className="relative inline-flex items-center rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200"
                >
                  <ChevronLeftIcon className="h-4 w-4 mr-1" />
                  Previous
                </button>
                <button
                  onClick={goToNextPage}
                  disabled={currentPage === totalPages - 1}
                  className="relative inline-flex items-center rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200"
                >
                  Next
                  <ChevronRightIcon className="h-4 w-4 ml-1" />
                </button>
              </div>
              <div className="hidden sm:flex sm:flex-1 sm:items-center sm:justify-between">
                <div className="flex items-center space-x-4">
                  <p className="text-sm text-gray-700">
                    Showing <span className="font-medium">{startRow}</span> to{' '}
                    <span className="font-medium">{endRow}</span> of{' '}
                    <span className="font-medium">{dataResponse?.total_count.toLocaleString()}</span> results
                  </p>
                  {refreshInterval && (
                    <div className="flex items-center text-xs text-gray-500">
                      <ArrowPathIcon className="h-3 w-3 mr-1 animate-spin" />
                      Auto-refresh: {refreshInterval}s
                    </div>
                  )}
                </div>
                <div className="flex items-center space-x-2">
                  <nav className="isolate inline-flex -space-x-px rounded-lg shadow-sm" aria-label="Pagination">
                    <button
                      onClick={goToPrevPage}
                      disabled={currentPage === 0}
                      className="relative inline-flex items-center rounded-l-lg px-3 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200"
                    >
                      <ChevronLeftIcon className="h-4 w-4" />
                    </button>
                    <span className="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-900 ring-1 ring-inset ring-gray-300 bg-white">
                      Page {currentPage + 1} of {totalPages}
                    </span>
                    <button
                      onClick={goToNextPage}
                      disabled={currentPage === totalPages - 1}
                      className="relative inline-flex items-center rounded-r-lg px-3 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200"
                    >
                      <ChevronRightIcon className="h-4 w-4" />
                    </button>
                  </nav>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Modal Dialogs */}
      {/* Keyboard Shortcuts Modal */}
      {showKeyboardShortcuts && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-white rounded-xl shadow-2xl max-w-md w-full max-h-[80vh] overflow-y-auto">
            <div className="p-6">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold text-gray-900">Keyboard Shortcuts</h3>
                <button
                  onClick={() => setShowKeyboardShortcuts(false)}
                  className="text-gray-400 hover:text-gray-600"
                >
                  <XMarkIcon className="h-5 w-5" />
                </button>
              </div>
              <div className="space-y-4">
                <div className="grid grid-cols-1 gap-3">
                  <div className="flex justify-between items-center py-2 border-b border-gray-100">
                    <span className="text-sm text-gray-700">Search</span>
                    <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 rounded">Ctrl+F</kbd>
                  </div>
                  <div className="flex justify-between items-center py-2 border-b border-gray-100">
                    <span className="text-sm text-gray-700">Refresh Data</span>
                    <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 rounded">Ctrl+R</kbd>
                  </div>
                  <div className="flex justify-between items-center py-2 border-b border-gray-100">
                    <span className="text-sm text-gray-700">Print</span>
                    <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 rounded">Ctrl+P</kbd>
                  </div>
                  <div className="flex justify-between items-center py-2 border-b border-gray-100">
                    <span className="text-sm text-gray-700">Save View</span>
                    <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 rounded">Ctrl+S</kbd>
                  </div>
                  <div className="flex justify-between items-center py-2 border-b border-gray-100">
                    <span className="text-sm text-gray-700">Help</span>
                    <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 rounded">Ctrl+H</kbd>
                  </div>
                  <div className="flex justify-between items-center py-2">
                    <span className="text-sm text-gray-700">Close</span>
                    <kbd className="px-2 py-1 text-xs font-semibold text-gray-800 bg-gray-100 rounded">Esc</kbd>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Save View Modal */}
      {showSaveViewDialog && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-white rounded-xl shadow-2xl max-w-md w-full">
            <div className="p-6">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold text-gray-900">Save Current View</h3>
                <button
                  onClick={() => setShowSaveViewDialog(false)}
                  className="text-gray-400 hover:text-gray-600"
                >
                  <XMarkIcon className="h-5 w-5" />
                </button>
              </div>
              <div className="space-y-4">
                <div>
                  <label htmlFor="view-name" className="block text-sm font-medium text-gray-700 mb-1">
                    View Name
                  </label>
                  <input
                    type="text"
                    id="view-name"
                    value={newViewName}
                    onChange={(e) => setNewViewName(e.target.value)}
                    placeholder="Enter a name for this view..."
                    className="block w-full border-gray-300 rounded-lg focus:ring-blue-500 focus:border-blue-500"
                    autoFocus
                  />
                </div>
                <div className="text-xs text-gray-500 bg-gray-50 p-3 rounded-lg">
                  <p className="font-medium mb-1">This view will save:</p>
                  <ul className="space-y-1">
                    <li>• Current filters and search terms</li>
                    <li>• Column visibility and sorting</li>
                    <li>• Rows per page setting</li>
                  </ul>
                </div>
                <div className="flex justify-end space-x-3">
                  <button
                    onClick={() => setShowSaveViewDialog(false)}
                    className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={saveCurrentView}
                    disabled={!newViewName.trim()}
                    className="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
                  >
                    Save View
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
} 
