import React, { useState, useEffect } from 'react';
import {
  ClipboardDocumentCheckIcon,
  BeakerIcon,
  CheckCircleIcon,
  XCircleIcon,
  ExclamationTriangleIcon,
  ChartBarIcon,
  ArrowPathIcon,
  DocumentMagnifyingGlassIcon
} from '@heroicons/react/24/outline';
import axios from '../../utils/axios';

interface QCReview {
  id: string;
  sample_id: string;
  review_type: 'extraction' | 'library_prep' | 'sequencing' | 'data_quality';
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  reviewer: string;
  created_at: string;
  completed_at?: string;
  results?: QCResults;
}

interface QCResults {
  passed: boolean;
  metrics: Record<string, any>;
  issues: string[];
  recommendations: string[];
}

interface QCMetric {
  id: string;
  name: string;
  description: string;
  metric_type: 'numeric' | 'boolean' | 'text';
  units?: string;
  min_value?: number;
  max_value?: number;
  critical: boolean;
}

interface ControlSample {
  id: string;
  name: string;
  type: 'positive' | 'negative' | 'standard';
  expected_values: Record<string, any>;
  last_run?: string;
  status?: 'pass' | 'fail' | 'warning';
}

export const QAQCWorkflowInterface: React.FC = () => {
  const [reviews, setReviews] = useState<QCReview[]>([]);
  const [metrics, setMetrics] = useState<QCMetric[]>([]);
  const [controlSamples, setControlSamples] = useState<ControlSample[]>([]);
  const [selectedTab, setSelectedTab] = useState<'reviews' | 'metrics' | 'controls' | 'trends'>('reviews');
  const [loading, setLoading] = useState(true);
  const [_selectedReview, setSelectedReview] = useState<QCReview | null>(null);

  useEffect(() => {
    fetchData();
  }, []);

  const fetchData = async () => {
    try {
      const [reviewsRes, metricsRes, controlsRes] = await Promise.all([
        axios.get('/api/qc/reviews'),
        axios.get('/api/qc/metrics/definitions'),
        axios.get('/api/qc/control-samples')
      ]);

      setReviews(reviewsRes.data.reviews || []);
      setMetrics(metricsRes.data.metrics || []);
      setControlSamples(controlsRes.data.control_samples || []);
    } catch (error) {
      console.error('Failed to fetch QC data:', error);
    } finally {
      setLoading(false);
    }
  };

  const _startQCReview = async (sampleId: string, reviewType: string) => {
    try {
      const response = await axios.post('/api/qc/reviews', {
        sample_id: sampleId,
        review_type: reviewType
      });
      
      setReviews([response.data.review, ...reviews]);
    } catch (error) {
      console.error('Failed to start QC review:', error);
    }
  };

  const _completeQCReview = async (reviewId: string, results: QCResults) => {
    try {
      await axios.post(`/api/qc/reviews/${reviewId}/complete`, { results });
      
      setReviews(reviews.map(r => 
        r.id === reviewId 
          ? { ...r, status: 'completed', completed_at: new Date().toISOString(), results }
          : r
      ));
    } catch (error) {
      console.error('Failed to complete QC review:', error);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircleIcon className="h-5 w-5 text-green-500" />;
      case 'failed':
        return <XCircleIcon className="h-5 w-5 text-red-500" />;
      case 'in_progress':
        return <ArrowPathIcon className="h-5 w-5 text-blue-500 animate-spin" />;
      default:
        return <ExclamationTriangleIcon className="h-5 w-5 text-yellow-500" />;
    }
  };

  const getReviewTypeIcon = (type: string) => {
    switch (type) {
      case 'extraction':
        return <BeakerIcon className="h-5 w-5 text-purple-500" />;
      case 'library_prep':
        return <ClipboardDocumentCheckIcon className="h-5 w-5 text-blue-500" />;
      case 'sequencing':
        return <ChartBarIcon className="h-5 w-5 text-green-500" />;
      case 'data_quality':
        return <DocumentMagnifyingGlassIcon className="h-5 w-5 text-orange-500" />;
      default:
        return <BeakerIcon className="h-5 w-5 text-gray-500" />;
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-gray-900">Quality Control Workflow</h1>
        <p className="text-sm text-gray-500">Manage QC reviews, metrics, and control samples</p>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200 mb-6">
        <nav className="-mb-px flex space-x-8">
          <button
            onClick={() => setSelectedTab('reviews')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              selectedTab === 'reviews'
                ? 'border-indigo-500 text-indigo-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            QC Reviews
          </button>
          <button
            onClick={() => setSelectedTab('metrics')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              selectedTab === 'metrics'
                ? 'border-indigo-500 text-indigo-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Metrics
          </button>
          <button
            onClick={() => setSelectedTab('controls')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              selectedTab === 'controls'
                ? 'border-indigo-500 text-indigo-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Control Samples
          </button>
          <button
            onClick={() => setSelectedTab('trends')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              selectedTab === 'trends'
                ? 'border-indigo-500 text-indigo-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Trends
          </button>
        </nav>
      </div>

      {/* QC Reviews Tab */}
      {selectedTab === 'reviews' && (
        <div className="space-y-6">
          <div className="bg-white rounded-lg shadow">
            <div className="px-6 py-4 border-b border-gray-200 flex justify-between items-center">
              <h3 className="text-lg font-medium text-gray-900">Active QC Reviews</h3>
              <button className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
                New Review
              </button>
            </div>
            
            <div className="divide-y divide-gray-200">
              {reviews.map((review) => (
                <div
                  key={review.id}
                  className="px-6 py-4 hover:bg-gray-50 cursor-pointer"
                  onClick={() => setSelectedReview(review)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                      <div className="flex-shrink-0">
                        {getReviewTypeIcon(review.review_type)}
                      </div>
                      <div>
                        <p className="text-sm font-medium text-gray-900">
                          Sample {review.sample_id}
                        </p>
                        <p className="text-sm text-gray-500">
                          {review.review_type.replace('_', ' ').charAt(0).toUpperCase() + 
                           review.review_type.replace('_', ' ').slice(1)}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-4">
                      <div className="flex items-center space-x-2">
                        {getStatusIcon(review.status)}
                        <span className="text-sm text-gray-500 capitalize">
                          {review.status.replace('_', ' ')}
                        </span>
                      </div>
                      <div className="text-sm text-gray-500">
                        {new Date(review.created_at).toLocaleDateString()}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Metrics Tab */}
      {selectedTab === 'metrics' && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {metrics.map((metric) => (
            <div key={metric.id} className="bg-white rounded-lg shadow p-6">
              <div className="flex items-center justify-between mb-4">
                <h4 className="text-lg font-medium text-gray-900">{metric.name}</h4>
                {metric.critical && (
                  <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                    Critical
                  </span>
                )}
              </div>
              
              <p className="text-sm text-gray-500 mb-4">{metric.description}</p>
              
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-500">Type:</span>
                  <span className="font-medium capitalize">{metric.metric_type}</span>
                </div>
                {metric.units && (
                  <div className="flex justify-between">
                    <span className="text-gray-500">Units:</span>
                    <span className="font-medium">{metric.units}</span>
                  </div>
                )}
                {metric.min_value !== undefined && (
                  <div className="flex justify-between">
                    <span className="text-gray-500">Range:</span>
                    <span className="font-medium">
                      {metric.min_value} - {metric.max_value}
                    </span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Control Samples Tab */}
      {selectedTab === 'controls' && (
        <div className="bg-white rounded-lg shadow">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">Control Samples</h3>
          </div>
          
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Sample Name
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Type
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Last Run
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {controlSamples.map((control) => (
                  <tr key={control.id}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                      {control.name}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        control.type === 'positive' ? 'bg-green-100 text-green-800' :
                        control.type === 'negative' ? 'bg-red-100 text-red-800' :
                        'bg-gray-100 text-gray-800'
                      }`}>
                        {control.type}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {control.last_run ? new Date(control.last_run).toLocaleDateString() : 'Never'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      {control.status && (
                        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                          control.status === 'pass' ? 'bg-green-100 text-green-800' :
                          control.status === 'fail' ? 'bg-red-100 text-red-800' :
                          'bg-yellow-100 text-yellow-800'
                        }`}>
                          {control.status}
                        </span>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      <button className="text-indigo-600 hover:text-indigo-900">
                        Run Test
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Trends Tab */}
      {selectedTab === 'trends' && (
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-medium text-gray-900 mb-4">QC Metric Trends</h3>
          <p className="text-sm text-gray-500">Quality control metric trends and analysis over time</p>
          {/* Add charts here */}
          <div className="mt-8 h-64 bg-gray-50 rounded-lg flex items-center justify-center">
            <ChartBarIcon className="h-12 w-12 text-gray-400" />
          </div>
        </div>
      )}
    </div>
  );
}; 