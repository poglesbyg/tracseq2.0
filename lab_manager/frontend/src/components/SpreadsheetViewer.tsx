import { useState } from 'react';
import { ChevronLeftIcon, ChevronRightIcon, BeakerIcon } from '@heroicons/react/24/outline';

interface SheetData {
  name: string;
  headers: string[];
  rows: string[][];
  total_rows: number;
  total_columns: number;
}

  interface SpreadsheetData {
    sheet_names: string[];
    sheets: SheetData[];
  }

interface Template {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  metadata: Record<string, any>;
}

  interface SpreadsheetViewerProps {
    template: Template;
    data: SpreadsheetData;
    onClose: () => void;
    onCreateSamples?: (template: Template, data: SpreadsheetData) => void;
  }

export default function SpreadsheetViewer({ template, data, onClose, onCreateSamples }: SpreadsheetViewerProps) {
  const [activeSheetIndex, setActiveSheetIndex] = useState(0);
  const [currentPage, setCurrentPage] = useState(0);
  const rowsPerPage = 50;

  // Add null/undefined checks for data and sheets
  if (!data || !data.sheets || !Array.isArray(data.sheets) || data.sheets.length === 0) {
    return (
      <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
        <div className="relative top-20 mx-auto p-5 border w-11/12 shadow-lg rounded-md bg-white">
          <div className="text-center">
            <h3 className="text-lg font-medium text-gray-900">No Data Available</h3>
            <p className="mt-2 text-sm text-gray-500">
              Unable to parse data from this spreadsheet or no data provided.
            </p>
            <div className="mt-4">
              <button
                onClick={onClose}
                className="inline-flex justify-center px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md hover:bg-indigo-700 focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-indigo-500"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  const activeSheet = data.sheets[activeSheetIndex];
  const totalPages = Math.ceil(activeSheet?.rows.length / rowsPerPage) || 1;
  const startRow = currentPage * rowsPerPage;
  const endRow = Math.min(startRow + rowsPerPage, activeSheet?.rows.length || 0);
  const currentRows = activeSheet?.rows.slice(startRow, endRow) || [];

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

  const switchSheet = (index: number) => {
    setActiveSheetIndex(index);
    setCurrentPage(0); // Reset to first page when switching sheets
  };

  if (!activeSheet) {
    return (
      <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
        <div className="relative top-20 mx-auto p-5 border w-11/12 shadow-lg rounded-md bg-white">
          <div className="text-center">
            <h3 className="text-lg font-medium text-gray-900">No Data Available</h3>
            <p className="mt-2 text-sm text-gray-500">
              Unable to parse data from this spreadsheet.
            </p>
            <div className="mt-4">
              <button
                onClick={onClose}
                className="inline-flex justify-center px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md hover:bg-indigo-700 focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-indigo-500"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-4 mx-auto p-5 border w-11/12 max-w-7xl shadow-lg rounded-md bg-white">
        {/* Header */}
        <div className="flex justify-between items-center mb-4">
          <div>
            <h3 className="text-lg font-medium text-gray-900">{template.name}</h3>
            <p className="text-sm text-gray-500">
              {template.description || 'Spreadsheet data viewer'}
            </p>
          </div>
          <div className="flex items-center space-x-4">
            {onCreateSamples && (
              <button
                onClick={() => onCreateSamples(template, data)}
                className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <BeakerIcon className="h-4 w-4 mr-2" />
                Create Samples
              </button>
            )}
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 transition-colors"
            >
              <span className="sr-only">Close</span>
              <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        {/* Sheet Tabs */}
        {data.sheets.length > 1 && (
          <div className="border-b border-gray-200 mb-4">
            <nav className="-mb-px flex space-x-8">
              {data.sheets.map((sheet, index) => (
                <button
                  key={index}
                  onClick={() => switchSheet(index)}
                  className={`py-2 px-1 border-b-2 font-medium text-sm ${
                    index === activeSheetIndex
                      ? 'border-indigo-500 text-indigo-600'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  }`}
                >
                  {sheet.name}
                </button>
              ))}
            </nav>
          </div>
        )}

        {/* Data Info */}
        <div className="bg-gray-50 px-4 py-2 rounded-md mb-4">
          <div className="flex justify-between text-sm text-gray-600">
            <span>
              Sheet: {activeSheet.name} • {activeSheet.total_rows} rows • {activeSheet.total_columns} columns
            </span>
            <span>
              Showing rows {startRow + 1}-{endRow} of {activeSheet.rows.length}
            </span>
          </div>
        </div>

        {/* Data Table */}
        <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 rounded-lg">
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-300">
              <thead className="bg-gray-50">
                <tr>
                  {activeSheet.headers.map((header, index) => (
                    <th
                      key={index}
                      scope="col"
                      className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                    >
                      {header}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {currentRows.map((row, rowIndex) => (
                  <tr key={startRow + rowIndex} className={rowIndex % 2 === 0 ? 'bg-white' : 'bg-gray-50'}>
                    {row.map((cell, cellIndex) => (
                      <td
                        key={cellIndex}
                        className="px-6 py-4 whitespace-nowrap text-sm text-gray-900"
                      >
                        {cell || '-'}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-between border-t border-gray-200 bg-white px-4 py-3 sm:px-6 mt-4">
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
                  Showing <span className="font-medium">{startRow + 1}</span> to{' '}
                  <span className="font-medium">{endRow}</span> of{' '}
                  <span className="font-medium">{activeSheet.rows.length}</span> results
                </p>
              </div>
              <div>
                <nav className="isolate inline-flex -space-x-px rounded-md shadow-sm" aria-label="Pagination">
                  <button
                    onClick={goToPrevPage}
                    disabled={currentPage === 0}
                    className="relative inline-flex items-center rounded-l-md px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <span className="sr-only">Previous</span>
                    <ChevronLeftIcon className="h-5 w-5" aria-hidden="true" />
                  </button>
                  <span className="relative inline-flex items-center px-4 py-2 text-sm font-semibold text-gray-900 ring-1 ring-inset ring-gray-300">
                    Page {currentPage + 1} of {totalPages}
                  </span>
                  <button
                    onClick={goToNextPage}
                    disabled={currentPage === totalPages - 1}
                    className="relative inline-flex items-center rounded-r-md px-2 py-2 text-gray-400 ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus:z-20 focus:outline-offset-0 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <span className="sr-only">Next</span>
                    <ChevronRightIcon className="h-5 w-5" aria-hidden="true" />
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
