import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '../utils/axios';
import { 
  XMarkIcon,
  MagnifyingGlassIcon,
  BeakerIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon
} from '@heroicons/react/24/outline';

interface Sample {
  id: string;
  barcode: string;
  sample_type: string;
  status: string;
  temperature_requirements?: string;
  created_at: string;
}

interface SampleAssignmentModalProps {
  isOpen: boolean;
  onClose: () => void;
  positionId: string;
  positionName: string;
  containerPath: string;
  temperatureZone?: string;
  onSuccess?: () => void;
}

export default function SampleAssignmentModal({
  isOpen,
  onClose,
  positionId,
  positionName,
  containerPath,
  temperatureZone,
  onSuccess
}: SampleAssignmentModalProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedSample, setSelectedSample] = useState<Sample | null>(null);
  const [assignmentNotes, setAssignmentNotes] = useState('');
  
  const queryClient = useQueryClient();

  // Fetch available samples (not already assigned)
  const { data: samplesData, isLoading: isLoadingSamples } = useQuery({
    queryKey: ['available-samples', searchQuery],
    queryFn: async () => {
      const params = new URLSearchParams();
      if (searchQuery) {
        params.append('search', searchQuery);
      }
      params.append('status', 'pending,validated'); // Only samples ready for storage
      params.append('not_assigned', 'true'); // Only unassigned samples
      
      const response = await api.get(`/api/samples?${params.toString()}`);
      return response.data;
    },
    enabled: isOpen,
  });

  // Sample assignment mutation
  const assignSampleMutation = useMutation({
    mutationFn: async (assignmentData: Record<string, unknown>) => {
      const response = await api.post('/api/storage/samples/assign', assignmentData);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['storage-container-details'] });
      queryClient.invalidateQueries({ queryKey: ['storage-grid'] });
      queryClient.invalidateQueries({ queryKey: ['available-samples'] });
      if (onSuccess) onSuccess();
      onClose();
    },
  });

  const handleAssignment = () => {
    if (!selectedSample) return;

    const assignmentData = {
      sample_id: selectedSample.id,
      container_id: positionId,
      position_identifier: positionName,
      notes: assignmentNotes || undefined,
      storage_conditions: {
        temperature_zone: temperatureZone,
        assigned_at: new Date().toISOString(),
      },
    };

    assignSampleMutation.mutate(assignmentData);
  };

  const isTemperatureCompatible = (sample: Sample) => {
    if (!sample.temperature_requirements || !temperatureZone) return true;
    
    const sampleTemp = sample.temperature_requirements.toLowerCase();
    const zoneTemp = temperatureZone.toLowerCase();
    
    // Basic temperature compatibility check
    if (sampleTemp.includes('-80') && zoneTemp.includes('-80')) return true;
    if (sampleTemp.includes('-20') && zoneTemp.includes('-20')) return true;
    if (sampleTemp.includes('4') && zoneTemp.includes('4')) return true;
    if (sampleTemp.includes('rt') && zoneTemp.includes('rt')) return true;
    
    return false;
  };

  const filteredSamples = samplesData?.data?.filter((sample: Sample) =>
    sample.barcode.toLowerCase().includes(searchQuery.toLowerCase()) ||
    sample.sample_type.toLowerCase().includes(searchQuery.toLowerCase())
  ) || [];

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] overflow-hidden">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-lg font-medium text-gray-900">Assign Sample to Position</h3>
              <p className="text-sm text-gray-500 mt-1">
                {containerPath} → {positionName}
                {temperatureZone && (
                  <span className="ml-2 inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                    {temperatureZone}
                  </span>
                )}
              </p>
            </div>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-500"
            >
              <XMarkIcon className="h-6 w-6" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto">
          {/* Search */}
          <div className="p-6 border-b border-gray-200">
            <div className="relative">
              <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
              </div>
              <input
                type="text"
                className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                placeholder="Search samples by barcode or type..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
            </div>
          </div>

          {/* Sample List */}
          <div className="p-6">
            {isLoadingSamples ? (
              <div className="flex justify-center py-8">
                <div className="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-600"></div>
              </div>
            ) : filteredSamples.length === 0 ? (
              <div className="text-center py-8">
                <BeakerIcon className="mx-auto h-12 w-12 text-gray-400" />
                <h3 className="mt-2 text-sm font-medium text-gray-900">No samples found</h3>
                <p className="mt-1 text-sm text-gray-500">
                  {searchQuery ? 'Try adjusting your search terms.' : 'No samples available for assignment.'}
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                <h4 className="text-sm font-medium text-gray-900">
                  Available Samples ({filteredSamples.length})
                </h4>
                <div className="max-h-60 overflow-y-auto space-y-2">
                  {filteredSamples.map((sample: Sample) => {
                    const isSelected = selectedSample?.id === sample.id;
                    const isCompatible = isTemperatureCompatible(sample);
                    
                    return (
                      <div
                        key={sample.id}
                        className={`
                          p-3 border rounded-lg cursor-pointer transition-colors
                          ${isSelected 
                            ? 'border-indigo-500 bg-indigo-50' 
                            : isCompatible
                              ? 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
                              : 'border-yellow-200 bg-yellow-50'
                          }
                        `}
                        onClick={() => setSelectedSample(sample)}
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center space-x-3">
                            <BeakerIcon className="h-5 w-5 text-gray-400" />
                            <div>
                              <div className="text-sm font-medium text-gray-900">
                                {sample.barcode}
                              </div>
                              <div className="text-sm text-gray-500">
                                {sample.sample_type} • {sample.status}
                                {sample.temperature_requirements && (
                                  <span className="ml-2">• {sample.temperature_requirements}</span>
                                )}
                              </div>
                            </div>
                          </div>
                          <div className="flex items-center space-x-2">
                            {!isCompatible && (
                              <ExclamationTriangleIcon 
                                className="h-5 w-5 text-yellow-500" 
                                title="Temperature requirements may not match storage zone"
                              />
                            )}
                            {isSelected && (
                              <CheckCircleIcon className="h-5 w-5 text-indigo-600" />
                            )}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </div>

          {/* Assignment Notes */}
          {selectedSample && (
            <div className="px-6 pb-6">
              <label htmlFor="assignment-notes" className="block text-sm font-medium text-gray-700 mb-2">
                Assignment Notes (Optional)
              </label>
              <textarea
                id="assignment-notes"
                rows={3}
                className="block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                placeholder="Add any notes about this sample assignment..."
                value={assignmentNotes}
                onChange={(e) => setAssignmentNotes(e.target.value)}
              />
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-200 bg-gray-50">
          <div className="flex justify-between items-center">
            <div className="text-sm text-gray-500">
              {selectedSample ? (
                <span>
                  Selected: <strong>{selectedSample.barcode}</strong>
                  {!isTemperatureCompatible(selectedSample) && (
                    <span className="ml-2 text-yellow-600">⚠️ Temperature mismatch</span>
                  )}
                </span>
              ) : (
                'Select a sample to assign to this position'
              )}
            </div>
            <div className="flex space-x-3">
              <button
                onClick={onClose}
                className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleAssignment}
                disabled={!selectedSample || assignSampleMutation.isPending}
                className="px-4 py-2 bg-indigo-600 border border-transparent rounded-md text-sm font-medium text-white hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {assignSampleMutation.isPending ? 'Assigning...' : 'Assign Sample'}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
} 