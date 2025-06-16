import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import axios from 'axios';
import SpreadsheetDataViewer from '../SpreadsheetDataViewer';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

// Mock dataset
const mockDataset = {
  id: 'test-dataset-id',
  original_filename: 'test-data.csv',
  file_type: 'csv',
  total_rows: 100,
  total_columns: 5,
  column_headers: ['Sample ID', 'Concentration', 'Date', 'Email', 'Active'],
  created_at: '2025-01-01T00:00:00Z',
  uploaded_by: 'test-user@lab.local',
  sheet_name: undefined,
};

// Mock data response
const mockDataResponse = {
  records: [
    {
      id: 'record-1',
      row_number: 1,
      row_data: {
        'Sample ID': 'SAMPLE001',
        'Concentration': '25.5',
        'Date': '2025-01-01',
        'Email': 'test@lab.com',
        'Active': 'true'
      },
      created_at: '2025-01-01T00:00:00Z'
    },
    {
      id: 'record-2',
      row_number: 2,
      row_data: {
        'Sample ID': 'SAMPLE002',
        'Concentration': '30.2',
        'Date': '2025-01-02',
        'Email': 'test2@lab.com',
        'Active': 'false'
      },
      created_at: '2025-01-01T00:00:00Z'
    }
  ],
  total_count: 100
};

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

describe('SpreadsheetDataViewer', () => {
  const mockOnClose = jest.fn();
  
  beforeEach(() => {
    jest.clearAllMocks();
    // Default successful response
    mockedAxios.get.mockResolvedValue({
      data: { data: mockDataResponse }
    });
  });

  const renderComponent = (dataset = mockDataset) => {
    const TestWrapper = createTestWrapper();
    return render(
      <TestWrapper>
        <SpreadsheetDataViewer dataset={dataset} onClose={mockOnClose} />
      </TestWrapper>
    );
  };

  describe('Basic Rendering', () => {
    it('renders the component with dataset information', async () => {
      renderComponent();
      
      expect(screen.getByText('test-data.csv')).toBeInTheDocument();
      expect(screen.getByText('CSV')).toBeInTheDocument();
      expect(screen.getByText('100 rows')).toBeInTheDocument();
      expect(screen.getByText('5 columns')).toBeInTheDocument();
      expect(screen.getByText('by test-user@lab.local')).toBeInTheDocument();
    });

    it('renders header action buttons', () => {
      renderComponent();
      
      expect(screen.getByTitle('Toggle Statistics')).toBeInTheDocument();
      expect(screen.getByTitle('Export Data')).toBeInTheDocument();
      expect(screen.getByTitle('Enter Fullscreen')).toBeInTheDocument();
      expect(screen.getByTitle('Close')).toBeInTheDocument();
    });

    it('calls onClose when close button is clicked', () => {
      renderComponent();
      
      fireEvent.click(screen.getByTitle('Close'));
      expect(mockOnClose).toHaveBeenCalled();
    });
  });

  describe('Data Loading and Display', () => {
    it('shows loading state initially', () => {
      renderComponent();
      expect(screen.getByText('Loading data...')).toBeInTheDocument();
    });

    it('displays data table when loaded', async () => {
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('SAMPLE001')).toBeInTheDocument();
        expect(screen.getByText('SAMPLE002')).toBeInTheDocument();
      });
    });

    it('displays column headers with data type icons', async () => {
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('Sample ID')).toBeInTheDocument();
        expect(screen.getByText('Concentration')).toBeInTheDocument();
        expect(screen.getByText('Date')).toBeInTheDocument();
        expect(screen.getByText('Email')).toBeInTheDocument();
        expect(screen.getByText('Active')).toBeInTheDocument();
      });
    });

    it('formats cell values according to data type', async () => {
      renderComponent();
      
      await waitFor(() => {
        // Check number formatting
        expect(screen.getByText('25.5')).toBeInTheDocument();
        
        // Check email links
        const emailLink = screen.getByText('test@lab.com');
        expect(emailLink.closest('a')).toHaveAttribute('href', 'mailto:test@lab.com');
        
        // Check boolean formatting
        expect(screen.getByText('Yes')).toBeInTheDocument();
        expect(screen.getByText('No')).toBeInTheDocument();
      });
    });
  });

  describe('Statistics Panel', () => {
    it('toggles statistics panel when stats button is clicked', async () => {
      renderComponent();
      
      const statsButton = screen.getByTitle('Toggle Statistics');
      fireEvent.click(statsButton);
      
      await waitFor(() => {
        expect(screen.getByText('Column Statistics')).toBeInTheDocument();
      });
      
      // Toggle off
      fireEvent.click(statsButton);
      await waitFor(() => {
        expect(screen.queryByText('Column Statistics')).not.toBeInTheDocument();
      });
    });

    it('displays column statistics correctly', async () => {
      renderComponent();
      
      const statsButton = screen.getByTitle('Toggle Statistics');
      fireEvent.click(statsButton);
      
      await waitFor(() => {
        expect(screen.getByText('Column Statistics')).toBeInTheDocument();
        expect(screen.getByText('Non-empty:')).toBeInTheDocument();
        expect(screen.getByText('Unique:')).toBeInTheDocument();
      });
    });
  });

  describe('Sorting Functionality', () => {
    it('sorts data when column header is clicked', async () => {
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('SAMPLE001')).toBeInTheDocument();
      });
      
      // Click on Sample ID column to sort
      const sampleIdHeader = screen.getByText('Sample ID');
      fireEvent.click(sampleIdHeader);
      
      // Should see sort indicators in pagination
      await waitFor(() => {
        expect(screen.getByText(/sorted by Sample ID/)).toBeInTheDocument();
      });
    });

    it('cycles through sort directions on repeated clicks', async () => {
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('SAMPLE001')).toBeInTheDocument();
      });
      
      const sampleIdHeader = screen.getByText('Sample ID');
      
      // First click - ascending
      fireEvent.click(sampleIdHeader);
      await waitFor(() => {
        expect(screen.getByText(/sorted by Sample ID ↑/)).toBeInTheDocument();
      });
      
      // Second click - descending
      fireEvent.click(sampleIdHeader);
      await waitFor(() => {
        expect(screen.getByText(/sorted by Sample ID ↓/)).toBeInTheDocument();
      });
    });
  });

  describe('Row Selection', () => {
    it('allows selecting individual rows', async () => {
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('SAMPLE001')).toBeInTheDocument();
      });
      
      const checkboxes = screen.getAllByRole('checkbox');
      const firstRowCheckbox = checkboxes[1]; // Skip header checkbox
      
      fireEvent.click(firstRowCheckbox);
      
      await waitFor(() => {
        expect(screen.getByText('1 selected')).toBeInTheDocument();
      });
    });

    it('allows selecting all rows with header checkbox', async () => {
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('SAMPLE001')).toBeInTheDocument();
      });
      
      const headerCheckbox = screen.getAllByRole('checkbox')[0];
      fireEvent.click(headerCheckbox);
      
      await waitFor(() => {
        expect(screen.getByText('2 selected')).toBeInTheDocument();
      });
    });
  });

  describe('Export Functionality', () => {
    beforeEach(() => {
      // Mock URL.createObjectURL and related functions
      (globalThis as any).URL = {
        createObjectURL: jest.fn(() => 'mock-blob-url'),
        revokeObjectURL: jest.fn(),
      };
      
      // Mock document.createElement for download link
      const mockAnchor = {
        href: '',
        download: '',
        click: jest.fn(),
      };
      jest.spyOn(document, 'createElement').mockReturnValue(mockAnchor as any);
    });

    it('shows export dropdown on hover', async () => {
      renderComponent();
      
      const exportButton = screen.getByTitle('Export Data');
      fireEvent.mouseEnter(exportButton.parentElement!);
      
      await waitFor(() => {
        expect(screen.getByText('Export CSV')).toBeInTheDocument();
        expect(screen.getByText('Export JSON')).toBeInTheDocument();
      });
    });
  });

  describe('Fullscreen Mode', () => {
    it('toggles fullscreen mode', () => {
      renderComponent();
      
      const fullscreenButton = screen.getByTitle('Enter Fullscreen');
      fireEvent.click(fullscreenButton);
      
      expect(screen.getByTitle('Exit Fullscreen')).toBeInTheDocument();
    });
  });

  describe('Search and Filtering', () => {
    it('renders search input', () => {
      renderComponent();
      
      expect(screen.getByPlaceholderText('Search within this dataset...')).toBeInTheDocument();
    });

    it('renders rows per page selector', () => {
      renderComponent();
      
      expect(screen.getByDisplayValue('50')).toBeInTheDocument();
      expect(screen.getByText('Show:')).toBeInTheDocument();
    });

    it('shows filters toggle button', () => {
      renderComponent();
      
      expect(screen.getByText('Filters')).toBeInTheDocument();
    });

    it('updates search term and resets page', async () => {
      renderComponent();
      
      const searchInput = screen.getByPlaceholderText('Search within this dataset...');
      await userEvent.type(searchInput, 'SAMPLE001');
      
      expect(searchInput).toHaveValue('SAMPLE001');
    });
  });

  describe('Pagination', () => {
    it('displays pagination when multiple pages exist', async () => {
      // Mock dataset with more rows to trigger pagination
      const largeMockDataResponse = {
        ...mockDataResponse,
        total_count: 150
      };
      
      mockedAxios.get.mockResolvedValue({
        data: { data: largeMockDataResponse }
      });
      
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText(/Showing 1 to/)).toBeInTheDocument();
        expect(screen.getByText(/of 150 results/)).toBeInTheDocument();
      });
    });
  });

  describe('Error Handling', () => {
    it('displays error message when data loading fails', async () => {
      mockedAxios.get.mockRejectedValue(new Error('Network error'));
      
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('Error Loading Data')).toBeInTheDocument();
      });
    });
  });

  describe('Empty Data Handling', () => {
    it('displays empty state when no data is available', async () => {
      mockedAxios.get.mockResolvedValue({
        data: { 
          data: {
            records: [],
            total_count: 0
          }
        }
      });
      
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('No data found')).toBeInTheDocument();
      });
    });

    it('handles dataset with missing column headers', async () => {
      const datasetWithoutHeaders = {
        ...mockDataset,
        column_headers: []
      };
      
      renderComponent(datasetWithoutHeaders);
      
      await waitFor(() => {
        // Should extract headers from data
        expect(screen.getByText('SAMPLE001')).toBeInTheDocument();
      });
    });
  });

  describe('Data Type Detection', () => {
    it('detects and formats different data types correctly', async () => {
      renderComponent();
      
      await waitFor(() => {
        // Check number detection and formatting
        expect(screen.getByText('25.5')).toBeInTheDocument();
        
        // Check email detection and linking
        const emailElements = screen.getAllByText(/test.*@lab\.com/);
        expect(emailElements.length).toBeGreaterThan(0);
        
        // Check boolean detection and formatting
        expect(screen.getByText('Yes')).toBeInTheDocument();
        expect(screen.getByText('No')).toBeInTheDocument();
      });
    });
  });
}); 
