import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import axios from 'axios';

interface Sample {
  id: string;
  name: string;
  barcode: string;
  location: string;
  status: 'Pending' | 'Validated' | 'InStorage' | 'InSequencing' | 'Completed';
  created_at: string;
  updated_at: string;
  metadata?: Record<string, unknown>;
}

interface UpdateSample {
  name?: string;
  barcode?: string;
  location?: string;
  status?: 'Pending' | 'Validated' | 'InStorage' | 'InSequencing' | 'Completed';
  metadata?: any;
}

interface SampleEditModalProps {
  isOpen: boolean;
  onClose: () => void;
  sample: Sample | null;
  onSave: (sample: Sample) => void;
}

const statusOptions = [
  { value: 'Pending', label: 'Pending', color: 'yellow' },
  { value: 'Validated', label: 'Validated', color: 'blue' },
  { value: 'InStorage', label: 'In Storage', color: 'purple' },
  { value: 'InSequencing', label: 'In Sequencing', color: 'indigo' },
  { value: 'Completed', label: 'Completed', color: 'green' },
];

export default function SampleEditModal({ isOpen, onClose, sample, onSave }: SampleEditModalProps) {
  const [formData, setFormData] = useState<Sample>({
    id: sample?.id || '',
    name: sample?.name || '',
    barcode: sample?.barcode || '',
    location: sample?.location || '',
    status: sample?.status || 'Pending',
    created_at: sample?.created_at || '',
    updated_at: sample?.updated_at || '',
    metadata: sample?.metadata || {},
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const queryClient = useQueryClient();

  const updateMutation = useMutation({
    mutationFn: async (updates: UpdateSample) => {
      const response = await axios.put(`/api/samples/${sample?.id}`, updates);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['samples'] });
      queryClient.invalidateQueries({ queryKey: ['dashboardStats'] });
      onClose();
    },
    onError: (error: any) => {
      console.error('Failed to update sample:', error);
      setErrors({ general: 'Failed to update sample. Please try again.' });
    },
  });

  const validateForm = () => {
    const newErrors: Record<string, string> = {};

    if (!formData.name.trim()) {
      newErrors.name = 'Sample name is required';
    }

    if (!formData.barcode.trim()) {
      newErrors.barcode = 'Barcode is required';
    }

    if (!formData.location.trim()) {
      newErrors.location = 'Location is required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    // Only send changed fields
    const updates: UpdateSample = {};
    if (formData.name !== sample?.name) updates.name = formData.name;
    if (formData.barcode !== sample?.barcode) updates.barcode = formData.barcode;
    if (formData.location !== sample?.location) updates.location = formData.location;
    if (formData.status !== sample?.status) updates.status = formData.status;

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
            <h3 className="text-lg font-medium text-gray-900">Edit Sample</h3>
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
              Sample Name
            </label>
            <input
              type="text"
              id="name"
              value={formData.name}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({...formData, name: e.target.value})}
              className={`w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm ${
                errors.name ? 'border-red-300' : ''
              }`}
              placeholder="Enter sample name"
            />
            {errors.name && <p className="mt-1 text-sm text-red-600">{errors.name}</p>}
          </div>

          <div>
            <label htmlFor="barcode" className="block text-sm font-medium text-gray-700 mb-1">
              Barcode
            </label>
            <input
              type="text"
              id="barcode"
              value={formData.barcode}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({...formData, barcode: e.target.value})}
              className={`w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm ${
                errors.barcode ? 'border-red-300' : ''
              }`}
              placeholder="Enter barcode"
            />
            {errors.barcode && <p className="mt-1 text-sm text-red-600">{errors.barcode}</p>}
          </div>

          <div>
            <label htmlFor="location" className="block text-sm font-medium text-gray-700 mb-1">
              Location
            </label>
            <input
              type="text"
              id="location"
              value={formData.location}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({...formData, location: e.target.value})}
              className={`w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm ${
                errors.location ? 'border-red-300' : ''
              }`}
              placeholder="Enter storage location"
            />
            {errors.location && <p className="mt-1 text-sm text-red-600">{errors.location}</p>}
          </div>

          <div>
            <label htmlFor="status" className="block text-sm font-medium text-gray-700 mb-1">
              Status
            </label>
            <select
              id="status"
              value={formData.status}
              onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setFormData({...formData, status: e.target.value as 'Pending' | 'Validated' | 'InStorage' | 'InSequencing' | 'Completed'})}
              className="w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
            >
              {statusOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          {/* Sample Info */}
          <div className="border-t border-gray-200 pt-4">
            <h4 className="text-sm font-medium text-gray-900 mb-2">Sample Information</h4>
                          <dl className="space-y-1 text-sm">
                <div className="flex justify-between">
                  <dt className="text-gray-500">ID:</dt>
                  <dd className="text-gray-900 font-mono text-xs">{sample?.id}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-gray-500">Created:</dt>
                  <dd className="text-gray-900">{sample?.created_at ? new Date(sample.created_at).toLocaleDateString() : 'N/A'}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-gray-500">Last Updated:</dt>
                  <dd className="text-gray-900">{sample?.updated_at ? new Date(sample.updated_at).toLocaleDateString() : 'N/A'}</dd>
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
