import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import TemplateEditModal from '../TemplateEditModal';

// Mock axios
jest.mock('axios');

// Mock template
const mockTemplate = {
  id: 'template-1',
  name: 'Sample Template',
  description: 'A test template',
  created_at: '2025-01-01T00:00:00Z',
  fields: [
    {
      name: 'Sample ID',
      type: 'text',
      required: true,
      defaultValue: '',
    },
    {
      name: 'Concentration',
      type: 'number',
      required: false,
      defaultValue: 0,
    },
  ],
  metadata: {},
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

describe('TemplateEditModal', () => {
  const mockOnClose = jest.fn();
  const mockOnSave = jest.fn();
  
  beforeEach(() => {
    jest.clearAllMocks();
  });

  const renderComponent = (isOpen = true, template: typeof mockTemplate | null = mockTemplate) => {
    const TestWrapper = createTestWrapper();
    return render(
      <TestWrapper>
        <TemplateEditModal
          isOpen={isOpen}
          template={template}
          onClose={mockOnClose}
          onSave={mockOnSave}
        />
      </TestWrapper>
    );
  };

  describe('Basic Rendering', () => {
    it('renders when open', () => {
      renderComponent();
      
      expect(screen.getByText('Edit Template')).toBeInTheDocument();
      expect(screen.getByDisplayValue('Sample Template')).toBeInTheDocument();
    });

    it('does not render when closed', () => {
      renderComponent(false);
      
      expect(screen.queryByText('Edit Template')).not.toBeInTheDocument();
    });

    it('displays template information', () => {
      renderComponent();
      
      expect(screen.getByDisplayValue('Sample Template')).toBeInTheDocument();
      expect(screen.getByDisplayValue('A test template')).toBeInTheDocument();
    });
  });

  describe('Form Interaction', () => {
    it('allows editing template name', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const nameInput = screen.getByDisplayValue('Sample Template');
      await user.clear(nameInput);
      await user.type(nameInput, 'Updated Template');
      
      expect(nameInput).toHaveValue('Updated Template');
    });

    it('allows editing template description', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const descriptionInput = screen.getByDisplayValue('A test template');
      await user.clear(descriptionInput);
      await user.type(descriptionInput, 'Updated description');
      
      expect(descriptionInput).toHaveValue('Updated description');
    });
  });

  describe('Field Management', () => {
    it('displays existing fields', () => {
      renderComponent();
      
      expect(screen.getByDisplayValue('Sample ID')).toBeInTheDocument();
      expect(screen.getByDisplayValue('Concentration')).toBeInTheDocument();
    });

    it('allows adding new fields', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const addButton = screen.getByText('Add Field');
      await user.click(addButton);
      
      // Check that a new field form appears
      const fieldNameInputs = screen.getAllByPlaceholderText(/field name/i);
      expect(fieldNameInputs.length).toBeGreaterThan(2);
    });

    it('allows removing fields', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const removeButtons = screen.getAllByText('Remove');
      await user.click(removeButtons[0]);
      
      // Should have one less field
      await waitFor(() => {
        expect(screen.queryByDisplayValue('Sample ID')).not.toBeInTheDocument();
      });
    });
  });

  describe('Modal Controls', () => {
    it('calls onClose when cancel button is clicked', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const cancelButton = screen.getByText('Cancel');
      await user.click(cancelButton);
      
      expect(mockOnClose).toHaveBeenCalled();
    });

    it('calls onClose when close (X) button is clicked', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const closeButton = screen.getByLabelText('Close');
      await user.click(closeButton);
      
      expect(mockOnClose).toHaveBeenCalled();
    });

    it('calls onSave when save button is clicked', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const saveButton = screen.getByText('Save Template');
      await user.click(saveButton);
      
      expect(mockOnSave).toHaveBeenCalled();
    });
  });

  describe('Validation', () => {
    it('requires template name', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const nameInput = screen.getByDisplayValue('Sample Template');
      await user.clear(nameInput);
      
      const saveButton = screen.getByText('Save Template');
      await user.click(saveButton);
      
      expect(screen.getByText('Template name is required')).toBeInTheDocument();
    });

    it('validates field names are unique', async () => {
      const user = userEvent.setup();
      renderComponent();
      
      const addButton = screen.getByText('Add Field');
      await user.click(addButton);
      
      const fieldNameInputs = screen.getAllByPlaceholderText(/field name/i);
      await user.type(fieldNameInputs[fieldNameInputs.length - 1], 'Sample ID');
      
      const saveButton = screen.getByText('Save Template');
      await user.click(saveButton);
      
      expect(screen.getByText('Field names must be unique')).toBeInTheDocument();
    });
  });

  describe('Null Template Handling', () => {
    it('handles null template gracefully', () => {
      renderComponent(true, null);
      
      expect(screen.queryByText('Edit Template')).not.toBeInTheDocument();
    });
  });
});