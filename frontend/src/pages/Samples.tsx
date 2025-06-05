import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import SampleSubmissionWizard from '../components/SampleSubmissionWizard';
import SampleEditModal from '../components/SampleEditModal';

interface Sample {
  id: string;
  name: string;
  barcode: string;
  location: string;
  status: 'Pending' | 'Validated' | 'InStorage' | 'InSequencing' | 'Completed';
  created_at: string;
  updated_at: string;
  metadata: any;
}



export default function Samples() {
  const [showWizard, setShowWizard] = useState(false);
  const [editingSample, setEditingSample] = useState<Sample | null>(null);

  // Fetch samples
  const { data: samples, isLoading: isLoadingSamples } = useQuery<Sample[]>({
    queryKey: ['samples'],
    queryFn: async () => {
      const response = await axios.get('/api/samples');
      return response.data;
    },
  });



  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Completed':
        return 'bg-green-100 text-green-800';
      case 'Pending':
        return 'bg-yellow-100 text-yellow-800';
      case 'Validated':
        return 'bg-blue-100 text-blue-800';
      case 'InStorage':
        return 'bg-purple-100 text-purple-800';
      case 'InSequencing':
        return 'bg-indigo-100 text-indigo-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-xl font-semibold text-gray-900">Samples</h1>
          <p className="mt-2 text-sm text-gray-700">
            A list of all samples in the system including their name, template, storage location, and status.
          </p>
        </div>
        <div className="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
          <button
            type="button"
            onClick={() => setShowWizard(true)}
            className="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 sm:w-auto"
          >
            Add Sample
          </button>
        </div>
      </div>

      {showWizard && (
        <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4">
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

      <div className="mt-8 flex flex-col">
        <div className="-my-2 -mx-4 overflow-x-auto sm:-mx-6 lg:-mx-8">
          <div className="inline-block min-w-full py-2 align-middle md:px-6 lg:px-8">
            <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
              <table className="min-w-full divide-y divide-gray-300">
                <thead className="bg-gray-50">
                  <tr>
                    <th scope="col" className="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-6">
                      Name
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Template
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Storage Location
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Barcode
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Status
                    </th>
                    <th scope="col" className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">
                      Created
                    </th>
                    <th scope="col" className="relative py-3.5 pl-3 pr-4 sm:pr-6">
                      <span className="sr-only">Actions</span>
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-200 bg-white">
                  {isLoadingSamples ? (
                    <tr>
                      <td colSpan={7} className="px-3 py-4 text-sm text-gray-500 text-center">
                        Loading samples...
                      </td>
                    </tr>
                  ) : samples?.length === 0 ? (
                    <tr>
                      <td colSpan={7} className="px-3 py-4 text-sm text-gray-500 text-center">
                        No samples found
                      </td>
                    </tr>
                  ) : (
                    samples?.map((sample) => (
                      <tr key={sample.id}>
                        <td className="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900 sm:pl-6">
                          {sample.name}
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          {sample.metadata?.template_name || 'N/A'}
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          {sample.location}
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          {sample.barcode}
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm">
                          <span className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${getStatusColor(sample.status)}`}>
                            {sample.status}
                          </span>
                        </td>
                        <td className="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                          {new Date(sample.created_at).toLocaleDateString()}
                        </td>
                        <td className="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-6">
                          <button
                            onClick={() => setEditingSample(sample)}
                            className="text-indigo-600 hover:text-indigo-900"
                          >
                            Edit
                          </button>
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
    </div>
  );
} 
