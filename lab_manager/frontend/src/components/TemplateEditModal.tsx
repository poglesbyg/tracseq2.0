import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import axios from 'axios';

  interface Template {
    id: string;
    name: string;
    description?: string;
    created_at: string;
    fields: TemplateField[];
    metadata?: Record<string, unknown>;
  }

interface TemplateField {
  name: string;
  type: string;
  required: boolean;
  defaultValue?: string | number | boolean | null;
}

interface UpdateTemplate {
  name?: string;
  description?: string;
  metadata?: Record<string, any>;
}

interface TemplateEditModalProps {
  isOpen: boolean;
  onClose: () => void;
  template: Template | null;
  onSave: (template: Template) => void;
}

export default function TemplateEditModal({ template, onClose }: TemplateEditModalProps) {
  const [formData, setFormData] = useState<Template>({
    id: template?.id || '',
    name: template?.name || '',
    description: template?.description || '',
    created_at: template?.created_at || '',
    fields: template?.fields || [],
    metadata: template?.metadata || {},
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const queryClient = useQueryClient();

  const updateMutation = useMutation({
    mutationFn: async (updates: UpdateTemplate) => {
      const response = await axios.put(`/api/templates/${template?.id}`, updates);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['templates'] });
      queryClient.invalidateQueries({ queryKey: ['dashboardStats'] });
      onClose();
    },
    onError: (error: any) => {
      console.error('Failed to update template:', error);
      setErrors({ general: 'Failed to update template. Please try again.' });
    },
  });

  const validateForm = () => {
    const newErrors: Record<string, string> = {};

    if (!formData.name.trim()) {
      newErrors.name = 'Template name is required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    // Only send changed fields
    const updates: UpdateTemplate = {};
    if (formData.name !== template?.name) updates.name = formData.name;
    if (formData.description !== (template?.description || '')) {
      updates.description = formData.description || undefined;
    }
    if (formData.metadata !== template?.metadata) updates.metadata = formData.metadata;

    // If no changes, just close the modal
    if (Object.keys(updates).length === 0) {
      onClose();
      return;
    }

    updateMutation.mutate(updates);
  };

  const handleInputChange = (field: keyof typeof formData, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    // Clear error when user starts typing
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: '' }));
    }
  };

  return (
    <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-md w-full">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-medium text-gray-900">Edit Template</h3>
            <button
              type="button"
              onClick={onClose}
              className="text-gray-400 hover:text-gray-500 focus:outline-none"
            >
              <span className="sr-only">Close</span>
              <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="px-6 py-4 space-y-4">
          {errors.general && (
            <div className="rounded-md bg-red-50 p-4">
              <div className="text-sm text-red-700">{errors.general}</div>
            </div>
          )}

          <div>
            <label htmlFor="name" className="block text-sm font-medium text-gray-700 mb-1">
              Template Name
            </label>
            <input
              type="text"
              id="name"
              value={formData.name}
              onChange={(e) => handleInputChange('name', e.target.value)}
              className={`w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm ${
                errors.name ? 'border-red-300' : ''
              }`}
              placeholder="Enter template name"
            />
            {errors.name && <p className="mt-1 text-sm text-red-600">{errors.name}</p>}
          </div>

          <div>
            <label htmlFor="description" className="block text-sm font-medium text-gray-700 mb-1">
              Description
            </label>
            <textarea
              id="description"
              rows={3}
              value={formData.description}
              onChange={(e) => handleInputChange('description', e.target.value)}
              className="w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
              placeholder="Enter template description (optional)"
            />
          </div>

          {/* Template Info */}
          <div className="border-t border-gray-200 pt-4">
            <h4 className="text-sm font-medium text-gray-900 mb-2">Template Information</h4>
            <dl className="space-y-1 text-sm">
              <div className="flex justify-between">
                <dt className="text-gray-500">ID:</dt>
                <dd className="text-gray-900 font-mono text-xs">{template?.id}</dd>
              </div>
              <div className="flex justify-between">
                <dt className="text-gray-500">Created:</dt>
                <dd className="text-gray-900">{new Date(template?.created_at || '').toLocaleDateString()}</dd>
              </div>
                              <div className="flex justify-between">
                  <dt className="text-gray-500">File Type:</dt>
                  <dd className="text-gray-900">{typeof template?.metadata?.file_type === 'string' ? template.metadata.file_type : 'Unknown'}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-gray-500">Original File:</dt>
                  <dd className="text-gray-900 text-xs">{typeof template?.metadata?.originalFileName === 'string' ? template.metadata.originalFileName : 'Unknown'}</dd>
                </div>
            </dl>
          </div>
        </form>

        <div className="px-6 py-4 bg-gray-50 border-t border-gray-200 flex justify-end space-x-3">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            disabled={updateMutation.isPending}
          >
            Cancel
          </button>
          <button
            type="submit"
            onClick={handleSubmit}
            disabled={updateMutation.isPending}
            className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {updateMutation.isPending ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  );
} 
