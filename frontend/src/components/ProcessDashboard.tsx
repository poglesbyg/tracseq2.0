import { 
  BeakerIcon, 
  ClockIcon, 
  CheckCircleIcon, 
  ExclamationTriangleIcon,
  ChartBarIcon,
  ArrowRightIcon,
  CalendarIcon,
  ArrowTrendingUpIcon
} from '@heroicons/react/24/outline';
import { useMemo } from 'react';

interface ProcessMetrics {
  totalSamples: number;
  byStatus: Record<string, number>;
  averageProcessingTime: {
    validation: number; // hours
    storage: number;
    sequencing: number;
    overall: number;
  };
  recentThroughput: {
    last24h: number;
    last7d: number;
    last30d: number;
  };
  bottlenecks: {
    stage: string;
    count: number;
    avgWaitTime: number;
  }[];
}

interface ProcessDashboardProps {
  metrics: ProcessMetrics;
  className?: string;
}

const statusConfig = {
  'Pending': { 
    color: 'bg-yellow-100 text-yellow-800 border-yellow-200', 
    icon: ClockIcon,
    description: 'Awaiting validation'
  },
  'Validated': { 
    color: 'bg-blue-100 text-blue-800 border-blue-200', 
    icon: CheckCircleIcon,
    description: 'Ready for storage'
  },
  'InStorage': { 
    color: 'bg-purple-100 text-purple-800 border-purple-200', 
    icon: BeakerIcon,
    description: 'Stored and available'
  },
  'InSequencing': { 
    color: 'bg-indigo-100 text-indigo-800 border-indigo-200', 
    icon: ChartBarIcon,
    description: 'Currently sequencing'
  },
  'Completed': { 
    color: 'bg-green-100 text-green-800 border-green-200', 
    icon: CheckCircleIcon,
    description: 'Processing complete'
  },
};

export default function ProcessDashboard({ metrics, className = '' }: ProcessDashboardProps) {
  
  const processFlow = useMemo(() => {
    const flow = [
      { stage: 'Pending', count: metrics.byStatus.Pending || 0 },
      { stage: 'Validated', count: metrics.byStatus.Validated || 0 },
      { stage: 'InStorage', count: metrics.byStatus.InStorage || 0 },
      { stage: 'InSequencing', count: metrics.byStatus.InSequencing || 0 },
      { stage: 'Completed', count: metrics.byStatus.Completed || 0 },
    ];
    return flow;
  }, [metrics.byStatus]);

  const totalActive = metrics.totalSamples - (metrics.byStatus.Completed || 0);
  
  const formatTime = (hours: number) => {
    if (hours < 24) return `${Math.round(hours)}h`;
    const days = Math.floor(hours / 24);
    const remainingHours = Math.round(hours % 24);
    return remainingHours > 0 ? `${days}d ${remainingHours}h` : `${days}d`;
  };

  const getBottleneckSeverity = (waitTime: number) => {
    if (waitTime > 72) return 'high'; // > 3 days
    if (waitTime > 24) return 'medium'; // > 1 day
    return 'low';
  };

  const severityColors = {
    high: 'bg-red-100 text-red-800 border-red-200',
    medium: 'bg-yellow-100 text-yellow-800 border-yellow-200',
    low: 'bg-green-100 text-green-800 border-green-200',
  };

  return (
    <div className={`process-dashboard ${className}`}>
      {/* Header */}
      <div className="mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-2">Process Dashboard</h2>
        <p className="text-sm text-gray-600">
          Real-time view of laboratory sample processing workflow
        </p>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
        <div className="bg-white p-4 rounded-lg border border-gray-200 shadow-sm">
          <div className="flex items-center">
            <BeakerIcon className="h-8 w-8 text-blue-600" />
            <div className="ml-3">
              <p className="text-sm font-medium text-gray-500">Total Samples</p>
              <p className="text-2xl font-semibold text-gray-900">{metrics.totalSamples}</p>
            </div>
          </div>
        </div>

        <div className="bg-white p-4 rounded-lg border border-gray-200 shadow-sm">
          <div className="flex items-center">
            <ClockIcon className="h-8 w-8 text-yellow-600" />
            <div className="ml-3">
              <p className="text-sm font-medium text-gray-500">Active</p>
              <p className="text-2xl font-semibold text-gray-900">{totalActive}</p>
            </div>
          </div>
        </div>

        <div className="bg-white p-4 rounded-lg border border-gray-200 shadow-sm">
          <div className="flex items-center">
            <ArrowTrendingUpIcon className="h-8 w-8 text-green-600" />
            <div className="ml-3">
              <p className="text-sm font-medium text-gray-500">Avg. Processing</p>
              <p className="text-2xl font-semibold text-gray-900">
                {formatTime(metrics.averageProcessingTime.overall)}
              </p>
            </div>
          </div>
        </div>

        <div className="bg-white p-4 rounded-lg border border-gray-200 shadow-sm">
          <div className="flex items-center">
            <CalendarIcon className="h-8 w-8 text-indigo-600" />
            <div className="ml-3">
              <p className="text-sm font-medium text-gray-500">Last 24h</p>
              <p className="text-2xl font-semibold text-gray-900">{metrics.recentThroughput.last24h}</p>
            </div>
          </div>
        </div>
      </div>

      {/* Process Flow Visualization */}
      <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6 mb-8">
        <h3 className="text-lg font-medium text-gray-900 mb-4">Sample Processing Flow</h3>
        
        <div className="flex items-center justify-between">
          {processFlow.map((stage, index) => {
            const config = statusConfig[stage.stage as keyof typeof statusConfig];
            const Icon = config.icon;
            const isLast = index === processFlow.length - 1;
            
            return (
              <div key={stage.stage} className="flex items-center">
                {/* Stage */}
                <div className="text-center">
                  <div className={`w-16 h-16 rounded-full border-2 flex items-center justify-center ${config.color} mb-2`}>
                    <Icon className="w-6 h-6" />
                  </div>
                  <div className="text-sm font-medium text-gray-900">{stage.stage}</div>
                  <div className="text-xs text-gray-500 mb-1">{config.description}</div>
                  <div className="text-lg font-semibold text-gray-900">{stage.count}</div>
                  <div className="text-xs text-gray-500">samples</div>
                </div>
                
                {/* Arrow */}
                {!isLast && (
                  <ArrowRightIcon className="w-6 h-6 text-gray-400 mx-4" />
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Processing Times */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
        <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
          <h3 className="text-lg font-medium text-gray-900 mb-4">Average Processing Times</h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-600">Validation</span>
              <span className="text-sm font-medium text-gray-900">
                {formatTime(metrics.averageProcessingTime.validation)}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-600">Storage</span>
              <span className="text-sm font-medium text-gray-900">
                {formatTime(metrics.averageProcessingTime.storage)}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-600">Sequencing</span>
              <span className="text-sm font-medium text-gray-900">
                {formatTime(metrics.averageProcessingTime.sequencing)}
              </span>
            </div>
            <div className="border-t pt-3 mt-3">
              <div className="flex justify-between items-center">
                <span className="text-sm font-medium text-gray-900">Overall</span>
                <span className="text-sm font-semibold text-gray-900">
                  {formatTime(metrics.averageProcessingTime.overall)}
                </span>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
          <h3 className="text-lg font-medium text-gray-900 mb-4">Recent Throughput</h3>
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-600">Last 24 hours</span>
              <span className="text-sm font-medium text-gray-900">
                {metrics.recentThroughput.last24h} samples
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-600">Last 7 days</span>
              <span className="text-sm font-medium text-gray-900">
                {metrics.recentThroughput.last7d} samples
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-600">Last 30 days</span>
              <span className="text-sm font-medium text-gray-900">
                {metrics.recentThroughput.last30d} samples
              </span>
            </div>
            <div className="border-t pt-3 mt-3">
              <div className="flex justify-between items-center">
                <span className="text-sm font-medium text-gray-900">Daily Average</span>
                <span className="text-sm font-semibold text-gray-900">
                  {Math.round(metrics.recentThroughput.last30d / 30)} samples/day
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Bottlenecks */}
      {metrics.bottlenecks && metrics.bottlenecks.length > 0 && (
        <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
          <div className="flex items-center mb-4">
            <ExclamationTriangleIcon className="h-5 w-5 text-yellow-500 mr-2" />
            <h3 className="text-lg font-medium text-gray-900">Identified Bottlenecks</h3>
          </div>
          
          <div className="space-y-3">
            {metrics.bottlenecks.map((bottleneck, index) => {
              const severity = getBottleneckSeverity(bottleneck.avgWaitTime);
              return (
                <div key={index} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                  <div className="flex-1">
                    <div className="flex items-center space-x-3">
                      <span className="text-sm font-medium text-gray-900">
                        {bottleneck.stage}
                      </span>
                      <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium border ${severityColors[severity]}`}>
                        {severity} impact
                      </span>
                    </div>
                    <div className="text-xs text-gray-500 mt-1">
                      {bottleneck.count} samples affected
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-sm font-medium text-gray-900">
                      {formatTime(bottleneck.avgWaitTime)}
                    </div>
                    <div className="text-xs text-gray-500">avg wait time</div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
