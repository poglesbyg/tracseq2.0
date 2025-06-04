import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import axios from 'axios';
import { QueueListIcon } from '@heroicons/react/24/outline';

interface SequencingJob {
  id: string;
  sample_id: string;
  status: string;
  created_at: string;
  updated_at: string;
  sample: {
    name: string;
    template_id: string;
  };
}

interface Sample {
  id: string;
  name: string;
  template_id: string;
  status: string;
  created_at: string;
  updated_at: string;
}

export default function Sequencing() {
  const [selectedSample, setSelectedSample] = useState<string>('');
  const queryClient = useQueryClient();

  const { data: jobs, isLoading: isLoadingJobs } = useQuery<SequencingJob[]>({
    queryKey: ['sequencingJobs'],
    queryFn: async () => {
      const response = await axios.get('/api/sequencing/jobs');
      return response.data;
    },
  });

  const { data: samples, isLoading: isLoadingSamples } = useQuery<Sample[]>({
    queryKey: ['samples'],
    queryFn: async () => {
      const response = await axios.get('/api/samples');
      return response.data;
    },
  });

  const createJobMutation = useMutation({
    mutationFn: async (sampleId: string) => {
      const response = await axios.post('/api/sequencing/jobs', { sample_id: sampleId });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sequencingJobs'] });
      setSelectedSample('');
    },
  });

  const handleCreateJob = () => {
    if (selectedSample) {
      createJobMutation.mutate(selectedSample);
    }
  };

  if (isLoadingJobs || isLoadingSamples) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  return (
    <div>
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-2xl font-semibold text-gray-900">Sequencing Jobs</h1>
          <p className="mt-2 text-sm text-gray-700">
            A list of all sequencing jobs and their current status.
          </p>
        </div>
        <div className="mt-4 sm:ml-16 sm:mt-0 sm:flex-none">
          <select
            value={selectedSample}
            onChange={(e) => setSelectedSample(e.target.value)}
            className="block w-full rounded-md border-0 py-1.5 pl-3 pr-10 text-gray-900 ring-1 ring-inset ring-gray-300 focus:ring-2 focus:ring-indigo-600 sm:text-sm sm:leading-6"
          >
            <option value="">Select a sample</option>
            {samples?.map((sample: Sample) => (
              <option key={sample.id} value={sample.id}>
                {sample.name}
              </option>
            ))}
          </select>
          <button
            type="button"
            onClick={handleCreateJob}
            disabled={!selectedSample || createJobMutation.isPending}
            className="mt-3 block rounded-md bg-indigo-600 px-3 py-2 text-center text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
          >
            {createJobMutation.isPending ? 'Creating...' : 'Create Job'}
          </button>
        </div>
      </div>

      <div className="mt-8 flow-root">
        <div className="-mx-4 -my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
          <div className="inline-block min-w-full py-2 align-middle sm:px-6 lg:px-8">
            <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 sm:rounded-lg">
              <table className="min-w-full divide-y divide-gray-300">
                <thead className="bg-gray-50">
                  <tr>
                    <th scope="col" className="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-6">
                      Sample
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Status
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Created
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Updated
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-200 bg-white">
                  {jobs?.map((job) => (
                    <tr key={job.id}>
                      <td className="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900 sm:pl-6">
                        <div className="flex items-center">
                          <QueueListIcon className="h-5 w-5 text-gray-400 mr-2" />
                          {job.sample.name}
                        </div>
                      </td>
                      <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                        <span
                          className={`inline-flex items-center rounded-md px-2 py-1 text-xs font-medium ${
                            job.status === 'completed'
                              ? 'bg-green-50 text-green-700 ring-1 ring-inset ring-green-600/20'
                              : job.status === 'running'
                              ? 'bg-blue-50 text-blue-700 ring-1 ring-inset ring-blue-600/20'
                              : job.status === 'pending'
                              ? 'bg-yellow-50 text-yellow-700 ring-1 ring-inset ring-yellow-600/20'
                              : 'bg-gray-50 text-gray-700 ring-1 ring-inset ring-gray-600/20'
                          }`}
                        >
                          {job.status}
                        </span>
                      </td>
                      <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                        {new Date(job.created_at).toLocaleDateString()}
                      </td>
                      <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                        {new Date(job.updated_at).toLocaleDateString()}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
} 
