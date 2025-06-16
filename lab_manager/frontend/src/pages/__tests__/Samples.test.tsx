import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import axios from 'axios';
import Samples from '../Samples';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

// Mock data - Updated to match component interface
const mockSamples = [
  {
    id: '1',
    name: 'Sample 1',
    barcode: 'BAR001',
    location: 'Location 1',
    status: 'Pending' as const,
    created_at: '2024-03-20T10:00:00Z',
    updated_at: '2024-03-20T10:00:00Z',
    metadata: {
      template_name: 'Template 1'
    }
  },
  {
    id: '2',
    name: 'Sample 2',
    barcode: 'BAR002',
    location: 'Location 2',
    status: 'Validated' as const,
    created_at: '2024-03-20T11:00:00Z',
    updated_at: '2024-03-20T11:00:00Z',
    metadata: {
      template_name: 'Template 2'
    }
  }
];

describe('Samples', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    jest.clearAllMocks();
  });

  it('renders loading state initially', () => {
    mockedAxios.get.mockImplementation(() => new Promise(() => {}));
    
    render(
      <QueryClientProvider client={queryClient}>
        <Samples />
      </QueryClientProvider>
    );

    expect(screen.getByText('Loading samples...')).toBeInTheDocument();
  });

  it('renders samples list correctly', async () => {
    mockedAxios.get.mockResolvedValueOnce({ data: mockSamples });

    render(
      <QueryClientProvider client={queryClient}>
        <Samples />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Sample 1')).toBeInTheDocument();
      expect(screen.getByText('Sample 2')).toBeInTheDocument();
      expect(screen.getByText('Template 1')).toBeInTheDocument();
      expect(screen.getByText('Template 2')).toBeInTheDocument();
      expect(screen.getByText('Location 1')).toBeInTheDocument();
      expect(screen.getByText('Location 2')).toBeInTheDocument();
    });
  });

  it('shows empty state when no samples exist', async () => {
    mockedAxios.get.mockResolvedValueOnce({ data: [] });

    render(
      <QueryClientProvider client={queryClient}>
        <Samples />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('No samples found')).toBeInTheDocument();
    });
  });

  it('opens sample submission wizard when Add Sample button is clicked', async () => {
    mockedAxios.get.mockResolvedValueOnce({ data: mockSamples });

    render(
      <QueryClientProvider client={queryClient}>
        <Samples />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Add Sample')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText('Add Sample'));

    expect(screen.getByText('Add New Sample')).toBeInTheDocument();
  });

  it('displays correct status colors', async () => {
    mockedAxios.get.mockResolvedValueOnce({ data: mockSamples });

    render(
      <QueryClientProvider client={queryClient}>
        <Samples />
      </QueryClientProvider>
    );

    await waitFor(() => {
      const pendingStatus = screen.getByText('Pending');
      const validatedStatus = screen.getByText('Validated');
      
      expect(pendingStatus).toHaveClass('bg-yellow-100');
      expect(validatedStatus).toHaveClass('bg-blue-100');
    });
  });
}); 
