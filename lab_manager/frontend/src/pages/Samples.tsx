import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import { 
  EyeIcon, 
  FunnelIcon, 
  CalendarIcon, 
  ClockIcon,
  ArrowPathIcon 
} from '@heroicons/react/24/outline';
import SampleSubmissionWizard from '../components/SampleSubmissionWizard';
import SampleEditModal from '../components/SampleEditModal';
import ProcessFlow from '../components/ProcessFlow';

interface Sample {
  id: string;
  name: string;
  barcode: string;
  location: string;
  status: 'Pending' | 'Validated' | 'InStorage' | 'InSequencing' | 'Completed';
  created_at: string;
  updated_at: string;
  metadata: any;
  // Enhanced temporal data
  timestamps?: {
    created_at?: string;
    validated_at?: string;
    stored_at?: string;
    sequencing_started_at?: string;
    completed_at?: string;
  };
}

export default function Samples() {
  const [showWizard, setShowWizard] = useState(false);
  const [editingSample, setEditingSample] = useState<Sample | null>(null);
  const [selectedSample, setSelectedSample] = useState<Sample | null>(null);
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [timeFilter, setTimeFilter] = useState<string>('all');
  const [viewMode, setViewMode] = useState<'table' | 'process'>('table');

  // Fetch samples
  const { data: samples, isLoading: isLoadingSamples, refetch } = useQuery<Sample[]>({
    queryKey: ['samples'],
    queryFn: async () => {
      const response = await axios.get('/api/samples');
      return response.data;
    },
  });

  // Filter samples based on current filters
  const filteredSamples = samples?.filter(sample => {
    // Status filter
    if (statusFilter !== 'all' && sample.status !== statusFilter) return false;
    
    // Time filter
    if (timeFilter !== 'all') {
      const sampleDate = new Date(sample.created_at);
      const now = new Date();
      const diffInHours = (now.getTime() - sampleDate.getTime()) / (1000 * 60 * 60);
      
      switch (timeFilter) {
        case '24h':
          if (diffInHours > 24) return false;
          break;
        case '7d':
          if (diffInHours > 168) return false;
          break;
        case '30d':
          if (diffInHours > 720) return false;
          break;
      }
    }
    
    return true;
  }) || [];

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Completed':
        return 'bg-green-100 text-green-800 border-green-200';
      case 'Pending':
        return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      case 'Validated':
        return 'bg-blue-100 text-blue-800 border-blue-200';
      case 'InStorage':
        return 'bg-purple-100 text-purple-800 border-purple-200';
      case 'InSequencing':
        return 'bg-indigo-100 text-indigo-800 border-indigo-200';
      default:
        return 'bg-gray-100 text-gray-800 border-gray-200';
    }
  };

  const formatRelativeTime = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 1) return 'Just now';
    if (diffInHours < 24) return `${diffInHours}h ago`;
    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays < 7) return `${diffInDays}d ago`;
    return date.toLocaleDateString();
  };

  const getProcessingDuration = (sample: Sample) => {
    const created = new Date(sample.created_at);
    const updated = new Date(sample.updated_at);
    const diffInHours = Math.floor((updated.getTime() - created.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 24) return `${diffInHours}h`;
    const diffInDays = Math.floor(diffInHours / 24);
    const remainingHours = diffInHours % 24;
    return remainingHours > 0 ? `${diffInDays}d ${remainingHours}h` : `${diffInDays}d`;
  };

  const statusCounts = samples?.reduce((acc, sample) => {
    acc[sample.status] = (acc[sample.status] || 0) + 1;
    return acc;
  }, {} as Record<string, number>) || {};

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-xl font-semibold text-gray-900">Sample Management</h1>
          <p className="mt-2 text-sm text-gray-700">
            Comprehensive view of laboratory samples with process tracking and temporal analysis.
          </p>
        </div>
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none space-x-3">
          <button
            type="button"
            onClick={() => refetch()}
            className="inline-flex items-center rounded-md border border-gray-300 bg-white px-3 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
          >
            <ArrowPathIcon className="h-4 w-4 mr-2" />
            Refresh
          </button>
          <button
            type="button"
            onClick={() => setShowWizard(true)}
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 sm:w-auto"
          >
            Add Sample
          </button>
        </div>
      </div>

      {/* Status Overview Cards */}
      <div className="mt-8 grid grid-cols-2 md:grid-cols-5 gap-4">
        {Object.entries(statusCounts).map(([status, count]) => (
          <div key={status} className="bg-white overflow-hidden shadow rounded-lg">
            <div className="p-5">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${getStatusColor(status)}`}>
                    {status}
                  </span>
                </div>
                <div className="ml-5 w-0 flex-1">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 truncate">Count</dt>
                    <dd className="text-lg font-medium text-gray-900">{count}</dd>
                  </dl>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Filters and View Controls */}
      <div className="mt-8 bg-white shadow rounded-lg">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <div className="flex items-center space-x-2">
                <FunnelIcon className="h-4 w-4 text-gray-500" />
                <span className="text-sm font-medium text-gray-700">Filters</span>
              </div>
              
              {/* Status Filter */}
              <select
                value={statusFilter}
                onChange={(e) => setStatusFilter(e.target.value)}
                className="text-sm border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="all">All Status</option>
                <option value="Pending">Pending</option>
                <option value="Validated">Validated</option>
                <option value="InStorage">In Storage</option>
                <option value="InSequencing">In Sequencing</option>
                <option value="Completed">Completed</option>
              </select>

              {/* Time Filter */}
              <select
                value={timeFilter}
                onChange={(e) => setTimeFilter(e.target.value)}
                className="text-sm border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="all">All Time</option>
                <option value="24h">Last 24 Hours</option>
                <option value="7d">Last 7 Days</option>
                <option value="30d">Last 30 Days</option>
              </select>
            </div>

            {/* View Mode Toggle */}
            <div className="flex items-center space-x-2">
              <span className="text-sm text-gray-500">View:</span>
              <button
                onClick={() => setViewMode('table')}
                className={`px-3 py-1 text-sm rounded-md ${viewMode === 'table' ? 'bg-indigo-100 text-indigo-700' : 'text-gray-500 hover:text-gray-700'}`}
              >
                Table
              </button>
              <button
                onClick={() => setViewMode('process')}
                className={`px-3 py-1 text-sm rounded-md ${viewMode === 'process' ? 'bg-indigo-100 text-indigo-700' : 'text-gray-500 hover:text-gray-700'}`}
              >
                Process
              </button>
            </div>
          </div>
        </div>

        {/* Results Count */}
        <div className="px-6 py-3 bg-gray-50 border-b border-gray-200">
          <p className="text-sm text-gray-600">
            Showing {filteredSamples.length} of {samples?.length || 0} samples
          </p>
        </div>
      </div>

      {/* Sample Process Detail Modal */}
      {selectedSample && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-y-auto">
            <div className="p-6">
              <div className="flex justify-between items-center mb-6">
                <div>
                  <h2 className="text-lg font-medium text-gray-900">Sample Process Flow</h2>
                  <p className="text-sm text-gray-500">{selectedSample.name} ({selectedSample.barcode})</p>
                </div>
                <button
                  type="button"
                  onClick={() => setSelectedSample(null)}
                  className="text-gray-400 hover:text-gray-500"
                >
                  <span className="sr-only">Close</span>
                  <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
              
              <ProcessFlow
                currentStatus={selectedSample.status}
                timestamps={selectedSample.timestamps || {
                  created_at: selectedSample.created_at,
                  validated_at: selectedSample.status === 'Validated' || selectedSample.status === 'InStorage' || selectedSample.status === 'InSequencing' || selectedSample.status === 'Completed' ? selectedSample.updated_at : undefined,
                  stored_at: selectedSample.status === 'InStorage' || selectedSample.status === 'InSequencing' || selectedSample.status === 'Completed' ? selectedSample.updated_at : undefined,
                  sequencing_started_at: selectedSample.status === 'InSequencing' || selectedSample.status === 'Completed' ? selectedSample.updated_at : undefined,
                  completed_at: selectedSample.status === 'Completed' ? selectedSample.updated_at : undefined,
                }}
              />

              {/* Sample Details */}
              <div className="mt-8 bg-gray-50 rounded-lg p-4">
                <h3 className="text-sm font-medium text-gray-900 mb-3">Sample Details</h3>
                <dl className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <dt className="font-medium text-gray-500">Location</dt>
                    <dd className="text-gray-900">{selectedSample.location}</dd>
                  </div>
                  <div>
                    <dt className="font-medium text-gray-500">Processing Duration</dt>
                    <dd className="text-gray-900">{getProcessingDuration(selectedSample)}</dd>
                  </div>
                  <div>
                    <dt className="font-medium text-gray-500">Created</dt>
                    <dd className="text-gray-900">{new Date(selectedSample.created_at).toLocaleString()}</dd>
                  </div>
                  <div>
                    <dt className="font-medium text-gray-500">Last Updated</dt>
                    <dd className="text-gray-900">{new Date(selectedSample.updated_at).toLocaleString()}</dd>
                  </div>
                </dl>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Sample Submission Wizard */}
      {showWizard && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-y-auto">
            <div className="p-6">
              <div className="flex justify-between items-center mb-4">
                <h2 className="text-lg font-medium">Add New Sample</h2>
                <button
                  type="button"
                  onClick={() => setShowWizard(false)}
                  className="text-gray-400 hover:text-gray-500"
                >
                  <span className="sr-only">Close</span>
                  <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
              <SampleSubmissionWizard 
                onClose={() => setShowWizard(false)}
                onSuccess={() => {
                  // Additional success handling can be added here
                  // (e.g., show success toast)
                }}
              />
            </div>
          </div>
        </div>
      )}

      {editingSample && (
        <SampleEditModal
          sample={editingSample}
          onClose={() => setEditingSample(null)}
        />
      )}

      {/* Samples Display */}
      <div className="mt-8">
        {viewMode === 'table' ? (
          /* Table View */
          <div className="flex flex-col">
            <div className="-my-2 -mx-4 overflow-x-auto sm:-mx-6 lg:-mx-8">
              <div className="inline-block min-w-full py-2 align-middle md:px-6 lg:px-8">
                <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
                  <table className="min-w-full divide-y divide-gray-300">
                    <thead className="bg-gray-50">
                      <tr>
                        <th scope="col" className="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-6">
                          Sample Info
                        </th>
                        <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                          Status & Location
                        </th>
                        <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                          Temporal Data
                        </th>
                        <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                          Processing Time
                        </th>
                        <th scope="col" className="relative py-3.5 pl-3 pr-4 sm:pr-6">
                          <span className="sr-only">Actions</span>
                        </th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-200 bg-white">
                      {isLoadingSamples ? (
                        <tr>
                          <td colSpan={5} className="px-3 py-4 text-sm text-gray-500 text-center">
                            Loading samples...
                          </td>
                        </tr>
                      ) : filteredSamples.length === 0 ? (
                        <tr>
                          <td colSpan={5} className="px-3 py-4 text-sm text-gray-500 text-center">
                            No samples found matching current filters
                          </td>
                        </tr>
                      ) : (
                        filteredSamples.map((sample) => (
                          <tr key={sample.id} className="hover:bg-gray-50">
                            <td className="whitespace-nowrap py-4 pl-4 pr-3 text-sm sm:pl-6">
                              <div>
                                <div className="font-medium text-gray-900">{sample.name}</div>
                                <div className="text-gray-500">
                                  <span className="font-mono text-xs">{sample.barcode}</span>
                                </div>
                                <div className="text-xs text-gray-400">
                                  Template: {sample.metadata?.template_name || 'N/A'}
                                </div>
                              </div>
                            </td>
                            <td className="whitespace-nowrap px-3 py-4 text-sm">
                              <div>
                                <span className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 border ${getStatusColor(sample.status)}`}>
                                  {sample.status}
                                </span>
                                <div className="text-gray-500 mt-1 text-xs">{sample.location}</div>
                              </div>
                            </td>
                            <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                              <div className="space-y-1">
                                <div className="flex items-center text-xs">
                                  <CalendarIcon className="h-3 w-3 mr-1" />
                                  Created: {formatRelativeTime(sample.created_at)}
                                </div>
                                <div className="flex items-center text-xs">
                                  <ClockIcon className="h-3 w-3 mr-1" />
                                  Updated: {formatRelativeTime(sample.updated_at)}
                                </div>
                              </div>
                            </td>
                            <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                              <span className="font-medium">{getProcessingDuration(sample)}</span>
                            </td>
                            <td className="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-6">
                              <div className="flex items-center space-x-2">
                                <button
                                  onClick={() => setSelectedSample(sample)}
                                  className="text-indigo-600 hover:text-indigo-900"
                                  title="View Process Flow"
                                >
                                  <EyeIcon className="h-4 w-4" />
                                </button>
                                <button
                                  onClick={() => setEditingSample(sample)}
                                  className="text-indigo-600 hover:text-indigo-900"
                                >
                                  Edit
                                </button>
                              </div>
                            </td>
                          </tr>
                        ))
                      )}
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </div>
        ) : (
          /* Process View */
          <div className="space-y-6">
            {filteredSamples.map((sample) => (
              <div key={sample.id} className="bg-white shadow rounded-lg p-6">
                <div className="flex items-center justify-between mb-4">
                  <div>
                    <h3 className="text-lg font-medium text-gray-900">{sample.name}</h3>
                    <p className="text-sm text-gray-500">
                      {sample.barcode} • {sample.location}
                    </p>
                  </div>
                  <div className="flex items-center space-x-3">
                    <span className={`inline-flex rounded-full px-3 py-1 text-sm font-semibold border ${getStatusColor(sample.status)}`}>
                      {sample.status}
                    </span>
                    <button
                      onClick={() => setEditingSample(sample)}
                      className="text-indigo-600 hover:text-indigo-900 text-sm"
                    >
                      Edit
                    </button>
                  </div>
                </div>
                
                <ProcessFlow
                  currentStatus={sample.status}
                  timestamps={{
                    created_at: sample.created_at,
                    validated_at: sample.status === 'Validated' || sample.status === 'InStorage' || sample.status === 'InSequencing' || sample.status === 'Completed' ? sample.updated_at : undefined,
                    stored_at: sample.status === 'InStorage' || sample.status === 'InSequencing' || sample.status === 'Completed' ? sample.updated_at : undefined,
                    sequencing_started_at: sample.status === 'InSequencing' || sample.status === 'Completed' ? sample.updated_at : undefined,
                    completed_at: sample.status === 'Completed' ? sample.updated_at : undefined,
                  }}
                  className="mt-4"
                />
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
} 
