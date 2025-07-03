import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import axios from 'axios';
import { BeakerIcon, DocumentIcon, MapPinIcon, CheckCircleIcon } from '@heroicons/react/24/outline';

interface Template {
  id: number;
  name: string;
  version: string;
}

interface StorageLocation {
  id: number;
  name: string;
  capacity: number;
  available: number;
}

interface SampleData {
  name: string;
  barcode: string;
  template_id: string;
  storage_location_id: string;
  metadata: Record<string, string>;
}

interface SampleSubmissionData {
  name: string;
  barcode: string;
  location: string;
  metadata: Record<string, string>;
}

interface SampleSubmissionWizardProps {
  onSuccess?: () => void;
  onClose?: () => void;
}

const steps = [
  { id: 'template', name: 'Template Selection', icon: DocumentIcon },
  { id: 'details', name: 'Sample Details', icon: BeakerIcon },
  { id: 'storage', name: 'Storage Location', icon: MapPinIcon },
  { id: 'confirm', name: 'Confirmation', icon: CheckCircleIcon },
];

export default function SampleSubmissionWizard({ onSuccess, onClose }: SampleSubmissionWizardProps) {
  const [currentStep, setCurrentStep] = useState(0);
  const [formData, setFormData] = useState<SampleData>({
    name: '',
    barcode: '',
    template_id: '',
    storage_location_id: '',
    metadata: {},
  });
  const queryClient = useQueryClient();

  // Fetch available templates
  const { data: templates } = useQuery<Template[]>({
    queryKey: ['templates'],
    queryFn: async () => {
      const response = await axios.get('/api/templates');
      // Handle API response format { data: [...], pagination: {...}, success: true }
      if (response.data && typeof response.data === 'object' && Array.isArray(response.data.data)) {
        return response.data.data;
      }
      // Fallback to direct array or templates property
      return Array.isArray(response.data) ? response.data : (response.data.templates || []);
    },
  });

  // Fetch storage locations
  const { data: storageLocations } = useQuery<StorageLocation[]>({
    queryKey: ['storage-locations'],
    queryFn: async () => {
      const response = await axios.get('/api/storage/locations');
      // Handle both response formats - direct array or nested in data/locations
      return Array.isArray(response.data) 
        ? response.data 
        : (response.data.data || response.data.locations || []);
    },
  });

  // Submit sample mutation
  const submitSample = useMutation({
    mutationFn: async (data: SampleSubmissionData) => {
      const response = await axios.post('/api/samples', data);
      return response.data;
    },
    onSuccess: () => {
      // Refresh the samples list
      queryClient.invalidateQueries({ queryKey: ['samples'] });
      
      // Notify parent component of success
      onSuccess?.();
      
      // Close the modal
      onClose?.();
    },
    onError: (error) => {
      console.error('Failed to create sample:', error);
    },
  });

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    }
  };

  const handleBack = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSubmit = () => {
    // Generate barcode if not provided
    const barcode = formData.barcode || `LAB-${Date.now()}-${Math.random().toString(36).substr(2, 4).toUpperCase()}`;
    
    // Get location name from storage locations
    const selectedLocation = storageLocations?.find(l => l.id === Number(formData.storage_location_id));
    const locationName = selectedLocation?.name || 'Unknown Location';
    
    const submissionData: SampleSubmissionData = {
      name: formData.name,
      barcode: barcode,
      location: locationName,
      metadata: {
        ...formData.metadata,
        template_id: formData.template_id,
        storage_location_id: formData.storage_location_id,
      },
    };
    
    submitSample.mutate(submissionData);
  };

  const renderStep = () => {
    switch (currentStep) {
      case 0:
        return (
          <div className="space-y-4">
            <h3 className="text-lg font-medium">Select Template</h3>
            <select
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              value={formData.template_id}
              onChange={(e) => setFormData({ ...formData, template_id: e.target.value })}
              data-testid="sample-type-select"
            >
              <option value="">Select a template</option>
              {templates?.map((template) => (
                <option key={template.id} value={template.id}>
                  {template.name} (v{template.version})
                </option>
              ))}
            </select>
          </div>
        );

      case 1:
        return (
          <div className="space-y-4">
            <h3 className="text-lg font-medium">Sample Details</h3>
            <div>
              <label className="block text-sm font-medium text-gray-700">Sample Name</label>
              <input
                type="text"
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                data-testid="sample-name-input"
                required
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">Barcode (optional)</label>
              <input
                type="text"
                placeholder="Leave empty to auto-generate"
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                value={formData.barcode}
                onChange={(e) => setFormData({ ...formData, barcode: e.target.value })}
                data-testid="sample-barcode-input"
              />
              <p className="mt-1 text-sm text-gray-500">
                If left empty, a unique barcode will be generated automatically
              </p>
            </div>
            {/* Add template-specific metadata fields here */}
          </div>
        );

      case 2:
        return (
          <div className="space-y-4">
            <h3 className="text-lg font-medium">Storage Location</h3>
            <select
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              value={formData.storage_location_id}
              onChange={(e) => setFormData({ ...formData, storage_location_id: e.target.value })}
              data-testid="project-select"
            >
              <option value="">Select a storage location</option>
              {storageLocations?.map((location) => (
                <option key={location.id} value={location.id}>
                  {location.name} ({location.available} slots available)
                </option>
              ))}
            </select>
          </div>
        );

      case 3: {
        const selectedTemplate = templates?.find((t) => t.id === Number(formData.template_id));
        const selectedLocation = storageLocations?.find((l) => l.id === Number(formData.storage_location_id));
        const displayBarcode = formData.barcode || `LAB-${Date.now()}-${Math.random().toString(36).substr(2, 4).toUpperCase()}`;
        
        return (
          <div className="space-y-4">
            <h3 className="text-lg font-medium">Confirm Submission</h3>
            <div className="bg-gray-50 p-4 rounded-md">
              <dl className="grid grid-cols-1 gap-x-4 gap-y-4 sm:grid-cols-2">
                <div>
                  <dt className="text-sm font-medium text-gray-500">Sample Name</dt>
                  <dd className="mt-1 text-sm text-gray-900">{formData.name || 'Not specified'}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-gray-500">Barcode</dt>
                  <dd className="mt-1 text-sm text-gray-900">
                    {formData.barcode ? formData.barcode : <span className="text-gray-500">{displayBarcode} (auto-generated)</span>}
                  </dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-gray-500">Template</dt>
                  <dd className="mt-1 text-sm text-gray-900">
                    {selectedTemplate?.name || 'Not selected'}
                  </dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-gray-500">Storage Location</dt>
                  <dd className="mt-1 text-sm text-gray-900">
                    {selectedLocation?.name || 'Not selected'}
                  </dd>
                </div>
              </dl>
            </div>
          </div>
        );
      }

      default:
        return null;
    }
  };

  return (
    <div className="max-w-3xl mx-auto py-8">
      <nav aria-label="Progress">
        <ol className="space-y-4 md:flex md:space-y-0 md:space-x-8">
          {steps.map((step, index) => (
            <li key={step.id} className="md:flex-1">
              <div
                className={`group pl-4 py-2 flex flex-col border-l-4 ${
                  index <= currentStep ? 'border-indigo-600' : 'border-gray-200'
                } md:pl-0 md:pt-4 md:pb-0 md:border-l-0 md:border-t-4`}
              >
                <span className="text-xs font-semibold tracking-wide uppercase">
                  {step.name}
                </span>
              </div>
            </li>
          ))}
        </ol>
      </nav>

      <div className="mt-8">
        {renderStep()}
      </div>

      {submitSample.error && (
        <div className="mt-6 bg-red-50 border border-red-200 rounded-lg p-4">
          <h4 className="text-sm font-medium text-red-800">Error Creating Sample</h4>
          <p className="mt-2 text-sm text-red-700">
            {submitSample.error.message || 'Failed to create sample. Please try again.'}
          </p>
        </div>
      )}

      <div className="mt-8 flex justify-between">
        <button
          type="button"
          onClick={handleBack}
          disabled={currentStep === 0}
          className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
        >
          Back
        </button>
        {currentStep === steps.length - 1 ? (
          <button
            type="button"
            onClick={handleSubmit}
            disabled={submitSample.isPending || !formData.name.trim() || !formData.storage_location_id}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
            data-testid="submit-sample-button"
          >
            {submitSample.isPending ? (
              <>
                <svg className="animate-spin -ml-1 mr-3 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Creating Sample...
              </>
            ) : (
              'Submit Sample'
            )}
          </button>
        ) : (
          <button
            type="button"
            onClick={handleNext}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            Next
          </button>
        )}
      </div>
    </div>
  );
} 
