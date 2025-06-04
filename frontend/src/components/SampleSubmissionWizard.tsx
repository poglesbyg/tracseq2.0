import React, { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
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
  template_id: number;
  storage_location_id: number;
  metadata: Record<string, string>;
}

const steps = [
  { id: 'template', name: 'Template Selection', icon: DocumentIcon },
  { id: 'details', name: 'Sample Details', icon: BeakerIcon },
  { id: 'storage', name: 'Storage Location', icon: MapPinIcon },
  { id: 'confirm', name: 'Confirmation', icon: CheckCircleIcon },
];

export default function SampleSubmissionWizard() {
  const [currentStep, setCurrentStep] = useState(0);
  const [formData, setFormData] = useState<SampleData>({
    name: '',
    template_id: 0,
    storage_location_id: 0,
    metadata: {},
  });

  // Fetch available templates
  const { data: templates } = useQuery<Template[]>({
    queryKey: ['templates'],
    queryFn: async () => {
      const response = await axios.get('/api/templates');
      return response.data;
    },
  });

  // Fetch storage locations
  const { data: storageLocations } = useQuery<StorageLocation[]>({
    queryKey: ['storage-locations'],
    queryFn: async () => {
      const response = await axios.get('/api/storage/locations');
      return response.data;
    },
  });

  // Submit sample mutation
  const submitSample = useMutation({
    mutationFn: async (data: SampleData) => {
      const response = await axios.post('/api/samples', data);
      return response.data;
    },
    onSuccess: () => {
      // Handle success (e.g., show success message, redirect)
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
    submitSample.mutate(formData);
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
              onChange={(e) => setFormData({ ...formData, template_id: Number(e.target.value) })}
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
              />
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
              onChange={(e) => setFormData({ ...formData, storage_location_id: Number(e.target.value) })}
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

      case 3:
        return (
          <div className="space-y-4">
            <h3 className="text-lg font-medium">Confirm Submission</h3>
            <div className="bg-gray-50 p-4 rounded-md">
              <dl className="grid grid-cols-1 gap-x-4 gap-y-4 sm:grid-cols-2">
                <div>
                  <dt className="text-sm font-medium text-gray-500">Sample Name</dt>
                  <dd className="mt-1 text-sm text-gray-900">{formData.name}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-gray-500">Template</dt>
                  <dd className="mt-1 text-sm text-gray-900">
                    {templates?.find((t) => t.id === formData.template_id)?.name}
                  </dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-gray-500">Storage Location</dt>
                  <dd className="mt-1 text-sm text-gray-900">
                    {storageLocations?.find((l) => l.id === formData.storage_location_id)?.name}
                  </dd>
                </div>
              </dl>
            </div>
          </div>
        );

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
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            Submit Sample
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
