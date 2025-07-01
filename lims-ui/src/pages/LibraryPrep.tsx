import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import api from '../utils/axios';
import {
  BeakerIcon,
  DocumentTextIcon,
  ClockIcon,
  CheckCircleIcon,
  XCircleIcon,
  MagnifyingGlassIcon,
  PlusIcon,
  ArrowPathIcon,
} from '@heroicons/react/24/outline';
import { format } from 'date-fns';

interface LibraryPrepProtocol {
  id: string;
  name: string;
  version: string;
  protocol_type: string;
  kit_name?: string;
  kit_manufacturer?: string;
  estimated_duration_hours: number;
  is_active: boolean;
}

interface LibraryPreparation {
  id: string;
  batch_id: string;
  project_id: string;
  project_name?: string;
  protocol_id: string;
  protocol_name?: string;
  sample_ids: string[];
  status: 'pending' | 'in_progress' | 'completed' | 'failed' | 'qc_review';
  prep_date: string;
  operator_name?: string;
  qc_status?: 'pending' | 'passed' | 'failed' | 'conditional';
  notes?: string;
  created_at: string;
}

export default function LibraryPrep() {
  const [selectedTab, setSelectedTab] = useState<'preparations' | 'protocols'>('preparations');
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedPrep, setSelectedPrep] = useState<LibraryPreparation | null>(null);

  // Fetch library preparations
  const { data: preparations, isLoading: isLoadingPreps } = useQuery<LibraryPreparation[]>({
    queryKey: ['library-preparations', searchTerm],
    queryFn: async () => {
      const params = searchTerm ? { search: searchTerm } : {};
      const response = await api.get('/api/library-prep/preparations', { params });
      return response.data;
    },
  });

  // Fetch protocols
  const { data: protocols, isLoading: isLoadingProtocols } = useQuery<LibraryPrepProtocol[]>({
    queryKey: ['library-prep-protocols'],
    queryFn: async () => {
      const response = await api.get('/api/library-prep/protocols');
      return response.data;
    },
  });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'bg-green-100 text-green-800';
      case 'in_progress':
        return 'bg-blue-100 text-blue-800';
      case 'failed':
        return 'bg-red-100 text-red-800';
      case 'qc_review':
        return 'bg-yellow-100 text-yellow-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getQCStatusIcon = (status?: string) => {
    switch (status) {
      case 'passed':
        return <CheckCircleIcon className="h-5 w-5 text-green-500" />;
      case 'failed':
        return <XCircleIcon className="h-5 w-5 text-red-500" />;
      case 'conditional':
        return <ClockIcon className="h-5 w-5 text-yellow-500" />;
      default:
        return <ClockIcon className="h-5 w-5 text-gray-400" />;
    }
  };

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      {/* Header */}
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-xl font-semibold text-gray-900">Library Preparation</h1>
          <p className="mt-2 text-sm text-gray-700">
            Manage library preparation workflows, protocols, and QC tracking
          </p>
        </div>
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
          <button
            type="button"
            onClick={() => {/* Navigate to create modal */}}
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 sm:w-auto"
          >
            <PlusIcon className="-ml-1 mr-2 h-5 w-5" />
            New Library Prep
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="mt-4">
        <div className="sm:hidden">
          <select
            value={selectedTab}
            onChange={(e) => setSelectedTab(e.target.value as 'preparations' | 'protocols')}
            className="block w-full rounded-md border-gray-300 focus:border-indigo-500 focus:ring-indigo-500"
          >
            <option value="preparations">Preparations</option>
            <option value="protocols">Protocols</option>
          </select>
        </div>
        <div className="hidden sm:block">
          <nav className="flex space-x-8" aria-label="Tabs">
            <button
              onClick={() => setSelectedTab('preparations')}
              className={`${
                selectedTab === 'preparations'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              } whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
            >
              Preparations
            </button>
            <button
              onClick={() => setSelectedTab('protocols')}
              className={`${
                selectedTab === 'protocols'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              } whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm`}
            >
              Protocols
            </button>
          </nav>
        </div>
      </div>

      {/* Search bar */}
      {selectedTab === 'preparations' && (
        <div className="mt-6">
          <div className="max-w-lg">
            <label htmlFor="search" className="sr-only">
              Search by batch number
            </label>
            <div className="relative">
              <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <MagnifyingGlassIcon className="h-5 w-5 text-gray-400" />
              </div>
              <input
                type="search"
                name="search"
                id="search"
                className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                placeholder="Search by batch number..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
              />
            </div>
          </div>
        </div>
      )}

      {/* Content */}
      <div className="mt-8">
        {selectedTab === 'preparations' ? (
          <div className="bg-white shadow overflow-hidden sm:rounded-md">
            {isLoadingPreps ? (
              <div className="flex justify-center py-8">
                <ArrowPathIcon className="h-8 w-8 animate-spin text-indigo-600" />
              </div>
            ) : (
              <ul className="divide-y divide-gray-200">
                {preparations?.map((prep) => (
                  <li key={prep.id}>
                    <div className="px-4 py-4 sm:px-6 hover:bg-gray-50 cursor-pointer"
                         onClick={() => setSelectedPrep(prep)}>
                      <div className="flex items-center justify-between">
                        <div className="flex items-center">
                          <div className="flex-shrink-0">
                            <BeakerIcon className="h-10 w-10 text-gray-400" />
                          </div>
                          <div className="ml-4">
                            <div className="text-sm font-medium text-gray-900">
                              Batch: {prep.batch_id}
                            </div>
                            <div className="text-sm text-gray-500">
                              Protocol: {prep.protocol_name || prep.protocol_id}
                            </div>
                            <div className="text-sm text-gray-500">
                              {prep.sample_ids.length} samples • {format(new Date(prep.prep_date), 'MMM dd, yyyy')}
                            </div>
                          </div>
                        </div>
                        <div className="flex items-center space-x-4">
                          <div className="flex items-center space-x-2">
                            {getQCStatusIcon(prep.qc_status)}
                            <span className="text-sm text-gray-500">QC</span>
                          </div>
                          <span
                            className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${getStatusColor(
                              prep.status
                            )}`}
                          >
                            {prep.status.replace('_', ' ')}
                          </span>
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        ) : (
          <div className="bg-white shadow overflow-hidden sm:rounded-md">
            {isLoadingProtocols ? (
              <div className="flex justify-center py-8">
                <ArrowPathIcon className="h-8 w-8 animate-spin text-indigo-600" />
              </div>
            ) : (
              <ul className="divide-y divide-gray-200">
                {protocols?.filter(p => p.is_active).map((protocol) => (
                  <li key={protocol.id}>
                    <div className="px-4 py-4 sm:px-6">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center">
                          <div className="flex-shrink-0">
                            <DocumentTextIcon className="h-10 w-10 text-gray-400" />
                          </div>
                          <div className="ml-4">
                            <div className="text-sm font-medium text-gray-900">
                              {protocol.name} v{protocol.version}
                            </div>
                            <div className="text-sm text-gray-500">
                              Type: {protocol.protocol_type} • Duration: {protocol.estimated_duration_hours}h
                            </div>
                            {protocol.kit_name && (
                              <div className="text-sm text-gray-500">
                                Kit: {protocol.kit_name} ({protocol.kit_manufacturer})
                              </div>
                            )}
                          </div>
                        </div>
                        <div>
                          <button className="text-indigo-600 hover:text-indigo-900 text-sm font-medium">
                            View Details
                          </button>
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </div>

      {/* Selected Prep Details Modal */}
      {selectedPrep && (
        <div className="fixed inset-0 z-10 overflow-y-auto">
          <div className="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
            <div className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" 
                 onClick={() => setSelectedPrep(null)} />
            <div className="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
              <div className="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
                <div className="sm:flex sm:items-start">
                  <div className="mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-indigo-100 sm:mx-0 sm:h-10 sm:w-10">
                    <BeakerIcon className="h-6 w-6 text-indigo-600" />
                  </div>
                  <div className="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left flex-1">
                    <h3 className="text-lg leading-6 font-medium text-gray-900">
                      Library Prep Details
                    </h3>
                    <div className="mt-4 space-y-3">
                      <div>
                        <span className="text-sm font-medium text-gray-500">Batch ID:</span>
                        <span className="ml-2 text-sm text-gray-900">{selectedPrep.batch_id}</span>
                      </div>
                      <div>
                        <span className="text-sm font-medium text-gray-500">Project:</span>
                        <span className="ml-2 text-sm text-gray-900">{selectedPrep.project_name || selectedPrep.project_id}</span>
                      </div>
                      <div>
                        <span className="text-sm font-medium text-gray-500">Status:</span>
                        <span className={`ml-2 inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${getStatusColor(selectedPrep.status)}`}>
                          {selectedPrep.status}
                        </span>
                      </div>
                      <div>
                        <span className="text-sm font-medium text-gray-500">QC Status:</span>
                        <span className="ml-2 text-sm text-gray-900 flex items-center inline-flex">
                          {getQCStatusIcon(selectedPrep.qc_status)}
                          <span className="ml-1">{selectedPrep.qc_status || 'Pending'}</span>
                        </span>
                      </div>
                      {selectedPrep.notes && (
                        <div>
                          <span className="text-sm font-medium text-gray-500">Notes:</span>
                          <p className="mt-1 text-sm text-gray-900">{selectedPrep.notes}</p>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              </div>
              <div className="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
                <button
                  type="button"
                  className="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-indigo-600 text-base font-medium text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:ml-3 sm:w-auto sm:text-sm"
                  onClick={() => {/* Navigate to QC page */}}
                >
                  View QC Results
                </button>
                <button
                  type="button"
                  className="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
                  onClick={() => setSelectedPrep(null)}
                >
                  Close
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}