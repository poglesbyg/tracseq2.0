import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import { 
  ExclamationTriangleIcon, 
  ArrowTrendingUpIcon, 
  ClockIcon,
  DocumentIcon,
  BeakerIcon,
  QueueListIcon,
  CheckCircleIcon
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
      case 'completed': return 'text-green-600';
      case 'pending': return 'text-yellow-600';
      case 'validated': return 'text-blue-600';
      case 'in_progress': return 'text-indigo-600';
      case 'failed': return 'text-red-600';
      default: return 'text-gray-600';
    }
  };

  if (statsLoading) {
    return (
      <div className="px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
          <span className="ml-3 text-gray-600">Loading dashboard...</span>
        </div>
      </div>
    );
  }

  if (statsError) {
    return (
      <div className="px-4 sm:px-6 lg:px-8">
        <div className="rounded-md bg-red-50 p-4">
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
                <p className="mt-1 text-xs">Error: {statsError.message}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-semibold text-gray-900">Dashboard</h1>
        <p className="mt-2 text-sm text-gray-700">
          Welcome to Lab Manager. Here's an overview of your lab's current status.
        </p>
      </div>
      
      {/* Stats Grid */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4 mb-8">
        {/* Templates Card */}
        <div className="relative overflow-hidden rounded-lg bg-white px-4 py-5 shadow hover:shadow-md transition-shadow sm:p-6">
          <dt className="truncate text-sm font-medium text-gray-500">Total Templates</dt>
          <dd className="mt-1 flex items-baseline">
            <div className="text-3xl font-semibold tracking-tight text-gray-900">
              {stats?.totalTemplates ?? 0}
            </div>
            <div className="ml-2 flex items-baseline text-sm font-semibold text-green-600">
              <ArrowTrendingUpIcon className="h-4 w-4 flex-shrink-0 self-center" />
            </div>
          </dd>
          <div className="absolute right-4 top-4">
            <DocumentIcon className="h-8 w-8 text-gray-300" />
          </div>
        </div>

        {/* Samples Card */}
        <div className="relative overflow-hidden rounded-lg bg-white px-4 py-5 shadow hover:shadow-md transition-shadow sm:p-6">
          <dt className="truncate text-sm font-medium text-gray-500">Total Samples</dt>
          <dd className="mt-1 flex items-baseline">
            <div className="text-3xl font-semibold tracking-tight text-gray-900">
              {stats?.totalSamples ?? 0}
            </div>
            <div className="ml-2 flex items-baseline text-sm font-semibold text-blue-600">
              <ArrowTrendingUpIcon className="h-4 w-4 flex-shrink-0 self-center" />
            </div>
          </dd>
          <div className="absolute right-4 top-4">
            <BeakerIcon className="h-8 w-8 text-gray-300" />
          </div>
        </div>

        {/* Pending Sequencing Card */}
        <div className="relative overflow-hidden rounded-lg bg-white px-4 py-5 shadow hover:shadow-md transition-shadow sm:p-6">
          <dt className="truncate text-sm font-medium text-gray-500">Pending Sequencing</dt>
          <dd className="mt-1 flex items-baseline">
            <div className="text-3xl font-semibold tracking-tight text-gray-900">
              {stats?.pendingSequencing ?? 0}
            </div>
            {stats?.pendingSequencing && stats.pendingSequencing > 0 && (
              <div className="ml-2 flex items-baseline text-sm font-semibold text-yellow-600">
                <ClockIcon className="h-4 w-4 flex-shrink-0 self-center" />
              </div>
            )}
          </dd>
          <div className="absolute right-4 top-4">
            <QueueListIcon className="h-8 w-8 text-gray-300" />
          </div>
        </div>

        {/* Completed Sequencing Card */}
        <div className="relative overflow-hidden rounded-lg bg-white px-4 py-5 shadow hover:shadow-md transition-shadow sm:p-6">
          <dt className="truncate text-sm font-medium text-gray-500">Completed Sequencing</dt>
          <dd className="mt-1 flex items-baseline">
            <div className="text-3xl font-semibold tracking-tight text-gray-900">
              {stats?.completedSequencing ?? 0}
            </div>
            {stats?.completedSequencing && stats.completedSequencing > 0 && (
              <div className="ml-2 flex items-baseline text-sm font-semibold text-green-600">
                <CheckCircleIcon className="h-4 w-4 flex-shrink-0 self-center" />
              </div>
            )}
          </dd>
          <div className="absolute right-4 top-4">
            <CheckCircleIcon className="h-8 w-8 text-gray-300" />
          </div>
        </div>
      </div>

      {/* Recent Activity Section */}
      <div className="bg-white shadow rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <h2 className="text-lg font-medium text-gray-900 mb-4">Recent Activity</h2>
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
                            className="absolute top-4 left-4 -ml-px h-full w-0.5 bg-gray-200"
                            aria-hidden="true"
                          />
                        ) : null}
                        <div className="relative flex space-x-3">
                          <div>
                            <span className={`h-8 w-8 rounded-full flex items-center justify-center ring-8 ring-white ${
                              activity.type === 'template' ? 'bg-blue-500' :
                              activity.type === 'sample' ? 'bg-green-500' :
                              'bg-indigo-500'
                            }`}>
                              <Icon className="h-4 w-4 text-white" aria-hidden="true" />
                            </span>
                          </div>
                          <div className="min-w-0 flex-1 pt-1.5 flex justify-between space-x-4">
                            <div>
                              <p className="text-sm text-gray-900">
                                {activity.description}
                                {activity.status && (
                                  <span className={`ml-2 text-xs font-medium ${getStatusColor(activity.status)}`}>
                                    ({activity.status})
                                  </span>
                                )}
                              </p>
                            </div>
                            <div className="text-right text-sm whitespace-nowrap text-gray-500">
                              <time dateTime={activity.timestamp}>
                                {formatTimeAgo(activity.timestamp)}
                              </time>
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
            <div className="text-center py-6">
              <ClockIcon className="mx-auto h-12 w-12 text-gray-400" />
              <h3 className="mt-2 text-sm font-medium text-gray-900">No recent activity</h3>
              <p className="mt-1 text-sm text-gray-500">
                Start by uploading templates or creating samples to see activity here.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
} 
