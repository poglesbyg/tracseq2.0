import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '../utils/axios';
import {
  BeakerIcon,
  CpuChipIcon,
  SparklesIcon,
  CheckIcon,
  XMarkIcon,
  InformationCircleIcon,
  ArrowPathIcon,
} from '@heroicons/react/24/outline';

interface FlowCellType {
  id: string;
  name: string;
  manufacturer: string;
  model: string;
  lane_count: number;
  reads_per_lane: number;
}

interface LibraryPrep {
  id: string;
  batch_id: string;
  sample_names: string[];
  concentration: number;
  fragment_size: number;
  index_type: string;
  project_name?: string;
}

interface LaneAssignment {
  lane_number: number;
  library_prep_ids: string[];
  target_reads: number;
  loading_concentration: number;
  color?: string;
}

interface FlowCellDesign {
  id: string;
  name: string;
  flow_cell_type_id: string;
  project_id: string;
  status: 'draft' | 'approved' | 'in_sequencing' | 'completed';
  lane_assignments: LaneAssignment[];
  ai_optimization_score?: number;
  ai_suggestions?: string[];
}

interface DragItem {
  library_prep: LibraryPrep;
}

const LANE_COLORS = [
  'bg-blue-100 border-blue-300',
  'bg-green-100 border-green-300',
  'bg-yellow-100 border-yellow-300',
  'bg-purple-100 border-purple-300',
  'bg-pink-100 border-pink-300',
  'bg-indigo-100 border-indigo-300',
  'bg-red-100 border-red-300',
  'bg-orange-100 border-orange-300',
];

export default function FlowCellDesign() {
  const [selectedFlowCellType, setSelectedFlowCellType] = useState<FlowCellType | null>(null);
  const [laneAssignments, setLaneAssignments] = useState<LaneAssignment[]>([]);
  const [draggedItem, setDraggedItem] = useState<DragItem | null>(null);
  const [showAISuggestions, setShowAISuggestions] = useState(false);
  const queryClient = useQueryClient();

  // Fetch flow cell types
  const { data: flowCellTypes } = useQuery<FlowCellType[]>({
    queryKey: ['flow-cell-types'],
    queryFn: async () => {
      const response = await api.get('/api/flow-cells/types');
      return response.data;
    },
  });

  // Fetch available libraries
  const { data: availableLibraries } = useQuery<LibraryPrep[]>({
    queryKey: ['available-libraries'],
    queryFn: async () => {
      const response = await api.get('/api/library-prep/preparations', { 
        params: { status: 'ready_for_sequencing' } 
      });
      return response.data;
    },
  });

  // Create flow cell design mutation
  const createDesignMutation = useMutation({
    mutationFn: async (design: { name: string; flow_cell_type_id: string; lane_assignments: LaneAssignment[] }) => {
      const response = await api.post('/api/flow-cells/designs', design);
      return response.data;
    },
    onSuccess: () => {
      // Handle success
      alert('Flow cell design created successfully!');
    },
  });

  // AI optimization mutation
  const optimizeMutation = useMutation({
    mutationFn: async () => {
      const response = await api.post('/api/flow-cells/optimize', {
        flow_cell_type_id: selectedFlowCellType?.id,
        libraries: Object.values(laneAssignments).flat(),
      });
      return response.data;
    },
    onSuccess: (data) => {
      // Apply optimized assignments
      setLaneAssignments(data.lane_assignments || {});
    },
  });

  const handleFlowCellTypeSelect = (type: FlowCellType) => {
    setSelectedFlowCellType(type);
    // Initialize empty lanes
    const emptyLanes: LaneAssignment[] = Array.from({ length: type.lane_count }, (_, i) => ({
      lane_number: i + 1,
      library_prep_ids: [],
      target_reads: Math.floor(type.reads_per_lane / type.lane_count),
      loading_concentration: 1.5,
      color: LANE_COLORS[i % LANE_COLORS.length],
    }));
    setLaneAssignments(emptyLanes);
  };

  const handleDragStart = (e: React.DragEvent, library: LibraryPrep) => {
    setDraggedItem({ library_prep: library });
    e.dataTransfer.effectAllowed = 'copy';
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  };

  const handleDrop = (e: React.DragEvent, laneNumber: number) => {
    e.preventDefault();
    if (!draggedItem) return;

    const updatedLanes = laneAssignments.map((lane) => {
      if (lane.lane_number === laneNumber) {
        return {
          ...lane,
          library_prep_ids: [...lane.library_prep_ids, draggedItem.library_prep.id],
        };
      }
      return lane;
    });

    setLaneAssignments(updatedLanes);
    setDraggedItem(null);
  };

  const removeFromLane = (laneNumber: number, libraryId: string) => {
    const updatedLanes = laneAssignments.map((lane) => {
      if (lane.lane_number === laneNumber) {
        return {
          ...lane,
          library_prep_ids: lane.library_prep_ids.filter((id) => id !== libraryId),
        };
      }
      return lane;
    });
    setLaneAssignments(updatedLanes);
  };

  const getLibraryById = (id: string): LibraryPrep | undefined => {
    return availableLibraries?.find((lib) => lib.id === id);
  };

  const calculateLaneBalance = (): number => {
    if (!laneAssignments.length) return 0;
    const laneCounts = laneAssignments.map((lane) => lane.library_prep_ids.length);
    const avg = laneCounts.reduce((a, b) => a + b, 0) / laneCounts.length;
    const variance = laneCounts.reduce((sum, count) => sum + Math.pow(count - avg, 2), 0) / laneCounts.length;
    return Math.max(0, 100 - variance * 10);
  };

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      {/* Header */}
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-xl font-semibold text-gray-900">Flow Cell Design</h1>
          <p className="mt-2 text-sm text-gray-700">
            Design flow cells with intelligent lane assignment and AI optimization
          </p>
        </div>
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none space-x-2">
          <button
            type="button"
            onClick={() => selectedFlowCellType && availableLibraries && 
              optimizeMutation.mutate()
            }
            disabled={!selectedFlowCellType || !availableLibraries?.length}
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-purple-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <SparklesIcon className="-ml-1 mr-2 h-5 w-5" />
            AI Optimize
          </button>
          <button
            type="button"
            onClick={() => createDesignMutation.mutate({
              name: `Flow Cell ${new Date().toISOString()}`,
              flow_cell_type_id: selectedFlowCellType!.id,
              lane_assignments: laneAssignments,
            })}
            disabled={!selectedFlowCellType || laneAssignments.every(lane => lane.library_prep_ids.length === 0)}
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <CheckIcon className="-ml-1 mr-2 h-5 w-5" />
            Save Design
          </button>
        </div>
      </div>

      {/* Flow Cell Type Selection */}
      {!selectedFlowCellType ? (
        <div className="mt-8">
          <h2 className="text-lg font-medium text-gray-900 mb-4">Select Flow Cell Type</h2>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {flowCellTypes?.map((type) => (
              <button
                key={type.id}
                onClick={() => handleFlowCellTypeSelect(type)}
                className="relative rounded-lg border border-gray-300 bg-white p-6 shadow-sm hover:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"
              >
                <div className="flex items-center space-x-3">
                  <CpuChipIcon className="h-10 w-10 text-indigo-600" />
                  <div className="text-left">
                    <h3 className="text-base font-medium text-gray-900">{type.name}</h3>
                    <p className="text-sm text-gray-500">{type.manufacturer} {type.model}</p>
                    <p className="text-sm text-gray-500">{type.lane_count} lanes</p>
                  </div>
                </div>
              </button>
            ))}
          </div>
        </div>
      ) : (
        <div className="mt-8">
          <div className="mb-4 flex items-center justify-between">
            <div>
              <h2 className="text-lg font-medium text-gray-900">
                {selectedFlowCellType.name} - {selectedFlowCellType.lane_count} Lanes
              </h2>
              <p className="text-sm text-gray-500">
                Drag libraries from the left panel to assign them to lanes
              </p>
            </div>
            <button
              onClick={() => {
                setSelectedFlowCellType(null);
                setLaneAssignments([]);
              }}
              className="text-sm text-gray-500 hover:text-gray-700"
            >
              Change flow cell type
            </button>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Available Libraries Panel */}
            <div className="lg:col-span-1">
              <div className="bg-white shadow rounded-lg p-4">
                <h3 className="text-base font-medium text-gray-900 mb-4">Available Libraries</h3>
                <div className="space-y-2 max-h-96 overflow-y-auto">
                  {availableLibraries?.map((library) => (
                    <div
                      key={library.id}
                      draggable
                      onDragStart={(e) => handleDragStart(e, library)}
                      className="p-3 border border-gray-200 rounded-md cursor-move hover:border-indigo-300 hover:shadow-sm"
                    >
                      <div className="flex items-center justify-between">
                        <BeakerIcon className="h-5 w-5 text-gray-400 flex-shrink-0" />
                        <div className="ml-2 flex-1">
                          <p className="text-sm font-medium text-gray-900">{library.batch_id}</p>
                          <p className="text-xs text-gray-500">
                            {library.sample_names.length} samples • {library.concentration} ng/μL
                          </p>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Stats Panel */}
              <div className="mt-4 bg-white shadow rounded-lg p-4">
                <h3 className="text-base font-medium text-gray-900 mb-4">Design Stats</h3>
                <div className="space-y-3">
                  <div>
                    <div className="flex justify-between text-sm">
                      <span className="text-gray-500">Lane Balance</span>
                      <span className="font-medium">{calculateLaneBalance().toFixed(0)}%</span>
                    </div>
                    <div className="mt-1 bg-gray-200 rounded-full h-2">
                      <div
                        className="bg-indigo-600 h-2 rounded-full"
                        style={{ width: `${calculateLaneBalance()}%` }}
                      />
                    </div>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500">Total Libraries</span>
                    <span className="font-medium">
                      {laneAssignments.reduce((sum, lane) => sum + lane.library_prep_ids.length, 0)}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            {/* Flow Cell Lanes */}
            <div className="lg:col-span-2">
              <div className="bg-white shadow rounded-lg p-4">
                <h3 className="text-base font-medium text-gray-900 mb-4">Lane Assignments</h3>
                <div className="grid grid-cols-2 gap-4">
                  {laneAssignments.map((lane) => (
                    <div
                      key={lane.lane_number}
                      onDragOver={handleDragOver}
                      onDrop={(e) => handleDrop(e, lane.lane_number)}
                      className={`border-2 border-dashed rounded-lg p-4 min-h-[120px] ${lane.color}`}
                    >
                      <div className="flex items-center justify-between mb-2">
                        <h4 className="text-sm font-medium text-gray-900">Lane {lane.lane_number}</h4>
                        <span className="text-xs text-gray-500">
                          {lane.library_prep_ids.length} libraries
                        </span>
                      </div>
                      <div className="space-y-1">
                        {lane.library_prep_ids.map((libId) => {
                          const library = getLibraryById(libId);
                          if (!library) return null;
                          return (
                            <div
                              key={libId}
                              className="flex items-center justify-between bg-white rounded p-1 text-xs"
                            >
                              <span className="truncate">{library.batch_id}</span>
                              <button
                                onClick={() => removeFromLane(lane.lane_number, libId)}
                                className="text-red-500 hover:text-red-700"
                              >
                                <XMarkIcon className="h-3 w-3" />
                              </button>
                            </div>
                          );
                        })}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* AI Suggestions */}
              {showAISuggestions && optimizeMutation.data && (
                <div className="mt-4 bg-blue-50 border border-blue-200 rounded-lg p-4">
                  <div className="flex items-start">
                    <InformationCircleIcon className="h-5 w-5 text-blue-400 flex-shrink-0" />
                    <div className="ml-3">
                      <h3 className="text-sm font-medium text-blue-800">AI Optimization Applied</h3>
                      <div className="mt-2 text-sm text-blue-700">
                        <p>Optimization Score: {optimizeMutation.data.score}%</p>
                        <ul className="mt-2 list-disc list-inside">
                          {optimizeMutation.data.suggestions?.map((suggestion: string, index: number) => (
                            <li key={index}>{suggestion}</li>
                          ))}
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}