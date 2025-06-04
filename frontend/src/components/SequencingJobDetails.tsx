import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import axios from 'axios';
import { DocumentIcon, ArrowPathIcon, CheckCircleIcon, XCircleIcon } from '@heroicons/react/24/outline';

interface SequencingJob {
  id: number;
  name: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  created_at: string;
  updated_at: string;
  sample_sheet_url: string | null;
  samples: {
    id: number;
    name: string;
    barcode: string;
  }[];
}

interface JobDetailsProps {
  jobId: number;
  onClose: () => void;
}

export default function SequencingJobDetails({ jobId, onClose }: JobDetailsProps) {
  const [isGeneratingSheet, setIsGeneratingSheet] = useState(false);
  const queryClient = useQueryClient();

  // Fetch job details
  const { data: job, isLoading } = useQuery<SequencingJob>({
    queryKey: ['sequencing-job', jobId],
    queryFn: async () => {
      const response = await axios.get(`/api/sequencing/jobs/${jobId}`);
      return response.data;
    },
  });

  // Generate sample sheet mutation
  const generateSampleSheet = useMutation({
    mutationFn: async () => {
      const response = await axios.post(`/api/sequencing/jobs/${jobId}/sample-sheet`);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sequencing-job', jobId] });
      setIsGeneratingSheet(false);
    },
  });

  // Update job status mutation
  const updateJobStatus = useMutation({
    mutationFn: async (status: SequencingJob['status']) => {
      const response = await axios.patch(`/api/sequencing/jobs/${jobId}`, { status });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sequencing-job', jobId] });
    },
  });

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

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <ArrowPathIcon className="h-8 w-8 animate-spin text-indigo-600" />
      </div>
    );
  }

  if (!job) {
    return null;
  }

  return (
    <div className="bg-white shadow sm:rounded-lg">
      <div className="px-4 py-5 sm:p-6">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-medium leading-6 text-gray-900">Job Details</h3>
          <button
            type="button"
            onClick={onClose}
            className="text-gray-400 hover:text-gray-500"
          >
            <span className="sr-only">Close</span>
            <XCircleIcon className="h-6 w-6" />
          </button>
        </div>

        <div className="mt-4">
          <dl className="grid grid-cols-1 gap-x-4 gap-y-4 sm:grid-cols-2">
            <div>
              <dt className="text-sm font-medium text-gray-500">Job Name</dt>
              <dd className="mt-1 text-sm text-gray-900">{job.name}</dd>
            </div>
            <div>
              <dt className="text-sm font-medium text-gray-500">Status</dt>
              <dd className="mt-1">
                <span className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${getStatusColor(job.status)}`}>
                  {job.status}
                </span>
              </dd>
            </div>
            <div>
              <dt className="text-sm font-medium text-gray-500">Created</dt>
              <dd className="mt-1 text-sm text-gray-900">
                {new Date(job.created_at).toLocaleString()}
              </dd>
            </div>
            <div>
              <dt className="text-sm font-medium text-gray-500">Last Updated</dt>
              <dd className="mt-1 text-sm text-gray-900">
                {new Date(job.updated_at).toLocaleString()}
              </dd>
            </div>
          </dl>
        </div>

        <div className="mt-6">
          <h4 className="text-sm font-medium text-gray-900">Samples</h4>
          <ul className="mt-2 divide-y divide-gray-200">
            {job.samples.map((sample) => (
              <li key={sample.id} className="py-3">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-900">{sample.name}</p>
                    <p className="text-sm text-gray-500">{sample.barcode}</p>
                  </div>
                </div>
              </li>
            ))}
          </ul>
        </div>

        <div className="mt-6">
          <h4 className="text-sm font-medium text-gray-900">Sample Sheet</h4>
          <div className="mt-2">
            {job.sample_sheet_url ? (
              <a
                href={job.sample_sheet_url}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <DocumentIcon className="h-5 w-5 mr-2 text-gray-400" />
                Download Sample Sheet
              </a>
            ) : (
              <button
                type="button"
                onClick={() => {
                  setIsGeneratingSheet(true);
                  generateSampleSheet.mutate();
                }}
                disabled={isGeneratingSheet}
                className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                {isGeneratingSheet ? (
                  <>
                    <ArrowPathIcon className="h-5 w-5 mr-2 animate-spin" />
                    Generating...
                  </>
                ) : (
                  <>
                    <DocumentIcon className="h-5 w-5 mr-2" />
                    Generate Sample Sheet
                  </>
                )}
              </button>
            )}
          </div>
        </div>

        <div className="mt-6">
          <h4 className="text-sm font-medium text-gray-900">Job Status</h4>
          <div className="mt-2 flex space-x-4">
            <button
              type="button"
              onClick={() => updateJobStatus.mutate('running')}
              disabled={job.status === 'running'}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
            >
              Start Job
            </button>
            <button
              type="button"
              onClick={() => updateJobStatus.mutate('completed')}
              disabled={job.status === 'completed'}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50"
            >
              <CheckCircleIcon className="h-5 w-5 mr-2" />
              Mark as Completed
            </button>
            <button
              type="button"
              onClick={() => updateJobStatus.mutate('failed')}
              disabled={job.status === 'failed'}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50"
            >
              <XCircleIcon className="h-5 w-5 mr-2" />
              Mark as Failed
            </button>
          </div>
        </div>
      </div>
    </div>
  );
} 
