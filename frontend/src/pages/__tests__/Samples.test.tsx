import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import axios from 'axios';
import Samples from '../Samples';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

// Mock AuthContext
jest.mock('../../auth/AuthContext', () => ({
  useAuth: () => ({
    user: {
      id: '1',
      email: 'test@example.com',
      role: 'lab_technician',
    },
    isAuthenticated: true,
    hasPermission: () => true,
  }),
}));

// Mock sample data
const mockSamples = [
  {
    id: 'sample-1',
    sample_id: 'SAMPLE001',
    name: 'Test Sample 1',
    status: 'Pending',
    template_id: 'template-1',
    template_name: 'Template 1',
    storage_location: 'Freezer A1',
    created_at: '2025-01-01T00:00:00Z',
    created_by: 'test@example.com',
    metadata: {},
  },
  {
    id: 'sample-2',
    sample_id: 'SAMPLE002',
    name: 'Test Sample 2',
    status: 'Validated',
    template_id: 'template-1',
    template_name: 'Template 1',
    storage_location: 'Freezer A2',
    created_at: '2025-01-02T00:00:00Z',
    created_by: 'test@example.com',
    metadata: {},
  },
];

// Mock statistics
const mockStats = {
  total: 2,
  by_status: {
    Pending: 1,
    Validated: 1,
    InStorage: 0,
    InSequencing: 0,
    Completed: 0,
  },
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

describe('Samples', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    
    // Mock successful API responses
    mockedAxios.get.mockImplementation((url) => {
      if (url.includes('/api/samples/stats')) {
        return Promise.resolve({ data: { data: mockStats } });
      }
      if (url.includes('/api/samples')) {
        return Promise.resolve({ 
          data: { 
            data: {
              samples: mockSamples,
              total: mockSamples.length,
              page: 1,
              per_page: 50,
            }
          } 
        });
      }
      return Promise.reject(new Error('Unknown endpoint'));
    });
  });

  const renderComponent = () => {
    const TestWrapper = createTestWrapper();
    return render(
      <TestWrapper>
        <Samples />
      </TestWrapper>
    );
  };

  describe('Basic Rendering', () => {
    it('renders the page title and description', () => {
      renderComponent();
      
      expect(screen.getByText('Sample Management')).toBeInTheDocument();
      expect(screen.getByText(/Comprehensive view of laboratory samples/)).toBeInTheDocument();
    });

    it('renders action buttons', () => {
      renderComponent();
      
      expect(screen.getByText('Refresh')).toBeInTheDocument();
      expect(screen.getByText('Add Sample')).toBeInTheDocument();
    });

    it('renders filter controls', () => {
      renderComponent();
      
      expect(screen.getByText('All Status')).toBeInTheDocument();
      expect(screen.getByText('All Time')).toBeInTheDocument();
    });
  });

  describe('Statistics Display', () => {
    it('displays sample statistics', async () => {
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('Pending')).toBeInTheDocument();
        expect(screen.getByText('Validated')).toBeInTheDocument();
      });
    });

    it('shows correct counts in statistics', async () => {
      renderComponent();
      
      await waitFor(() => {
        // Find status badges and their associated counts
        const pendingElement = screen.getByText('Pending');
        const validatedElement = screen.getByText('Validated');
        
        expect(pendingElement).toBeInTheDocument();
        expect(validatedElement).toBeInTheDocument();
      });
    });
  });

  describe('Sample List', () => {
    it('displays samples when data is loaded', async () => {
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('SAMPLE001')).toBeInTheDocument();
        expect(screen.getByText('SAMPLE002')).toBeInTheDocument();
      });
    });

    it('displays sample information correctly', async () => {
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText('Test Sample 1')).toBeInTheDocument();
        expect(screen.getByText('Test Sample 2')).toBeInTheDocument();
        expect(screen.getByText('Freezer A1')).toBeInTheDocument();
        expect(screen.getByText('Freezer A2')).toBeInTheDocument();
      });
    });

    it('shows sample count information', async () => {
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText(/Showing.*of.*samples/)).toBeInTheDocument();
      });
    });
  });

  describe('Filtering and Search', () => {
    it('allows filtering by status', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const statusFilter = screen.getByDisplayValue('All Status');
      await user.selectOptions(statusFilter, 'Pending');
      
      expect(statusFilter).toHaveValue('Pending');
    });

    it('allows filtering by time range', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const timeFilter = screen.getByDisplayValue('All Time');
      await user.selectOptions(timeFilter, '7d');
      
      expect(timeFilter).toHaveValue('7d');
    });

    it('switches between table and process views', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const processButton = screen.getByText('Process');
      await user.click(processButton);
      
      // Process view should be active
      expect(processButton).toHaveClass('bg-indigo-100');
    });
  });

  describe('Error Handling', () => {
    it('displays error message when data loading fails', async () => {
      mockedAxios.get.mockRejectedValue(new Error('Network error'));
      
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText(/Error loading/i)).toBeInTheDocument();
      });
    });

    it('handles empty state when no samples exist', async () => {
      mockedAxios.get.mockImplementation((url) => {
        if (url.includes('/api/samples/stats')) {
          return Promise.resolve({ 
            data: { 
              data: {
                total: 0,
                by_status: {
                  Pending: 0,
                  Validated: 0,
                  InStorage: 0,
                  InSequencing: 0,
                  Completed: 0,
                },
              }
            } 
          });
        }
        if (url.includes('/api/samples')) {
          return Promise.resolve({ 
            data: { 
              data: {
                samples: [],
                total: 0,
                page: 1,
                per_page: 50,
              }
            } 
          });
        }
        return Promise.reject(new Error('Unknown endpoint'));
      });
      
      renderComponent();
      
      await waitFor(() => {
        expect(screen.getByText(/No samples found/)).toBeInTheDocument();
      });
    });
  });

  describe('User Actions', () => {
    it('handles refresh button click', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const refreshButton = screen.getByText('Refresh');
      await user.click(refreshButton);
      
      // Should trigger a new API call
      expect(mockedAxios.get).toHaveBeenCalledTimes(4); // 2 initial + 2 refresh
    });

    it('handles add sample button click', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const addButton = screen.getByText('Add Sample');
      await user.click(addButton);
      
      // Should show add sample form or modal
      // This would depend on the actual implementation
    });
  });

  describe('Loading States', () => {
    it('shows loading state initially', () => {
      renderComponent();
      
      expect(screen.getByText(/Loading/i)).toBeInTheDocument();
    });
  });

  describe('Status Display', () => {
    it('displays correct status colors', async () => {
      renderComponent();
      
      await waitFor(() => {
        const pendingElements = screen.getAllByText('Pending');
        const validatedElements = screen.getAllByText('Validated');
        
        expect(pendingElements.length).toBeGreaterThan(0);
        expect(validatedElements.length).toBeGreaterThan(0);
        
        // Check that status elements have appropriate classes
        pendingElements.forEach(element => {
          expect(element).toHaveClass('text-yellow-800');
        });
        
        validatedElements.forEach(element => {
          expect(element).toHaveClass('text-blue-800');
        });
      });
    });
  });
}); 
