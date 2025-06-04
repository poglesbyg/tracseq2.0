import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import axios from 'axios';
import { QueueListIcon, ArrowPathIcon } from '@heroicons/react/24/outline';
import SequencingJobDetails from '../components/SequencingJobDetails';

interface SequencingJob {
  id: number;
  name: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  created_at: string;
  updated_at: string;
  sample_count: number;
}

interface Sample {
  id: number;
  name: string;
  barcode: string;
}

export default function Sequencing() {
  const [selectedJobId, setSelectedJobId] = useState<number | null>(null);
  const [selectedSamples, setSelectedSamples] = useState<number[]>([]);
  const queryClient = useQueryClient();

  // Fetch sequencing jobs
  const { data: jobs, isLoading: isLoadingJobs } = useQuery<SequencingJob[]>({
    queryKey: ['sequencing-jobs'],
    queryFn: async () => {
      const response = await axios.get('/api/sequencing/jobs');
      return response.data;
    },
  });

  // Fetch available samples
  const { data: samples } = useQuery<Sample[]>({
    queryKey: ['samples'],
    queryFn: async () => {
      const response = await axios.get('/api/samples');
      return response.data;
    },
  });

  // Create job mutation
  const createJob = useMutation({
    mutationFn: async (data: { name: string; sample_ids: number[] }) => {
      const response = await axios.post('/api/sequencing/jobs', data);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sequencing-jobs'] });
      setSelectedSamples([]);
    },
  });

  const handleCreateJob = () => {
    if (selectedSamples.length > 0) {
      createJob.mutate({
        name: `Job ${new Date().toLocaleString()}`,
        sample_ids: selectedSamples,
      });
    }
  };

  const getStatusColor = (status: SequencingJob['status']) => {
    switch (status) {
      case 'completed':
        return 'bg-green-100 text-green-800';
      case 'running':
        return 'bg-blue-100 text-blue-800';
      case 'failed':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-yellow-100 text-yellow-800';
    }
  };

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-xl font-semibold text-gray-900">Sequencing Jobs</h1>
          <p className="mt-2 text-sm text-gray-700">
            A list of all sequencing jobs and their current status.
          </p>
        </div>
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
          <button
            type="button"
            onClick={handleCreateJob}
            disabled={selectedSamples.length === 0 || createJob.isPending}
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 sm:w-auto disabled:opacity-50"
          >
            {createJob.isPending ? 'Creating...' : 'Create Job'}
          </button>
        </div>
      </div>

      <div className="mt-8 grid grid-cols-1 gap-6 sm:grid-cols-2">
        <div>
          <div className="bg-white shadow sm:rounded-lg">
            <div className="px-4 py-5 sm:p-6">
              <h3 className="text-lg font-medium leading-6 text-gray-900">Available Samples</h3>
              <div className="mt-2 max-w-xl text-sm text-gray-500">
                <p>Select samples to include in a new sequencing job.</p>
              </div>
              <div className="mt-4">
                <select
                  multiple
                  className="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                  value={selectedSamples.map(String)}
                  onChange={(e) => {
                    const values = Array.from(e.target.selectedOptions, (option) => Number(option.value));
                    setSelectedSamples(values);
                  }}
                >
                  {samples?.map((sample) => (
                    <option key={sample.id} value={sample.id}>
                      {sample.name} ({sample.barcode})
                    </option>
                  ))}
                </select>
              </div>
            </div>
          </div>
        </div>

        <div>
          <div className="bg-white shadow sm:rounded-lg">
            <div className="px-4 py-5 sm:p-6">
              <h3 className="text-lg font-medium leading-6 text-gray-900">Recent Jobs</h3>
              <div className="mt-2 max-w-xl text-sm text-gray-500">
                <p>View and manage your sequencing jobs.</p>
              </div>
              <div className="mt-4">
                {isLoadingJobs ? (
                  <div className="flex justify-center py-4">
                    <ArrowPathIcon className="h-8 w-8 animate-spin text-indigo-600" />
                  </div>
                ) : (
                  <ul className="divide-y divide-gray-200">
                    {jobs?.map((job) => (
                      <li key={job.id} className="py-4">
                        <div className="flex items-center justify-between">
                          <div>
                            <p className="text-sm font-medium text-gray-900">{job.name}</p>
                            <p className="text-sm text-gray-500">
                              {job.sample_count} samples • Created {new Date(job.created_at).toLocaleDateString()}
                            </p>
                          </div>
                          <div className="flex items-center space-x-4">
                            <span
                              className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${getStatusColor(
                                job.status
                              )}`}
                            >
                              {job.status}
                            </span>
                            <button
                              type="button"
                              onClick={() => setSelectedJobId(job.id)}
                              className="text-indigo-600 hover:text-indigo-900 text-sm font-medium"
                            >
                              View Details
                            </button>
                          </div>
                        </div>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>

      {selectedJobId && (
        <div className="mt-8">
          <SequencingJobDetails jobId={selectedJobId} onClose={() => setSelectedJobId(null)} />
        </div>
      )}
    </div>
  );
} 
