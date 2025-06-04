import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import axios from 'axios';
import Samples from '../Samples';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

// Mock data
const mockSamples = [
  {
    id: 1,
    name: 'Sample 1',
    template_id: 1,
    storage_location_id: 1,
    barcode: 'BAR001',
    created_at: '2024-03-20T10:00:00Z',
    updated_at: '2024-03-20T10:00:00Z',
    status: 'active'
  },
  {
    id: 2,
    name: 'Sample 2',
    template_id: 2,
    storage_location_id: 2,
    barcode: 'BAR002',
    created_at: '2024-03-20T11:00:00Z',
    updated_at: '2024-03-20T11:00:00Z',
    status: 'pending'
  }
];

const mockTemplates = [
  { id: 1, name: 'Template 1', version: '1.0' },
  { id: 2, name: 'Template 2', version: '1.0' }
];

const mockStorageLocations = [
  { id: 1, name: 'Location 1' },
  { id: 2, name: 'Location 2' }
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
    mockedAxios.get
      .mockResolvedValueOnce({ data: mockSamples })
      .mockResolvedValueOnce({ data: mockTemplates })
      .mockResolvedValueOnce({ data: mockStorageLocations });

    render(
      <QueryClientProvider client={queryClient}>
        <Samples />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Sample 1')).toBeInTheDocument();
      expect(screen.getByText('Sample 2')).toBeInTheDocument();
      expect(screen.getByText('Template 1')).toBeInTheDocument();
      expect(screen.getByText('Location 1')).toBeInTheDocument();
    });
  });

  it('shows empty state when no samples exist', async () => {
    mockedAxios.get
      .mockResolvedValueOnce({ data: [] })
      .mockResolvedValueOnce({ data: mockTemplates })
      .mockResolvedValueOnce({ data: mockStorageLocations });

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
    mockedAxios.get
      .mockResolvedValueOnce({ data: mockSamples })
      .mockResolvedValueOnce({ data: mockTemplates })
      .mockResolvedValueOnce({ data: mockStorageLocations });

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
    mockedAxios.get
      .mockResolvedValueOnce({ data: mockSamples })
      .mockResolvedValueOnce({ data: mockTemplates })
      .mockResolvedValueOnce({ data: mockStorageLocations });

    render(
      <QueryClientProvider client={queryClient}>
        <Samples />
      </QueryClientProvider>
    );

    await waitFor(() => {
      const activeStatus = screen.getByText('active');
      const pendingStatus = screen.getByText('pending');
      
      expect(activeStatus).toHaveClass('bg-green-100');
      expect(pendingStatus).toHaveClass('bg-yellow-100');
    });
  });
}); 
