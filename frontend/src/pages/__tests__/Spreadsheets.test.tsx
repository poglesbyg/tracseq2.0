import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import axios from 'axios';
import Spreadsheets from '../Spreadsheets';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

// Mock components
jest.mock('../../components/FileUploadModal', () => {
  return function MockFileUploadModal({ onClose }: any) {
    return (
      <div data-testid="file-upload-modal">
        <button onClick={onClose}>Close Modal</button>
      </div>
    );
  };
});

jest.mock('../../components/SpreadsheetSearchModal', () => {
  return function MockSpreadsheetSearchModal({ onClose }: any) {
    return (
      <div data-testid="search-modal">
        <button onClick={onClose}>Close Search</button>
      </div>
    );
  };
});

jest.mock('../../components/SpreadsheetDataViewer', () => {
  return function MockSpreadsheetDataViewer({ dataset, onClose }: any) {
    return (
      <div data-testid="data-viewer">
        <span>Viewing: {dataset.original_filename}</span>
        <button onClick={onClose}>Close Viewer</button>
      </div>
    );
  };
});

// Mock datasets
const mockDatasets = [
  {
    id: 'dataset-1',
    filename: 'file1.csv',
    original_filename: 'Sample_Data_2024.csv',
    file_type: 'csv',
    file_size: 1024000,
    total_rows: 500,
    total_columns: 8,
    column_headers: ['Sample ID', 'Concentration'],
    upload_status: 'completed',
    error_message: null,
    uploaded_by: 'user@lab.local',
    created_at: '2025-01-01T00:00:00Z',
    updated_at: '2025-01-01T00:01:00Z',
    metadata: {}
  },
  {
    id: 'dataset-2', 
    filename: 'file2.xlsx',
    original_filename: 'Lab_Results.xlsx',
    file_type: 'xlsx',
    file_size: 2048000,
    total_rows: 1200,
    total_columns: 12,
    column_headers: ['ID', 'Value'],
    upload_status: 'processing',
    error_message: null,
    uploaded_by: 'admin@lab.local',
    created_at: '2025-01-02T00:00:00Z',
    updated_at: '2025-01-02T00:01:00Z',
    metadata: {}
  }
];

// Test wrapper with React Query
const createTestWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

describe('Spreadsheets Page', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    
    // Default mock responses
    mockedAxios.get.mockImplementation((url) => {
      if (url === '/api/spreadsheets/datasets') {
        return Promise.resolve({
          data: { data: mockDatasets }
        });
      }
      if (url === '/api/spreadsheets/filters') {
        return Promise.resolve({
          data: { data: { pools: [], samples: [], projects: [], all_columns: [] } }
        });
      }
      return Promise.reject(new Error('Unexpected URL'));
    });
    
    // Mock delete
    mockedAxios.delete.mockResolvedValue({ data: { success: true } });
    
    // Mock window.confirm
    (globalThis as any).confirm = jest.fn(() => true);
  });

  const renderComponent = () => {
    const TestWrapper = createTestWrapper();
    return render(
      <TestWrapper>
        <Spreadsheets />
      </TestWrapper>
    );
  };

  describe('Basic Rendering', () => {
    it('renders the page header and description', async () => {
      renderComponent();
      
      expect(screen.getByText('Spreadsheet Data')).toBeInTheDocument();
      expect(screen.getByText(/Upload and manage CSV\/Excel files/)).toBeInTheDocument();
    });

    it('renders action buttons', () => {
      renderComponent();
      
      expect(screen.getByText('Search Data')).toBeInTheDocument();
      expect(screen.getByText('Upload File')).toBeInTheDocument();
    });
  });

  describe('Dataset Display', () => {
    it('displays datasets in table format', async () => {
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('Sample_Data_2024.csv')).toBeInTheDocument();
        expect(screen.getByText('Lab_Results.xlsx')).toBeInTheDocument();
      });
    });

    it('shows correct file type badges', async () => {
      renderComponent();
      
      await waitFor(() => {
        // Find file type badges specifically by looking for spans with the right classes
        const csvBadge = screen.getByText('csv').closest('span');
        const xlsxBadge = screen.getByText('xlsx').closest('span');
        
        expect(csvBadge).toBeInTheDocument();
        expect(xlsxBadge).toBeInTheDocument();
        
        // Verify they have the correct styling classes
        expect(csvBadge).toHaveClass('uppercase', 'font-mono', 'text-xs', 'bg-gray-100');
        expect(xlsxBadge).toHaveClass('uppercase', 'font-mono', 'text-xs', 'bg-gray-100');
      });
    });

    it('displays upload status with correct colors', async () => {
      renderComponent();
      
      await waitFor(() => {
        const completedBadge = screen.getByText('completed');
        const processingBadge = screen.getByText('processing');
        
        expect(completedBadge).toBeInTheDocument();
        expect(processingBadge).toBeInTheDocument();
        
        expect(completedBadge).toHaveClass('bg-green-100', 'text-green-800');
        expect(processingBadge).toHaveClass('bg-yellow-100', 'text-yellow-800');
      });
    });

    it('shows View Data button only for completed datasets', async () => {
      renderComponent();
      
      await waitFor(() => {
        const viewButtons = screen.getAllByText('View Data');
        expect(viewButtons).toHaveLength(1); // Only for completed dataset
      });
    });
  });

  describe('Dataset Interaction', () => {
    it('opens data viewer when View Data button is clicked', async () => {
      renderComponent();
      
      await waitFor(() => {
        const viewButton = screen.getByText('View Data');
        fireEvent.click(viewButton);
      });
      
      await waitFor(() => {
        expect(screen.getByTestId('data-viewer')).toBeInTheDocument();
        expect(screen.getByText('Viewing: Sample_Data_2024.csv')).toBeInTheDocument();
      });
    });

    it('opens data viewer when clicking on completed dataset row', async () => {
      renderComponent();
      
      await waitFor(() => {
        const datasetRow = screen.getByText('Sample_Data_2024.csv').closest('tr');
        fireEvent.click(datasetRow!);
      });
      
      await waitFor(() => {
        expect(screen.getByTestId('data-viewer')).toBeInTheDocument();
      });
    });

    it('does not open data viewer when clicking on non-completed dataset row', async () => {
      renderComponent();
      
      await waitFor(() => {
        const processingRow = screen.getByText('Lab_Results.xlsx').closest('tr');
        fireEvent.click(processingRow!);
      });
      
      // Should not open data viewer
      expect(screen.queryByTestId('data-viewer')).not.toBeInTheDocument();
    });

    it('applies hover styles to completed datasets', async () => {
      renderComponent();
      
      await waitFor(() => {
        const completedRow = screen.getByText('Sample_Data_2024.csv').closest('tr');
        expect(completedRow).toHaveClass('hover:bg-gray-50', 'cursor-pointer');
      });
    });
  });

  describe('Modal Interactions', () => {
    it('opens upload modal when Upload File is clicked', () => {
      renderComponent();
      
      const uploadButton = screen.getByText('Upload File');
      fireEvent.click(uploadButton);
      
      expect(screen.getByTestId('file-upload-modal')).toBeInTheDocument();
    });

    it('closes upload modal when close is clicked', () => {
      renderComponent();
      
      const uploadButton = screen.getByText('Upload File');
      fireEvent.click(uploadButton);
      
      const closeButton = screen.getByText('Close Modal');
      fireEvent.click(closeButton);
      
      expect(screen.queryByTestId('file-upload-modal')).not.toBeInTheDocument();
    });

    it('opens search modal when Search Data is clicked', () => {
      renderComponent();
      
      const searchButton = screen.getByText('Search Data');
      fireEvent.click(searchButton);
      
      expect(screen.getByTestId('search-modal')).toBeInTheDocument();
    });
  });

  describe('Case Sensitivity Fixes', () => {
    it('handles case-insensitive status comparison for View Data button', async () => {
      const datasetsWithMixedCase = [
        { ...mockDatasets[0], upload_status: 'completed' }, // lowercase
        { ...mockDatasets[1], upload_status: 'COMPLETED' }  // uppercase
      ];
      
      mockedAxios.get.mockImplementation((url) => {
        if (url === '/api/spreadsheets/datasets') {
          return Promise.resolve({ data: { data: datasetsWithMixedCase } });
        }
        if (url === '/api/spreadsheets/filters') {
          return Promise.resolve({
            data: { data: { pools: [], samples: [], projects: [], all_columns: [] } }
          });
        }
        return Promise.reject(new Error('Unexpected URL'));
      });
      
      renderComponent();
      
      await waitFor(() => {
        const viewButtons = screen.getAllByText('View Data');
        expect(viewButtons).toHaveLength(2); // Both should show View Data button
      });
    });
  });

  describe('Error Handling', () => {
    it('displays error message when datasets fail to load', async () => {
      mockedAxios.get.mockRejectedValue(new Error('Network error'));
      
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('Error Loading Datasets')).toBeInTheDocument();
      });
    });
  });
});
