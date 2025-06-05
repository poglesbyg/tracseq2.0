import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import axios from 'axios';
import { CheckCircleIcon, BeakerIcon } from '@heroicons/react/24/outline';

interface SheetData {
  name: string;
  headers: string[];
  rows: string[][];
  total_rows: number;
  total_columns: number;
}

interface SpreadsheetData {
  sheet_names: string[];
  sheets: SheetData[];
}

interface Template {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  metadata: Record<string, any>;
}

interface StorageLocation {
  id: number;
  name: string;
  capacity?: number;
  available?: number;
}

interface ParsedTemplateResponse {
  template: Template;
  data: SpreadsheetData;
}

interface BatchSampleCreationProps {
  templateData: ParsedTemplateResponse;
  onClose: () => void;
  onComplete: () => void;
}

export default function BatchSampleCreation({ templateData, onClose, onComplete }: BatchSampleCreationProps) {
  const { template, data } = templateData;
  const activeSheet = data.sheets[0];
  
  const [nameColumnIndex, setNameColumnIndex] = useState(0);
  const [defaultStorageLocation, setDefaultStorageLocation] = useState<string>('');
  const queryClient = useQueryClient();

  // Fetch storage locations (optional, fallback to default locations)
  const { data: storageLocations } = useQuery<StorageLocation[]>({
    queryKey: ['storage-locations'],
    queryFn: async () => {
      try {
        const response = await axios.get('/api/storage/locations');
        return response.data;
      } catch (error) {
        // Return default locations if API fails
        return [
          { id: 1, name: 'Lab Room A' },
          { id: 2, name: 'Lab Room B' },
          { id: 3, name: 'Storage Freezer' },
          { id: 4, name: 'Sample Storage' },
        ];
      }
    },
  });

  // Batch create samples mutation
  const createSamplesMutation = useMutation({
    mutationFn: async (samples: any[]) => {
      const response = await axios.post('/api/samples/batch', { samples });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['samples'] });
      onComplete();
    },
  });

  const generateBarcode = (index: number): string => {
    const year = new Date().getFullYear();
    const paddedIndex = String(index + 1).padStart(4, '0');
    return `LAB-${year}-${paddedIndex}`;
  };

  const handleCreateSamples = () => {
    const samples = activeSheet.rows.map((row, index) => ({
      name: row[nameColumnIndex] || `Sample ${index + 1}`,
      barcode: generateBarcode(index),
      location: defaultStorageLocation,
      metadata: {
        template_id: template.id,
        template_name: template.name,
        source_template: template.name,
        original_row: index,
        template_data: Object.fromEntries(
          activeSheet.headers.map((header, headerIndex) => [header, row[headerIndex]])
        ),
      },
    }));

    createSamplesMutation.mutate(samples);
  };

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-4 mx-auto p-5 border w-11/12 max-w-2xl shadow-lg rounded-md bg-white">
        <div className="flex justify-between items-center mb-6">
          <div>
            <h2 className="text-xl font-semibold text-gray-900">Create Samples from Template</h2>
            <p className="text-sm text-gray-500">
              {template.name} • {activeSheet.rows.length} rows
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
          >
            <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="space-y-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Sample Name Column
            </label>
            <select
              value={nameColumnIndex}
              onChange={(e) => setNameColumnIndex(Number(e.target.value))}
              className="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
            >
              {activeSheet.headers.map((header, index) => (
                <option key={index} value={index}>
                  {header} (Column {index + 1})
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Default Storage Location
            </label>
            <select
              value={defaultStorageLocation}
              onChange={(e) => setDefaultStorageLocation(e.target.value)}
              className="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
            >
              <option value="">Select a location</option>
              {storageLocations?.map((location) => (
                <option key={location.id} value={location.name}>
                  {location.name} {location.available ? `(${location.available} available)` : ''}
                </option>
              ))}
            </select>
          </div>

          <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <div className="flex">
              <BeakerIcon className="h-5 w-5 text-blue-400 mt-0.5 mr-3" />
              <div>
                <h4 className="text-sm font-medium text-blue-800">Sample Creation Summary</h4>
                <div className="mt-2 text-sm text-blue-700">
                  <ul className="list-disc list-inside space-y-1">
                    <li>{activeSheet.rows.length} samples will be created</li>
                    <li>Each sample will receive a unique barcode</li>
                    <li>Template data will be preserved in sample metadata</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>

          <div className="bg-gray-50 rounded-lg p-4">
            <h4 className="text-sm font-medium text-gray-900 mb-2">Preview Sample Names</h4>
            <div className="grid grid-cols-2 gap-2 text-sm">
              {activeSheet.rows.slice(0, 6).map((row, index) => (
                <div key={index} className="text-gray-600">
                  {row[nameColumnIndex] || `Sample ${index + 1}`}
                </div>
              ))}
              {activeSheet.rows.length > 6 && (
                <div className="text-gray-400 col-span-2">
                  ... and {activeSheet.rows.length - 6} more
                </div>
              )}
            </div>
          </div>

          {createSamplesMutation.error && (
            <div className="bg-red-50 border border-red-200 rounded-lg p-4">
              <h4 className="text-sm font-medium text-red-800">Error Creating Samples</h4>
              <p className="mt-2 text-sm text-red-700">
                {createSamplesMutation.error.message || 'Failed to create samples. Please try again.'}
              </p>
            </div>
          )}
        </div>

        <div className="flex justify-between mt-8">
          <button
            onClick={onClose}
            className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
          >
            Cancel
          </button>
          <button
            onClick={handleCreateSamples}
            disabled={createSamplesMutation.isPending || defaultStorageLocation === ''}
            className="inline-flex items-center px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50"
          >
            {createSamplesMutation.isPending ? (
              <>
                <svg className="animate-spin -ml-1 mr-3 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Creating...
              </>
            ) : (
              <>
                <CheckCircleIcon className="h-4 w-4 mr-2" />
                Create {activeSheet.rows.length} Samples
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
} 
