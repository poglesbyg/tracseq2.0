import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

// Mock the utils/axios module
jest.mock('../../utils/axios', () => ({
  default: {
    get: jest.fn(),
    post: jest.fn(),
    put: jest.fn(),
    delete: jest.fn()
  }
}));

import Templates from '../Templates';
// Import the mocked module
import axios from '../../utils/axios';

// Mock template data
const mockTemplates = [
  {
    id: 'template-1',
    name: 'Blood Sample Template',
    description: 'Template for blood samples',
    created_at: '2025-01-01T00:00:00Z',
    created_by: 'user@example.com',
    fields: [],
    usage_count: 10,
    last_used: '2025-01-01T00:00:00Z'
  },
  {
    id: 'template-2',
    name: 'DNA Extraction Template',
    description: 'Template for DNA extraction',
    created_at: '2025-01-02T00:00:00Z',
    created_by: 'user@example.com',
    fields: [],
    usage_count: 5,
    last_used: '2025-01-02T00:00:00Z'
  }
];

describe('Templates', () => {
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
    axios.get.mockImplementation(() => new Promise(() => {}));
    
    render(
      <QueryClientProvider client={queryClient}>
        <Templates />
      </QueryClientProvider>
    );

    expect(screen.getByRole('status')).toBeInTheDocument();
  });

  it('renders templates list correctly', async () => {
    axios.get.mockResolvedValueOnce({ data: mockTemplates });

    render(
      <QueryClientProvider client={queryClient}>
        <Templates />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Blood Sample Template')).toBeInTheDocument();
      expect(screen.getByText('DNA Extraction Template')).toBeInTheDocument();
    });
  });

  it('handles file selection via click', async () => {
    axios.get.mockResolvedValueOnce({ data: mockTemplates });
    
    render(
      <QueryClientProvider client={queryClient}>
        <Templates />
      </QueryClientProvider>
    );

    // Wait for loading to complete
    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });

    const file = new File(['test content'], 'test.xlsx', { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' });
    const input = screen.getByLabelText(/click to select/i);

    Object.defineProperty(input, 'files', {
      value: [file],
    });

    fireEvent.change(input);

    await waitFor(() => {
      expect(screen.getByText('Confirm Upload')).toBeInTheDocument();
    });
  });

  it('handles drag and drop of valid file', async () => {
    axios.get.mockResolvedValueOnce({ data: mockTemplates });
    
    render(
      <QueryClientProvider client={queryClient}>
        <Templates />
      </QueryClientProvider>
    );

    // Wait for loading to complete
    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });

    const dropZone = screen.getByText(/Drag and drop your Excel file here/i).closest('div');
    const file = new File(['test content'], 'test.xlsx', { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' });

    // Simulate drag enter
    fireEvent.dragEnter(dropZone!, {
      dataTransfer: {
        files: [file],
      },
    });

    // Simulate drop
    fireEvent.drop(dropZone!, {
      dataTransfer: {
        files: [file],
      },
    });

    await waitFor(() => {
      expect(screen.getByText('Confirm Upload')).toBeInTheDocument();
    });
  });

  it('rejects invalid file types', async () => {
    axios.get.mockResolvedValueOnce({ data: mockTemplates });
    
    render(
      <QueryClientProvider client={queryClient}>
        <Templates />
      </QueryClientProvider>
    );

    // Wait for loading to complete
    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });

    const dropZone = screen.getByText(/Drag and drop your Excel file here/i).closest('div');
    const file = new File(['test content'], 'test.txt', { type: 'text/plain' });

    // Simulate drop
    fireEvent.drop(dropZone!, {
      dataTransfer: {
        files: [file],
      },
    });

    await waitFor(() => {
      expect(screen.queryByText('Confirm Upload')).not.toBeInTheDocument();
    });
  });

  it('handles successful file upload', async () => {
    axios.get.mockResolvedValueOnce({ data: mockTemplates });
    axios.post.mockResolvedValueOnce({ data: { id: '3', name: 'New Template' } });
    
    render(
      <QueryClientProvider client={queryClient}>
        <Templates />
      </QueryClientProvider>
    );

    // Wait for loading to complete
    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });

    const file = new File(['test content'], 'test.xlsx', { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' });
    const input = screen.getByLabelText(/click to select/i);

    Object.defineProperty(input, 'files', {
      value: [file],
    });

    fireEvent.change(input);

    const uploadButton = await screen.findByText('Confirm Upload');
    fireEvent.click(uploadButton);

    await waitFor(() => {
      expect(axios.post).toHaveBeenCalledWith(
        '/api/templates/upload',
        expect.any(FormData),
        expect.any(Object)
      );
    });
  });

  it('shows loading state during upload', async () => {
    axios.get.mockResolvedValueOnce({ data: mockTemplates });
    axios.post.mockImplementation(() => new Promise(() => {}));
    
    render(
      <QueryClientProvider client={queryClient}>
        <Templates />
      </QueryClientProvider>
    );

    // Wait for loading to complete
    await waitFor(() => {
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });

    const file = new File(['test content'], 'test.xlsx', { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' });
    const input = screen.getByLabelText(/click to select/i);

    Object.defineProperty(input, 'files', {
      value: [file],
    });

    fireEvent.change(input);

    const uploadButton = await screen.findByText('Confirm Upload');
    fireEvent.click(uploadButton);

    await waitFor(() => {
      expect(screen.getByText('Uploading...')).toBeInTheDocument();
    });
  });
}); 
