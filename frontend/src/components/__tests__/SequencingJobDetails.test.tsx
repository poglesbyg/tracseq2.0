import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import axios from 'axios';
import SequencingJobDetails from '../SequencingJobDetails';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

// Mock data - Updated to match component interface
const mockJob = {
  id: '1',
  name: 'Test Job',
  status: 'Pending' as const,
  created_at: '2024-03-20T10:00:00Z',
  updated_at: '2024-03-20T10:00:00Z',
  sample_sheet_path: null,
  metadata: {
    sample_ids: [1, 2]
  }
};

describe('SequencingJobDetails', () => {
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
        <SequencingJobDetails jobId="1" onClose={() => {}} />
      </QueryClientProvider>
    );

    expect(screen.getByRole('status', { hidden: true })).toBeInTheDocument();
  });

  it('renders job details correctly', async () => {
    mockedAxios.get.mockResolvedValueOnce({ data: mockJob });

    render(
      <QueryClientProvider client={queryClient}>
        <SequencingJobDetails jobId="1" onClose={() => {}} />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Test Job')).toBeInTheDocument();
      expect(screen.getByText('Pending')).toBeInTheDocument();
      expect(screen.getByText('Sample 1')).toBeInTheDocument();
      expect(screen.getByText('Sample 2')).toBeInTheDocument();
    });
  });

  it('handles sample sheet generation', async () => {
    mockedAxios.get.mockResolvedValueOnce({ data: mockJob });
    mockedAxios.post.mockResolvedValueOnce({ data: { sample_sheet_path: 'http://example.com/sheet' } });

    render(
      <QueryClientProvider client={queryClient}>
        <SequencingJobDetails jobId="1" onClose={() => {}} />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Generate Sample Sheet')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText('Generate Sample Sheet'));

    await waitFor(() => {
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/sequencing/jobs/1/sample-sheet');
    });
  });

  it('handles job status updates', async () => {
    mockedAxios.get.mockResolvedValueOnce({ data: mockJob });
    mockedAxios.patch.mockResolvedValueOnce({ data: { ...mockJob, status: 'InProgress' } });

    render(
      <QueryClientProvider client={queryClient}>
        <SequencingJobDetails jobId="1" onClose={() => {}} />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Start Job')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText('Start Job'));

    await waitFor(() => {
      expect(mockedAxios.patch).toHaveBeenCalledWith('/api/sequencing/jobs/1', { status: 'InProgress' });
    });
  });

  it('calls onClose when close button is clicked', async () => {
    mockedAxios.get.mockResolvedValueOnce({ data: mockJob });
    const onClose = jest.fn();

    render(
      <QueryClientProvider client={queryClient}>
        <SequencingJobDetails jobId="1" onClose={onClose} />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /close/i })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole('button', { name: /close/i }));
    expect(onClose).toHaveBeenCalled();
  });
}); 
