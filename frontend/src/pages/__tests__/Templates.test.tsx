import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import axios from 'axios';
import Templates from '../Templates';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

// Mock data
const mockTemplates = [
  {
    id: '1',
    name: 'Template 1',
    version: '1.0',
    created_at: '2024-03-20T10:00:00Z',
    updated_at: '2024-03-20T10:00:00Z'
  },
  {
    id: '2',
    name: 'Template 2',
    version: '1.0',
    created_at: '2024-03-20T11:00:00Z',
    updated_at: '2024-03-20T11:00:00Z'
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
    mockedAxios.get.mockImplementation(() => new Promise(() => {}));
    
    render(
      <QueryClientProvider client={queryClient}>
        <Templates />
      </QueryClientProvider>
    );

    expect(screen.getByRole('status')).toBeInTheDocument();
  });

  it('renders templates list correctly', async () => {
    mockedAxios.get.mockResolvedValueOnce({ data: mockTemplates });

    render(
      <QueryClientProvider client={queryClient}>
        <Templates />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Template 1')).toBeInTheDocument();
      expect(screen.getByText('Template 2')).toBeInTheDocument();
    });
  });

  it('handles file selection via click', async () => {
    mockedAxios.get.mockResolvedValueOnce({ data: mockTemplates });
    
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
    mockedAxios.get.mockResolvedValueOnce({ data: mockTemplates });
    
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
    mockedAxios.get.mockResolvedValueOnce({ data: mockTemplates });
    
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
    mockedAxios.get.mockResolvedValueOnce({ data: mockTemplates });
    mockedAxios.post.mockResolvedValueOnce({ data: { id: '3', name: 'New Template' } });
    
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
      expect(mockedAxios.post).toHaveBeenCalledWith(
        '/api/templates/upload',
        expect.any(FormData),
        expect.any(Object)
      );
    });
  });

  it('shows loading state during upload', async () => {
    mockedAxios.get.mockResolvedValueOnce({ data: mockTemplates });
    mockedAxios.post.mockImplementation(() => new Promise(() => {}));
    
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
