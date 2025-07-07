import { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '../utils/axios';
import { 
  MapPinIcon, 
  QrCodeIcon, 
  ArrowPathIcon,
  MagnifyingGlassIcon,
  BeakerIcon,
  ClockIcon,
  ChartBarIcon,
  ExclamationTriangleIcon,
  CheckCircleIcon,
  XMarkIcon,
  PlusIcon,
  EyeIcon,
  ArrowRightIcon,
  DocumentArrowDownIcon
} from '@heroicons/react/24/outline';

interface StorageLocation {
  id: string;
  name: string;
  description?: string;
  location_type: string;
  temperature_zone: string;
  max_capacity: number;
  current_capacity: number;
  coordinates?: any;
  status: string;
  metadata: any;
  created_at: string;
  updated_at: string;
}

interface StoredSample {
  id: string;
  barcode: string;
  sample_type: string;
  storage_location_id?: string;
  position?: any;
  temperature_requirements?: string;
  status: string;
  metadata: any;
  chain_of_custody: any[];
  stored_at?: string;
  created_at: string;
  updated_at: string;
}

interface StorageUtilization {
  location_id: string;
  max_capacity: number;
  current_capacity: number;
  utilization_percentage: number;
  available_capacity: number;
  status: string;
}

interface MobileSample {
  id: string;
  barcode: string;
  sample_type: string;
  location: string;
  temperature: number;
  status: string;
  priority: string;
  submitter: string;
  submitted_date: string;
  thumbnail_image?: string;
  qr_code_data: string;
}

export default function StorageManagement() {
  const [activeTab, setActiveTab] = useState<'locations' | 'samples' | 'analytics'>('locations');
  const [scanningBarcode, setScanningBarcode] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedLocation, setSelectedLocation] = useState<StorageLocation | null>(null);
  const [selectedSample, setSelectedSample] = useState<StoredSample | null>(null);
  const [showCreateLocation, setShowCreateLocation] = useState(false);
  const [showMoveModal, setShowMoveModal] = useState(false);
  const [newLocationForm, setNewLocationForm] = useState({
    name: '',
    description: '',
    location_type: 'freezer',
    temperature_zone: '-80C',
    max_capacity: 100
  });
  
  const queryClient = useQueryClient();

  // Fetch storage locations
  const { data: locationsResponse, isLoading: isLoadingLocations } = useQuery({
    queryKey: ['storage-locations'],
    queryFn: async () => {
      const response = await api.get('/api/storage/locations');
      return response.data;
    },
  });

  // Fetch stored samples (with fallback)
  const { data: samplesResponse, isLoading: isLoadingSamples } = useQuery({
    queryKey: ['storage-samples'],
    queryFn: async () => {
      try {
        const response = await api.get('/api/storage/samples');
        return response.data;
      } catch {
        // Fallback mock data
        return {
          data: [
            {
              id: "SMPL-001",
              barcode: "BC001",
              sample_type: "DNA",
              location_id: "freezer-a1",
              location_name: "Freezer A1 (-80°C)",
              status: "stored",
              stored_at: "2024-01-15T10:30:00Z",
              volume: 100.0,
              concentration: 50.0
            },
            {
              id: "SMPL-002", 
              barcode: "BC002",
              sample_type: "RNA",
              location_id: "fridge-b2",
              location_name: "Refrigerator B2 (4°C)",
              status: "stored",
              stored_at: "2024-01-16T14:20:00Z",
              volume: 75.0,
              concentration: 30.0
            }
          ]
        };
      }
    },
  });

  // Fetch storage analytics (with fallback)
  const { data: analyticsData } = useQuery({
    queryKey: ['storage-analytics'],
    queryFn: async () => {
      try {
        const response = await api.get('/api/storage/analytics/utilization');
        return response.data;
      } catch {
        // Fallback mock data
        return {
          data: {
            total_capacity: 5000,
            total_used: 3200,
            utilization_percentage: 64.0,
            zones: [
              {"name": "-80C", "capacity": 1000, "used": 800, "utilization": 80.0},
              {"name": "-20C", "capacity": 800, "used": 600, "utilization": 75.0},
              {"name": "4C", "capacity": 1200, "used": 900, "utilization": 75.0},
              {"name": "RT", "capacity": 2000, "used": 900, "utilization": 45.0}
            ]
          }
        };
      }
    },
  });

  // Fetch mobile samples for barcode scanning (with fallback)
  const { data: mobileSamplesResponse } = useQuery({
    queryKey: ['mobile-samples'],
    queryFn: async () => {
      try {
        const response = await api.get('/api/storage/mobile/samples');
        return response.data;
      } catch {
        // Fallback mock data
        return {
          data: [
            {
              id: "SMPL-001",
              barcode: "BC001",
              type: "DNA",
              location: "A1",
              status: "stored",
              temp: "-80°C"
            },
            {
              id: "SMPL-002",
              barcode: "BC002", 
              type: "RNA",
              location: "B2",
              status: "stored",
              temp: "4°C"
            }
          ]
        };
      }
    },
  });

  const locations = locationsResponse?.data || [];
  const samples = samplesResponse?.data || [];
  const mobilesamples = mobileSamplesResponse?.data?.samples || [];

  // Create location mutation
  const createLocationMutation = useMutation({
    mutationFn: async (locationData: any) => {
      const response = await api.post('/api/storage/locations', locationData);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['storage-locations'] });
      setShowCreateLocation(false);
      setNewLocationForm({
        name: '',
        description: '',
        location_type: 'freezer',
        temperature_zone: '-80C',
        max_capacity: 100
      });
    },
  });

  // Move sample mutation
  const moveSampleMutation = useMutation({
    mutationFn: async ({ sampleId, newLocationId, reason }: { 
      sampleId: string; 
      newLocationId: string; 
      reason: string;
    }) => {
      const response = await api.post(`/api/storage/samples/${sampleId}/move`, {
        new_location_id: newLocationId,
        reason: reason
      });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['storage-samples'] });
      queryClient.invalidateQueries({ queryKey: ['storage-locations'] });
      setShowMoveModal(false);
      setSelectedSample(null);
    },
  });

  // Retrieve sample mutation
  const retrieveSampleMutation = useMutation({
    mutationFn: async (sampleId: string) => {
      const response = await api.post(`/api/storage/samples/${sampleId}/retrieve`);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['storage-samples'] });
      queryClient.invalidateQueries({ queryKey: ['storage-locations'] });
    },
  });

  // Search samples mutation
  const searchSamplesMutation = useMutation({
    mutationFn: async (searchParams: any) => {
      const response = await api.get('/api/storage/samples/search', { params: searchParams });
      return response.data;
    },
  });

  const handleBarcodeScan = (barcode: string) => {
    searchSamplesMutation.mutate({ barcode });
  };

  const handleCreateLocation = () => {
    createLocationMutation.mutate(newLocationForm);
  };

  const handleMoveSample = (newLocationId: string, reason: string) => {
    if (selectedSample) {
      moveSampleMutation.mutate({
        sampleId: selectedSample.id,
        newLocationId,
        reason
      });
    }
  };

  const getLocationStatus = (location: StorageLocation) => {
    const utilization = (location.current_capacity / location.max_capacity) * 100;
    if (utilization >= 95) return { color: 'bg-red-100 text-red-800', status: 'Critical' };
    if (utilization >= 80) return { color: 'bg-yellow-100 text-yellow-800', status: 'Warning' };
    return { color: 'bg-green-100 text-green-800', status: 'Normal' };
  };

  const getTemperatureIcon = (temperature: string | undefined) => {
    if (!temperature) return '🌡️';
    const tempStr = temperature.toString().toLowerCase();
    if (tempStr.includes('-80') || tempStr.includes('-20')) {
      return '🧊'; // Freezer
    } else if (tempStr.includes('4') || tempStr.includes('rt')) {
      return '🌡️'; // Refrigerator/Room temp
    }
    return '🌡️';
  };

  const filteredSamples = samples.filter((sample: StoredSample) =>
    sample.barcode.toLowerCase().includes(searchQuery.toLowerCase()) ||
    sample.sample_type.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const filteredLocations = locations.filter((location: StorageLocation) =>
    location.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    location.temperature_zone.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="space-y-6">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h2 className="text-xl font-semibold text-gray-900">Storage Management</h2>
          <p className="mt-2 text-sm text-gray-700">
            Manage storage locations, track samples, and monitor storage conditions.
          </p>
        </div>
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none space-x-2">
          <button
            type="button"
            onClick={() => setScanningBarcode(!scanningBarcode)}
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
          >
            <QrCodeIcon className="h-5 w-5 mr-2" />
            {scanningBarcode ? 'Stop Scanning' : 'Scan Barcode'}
          </button>
          {activeTab === 'locations' && (
            <button
              type="button"
              onClick={() => setShowCreateLocation(true)}
              className="inline-flex items-center justify-center rounded-md border border-transparent bg-green-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2"
            >
              <PlusIcon className="h-5 w-5 mr-2" />
              Add Location
            </button>
          )}
        </div>
      </div>

      {/* Barcode Scanner */}
      {scanningBarcode && (
        <div className="bg-white shadow sm:rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg font-medium leading-6 text-gray-900">Scan Sample Barcode</h3>
            <div className="mt-2 max-w-xl text-sm text-gray-500">
              <p>Scan or enter a sample barcode to find its location and details.</p>
            </div>
            <div className="mt-5 flex space-x-3">
              <input
                type="text"
                className="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                placeholder="Enter or scan barcode"
                onKeyPress={(e) => {
                  if (e.key === 'Enter') {
                    handleBarcodeScan(e.currentTarget.value);
                    e.currentTarget.value = '';
                  }
                }}
              />
              <button
                type="button"
                onClick={() => setScanningBarcode(false)}
                className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <XMarkIcon className="h-4 w-4" />
              </button>
            </div>
            {searchSamplesMutation.data && (
              <div className="mt-4 p-4 bg-gray-50 rounded-md">
                <h4 className="font-medium text-gray-900">Search Results</h4>
                <div className="mt-2 space-y-2">
                  {searchSamplesMutation.data.data?.map((sample: StoredSample) => (
                    <div key={sample.id} className="flex justify-between items-center py-2 border-b">
                      <div>
                        <p className="font-medium">{sample.barcode}</p>
                        <p className="text-sm text-gray-500">{sample.sample_type}</p>
                      </div>
                      <div className="text-right">
                        <p className="text-sm">{sample.status}</p>
                        <p className="text-xs text-gray-500">
                          {sample.storage_location_id ? 'In Storage' : 'Not Stored'}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Tabs */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {[
            { id: 'locations', name: 'Storage Locations', icon: MapPinIcon },
            { id: 'samples', name: 'Stored Samples', icon: BeakerIcon },
            { id: 'analytics', name: 'Analytics', icon: ChartBarIcon },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`${
                activeTab === tab.id
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              } whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm flex items-center`}
            >
              <tab.icon className="h-5 w-5 mr-2" />
              {tab.name}
            </button>
          ))}
        </nav>
      </div>

      {/* Search Bar */}
      <div className="relative">
        <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
          <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
        </div>
        <input
          type="text"
          className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          placeholder={`Search ${activeTab}...`}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>

      {/* Content based on active tab */}
      {activeTab === 'locations' && (
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {isLoadingLocations ? (
            <div className="col-span-full text-center py-8">
              <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
              <p className="mt-2 text-sm text-gray-500">Loading locations...</p>
            </div>
          ) : (
            filteredLocations.map((location: StorageLocation) => {
              const statusInfo = getLocationStatus(location);
              const utilization = (location.current_capacity / location.max_capacity) * 100;
              
              return (
                <div key={location.id} className="bg-white overflow-hidden shadow rounded-lg">
                  <div className="p-5">
                    <div className="flex items-center">
                      <div className="flex-shrink-0">
                        <div className="text-2xl">
                          {getTemperatureIcon(location.temperature_zone)}
                        </div>
                      </div>
                      <div className="ml-5 w-0 flex-1">
                        <dl>
                          <dt className="text-sm font-medium text-gray-500 truncate">
                            {location.name}
                          </dt>
                          <dd className="text-lg font-medium text-gray-900">
                            {location.temperature_zone}
                          </dd>
                        </dl>
                      </div>
                    </div>
                    
                    <div className="mt-4">
                      <div className="flex justify-between text-sm text-gray-500">
                        <span>Capacity</span>
                        <span>{location.current_capacity}/{location.max_capacity}</span>
                      </div>
                      <div className="mt-1 w-full bg-gray-200 rounded-full h-2">
                        <div
                          className={`h-2 rounded-full ${
                            utilization >= 95 ? 'bg-red-500' : 
                            utilization >= 80 ? 'bg-yellow-500' : 'bg-green-500'
                          }`}
                          style={{ width: `${utilization}%` }}
                        ></div>
                      </div>
                      <div className="mt-2 flex justify-between items-center">
                        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${statusInfo.color}`}>
                          {statusInfo.status}
                        </span>
                        <button
                          onClick={() => setSelectedLocation(location)}
                          className="text-indigo-600 hover:text-indigo-900 text-sm font-medium"
                        >
                          View Details
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              );
            })
          )}
        </div>
      )}

      {activeTab === 'samples' && (
        <div className="bg-white shadow overflow-hidden sm:rounded-md">
          <ul className="divide-y divide-gray-200">
            {isLoadingSamples ? (
              <li className="px-6 py-4 text-center">
                <div className="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-600"></div>
                <p className="mt-2 text-sm text-gray-500">Loading samples...</p>
              </li>
            ) : filteredSamples.length === 0 ? (
              <li className="px-6 py-4 text-center text-gray-500">
                No samples found matching your search.
              </li>
            ) : (
              filteredSamples.map((sample: StoredSample) => (
                <li key={sample.id}>
                  <div className="px-6 py-4 flex items-center justify-between">
                    <div className="flex items-center">
                      <div className="flex-shrink-0">
                        <BeakerIcon className="h-8 w-8 text-gray-400" />
                      </div>
                      <div className="ml-4">
                        <div className="text-sm font-medium text-gray-900">
                          {sample.barcode}
                        </div>
                        <div className="text-sm text-gray-500">
                          {sample.sample_type} • {sample.status}
                        </div>
                        {sample.stored_at && (
                          <div className="text-xs text-gray-400">
                            Stored: {new Date(sample.stored_at).toLocaleDateString()}
                          </div>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      <button
                        onClick={() => setSelectedSample(sample)}
                        className="text-indigo-600 hover:text-indigo-900 text-sm font-medium"
                      >
                        <EyeIcon className="h-4 w-4" />
                      </button>
                      <button
                        onClick={() => {
                          setSelectedSample(sample);
                          setShowMoveModal(true);
                        }}
                        className="text-blue-600 hover:text-blue-900 text-sm font-medium"
                      >
                        <ArrowRightIcon className="h-4 w-4" />
                      </button>
                      <button
                        onClick={() => retrieveSampleMutation.mutate(sample.id)}
                        className="text-green-600 hover:text-green-900 text-sm font-medium"
                      >
                        <DocumentArrowDownIcon className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                </li>
              ))
            )}
          </ul>
        </div>
      )}

      {activeTab === 'analytics' && (
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {/* Storage Utilization Chart */}
          <div className="bg-white overflow-hidden shadow rounded-lg">
            <div className="p-5">
              <h3 className="text-lg leading-6 font-medium text-gray-900">
                Storage Utilization
              </h3>
              <div className="mt-4 space-y-4">
                {locations.map((location: StorageLocation) => {
                  const utilization = (location.current_capacity / location.max_capacity) * 100;
                  return (
                    <div key={location.id}>
                      <div className="flex justify-between text-sm">
                        <span className="font-medium">{location.name}</span>
                        <span>{utilization.toFixed(1)}%</span>
                      </div>
                      <div className="mt-1 w-full bg-gray-200 rounded-full h-2">
                        <div
                          className={`h-2 rounded-full ${
                            utilization >= 95 ? 'bg-red-500' : 
                            utilization >= 80 ? 'bg-yellow-500' : 'bg-green-500'
                          }`}
                          style={{ width: `${utilization}%` }}
                        ></div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>

          {/* Temperature Monitoring */}
          <div className="bg-white overflow-hidden shadow rounded-lg">
            <div className="p-5">
              <h3 className="text-lg leading-6 font-medium text-gray-900">
                Temperature Zones
              </h3>
              <div className="mt-4 space-y-3">
                                                  {Array.from(new Set(locations.map((l: StorageLocation) => l.temperature_zone))).map((zone) => {
                   const zoneString = zone as string;
                   const zoneLocations = locations.filter((l: StorageLocation) => l.temperature_zone === zoneString);
                   const totalCapacity = zoneLocations.reduce((sum: number, l: StorageLocation) => sum + l.max_capacity, 0);
                   const totalUsed = zoneLocations.reduce((sum: number, l: StorageLocation) => sum + l.current_capacity, 0);
                   
                   return (
                     <div key={zoneString} className="flex items-center justify-between py-2 border-b">
                       <div className="flex items-center">
                         <span className="text-2xl mr-3">{getTemperatureIcon(zoneString)}</span>
                         <div>
                           <p className="font-medium">{zoneString}</p>
                           <p className="text-sm text-gray-500">{zoneLocations.length} locations</p>
                         </div>
                       </div>
                       <div className="text-right">
                         <p className="font-medium">{totalUsed}/{totalCapacity}</p>
                         <p className="text-sm text-gray-500">
                           {((totalUsed / totalCapacity) * 100).toFixed(1)}% used
                         </p>
                       </div>
                     </div>
                   );
                 })}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Create Location Modal */}
      {showCreateLocation && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-md w-full">
            <div className="p-6">
              <h3 className="text-lg font-medium text-gray-900 mb-4">Create Storage Location</h3>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">Name</label>
                  <input
                    type="text"
                    value={newLocationForm.name}
                    onChange={(e) => setNewLocationForm({ ...newLocationForm, name: e.target.value })}
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Description</label>
                  <textarea
                    value={newLocationForm.description}
                    onChange={(e) => setNewLocationForm({ ...newLocationForm, description: e.target.value })}
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                    rows={2}
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Type</label>
                  <select
                    value={newLocationForm.location_type}
                    onChange={(e) => setNewLocationForm({ ...newLocationForm, location_type: e.target.value })}
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                  >
                    <option value="freezer">Freezer</option>
                    <option value="refrigerator">Refrigerator</option>
                    <option value="rack">Rack</option>
                    <option value="cabinet">Cabinet</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Temperature Zone</label>
                  <select
                    value={newLocationForm.temperature_zone}
                    onChange={(e) => setNewLocationForm({ ...newLocationForm, temperature_zone: e.target.value })}
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                  >
                    <option value="-80C">-80°C</option>
                    <option value="-20C">-20°C</option>
                    <option value="4C">4°C</option>
                    <option value="RT">Room Temperature</option>
                    <option value="37C">37°C</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Max Capacity</label>
                  <input
                    type="number"
                    value={newLocationForm.max_capacity}
                    onChange={(e) => setNewLocationForm({ ...newLocationForm, max_capacity: parseInt(e.target.value) })}
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                  />
                </div>
              </div>
              <div className="mt-6 flex justify-end space-x-3">
                <button
                  onClick={() => setShowCreateLocation(false)}
                  className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
                >
                  Cancel
                </button>
                <button
                  onClick={handleCreateLocation}
                  disabled={createLocationMutation.isPending}
                  className="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 disabled:opacity-50"
                >
                  {createLocationMutation.isPending ? 'Creating...' : 'Create'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Move Sample Modal */}
      {showMoveModal && selectedSample && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-md w-full">
            <div className="p-6">
              <h3 className="text-lg font-medium text-gray-900 mb-4">
                Move Sample: {selectedSample.barcode}
              </h3>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">New Location</label>
                  <select
                    id="newLocation"
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                  >
                    <option value="">Select a location</option>
                    {locations
                      .filter((loc: StorageLocation) => loc.id !== selectedSample.storage_location_id)
                      .map((location: StorageLocation) => (
                        <option key={location.id} value={location.id}>
                          {location.name} ({location.max_capacity - location.current_capacity} available)
                        </option>
                      ))}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">Reason</label>
                  <textarea
                    id="moveReason"
                    className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                    rows={2}
                    placeholder="Reason for moving sample..."
                  />
                </div>
              </div>
              <div className="mt-6 flex justify-end space-x-3">
                <button
                  onClick={() => setShowMoveModal(false)}
                  className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
                >
                  Cancel
                </button>
                <button
                  onClick={() => {
                    const newLocationSelect = document.getElementById('newLocation') as HTMLSelectElement;
                    const reasonTextarea = document.getElementById('moveReason') as HTMLTextAreaElement;
                    if (newLocationSelect.value && reasonTextarea.value) {
                      handleMoveSample(newLocationSelect.value, reasonTextarea.value);
                    }
                  }}
                  disabled={moveSampleMutation.isPending}
                  className="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 disabled:opacity-50"
                >
                  {moveSampleMutation.isPending ? 'Moving...' : 'Move Sample'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Sample Details Modal */}
      {selectedSample && !showMoveModal && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] overflow-y-auto">
            <div className="p-6">
              <div className="flex justify-between items-center mb-4">
                <h3 className="text-lg font-medium text-gray-900">Sample Details</h3>
                <button
                  onClick={() => setSelectedSample(null)}
                  className="text-gray-400 hover:text-gray-500"
                >
                  <XMarkIcon className="h-6 w-6" />
                </button>
              </div>
              
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-500">Barcode</label>
                    <p className="mt-1 text-sm text-gray-900 font-mono">{selectedSample.barcode}</p>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-500">Sample Type</label>
                    <p className="mt-1 text-sm text-gray-900">{selectedSample.sample_type}</p>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-500">Status</label>
                    <p className="mt-1 text-sm text-gray-900">{selectedSample.status}</p>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-500">Temperature Requirements</label>
                    <p className="mt-1 text-sm text-gray-900">{selectedSample.temperature_requirements || 'Not specified'}</p>
                  </div>
                </div>
                
                {selectedSample.chain_of_custody && selectedSample.chain_of_custody.length > 0 && (
                  <div>
                    <label className="block text-sm font-medium text-gray-500 mb-2">Chain of Custody</label>
                    <div className="space-y-2">
                      {selectedSample.chain_of_custody.map((entry: any, index: number) => (
                        <div key={index} className="flex items-center text-sm">
                          <ClockIcon className="h-4 w-4 text-gray-400 mr-2" />
                          <span className="text-gray-900">{entry.action}</span>
                          <span className="text-gray-500 ml-2">
                            {new Date(entry.timestamp).toLocaleString()}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Location Details Modal */}
      {selectedLocation && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] overflow-y-auto">
            <div className="p-6">
              <div className="flex justify-between items-center mb-4">
                <h3 className="text-lg font-medium text-gray-900">Location Details</h3>
                <button
                  onClick={() => setSelectedLocation(null)}
                  className="text-gray-400 hover:text-gray-500"
                >
                  <XMarkIcon className="h-6 w-6" />
                </button>
              </div>
              
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-500">Name</label>
                    <p className="mt-1 text-sm text-gray-900">{selectedLocation.name}</p>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-500">Type</label>
                    <p className="mt-1 text-sm text-gray-900">{selectedLocation.location_type}</p>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-500">Temperature Zone</label>
                    <p className="mt-1 text-sm text-gray-900">{selectedLocation.temperature_zone}</p>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-500">Capacity</label>
                    <p className="mt-1 text-sm text-gray-900">
                      {selectedLocation.current_capacity}/{selectedLocation.max_capacity}
                    </p>
                  </div>
                </div>
                
                {selectedLocation.description && (
                  <div>
                    <label className="block text-sm font-medium text-gray-500">Description</label>
                    <p className="mt-1 text-sm text-gray-900">{selectedLocation.description}</p>
                  </div>
                )}

                <div className="mt-4">
                  <label className="block text-sm font-medium text-gray-500 mb-2">Utilization</label>
                  <div className="w-full bg-gray-200 rounded-full h-3">
                    <div
                      className={`h-3 rounded-full ${
                        (selectedLocation.current_capacity / selectedLocation.max_capacity) * 100 >= 95 ? 'bg-red-500' : 
                        (selectedLocation.current_capacity / selectedLocation.max_capacity) * 100 >= 80 ? 'bg-yellow-500' : 'bg-green-500'
                      }`}
                      style={{ width: `${(selectedLocation.current_capacity / selectedLocation.max_capacity) * 100}%` }}
                    ></div>
                  </div>
                  <p className="mt-1 text-sm text-gray-500">
                    {((selectedLocation.current_capacity / selectedLocation.max_capacity) * 100).toFixed(1)}% utilized
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
} 
