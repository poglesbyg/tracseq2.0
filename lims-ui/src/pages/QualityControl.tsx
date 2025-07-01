import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '../utils/axios';
import {
  ChartBarIcon,
  CheckCircleIcon,
  XCircleIcon,
  ExclamationTriangleIcon,
  DocumentCheckIcon,
  ArrowTrendingUpIcon,
  ArrowTrendingDownIcon,
  ArrowPathIcon,
  ClipboardDocumentCheckIcon,
} from '@heroicons/react/24/outline';
import { format } from 'date-fns';

interface QCMetric {
  id: string;
  name: string;
  value: number;
  unit?: string;
  status: 'pass' | 'warning' | 'fail';
  threshold_low?: number;
  threshold_high?: number;
  trend?: 'up' | 'down' | 'stable';
}

interface QCReview {
  id: string;
  entity_type: 'library_prep' | 'sequencing_run' | 'sample';
  entity_id: string;
  entity_name?: string;
  review_status: 'pending' | 'in_progress' | 'completed';
  decision?: 'approved' | 'rejected' | 'conditional' | 'repeat_required';
  reviewer_name?: string;
  created_at: string;
  comments?: string;
}

interface QCDashboardStats {
  total_pending_reviews: number;
  pass_rate_last_30_days: number;
  active_control_samples: number;
  failing_metrics_count: number;
}

interface QcControlSample {
  id: string;
  name: string;
  sample_type: string;
  lot_number: string;
  expiry_date: string;
  target_values: Record<string, number>;
  is_active: boolean;
}

export default function QualityControl() {
  const [selectedTab, setSelectedTab] = useState<'dashboard' | 'reviews' | 'metrics' | 'controls'>('dashboard');
  const [selectedReview, setSelectedReview] = useState<QCReview | null>(null);
  const queryClient = useQueryClient();

  // Fetch QC dashboard stats
  const { data: dashboardStats } = useQuery<QCDashboardStats>({
    queryKey: ['qc-dashboard-stats'],
    queryFn: async () => {
      const response = await api.get('/api/qc/dashboard/stats');
      return response.data;
    },
  });

  // Fetch pending QC reviews
  const { data: pendingReviews, isLoading: isLoadingReviews } = useQuery<QCReview[]>({
    queryKey: ['qc-reviews', 'pending'],
    queryFn: async () => {
      const response = await api.get('/api/qc/reviews', { params: { status: 'pending' } });
      return response.data;
    },
  });

  // Fetch recent QC metrics
  const { data: recentMetrics } = useQuery<QCMetric[]>({
    queryKey: ['qc-recent-metrics'],
    queryFn: async () => {
      const response = await api.get('/api/qc/metrics/recent');
      return response.data;
    },
  });

  // Fetch QC metrics
  const { data: metrics, isLoading: isLoadingMetrics } = useQuery<QCMetric[]>({
    queryKey: ['qc-metrics'],
    queryFn: async () => {
      const response = await api.get('/api/qc/metrics');
      return response.data;
    },
  });

  // Fetch control samples
  const { data: controlSamples } = useQuery<QcControlSample[]>({
    queryKey: ['qc-control-samples'],
    queryFn: async () => {
      const response = await api.get('/api/qc/control-samples');
      return response.data;
    },
    enabled: selectedTab === 'controls',
  });

  // Submit QC review decision
  const submitReviewMutation = useMutation({
    mutationFn: async (data: { reviewId: string; decision: string; comments?: string }) => {
      const response = await api.post(`/api/qc/reviews/${data.reviewId}/decision`, {
        decision: data.decision,
        comments: data.comments,
      });
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['qc-pending-reviews'] });
      setSelectedReview(null);
    },
  });

  const getStatusIcon = (status: 'pass' | 'warning' | 'fail') => {
    switch (status) {
      case 'pass':
        return <CheckCircleIcon className="h-5 w-5 text-green-500" />;
      case 'warning':
        return <ExclamationTriangleIcon className="h-5 w-5 text-yellow-500" />;
      case 'fail':
        return <XCircleIcon className="h-5 w-5 text-red-500" />;
    }
  };

  const getTrendIcon = (trend?: 'up' | 'down' | 'stable') => {
    switch (trend) {
      case 'up':
        return <ArrowTrendingUpIcon className="h-4 w-4 text-green-500" />;
      case 'down':
        return <ArrowTrendingDownIcon className="h-4 w-4 text-red-500" />;
      default:
        return null;
    }
  };

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      {/* Header */}
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-xl font-semibold text-gray-900">Quality Control</h1>
          <p className="mt-2 text-sm text-gray-700">
            Monitor QC metrics, review results, and manage control samples
          </p>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="mt-6 grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ClipboardDocumentCheckIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">Pending Reviews</dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {dashboardStats?.total_pending_reviews || 0}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ChartBarIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">Pass Rate (30d)</dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {dashboardStats?.pass_rate_last_30_days || 0}%
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <DocumentCheckIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">Active Controls</dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {dashboardStats?.active_control_samples || 0}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ExclamationTriangleIcon className="h-6 w-6 text-yellow-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 truncate">Failing Metrics</dt>
                  <dd className="text-lg font-medium text-gray-900">
                    {dashboardStats?.failing_metrics_count || 0}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="mt-8">
        <div className="sm:hidden">
          <select
            value={selectedTab}
            onChange={(e) => setSelectedTab(e.target.value as 'dashboard' | 'reviews' | 'metrics' | 'controls')}
            className="block w-full rounded-md border-gray-300 focus:border-indigo-500 focus:ring-indigo-500"
          >
            <option value="dashboard">Dashboard</option>
            <option value="reviews">Reviews</option>
            <option value="metrics">Metrics</option>
            <option value="controls">Controls</option>
          </select>
        </div>
        <div className="hidden sm:block">
          <nav className="flex space-x-8" aria-label="Tabs">
            {['dashboard', 'reviews', 'metrics', 'controls'].map((tab) => (
              <button
                key={tab}
                onClick={() => setSelectedTab(tab as 'dashboard' | 'reviews' | 'metrics' | 'controls')}
                className={`${
                  selectedTab === tab
                    ? 'border-indigo-500 text-indigo-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                } whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm capitalize`}
              >
                {tab}
              </button>
            ))}
          </nav>
        </div>
      </div>

      {/* Tab Content */}
      <div className="mt-8">
        {selectedTab === 'dashboard' && (
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
            {/* Recent Metrics */}
            <div className="bg-white shadow sm:rounded-lg">
              <div className="px-4 py-5 sm:p-6">
                <h3 className="text-lg leading-6 font-medium text-gray-900">Recent Metrics</h3>
                <div className="mt-4 space-y-4">
                  {recentMetrics?.map((metric) => (
                    <div key={metric.id} className="flex items-center justify-between">
                      <div className="flex items-center space-x-3">
                        {getStatusIcon(metric.status)}
                        <div>
                          <p className="text-sm font-medium text-gray-900">{metric.name}</p>
                          <p className="text-sm text-gray-500">
                            {metric.value} {metric.unit}
                          </p>
                        </div>
                      </div>
                      <div className="flex items-center space-x-2">
                        {getTrendIcon(metric.trend)}
                        {metric.threshold_low && metric.threshold_high && (
                          <span className="text-xs text-gray-500">
                            ({metric.threshold_low} - {metric.threshold_high})
                          </span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Trend Chart Placeholder */}
            <div className="bg-white shadow sm:rounded-lg">
              <div className="px-4 py-5 sm:p-6">
                <h3 className="text-lg leading-6 font-medium text-gray-900">QC Trends</h3>
                <div className="mt-4 h-64 flex items-center justify-center border-2 border-dashed border-gray-300 rounded-lg">
                  <p className="text-gray-500">Trend chart visualization</p>
                </div>
              </div>
            </div>
          </div>
        )}

        {selectedTab === 'reviews' && (
          <div className="bg-white shadow overflow-hidden sm:rounded-md">
            {isLoadingReviews ? (
              <div className="flex justify-center py-8">
                <ArrowPathIcon className="h-8 w-8 animate-spin text-indigo-600" />
              </div>
            ) : (
              <ul className="divide-y divide-gray-200">
                {pendingReviews?.map((review) => (
                  <li key={review.id}>
                    <div className="px-4 py-4 sm:px-6 hover:bg-gray-50 cursor-pointer"
                         onClick={() => setSelectedReview(review)}>
                      <div className="flex items-center justify-between">
                        <div>
                          <p className="text-sm font-medium text-gray-900">
                            {review.entity_type.replace('_', ' ')} - {review.entity_name || review.entity_id}
                          </p>
                          <p className="text-sm text-gray-500">
                            Created {format(new Date(review.created_at), 'MMM dd, yyyy HH:mm')}
                          </p>
                        </div>
                        <div className="flex items-center">
                          <span className="inline-flex rounded-full bg-yellow-100 px-2 text-xs font-semibold leading-5 text-yellow-800">
                            {review.review_status}
                          </span>
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {selectedTab === 'metrics' && (
          <div className="bg-white shadow overflow-hidden sm:rounded-md">
            {isLoadingMetrics ? (
              <div className="flex justify-center py-8">
                <ArrowPathIcon className="h-8 w-8 animate-spin text-indigo-600" />
              </div>
            ) : (
              <ul className="divide-y divide-gray-200">
                {metrics?.map((metric) => (
                  <li key={metric.id}>
                    <div className="px-4 py-4 sm:px-6">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-3">
                          {getStatusIcon(metric.status)}
                          <div>
                            <p className="text-sm font-medium text-gray-900">{metric.name}</p>
                            <p className="text-sm text-gray-500">
                              {metric.value} {metric.unit}
                            </p>
                          </div>
                        </div>
                        <div className="flex items-center space-x-2">
                          {getTrendIcon(metric.trend)}
                          {metric.threshold_low && metric.threshold_high && (
                            <span className="text-xs text-gray-500">
                              Range: {metric.threshold_low} - {metric.threshold_high}
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {selectedTab === 'controls' && (
          <div className="bg-white shadow overflow-hidden sm:rounded-md">
            <ul className="divide-y divide-gray-200">
              {controlSamples?.map((control) => (
                <li key={control.id}>
                  <div className="px-4 py-4 sm:px-6">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm font-medium text-gray-900">{control.name}</p>
                        <p className="text-sm text-gray-500">
                          Type: {control.sample_type} • Lot: {control.lot_number}
                        </p>
                        <p className="text-sm text-gray-500">
                          Expires: {format(new Date(control.expiry_date), 'MMM dd, yyyy')}
                        </p>
                      </div>
                      <div className="flex items-center">
                        <span className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${
                          control.is_active ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                        }`}>
                          {control.is_active ? 'Active' : 'Inactive'}
                        </span>
                      </div>
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {/* Review Decision Modal */}
      {selectedReview && (
        <div className="fixed inset-0 z-10 overflow-y-auto">
          <div className="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
            <div className="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" 
                 onClick={() => setSelectedReview(null)} />
            <div className="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
              <div className="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
                <h3 className="text-lg leading-6 font-medium text-gray-900">
                  QC Review Decision
                </h3>
                <div className="mt-4">
                  <p className="text-sm text-gray-500">
                    {selectedReview.entity_type} - {selectedReview.entity_name}
                  </p>
                  <div className="mt-4 space-x-2">
                    <button
                      onClick={() => submitReviewMutation.mutate({ 
                        reviewId: selectedReview.id, 
                        decision: 'approved' 
                      })}
                      className="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                    >
                      Approve
                    </button>
                    <button
                      onClick={() => submitReviewMutation.mutate({ 
                        reviewId: selectedReview.id, 
                        decision: 'rejected' 
                      })}
                      className="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
                    >
                      Reject
                    </button>
                    <button
                      onClick={() => submitReviewMutation.mutate({ 
                        reviewId: selectedReview.id, 
                        decision: 'conditional' 
                      })}
                      className="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-yellow-600 hover:bg-yellow-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-yellow-500"
                    >
                      Conditional
                    </button>
                  </div>
                </div>
              </div>
              <div className="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
                <button
                  type="button"
                  className="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
                  onClick={() => setSelectedReview(null)}
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}