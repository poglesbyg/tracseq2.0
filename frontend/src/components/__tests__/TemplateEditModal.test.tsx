import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import axios from 'axios';
import { act } from 'react';
import TemplateEditModal from '../TemplateEditModal';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe('TemplateEditModal', () => {
  const mockOnClose = jest.fn();
  const mockOnSave = jest.fn();
  
  const mockTemplate = {
    id: 'template-1',
    name: 'Sample Template',
    description: 'A test template',
    created_at: '2025-01-01T00:00:00Z',
    fields: [
      { name: 'Sample ID', type: 'string', required: true },
      { name: 'Concentration', type: 'number', required: false }
    ],
    metadata: {
      file_type: 'csv',
      originalFileName: 'template.csv'
    }
  };

  let queryClient: QueryClient;

  beforeEach(() => {
    jest.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    });
  });

  const renderComponent = (isOpen = true, template: typeof mockTemplate | null = mockTemplate) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <TemplateEditModal
          isOpen={isOpen}
          onClose={mockOnClose}
          template={template}
          onSave={mockOnSave}
        />
      </QueryClientProvider>
    );
  };

  describe('Basic Rendering', () => {
    it('renders modal with template data', () => {
      renderComponent();
      
      expect(screen.getByText('Edit Template')).toBeInTheDocument();
      expect(screen.getByDisplayValue('Sample Template')).toBeInTheDocument();
      expect(screen.getByDisplayValue('A test template')).toBeInTheDocument();
    });

    it('renders even when closed (component always renders when mounted)', () => {
      renderComponent(false);
      
      // The component doesn't check isOpen, so it always renders
      expect(screen.getByText('Edit Template')).toBeInTheDocument();
    });

    it('displays template information', () => {
      renderComponent();
      
      const expectedDate = new Date(mockTemplate.created_at).toLocaleDateString();
      
      expect(screen.getByText('template-1')).toBeInTheDocument();
      expect(screen.getByText(expectedDate)).toBeInTheDocument();
      expect(screen.getByText('csv')).toBeInTheDocument();
      expect(screen.getByText('template.csv')).toBeInTheDocument();
    });

    it('displays form inputs', () => {
      renderComponent();
      
      expect(screen.getByLabelText('Template Name')).toBeInTheDocument();
      expect(screen.getByLabelText('Description')).toBeInTheDocument();
    });

    it('displays action buttons', () => {
      renderComponent();
      
      expect(screen.getByText('Cancel')).toBeInTheDocument();
      expect(screen.getByText('Save Changes')).toBeInTheDocument();
    });
  });

  describe('Form Interactions', () => {
    it('allows editing template name', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const nameInput = screen.getByLabelText('Template Name');
      
      await act(async () => {
        await user.clear(nameInput);
        await user.type(nameInput, 'Updated Template');
      });
      
      expect(nameInput).toHaveValue('Updated Template');
    });

    it('allows editing description', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const descriptionInput = screen.getByLabelText('Description');
      
      await act(async () => {
        await user.clear(descriptionInput);
        await user.type(descriptionInput, 'Updated description');
      });
      
      expect(descriptionInput).toHaveValue('Updated description');
    });
  });

  describe('Modal Controls', () => {
    it('calls onClose when Cancel button is clicked', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const cancelButton = screen.getByText('Cancel');
      await user.click(cancelButton);
      
      expect(mockOnClose).toHaveBeenCalled();
    });

    it('calls onClose when close (X) button is clicked', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      // The close button has sr-only text, so we need to find it by its parent button
      const closeButton = screen.getByRole('button', { name: /close/i });
      await user.click(closeButton);
      
      expect(mockOnClose).toHaveBeenCalled();
    });

    it('submits form when save button is clicked', async () => {
      const user = userEvent.setup();
      mockedAxios.put.mockResolvedValueOnce({ data: { success: true } });
      
      renderComponent();
      
      const nameInput = screen.getByLabelText('Template Name');
      
      await act(async () => {
        await user.clear(nameInput);
        await user.type(nameInput, 'Updated Template');
      });
      
      const saveButton = screen.getByText('Save Changes');
      await user.click(saveButton);
      
      await waitFor(() => {
        expect(mockedAxios.put).toHaveBeenCalledWith(
          '/api/templates/template-1',
          { name: 'Updated Template' }
        );
      });
    });
  });

  describe('Validation', () => {
    it('shows error when template name is empty', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const nameInput = screen.getByLabelText('Template Name');
      
      await act(async () => {
        await user.clear(nameInput);
      });
      
      const saveButton = screen.getByText('Save Changes');
      await user.click(saveButton);
      
      await waitFor(() => {
        expect(screen.getByText('Template name is required')).toBeInTheDocument();
      });
    });
  });

  describe('Null Template Handling', () => {
    it('handles null template gracefully', () => {
      // The component still renders with null template, just with empty values
      renderComponent(true, null);
      
      // The modal still shows "Edit Template"
      expect(screen.getByText('Edit Template')).toBeInTheDocument();
      expect(screen.getByLabelText('Template Name')).toHaveValue('');
    });
  });
});