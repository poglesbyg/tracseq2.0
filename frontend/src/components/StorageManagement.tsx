import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '../utils/axios';
import { MapPinIcon, QrCodeIcon, ArrowPathIcon } from '@heroicons/react/24/outline';

interface StorageLocation {
  id: number;
  name: string;
  description: string | null;
  temperature_zone: string;
  capacity: number;
  available: number;
  utilization_percentage: number;
  is_active: boolean;
  samples: StoredSample[];
}

interface StoredSample {
  id: number;
  sample_id: number;
  name: string;
  barcode: string;
  position: string | null;
  storage_state: string;
  stored_at: string;
  stored_by: string | null;
}

export default function StorageManagement() {
  const [scanningBarcode, setScanningBarcode] = useState(false);
  const queryClient = useQueryClient();

  // Fetch storage locations
  const { data: locations, isLoading: isLoadingLocations } = useQuery<StorageLocation[]>({
    queryKey: ['storage-locations'],
    queryFn: async () => {
      const response = await api.get('/api/storage/locations');
      return response.data;
    },
  });

  // Move sample mutation
  const moveSample = useMutation({
    mutationFn: async ({ barcode, locationId }: { barcode: string; locationId: number }) => {
      const response = await api.post(`/api/storage/move`, { 
        barcode, 
        location_id: locationId,
        reason: "Sample moved via storage management interface",
        moved_by: "admin"
      });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['storage-locations'] });
    },
  });

  // Scan barcode mutation
  const scanBarcode = useMutation({
    mutationFn: async (barcode: string) => {
      const response = await api.get(`/api/storage/scan/${barcode}`);
      return response.data;
    },
    onSuccess: (data) => {
      // Handle successful barcode scan
      console.log('Scanned sample:', data);
      setScanningBarcode(false);
    },
  });

  const handleBarcodeScan = (barcode: string) => {
    scanBarcode.mutate(barcode);
  };

  const handleMoveSample = (barcode: string, locationId: number) => {
    moveSample.mutate({ barcode, locationId });
  };

  const getLocationStatus = (location: StorageLocation) => {
    const percentage = (location.available / location.capacity) * 100;
    if (percentage === 0) return 'bg-red-100 text-red-800';
    if (percentage < 25) return 'bg-yellow-100 text-yellow-800';
    return 'bg-green-100 text-green-800';
  };

  return (
    <div className="space-y-6">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h2 className="text-xl font-semibold text-gray-900">Storage Management</h2>
          <p className="mt-2 text-sm text-gray-700">
            Manage storage locations and track samples using barcodes.
          </p>
        </div>
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
          <button
            type="button"
            onClick={() => setScanningBarcode(!scanningBarcode)}
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 sm:w-auto"
          >
            <QrCodeIcon className="h-5 w-5 mr-2" />
            {scanningBarcode ? 'Stop Scanning' : 'Scan Barcode'}
          </button>
        </div>
      </div>

      {scanningBarcode && (
        <div className="bg-white shadow sm:rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg font-medium leading-6 text-gray-900">Scan Barcode</h3>
            <div className="mt-2 max-w-xl text-sm text-gray-500">
              <p>Scan a sample barcode to view its details and current location.</p>
            </div>
            <div className="mt-5">
              <input
                type="text"
                className="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                placeholder="Enter or scan barcode"
                onKeyPress={(e) => {
                  if (e.key === 'Enter') {
                    handleBarcodeScan(e.currentTarget.value);
                  }
                }}
              />
            </div>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {isLoadingLocations ? (
          <div className="col-span-full flex justify-center py-12">
            <ArrowPathIcon className="h-8 w-8 animate-spin text-indigo-600" />
          </div>
        ) : (
          locations?.map((location) => (
            <div
              key={location.id}
              className="bg-white overflow-hidden shadow rounded-lg"
            >
              <div className="px-4 py-5 sm:p-6">
                <div className="flex items-center">
                  <MapPinIcon className="h-5 w-5 text-gray-400 mr-2" />
                  <h3 className="text-lg font-medium text-gray-900">{location.name}</h3>
                </div>
                <div className="mt-2">
                  <span
                    className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${getLocationStatus(
                      location
                    )}`}
                  >
                    {location.available} / {location.capacity} slots available
                  </span>
                </div>
                <div className="mt-4">
                  <h4 className="text-sm font-medium text-gray-900">Stored Samples</h4>
                  <ul className="mt-2 divide-y divide-gray-200">
                    {location.samples.map((sample) => (
                      <li key={sample.id} className="py-2">
                        <div className="flex items-center justify-between">
                          <div>
                            <p className="text-sm font-medium text-gray-900">{sample.name}</p>
                            <p className="text-sm text-gray-500">{sample.barcode}</p>
                          </div>
                          <select
                            className="text-indigo-600 text-sm font-medium bg-transparent border-0 cursor-pointer"
                            onChange={(e) => {
                              const newLocationId = Number(e.target.value);
                              if (newLocationId && newLocationId !== location.id) {
                                handleMoveSample(sample.barcode, newLocationId);
                              }
                            }}
                          >
                            <option value="">Move to...</option>
                            {locations?.filter(loc => loc.id !== location.id).map((loc) => (
                              <option key={loc.id} value={loc.id}>
                                {loc.name} ({loc.available} slots)
                              </option>
                            ))}
                          </select>
                        </div>
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
            </div>
          ))
        )}
      </div>


    </div>
  );
} 
