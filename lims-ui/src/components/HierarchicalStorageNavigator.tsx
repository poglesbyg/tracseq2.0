import { useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import api from '../utils/axios';
import SampleAssignmentModal from './SampleAssignmentModal';
import { 
  ChevronRightIcon,
  ArchiveBoxIcon,
  MapPinIcon,
  BeakerIcon,
  MagnifyingGlassIcon,
  CubeIcon,
  Squares2X2Icon,
  ListBulletIcon
} from '@heroicons/react/24/outline';

// Type definitions
interface StorageContainer {
  id: string;
  name: string;
  container_type: 'freezer' | 'rack' | 'box' | 'position';
  parent_container_id?: string;
  location_id?: string;
  grid_position?: Record<string, unknown>;
  capacity: number;
  occupied_count: number;
  temperature_zone?: string;
  barcode?: string;
  description?: string;
  status: string;
  created_at: string;
  updated_at: string;
}

interface SamplePosition {
  id: string;
  sample_id: string;
  container_id: string;
  position_identifier?: string;
  assigned_at: string;
  status: string;
  notes?: string;
}

interface GridPosition {
  container_id: string;
  position_identifier: string;
  row: number;
  column: number;
  is_occupied: boolean;
  sample_id?: string;
  sample_barcode?: string;
  sample_type?: string;
  status: string;
  temperature_zone?: string;
}

interface NavigationBreadcrumb {
  id: string;
  name: string;
  type: string;
}

export default function HierarchicalStorageNavigator() {
  const [currentContainerId, setCurrentContainerId] = useState<string | null>(null);
  const [navigationPath, setNavigationPath] = useState<NavigationBreadcrumb[]>([]);
  const [viewMode, setViewMode] = useState<'list' | 'grid'>('list');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedPosition, setSelectedPosition] = useState<{
    id: string;
    name: string;
    path: string;
    temperatureZone?: string;
  } | null>(null);
  const [showAssignModal, setShowAssignModal] = useState(false);
  
  const queryClient = useQueryClient();

  // Fetch top-level containers (freezers)
  const { data: topLevelContainers, isLoading: isLoadingTopLevel } = useQuery({
    queryKey: ['storage-containers', 'top-level'],
    queryFn: async () => {
      const response = await api.get('/api/storage/containers?container_type=freezer');
      return response.data;
    },
    enabled: !currentContainerId,
  });

  // Fetch current container details with children
  const { data: currentContainerData, isLoading: isLoadingContainer } = useQuery({
    queryKey: ['storage-container-details', currentContainerId],
    queryFn: async () => {
      if (!currentContainerId) return null;
      const response = await api.get(`/api/storage/containers/${currentContainerId}?include_samples=true`);
      return response.data;
    },
    enabled: !!currentContainerId,
  });

  // Fetch grid view for box-level containers
  const { data: gridData } = useQuery({
    queryKey: ['storage-grid', currentContainerId],
    queryFn: async () => {
      if (!currentContainerId) return null;
      const response = await api.get(`/api/storage/containers/${currentContainerId}/grid?include_empty=true`);
      return response.data;
    },
    enabled: !!currentContainerId && currentContainerData?.data?.container?.container_type === 'box',
  });

  const navigateToContainer = (container: StorageContainer) => {
    setCurrentContainerId(container.id);
    setNavigationPath(prev => [
      ...prev,
      { id: container.id, name: container.name, type: container.container_type }
    ]);
  };

  const navigateBack = (index: number) => {
    if (index === -1) {
      // Navigate to top level
      setCurrentContainerId(null);
      setNavigationPath([]);
    } else {
      // Navigate to specific breadcrumb
      const targetBreadcrumb = navigationPath[index];
      setCurrentContainerId(targetBreadcrumb.id);
      setNavigationPath(prev => prev.slice(0, index + 1));
    }
  };

  const getContainerIcon = (type: string) => {
    switch (type) {
      case 'freezer': return <CubeIcon className="h-5 w-5" />;
      case 'rack': return <ArchiveBoxIcon className="h-5 w-5" />;
      case 'box': return <Squares2X2Icon className="h-5 w-5" />;
      case 'position': return <MapPinIcon className="h-5 w-5" />;
      default: return <ArchiveBoxIcon className="h-5 w-5" />;
    }
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

  const getCapacityStatus = (utilization: number) => {
    if (utilization >= 95) return { color: 'bg-red-100 text-red-800', status: 'Critical' };
    if (utilization >= 80) return { color: 'bg-yellow-100 text-yellow-800', status: 'Warning' };
    return { color: 'bg-green-100 text-green-800', status: 'Normal' };
  };

  const filteredContainers = currentContainerData?.data?.children?.filter((container: StorageContainer) =>
    container.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    container.barcode?.toLowerCase().includes(searchQuery.toLowerCase())
  ) || [];

  const renderBreadcrumbs = () => (
    <nav className="flex mb-6" aria-label="Breadcrumb">
      <ol className="flex items-center space-x-2">
        <li>
          <button
            onClick={() => navigateBack(-1)}
            className="text-gray-500 hover:text-gray-700 font-medium"
          >
            Storage
          </button>
        </li>
        {navigationPath.map((breadcrumb, index) => (
          <li key={breadcrumb.id} className="flex items-center">
            <ChevronRightIcon className="h-4 w-4 text-gray-400 mx-2" />
            <button
              onClick={() => navigateBack(index)}
              className={`font-medium ${
                index === navigationPath.length - 1
                  ? 'text-gray-900'
                  : 'text-gray-500 hover:text-gray-700'
              }`}
            >
              {breadcrumb.name}
            </button>
          </li>
        ))}
      </ol>
    </nav>
  );

  const renderTopLevelContainers = () => (
    <div className="space-y-6">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h3 className="text-lg font-medium text-gray-900">Storage Units</h3>
          <p className="mt-2 text-sm text-gray-700">
            Select a storage unit to navigate its contents.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {topLevelContainers?.data?.data?.map((container: StorageContainer) => {
          const utilization = (container.occupied_count / container.capacity) * 100;
          const statusInfo = getCapacityStatus(utilization);
          
          return (
            <div
              key={container.id}
              className="bg-white overflow-hidden shadow rounded-lg cursor-pointer hover:shadow-md transition-shadow"
              onClick={() => navigateToContainer(container)}
            >
              <div className="p-5">
                <div className="flex items-center">
                  <div className="flex-shrink-0">
                    <div className="text-2xl">
                      {getTemperatureIcon(container.temperature_zone)}
                    </div>
                  </div>
                  <div className="ml-5 w-0 flex-1">
                    <dl>
                      <dt className="text-sm font-medium text-gray-500 truncate">
                        {container.name}
                      </dt>
                      <dd className="text-lg font-medium text-gray-900">
                        {container.temperature_zone}
                      </dd>
                    </dl>
                  </div>
                  <div className="flex-shrink-0">
                    <ChevronRightIcon className="h-5 w-5 text-gray-400" />
                  </div>
                </div>
                
                <div className="mt-4">
                  <div className="flex justify-between text-sm text-gray-500">
                    <span>Capacity</span>
                    <span>{container.occupied_count}/{container.capacity}</span>
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
                    <span className="text-xs text-gray-500">
                      {container.barcode}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );

  const renderContainerContents = () => {
    if (!currentContainerData?.data) return null;

    const { container, children, samples, capacity_info } = currentContainerData.data;

    // If this is a box, show grid view
    if (container.container_type === 'box' && viewMode === 'grid') {
      return renderGridView();
    }

    return (
      <div className="space-y-6">
        {/* Container Info Header */}
        <div className="bg-white shadow rounded-lg">
          <div className="px-6 py-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center">
                {getContainerIcon(container.container_type)}
                <div className="ml-3">
                  <h3 className="text-lg font-medium text-gray-900">{container.name}</h3>
                  <p className="text-sm text-gray-500">
                    {container.container_type.charAt(0).toUpperCase() + container.container_type.slice(1)} • {container.temperature_zone}
                  </p>
                </div>
              </div>
              
              {container.container_type === 'box' && (
                <div className="flex space-x-2">
                  <button
                    onClick={() => setViewMode('list')}
                    className={`p-2 rounded-md ${viewMode === 'list' ? 'bg-indigo-100 text-indigo-700' : 'text-gray-400 hover:text-gray-500'}`}
                  >
                    <ListBulletIcon className="h-5 w-5" />
                  </button>
                  <button
                    onClick={() => setViewMode('grid')}
                    className={`p-2 rounded-md ${viewMode === 'grid' ? 'bg-indigo-100 text-indigo-700' : 'text-gray-400 hover:text-gray-500'}`}
                  >
                    <Squares2X2Icon className="h-5 w-5" />
                  </button>
                </div>
              )}
            </div>

            <div className="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-4">
              <div className="bg-gray-50 p-3 rounded-lg">
                <dt className="text-sm font-medium text-gray-500">Total Capacity</dt>
                <dd className="text-lg font-semibold text-gray-900">{capacity_info.total_capacity}</dd>
              </div>
              <div className="bg-gray-50 p-3 rounded-lg">
                <dt className="text-sm font-medium text-gray-500">Occupied</dt>
                <dd className="text-lg font-semibold text-gray-900">{capacity_info.occupied_count}</dd>
              </div>
              <div className="bg-gray-50 p-3 rounded-lg">
                <dt className="text-sm font-medium text-gray-500">Available</dt>
                <dd className="text-lg font-semibold text-gray-900">{capacity_info.available_count}</dd>
              </div>
              <div className="bg-gray-50 p-3 rounded-lg">
                <dt className="text-sm font-medium text-gray-500">Utilization</dt>
                <dd className="text-lg font-semibold text-gray-900">
                  {capacity_info.utilization_percentage.toFixed(1)}%
                </dd>
              </div>
            </div>
          </div>
        </div>

        {/* Search */}
        <div className="relative">
          <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
            <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
          </div>
          <input
            type="text"
            className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            placeholder="Search containers..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>

        {/* Child Containers */}
        {children.length > 0 && (
          <div className="bg-white shadow rounded-lg">
            <div className="px-6 py-4 border-b border-gray-200">
              <h4 className="text-lg font-medium text-gray-900">
                {container.container_type === 'freezer' ? 'Racks' : 
                 container.container_type === 'rack' ? 'Boxes' : 
                 container.container_type === 'box' ? 'Positions' : 'Contents'}
              </h4>
            </div>
            <ul className="divide-y divide-gray-200">
              {filteredContainers.map((child: StorageContainer) => {
                const utilization = (child.occupied_count / child.capacity) * 100;
                const statusInfo = getCapacityStatus(utilization);
                
                return (
                  <li key={child.id}>
                    <div
                      className="px-6 py-4 flex items-center justify-between hover:bg-gray-50 cursor-pointer"
                      onClick={() => navigateToContainer(child)}
                    >
                      <div className="flex items-center">
                        {getContainerIcon(child.container_type)}
                        <div className="ml-4">
                          <div className="text-sm font-medium text-gray-900">
                            {child.name}
                          </div>
                          <div className="text-sm text-gray-500">
                            {child.barcode} • {child.occupied_count}/{child.capacity} occupied
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center space-x-4">
                        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${statusInfo.color}`}>
                          {utilization.toFixed(1)}%
                        </span>
                        <ChevronRightIcon className="h-5 w-5 text-gray-400" />
                      </div>
                    </div>
                  </li>
                );
              })}
            </ul>
          </div>
        )}

        {/* Direct Samples */}
        {samples.length > 0 && (
          <div className="bg-white shadow rounded-lg">
            <div className="px-6 py-4 border-b border-gray-200">
              <h4 className="text-lg font-medium text-gray-900">Samples</h4>
            </div>
            <ul className="divide-y divide-gray-200">
              {samples.map((sample: SamplePosition) => (
                <li key={sample.id}>
                  <div className="px-6 py-4 flex items-center justify-between">
                    <div className="flex items-center">
                      <BeakerIcon className="h-5 w-5 text-gray-400" />
                      <div className="ml-4">
                        <div className="text-sm font-medium text-gray-900">
                          Sample {sample.sample_id}
                        </div>
                        <div className="text-sm text-gray-500">
                          Position: {sample.position_identifier || 'N/A'} • Assigned: {new Date(sample.assigned_at).toLocaleDateString()}
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        sample.status === 'occupied' ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                      }`}>
                        {sample.status}
                      </span>
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    );
  };

  const renderGridView = () => {
    if (!gridData?.data) return null;

    const { grid_dimensions, positions } = gridData.data;

    return (
      <div className="space-y-6">
        <div className="bg-white shadow rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h4 className="text-lg font-medium text-gray-900">
              Grid View ({grid_dimensions.rows} × {grid_dimensions.columns})
            </h4>
            <div className="text-sm text-gray-500">
              {positions.filter((p: GridPosition) => p.is_occupied).length} / {grid_dimensions.total_positions} occupied
            </div>
          </div>

          <div className="grid gap-1" style={{
            gridTemplateColumns: `repeat(${grid_dimensions.columns}, minmax(0, 1fr))`
          }}>
            {Array.from({ length: grid_dimensions.rows }, (_, row) =>
              Array.from({ length: grid_dimensions.columns }, (_, col) => {
                const position = positions.find((p: GridPosition) => p.row === row + 1 && p.column === col + 1);
                
                return (
                  <div
                    key={`${row}-${col}`}
                    className={`
                      aspect-square border-2 rounded-md flex items-center justify-center text-xs font-medium cursor-pointer
                      ${position?.is_occupied 
                        ? 'bg-red-100 border-red-300 text-red-800 hover:bg-red-200' 
                        : 'bg-green-100 border-green-300 text-green-800 hover:bg-green-200'
                      }
                    `}
                    title={position ? `${position.position_identifier} - ${position.is_occupied ? 'Occupied' : 'Available'}` : 'Empty'}
                    onClick={() => {
                      if (position && !position.is_occupied) {
                        setSelectedPosition({
                          id: position.container_id,
                          name: position.position_identifier,
                          path: `${currentContainerData?.data?.container?.name || 'Container'}`,
                          temperatureZone: position.temperature_zone
                        });
                        setShowAssignModal(true);
                      }
                    }}
                  >
                    {position?.position_identifier || `${String.fromCharCode(65 + row)}${col + 1}`}
                  </div>
                );
              })
            )}
          </div>

          <div className="mt-4 flex items-center space-x-6 text-sm">
            <div className="flex items-center">
              <div className="w-4 h-4 bg-green-100 border-2 border-green-300 rounded mr-2"></div>
              <span>Available</span>
            </div>
            <div className="flex items-center">
              <div className="w-4 h-4 bg-red-100 border-2 border-red-300 rounded mr-2"></div>
              <span>Occupied</span>
            </div>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="space-y-6">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h2 className="text-xl font-semibold text-gray-900">Hierarchical Storage</h2>
          <p className="mt-2 text-sm text-gray-700">
            Navigate through storage units, racks, boxes, and positions to manage sample locations.
          </p>
        </div>
      </div>

      {renderBreadcrumbs()}

      {isLoadingTopLevel || isLoadingContainer ? (
        <div className="flex justify-center py-8">
          <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
        </div>
      ) : currentContainerId ? (
        renderContainerContents()
      ) : (
        renderTopLevelContainers()
      )}

      {/* Sample Assignment Modal */}
      {showAssignModal && selectedPosition && (
        <SampleAssignmentModal
          isOpen={showAssignModal}
          onClose={() => {
            setShowAssignModal(false);
            setSelectedPosition(null);
          }}
          positionId={selectedPosition.id}
          positionName={selectedPosition.name}
          containerPath={selectedPosition.path}
          temperatureZone={selectedPosition.temperatureZone}
          onSuccess={() => {
            queryClient.invalidateQueries({ queryKey: ['storage-container-details'] });
            queryClient.invalidateQueries({ queryKey: ['storage-grid'] });
          }}
        />
      )}
    </div>
  );
} 