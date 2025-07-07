import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import api from '../utils/axios';
import { CheckCircleIcon, BeakerIcon, MapPinIcon, ExclamationTriangleIcon } from '@heroicons/react/24/outline';

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
  metadata: Record<string, unknown>;
}

interface StorageLocation {
  id: number | string;
  name: string;
  capacity?: number;
  available?: number;
  max_capacity?: number;
  current_capacity?: number;
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

interface SampleData {
  name: string;
  barcode: string;
  location: string;
  metadata: {
    template_id: string;
    template_name: string;
    source_template: string;
    original_row: number;
    template_data: Record<string, string>;
  };
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
      const defaultLocations = [
        { id: 1, name: 'Lab Room A', capacity: 100, available: 80 },
        { id: 2, name: 'Lab Room B', capacity: 150, available: 120 },
        { id: 3, name: 'Storage Freezer (-80°C)', capacity: 500, available: 350 },
        { id: 4, name: 'Sample Storage (4°C)', capacity: 200, available: 180 },
      ];

      try {
        const response = await api.get('/api/storage/locations');
        // Handle both response formats - direct array or nested in data/locations
        const locations = response.data?.data || response.data?.locations || response.data || [];
        
        // If API returns empty array or invalid data, use default locations
        if (!Array.isArray(locations) || locations.length === 0) {
          console.log('🏢 Storage locations API returned empty data, using fallback locations');
          return defaultLocations;
        }
        
        console.log('🏢 Storage locations loaded from API:', locations.length, 'locations');
        return locations;
      } catch (error) {
        console.warn('🏢 Storage locations API failed, using fallback locations:', error);
        return defaultLocations;
      }
    },
  });

  // Batch create samples mutation with storage integration
  const createSamplesMutation = useMutation({
    mutationFn: async (samples: SampleData[]) => {
      const response = await api.post('/api/samples/batch', { samples });
      
      // If samples were created successfully and we have a storage location, store them
      if (response.data?.success && defaultStorageLocation && samples.length > 0) {
        const selectedLocation = Array.isArray(storageLocations) ? storageLocations.find(loc => loc.name === defaultStorageLocation) : null;
        
        if (selectedLocation && response.data.data?.created_samples) {
          // Store each sample in the selected location
          for (const sample of response.data.data.created_samples) {
            try {
              await api.post('/api/storage/samples', {
                barcode: sample.barcode,
                sample_type: sample.sample_type || 'unknown',
                storage_location_id: selectedLocation.id,
                temperature_requirements: getTemperatureForLocation(selectedLocation),
                metadata: {
                  batch_created: true,
                  template_id: template.id,
                  created_from_batch: new Date().toISOString()
                }
              });
            } catch (error) {
              console.warn(`Failed to store sample ${sample.barcode} in storage:`, error);
            }
          }
        }
      }
      
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['samples'] });
      queryClient.invalidateQueries({ queryKey: ['storage-samples'] });
      queryClient.invalidateQueries({ queryKey: ['storage-locations'] });
      onComplete();
    },
  });

  // Helper function to determine temperature requirements based on location
  const getTemperatureForLocation = (location: StorageLocation) => {
    const name = location.name.toLowerCase();
    if (name.includes('-80') || name.includes('freezer')) return '-80C';
    if (name.includes('-20')) return '-20C';
    if (name.includes('4°c') || name.includes('4c') || name.includes('refrigerator')) return '4C';
    return 'RT'; // Room temperature default
  };

  // Helper function to get temperature color and icon
  const getTemperatureInfo = (location: StorageLocation) => {
    const temp = getTemperatureForLocation(location);
    switch (temp) {
      case '-80C':
        return { color: 'text-blue-600 bg-blue-50', icon: '🧊', label: '-80°C (Ultra-Low)' };
      case '-20C':
        return { color: 'text-cyan-600 bg-cyan-50', icon: '❄️', label: '-20°C (Freezer)' };
      case '4C':
        return { color: 'text-green-600 bg-green-50', icon: '🧊', label: '4°C (Refrigerated)' };
      default:
        return { color: 'text-orange-600 bg-orange-50', icon: '🌡️', label: 'Room Temperature' };
    }
  };

  // Helper function to get capacity status
  const getCapacityStatus = (location: StorageLocation) => {
    const available = location.available || (location.max_capacity ? location.max_capacity - (location.current_capacity || 0) : 0);
    const total = location.capacity || location.max_capacity || 100;
    const percentage = total > 0 ? ((total - available) / total) * 100 : 0;
    
    if (percentage >= 90) return { status: 'critical', color: 'bg-red-500', textColor: 'text-red-700' };
    if (percentage >= 75) return { status: 'warning', color: 'bg-yellow-500', textColor: 'text-yellow-700' };
    return { status: 'good', color: 'bg-green-500', textColor: 'text-green-700' };
  };

  const generateBarcode = (index: number): string => {
    const year = new Date().getFullYear();
    const paddedIndex = String(index + 1).padStart(4, '0');
    return `LAB-${year}-${paddedIndex}`;
  };

  const handleCreateSamples = () => {
    const samples = activeSheet.rows.map((row, index) => ({
      name: row[nameColumnIndex] || `Sample ${index + 1}`,
      sample_type: "DNA", // Default sample type - could be made configurable
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

  const selectedLocationData = Array.isArray(storageLocations) 
    ? storageLocations.find(loc => loc.name === defaultStorageLocation)
    : null;

  return (
    <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
      <div className="relative top-4 mx-auto p-5 border w-11/12 max-w-4xl shadow-lg rounded-md bg-white">
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

          {/* Enhanced Storage Location Selection */}
          <div>
            <div className="flex items-center mb-3">
              <MapPinIcon className="h-5 w-5 text-indigo-600 mr-2" />
              <label className="block text-sm font-medium text-gray-700">
                Storage Location
              </label>
              <span className="ml-2 text-red-500">*</span>
            </div>
            
            {!defaultStorageLocation && (
              <div className="mb-4 p-4 bg-amber-50 border border-amber-200 rounded-lg">
                <div className="flex items-center">
                  <ExclamationTriangleIcon className="h-5 w-5 text-amber-500 mr-2" />
                  <p className="text-sm text-amber-800 font-medium">
                    Please select a storage location to continue
                  </p>
                </div>
                <p className="text-sm text-amber-700 mt-1">
                  All {activeSheet.rows.length} samples will be automatically stored in the selected location after creation.
                </p>
              </div>
            )}

            {/* Debug info for storage locations */}
            <div className="mb-4 p-3 bg-gray-100 border border-gray-300 rounded text-xs">
              <strong>Debug:</strong> Storage locations available: {Array.isArray(storageLocations) ? storageLocations.length : 'undefined'} 
              {Array.isArray(storageLocations) && storageLocations.length > 0 && (
                <span> • Names: {storageLocations.map(loc => loc.name).join(', ')}</span>
              )}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {Array.isArray(storageLocations) && storageLocations.map((location) => {
                const available = location.available || (location.max_capacity ? location.max_capacity - (location.current_capacity || 0) : 0);
                const total = location.capacity || location.max_capacity || 100;
                const tempInfo = getTemperatureInfo(location);
                const capacityStatus = getCapacityStatus(location);
                const isSelected = defaultStorageLocation === location.name;
                const hasEnoughSpace = available >= activeSheet.rows.length;

                return (
                  <div
                    key={location.id}
                    onClick={() => hasEnoughSpace ? setDefaultStorageLocation(location.name) : null}
                    className={`
                      relative p-4 border-2 rounded-lg cursor-pointer transition-all duration-200
                      ${isSelected 
                        ? 'border-indigo-500 bg-indigo-50 ring-2 ring-indigo-200' 
                        : hasEnoughSpace
                          ? 'border-gray-200 hover:border-indigo-300 hover:bg-gray-50'
                          : 'border-gray-200 bg-gray-50 cursor-not-allowed opacity-60'
                      }
                    `}
                  >
                    {isSelected && (
                      <div className="absolute top-2 right-2">
                        <CheckCircleIcon className="h-5 w-5 text-indigo-600" />
                      </div>
                    )}
                    
                    <div className="flex items-start justify-between mb-2">
                      <h3 className="font-medium text-gray-900">{location.name}</h3>
                      <div className={`px-2 py-1 rounded-full text-xs font-medium ${tempInfo.color}`}>
                        {tempInfo.icon} {tempInfo.label}
                      </div>
                    </div>
                    
                    <div className="space-y-2">
                      <div className="flex items-center justify-between text-sm">
                        <span className="text-gray-600">Available Space:</span>
                        <span className={`font-medium ${hasEnoughSpace ? 'text-green-600' : 'text-red-600'}`}>
                          {available} / {total} slots
                        </span>
                      </div>
                      
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div 
                          className={`h-2 rounded-full ${capacityStatus.color}`}
                          style={{ width: `${Math.min(100, ((total - available) / total) * 100)}%` }}
                        />
                      </div>
                      
                      {!hasEnoughSpace && (
                        <div className="flex items-center text-xs text-red-600 mt-2">
                          <ExclamationTriangleIcon className="h-4 w-4 mr-1" />
                          Insufficient space for {activeSheet.rows.length} samples
                        </div>
                      )}
                      
                      {hasEnoughSpace && (
                        <div className="text-xs text-green-600 mt-2">
                          ✓ Can accommodate {activeSheet.rows.length} samples
                        </div>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>

            {selectedLocationData && (
              <div className="mt-4 p-4 bg-green-50 border border-green-200 rounded-lg">
                <div className="flex items-center">
                  <CheckCircleIcon className="h-5 w-5 text-green-600 mr-2" />
                  <h4 className="text-sm font-medium text-green-800">Storage Location Selected</h4>
                </div>
                <p className="text-sm text-green-700 mt-1">
                  All {activeSheet.rows.length} samples will be automatically stored in{' '}
                  <span className="font-medium">{selectedLocationData.name}</span>{' '}
                  ({getTemperatureInfo(selectedLocationData).label}) after creation.
                </p>
              </div>
            )}
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
                    {selectedLocationData && (
                      <li>Samples will be stored in {selectedLocationData.name}</li>
                    )}
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
            className={`
              inline-flex items-center px-6 py-3 rounded-md font-medium transition-all duration-200
              ${defaultStorageLocation === ''
                ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                : createSamplesMutation.isPending
                  ? 'bg-indigo-400 text-white cursor-wait'
                  : 'bg-indigo-600 text-white hover:bg-indigo-700 shadow-md hover:shadow-lg'
              }
            `}
          >
            {createSamplesMutation.isPending ? (
              <>
                <svg className="animate-spin -ml-1 mr-3 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Creating {activeSheet.rows.length} Samples...
              </>
            ) : defaultStorageLocation === '' ? (
              <>
                <MapPinIcon className="h-4 w-4 mr-2" />
                Select Storage Location First
              </>
            ) : (
              <>
                <CheckCircleIcon className="h-4 w-4 mr-2" />
                Create & Store {activeSheet.rows.length} Samples
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
} 
