import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import SequencingJobDetails from '../SequencingJobDetails';

// Mock the utils/axios module
jest.mock('../../utils/axios', () => ({
  default: {
    get: jest.fn(),
    post: jest.fn(),
    put: jest.fn(),
    delete: jest.fn(),
    patch: jest.fn()
  }
}));

// Import the mocked module
import api from '../../utils/axios';

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
    api.get.mockImplementation(() => new Promise(() => {}));
    
    render(
      <QueryClientProvider client={queryClient}>
        <SequencingJobDetails jobId="1" onClose={() => {}} />
      </QueryClientProvider>
    );

    expect(screen.getByRole('status', { hidden: true })).toBeInTheDocument();
  });

  it('renders job details correctly', async () => {
    api.get.mockResolvedValueOnce({ data: mockJob });

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
    api.get.mockResolvedValueOnce({ data: mockJob });
    api.post.mockResolvedValueOnce({ data: { sample_sheet_path: 'http://example.com/sheet' } });

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
      expect(api.post).toHaveBeenCalledWith('/api/sequencing/jobs/1/sample-sheet');
    });
  });

  it('handles job status updates', async () => {
    api.get.mockResolvedValueOnce({ data: mockJob });
    api.patch.mockResolvedValueOnce({ data: { ...mockJob, status: 'InProgress' } });

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
      expect(api.patch).toHaveBeenCalledWith('/api/sequencing/jobs/1', { status: 'InProgress' });
    });
  });

  it('calls onClose when close button is clicked', async () => {
    api.get.mockResolvedValueOnce({ data: mockJob });
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
