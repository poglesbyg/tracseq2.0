import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import axios from 'axios';
import { 
  ExclamationTriangleIcon, 
  ArrowTrendingUpIcon, 
  ClockIcon,
  DocumentIcon,
  BeakerIcon,
  QueueListIcon,
  CheckCircleIcon,
  SparklesIcon,
  DocumentArrowUpIcon,
  EyeIcon
} from '@heroicons/react/24/outline';

interface DashboardStats {
  totalTemplates: number;
  totalSamples: number;
  pendingSequencing: number;
  completedSequencing: number;
}

interface RecentActivity {
  id: string;
  type: string;
  description: string;
  timestamp: string;
  status?: string;
}

interface Template {
  id: string;
  name: string;
  description: string;
  created_at: string;
  metadata: any;
}

interface Sample {
  id: string;
  name: string;
  barcode: string;
  location: string;
  status: string;
  created_at: string;
  metadata: any;
}

interface SequencingJob {
  id: string;
  name: string;
  status: string;
  sample_sheet_path: string;
  created_at: string;
  metadata: any;
}

export default function Dashboard() {
  const { data: stats, isLoading: statsLoading, error: statsError } = useQuery<DashboardStats>({
    queryKey: ['dashboardStats'],
    queryFn: async () => {
      const response = await axios.get('/api/dashboard/stats');
      return response.data;
    },
    retry: 3,
    staleTime: 30000, // 30 seconds
  });

  // Fetch recent templates for activity feed
  const { data: recentTemplates } = useQuery<Template[]>({
    queryKey: ['recentTemplates'],
    queryFn: async () => {
      const response = await axios.get('/api/templates');
      return response.data.slice(0, 3); // Get latest 3
    }
  });

  // Fetch recent samples for activity feed
  const { data: recentSamples } = useQuery<Sample[]>({
    queryKey: ['recentSamples'],
    queryFn: async () => {
      const response = await axios.get('/api/samples');
      return response.data.slice(0, 3); // Get latest 3
    }
  });

  // Fetch recent sequencing jobs for activity feed  
  const { data: recentJobs } = useQuery<SequencingJob[]>({
    queryKey: ['recentJobs'],
    queryFn: async () => {
      const response = await axios.get('/api/sequencing/jobs');
      return response.data.slice(0, 3); // Get latest 3
    }
  });

  // Combine recent activities
  const recentActivity: RecentActivity[] = [
    ...(recentTemplates?.map(template => ({
      id: template.id,
      type: 'template',
      description: `Template "${template.name}" was uploaded`,
      timestamp: template.created_at,
    })) || []),
    ...(recentSamples?.map(sample => ({
      id: sample.id,
      type: 'sample',
      description: `Sample "${sample.name}" was created with barcode ${sample.barcode}`,
      timestamp: sample.created_at,
      status: sample.status,
    })) || []),
    ...(recentJobs?.map(job => ({
      id: job.id,
      type: 'sequencing',
      description: `Sequencing job "${job.name}" was created`,
      timestamp: job.created_at,
      status: job.status,
    })) || []),
  ].sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()).slice(0, 5);

  const formatTimeAgo = (timestamp: string) => {
    const now = new Date();
    const time = new Date(timestamp);
    const diffInHours = Math.floor((now.getTime() - time.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 1) return 'Just now';
    if (diffInHours < 24) return `${diffInHours}h ago`;
    return `${Math.floor(diffInHours / 24)}d ago`;
  };

  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'template': return DocumentIcon;
      case 'sample': return BeakerIcon;
      case 'sequencing': return QueueListIcon;
      default: return ClockIcon;
    }
  };

  const getStatusColor = (status?: string) => {
    switch (status) {
      case 'completed': return 'text-green-600 bg-green-50';
      case 'pending': return 'text-yellow-600 bg-yellow-50';
      case 'validated': return 'text-blue-600 bg-blue-50';
      case 'in_progress': return 'text-indigo-600 bg-indigo-50';
      case 'failed': return 'text-red-600 bg-red-50';
      default: return 'text-gray-600 bg-gray-50';
    }
  };

  if (statsLoading) {
    return (
      <div className="px-4 sm:px-6 lg:px-8 py-8">
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
          <span className="ml-3 text-gray-600 font-medium">Loading dashboard...</span>
        </div>
      </div>
    );
  }

  if (statsError) {
    return (
      <div className="px-4 sm:px-6 lg:px-8 py-8">
        <div className="rounded-lg bg-red-50 p-6 border border-red-200">
          <div className="flex">
            <div className="flex-shrink-0">
              <ExclamationTriangleIcon className="h-5 w-5 text-red-400" aria-hidden="true" />
            </div>
            <div className="ml-3">
              <h3 className="text-sm font-medium text-red-800">
                Error loading dashboard
              </h3>
              <div className="mt-2 text-sm text-red-700">
                <p>Unable to connect to the server. Please check if the backend service is running.</p>
                <p className="mt-1 text-xs font-mono bg-red-100 p-2 rounded">Error: {statsError.message}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="px-4 sm:px-6 lg:px-8 py-8">
      {/* Header */}
      <div className="mb-8">
        <div className="md:flex md:items-center md:justify-between">
          <div className="flex-1 min-w-0">
            <h1 className="text-3xl font-bold leading-tight text-gray-900">Dashboard</h1>
            <p className="mt-2 text-sm text-gray-600">
              Welcome to Lab Manager. Here's an overview of your lab's current status.
            </p>
          </div>
          <div className="mt-4 flex md:mt-0 md:ml-4">
            <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
              System Online
            </span>
          </div>
        </div>
      </div>
      
      {/* Stats Grid */}
      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4 mb-8">
        {/* Templates Card */}
        <div className="dashboard-card group">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <DocumentIcon className="h-8 w-8 text-blue-600" />
            </div>
            <div className="ml-5 w-0 flex-1">
              <dl>
                <dt className="dashboard-stat-label">Total Templates</dt>
                <dd className="dashboard-stat-number">
                  {stats?.totalTemplates ?? 0}
                </dd>
              </dl>
            </div>
          </div>
          <div className="mt-4 flex items-center text-sm text-gray-500">
            <ArrowTrendingUpIcon className="h-4 w-4 text-green-500 mr-1" />
            <span>Active templates in system</span>
          </div>
        </div>

        {/* Samples Card */}
        <div className="dashboard-card group">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <BeakerIcon className="h-8 w-8 text-green-600" />
            </div>
            <div className="ml-5 w-0 flex-1">
              <dl>
                <dt className="dashboard-stat-label">Total Samples</dt>
                <dd className="dashboard-stat-number">
                  {stats?.totalSamples ?? 0}
                </dd>
              </dl>
            </div>
          </div>
          <div className="mt-4 flex items-center text-sm text-gray-500">
            <ArrowTrendingUpIcon className="h-4 w-4 text-green-500 mr-1" />
            <span>Samples in storage</span>
          </div>
        </div>

        {/* Pending Sequencing Card */}
        <div className="dashboard-card group">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <QueueListIcon className="h-8 w-8 text-yellow-600" />
            </div>
            <div className="ml-5 w-0 flex-1">
              <dl>
                <dt className="dashboard-stat-label">Pending Sequencing</dt>
                <dd className="dashboard-stat-number">
                  {stats?.pendingSequencing ?? 0}
                </dd>
              </dl>
            </div>
          </div>
          <div className="mt-4 flex items-center text-sm text-gray-500">
            {stats?.pendingSequencing && stats.pendingSequencing > 0 ? (
              <>
                <ClockIcon className="h-4 w-4 text-yellow-500 mr-1" />
                <span>Awaiting processing</span>
              </>
            ) : (
              <>
                <CheckCircleIcon className="h-4 w-4 text-green-500 mr-1" />
                <span>All caught up!</span>
              </>
            )}
          </div>
        </div>

        {/* Completed Sequencing Card */}
        <div className="dashboard-card group">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <CheckCircleIcon className="h-8 w-8 text-indigo-600" />
            </div>
            <div className="ml-5 w-0 flex-1">
              <dl>
                <dt className="dashboard-stat-label">Completed Sequencing</dt>
                <dd className="dashboard-stat-number">
                  {stats?.completedSequencing ?? 0}
                </dd>
              </dl>
            </div>
          </div>
          <div className="mt-4 flex items-center text-sm text-gray-500">
            <CheckCircleIcon className="h-4 w-4 text-green-500 mr-1" />
            <span>Successfully processed</span>
          </div>
        </div>
      </div>

      {/* AI-Powered Document Submissions Section */}
      <div className="mb-8">
        <div className="bg-gradient-to-br from-indigo-50 to-purple-50 rounded-lg border border-indigo-200 p-6">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="h-10 w-10 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-lg flex items-center justify-center">
                  <SparklesIcon className="h-6 w-6 text-white" />
                </div>
              </div>
              <div className="ml-4">
                <h2 className="text-lg font-semibold text-gray-900">AI-Powered Document Submissions</h2>
                <p className="text-sm text-gray-600">Upload laboratory documents to automatically extract sample data using AI</p>
              </div>
            </div>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {/* Quick Upload */}
            <Link
              to="/rag-submissions"
              className="group bg-white rounded-lg p-4 border border-gray-200 hover:border-indigo-300 hover:shadow-md transition-all duration-200"
            >
              <div className="flex items-center">
                <DocumentArrowUpIcon className="h-8 w-8 text-indigo-600 group-hover:text-indigo-700" />
                <div className="ml-3">
                  <h3 className="text-sm font-medium text-gray-900 group-hover:text-indigo-700">Upload Document</h3>
                  <p className="text-xs text-gray-500">PDF, DOCX, TXT</p>
                </div>
              </div>
            </Link>

            {/* Preview Mode */}
            <Link
              to="/rag-submissions?mode=preview"
              className="group bg-white rounded-lg p-4 border border-gray-200 hover:border-indigo-300 hover:shadow-md transition-all duration-200"
            >
              <div className="flex items-center">
                <EyeIcon className="h-8 w-8 text-purple-600 group-hover:text-purple-700" />
                <div className="ml-3">
                  <h3 className="text-sm font-medium text-gray-900 group-hover:text-purple-700">Preview Extract</h3>
                  <p className="text-xs text-gray-500">Review before creating</p>
                </div>
              </div>
            </Link>

            {/* Features */}
            <div className="bg-white rounded-lg p-4 border border-gray-200">
              <div className="space-y-2">
                <div className="flex items-center text-xs text-gray-600">
                  <CheckCircleIcon className="h-4 w-4 text-green-500 mr-2" />
                  AI-powered extraction
                </div>
                <div className="flex items-center text-xs text-gray-600">
                  <CheckCircleIcon className="h-4 w-4 text-green-500 mr-2" />
                  Confidence scoring
                </div>
                <div className="flex items-center text-xs text-gray-600">
                  <CheckCircleIcon className="h-4 w-4 text-green-500 mr-2" />
                  Natural language queries
                </div>
              </div>
            </div>
          </div>
          
          <div className="mt-4 text-center">
            <Link
              to="/rag-submissions"
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-700 hover:to-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-all duration-200"
            >
              <SparklesIcon className="h-4 w-4 mr-2" />
              Open AI Document Processor
            </Link>
          </div>
        </div>
      </div>

      {/* Recent Activity Section */}
      <div className="dashboard-card">
        <div className="px-6 py-5 border-b border-gray-200">
          <h2 className="text-lg font-medium text-gray-900 flex items-center">
            <ClockIcon className="h-5 w-5 text-gray-400 mr-2" />
            Recent Activity
          </h2>
          <p className="mt-1 text-sm text-gray-500">Latest updates from your lab operations</p>
        </div>
        <div className="px-6 py-5">
          {recentActivity.length > 0 ? (
            <div className="flow-root">
              <ul className="-mb-8">
                {recentActivity.map((activity, activityIdx) => {
                  const Icon = getActivityIcon(activity.type);
                  return (
                    <li key={activity.id}>
                      <div className="relative pb-8">
                        {activityIdx !== recentActivity.length - 1 ? (
                          <span
                            className="absolute top-5 left-5 -ml-px h-full w-0.5 bg-gray-200"
                            aria-hidden="true"
                          />
                        ) : null}
                        <div className="relative flex items-start space-x-3">
                          <div>
                            <span className={`activity-icon ${
                              activity.type === 'template' ? 'bg-blue-500' :
                              activity.type === 'sample' ? 'bg-green-500' :
                              'bg-indigo-500'
                            }`}>
                              <Icon className="h-4 w-4 text-white" aria-hidden="true" />
                            </span>
                          </div>
                          <div className="min-w-0 flex-1">
                            <div>
                              <div className="text-sm">
                                <p className="font-medium text-gray-900">
                                  {activity.description}
                                </p>
                              </div>
                              <div className="mt-2 flex items-center space-x-2">
                                <p className="text-sm text-gray-500">
                                  <time dateTime={activity.timestamp}>
                                    {formatTimeAgo(activity.timestamp)}
                                  </time>
                                </p>
                                {activity.status && (
                                  <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(activity.status)}`}>
                                    {activity.status}
                                  </span>
                                )}
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>
                    </li>
                  );
                })}
              </ul>
            </div>
          ) : (
            <div className="text-center py-12">
              <ClockIcon className="mx-auto h-12 w-12 text-gray-400" />
              <h3 className="mt-4 text-sm font-medium text-gray-900">No recent activity</h3>
              <p className="mt-2 text-sm text-gray-500">
                Start by uploading templates or creating samples to see activity here.
              </p>
              <div className="mt-6">
                <button
                  type="button"
                  className="btn-primary"
                >
                  Get Started
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
} 
