// Process and temporal data utilities for TracSeq 2.0 frontend

export interface ProcessStage {
  id: string;
  name: string;
  description: string;
  order: number;
  isCompleted: boolean;
  isCurrent: boolean;
  timestamp?: string;
  duration?: number; // in hours
}

export interface TimelineEvent {
  id: string;
  type: 'created' | 'validated' | 'stored' | 'sequencing_started' | 'completed' | 'failed' | 'status_change';
  title: string;
  description: string;
  timestamp: string;
  entity: {
    id: string;
    name: string;
    type: 'sample' | 'job' | 'template' | 'user';
  };
  metadata?: Record<string, unknown>;
}

export interface ProcessMetrics {
  totalSamples: number;
  byStatus: Record<string, number>;
  averageProcessingTime: {
    validation: number;
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

// Status configuration for consistent UI rendering
export const statusConfig = {
  'Pending': {
    color: 'bg-yellow-100 text-yellow-800 border-yellow-200',
    description: 'Awaiting validation',
    order: 0
  },
  'Validated': {
    color: 'bg-blue-100 text-blue-800 border-blue-200',
    description: 'Ready for storage',
    order: 1
  },
  'InStorage': {
    color: 'bg-purple-100 text-purple-800 border-purple-200',
    description: 'Stored and available',
    order: 2
  },
  'InSequencing': {
    color: 'bg-indigo-100 text-indigo-800 border-indigo-200',
    description: 'Currently sequencing',
    order: 3
  },
  'Completed': {
    color: 'bg-green-100 text-green-800 border-green-200',
    description: 'Processing complete',
    order: 4
  },
};

// Time formatting utilities
export const formatRelativeTime = (timestamp: string): string => {
  const date = new Date(timestamp);
  const now = new Date();
  const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));

  if (diffInHours < 1) return 'Just now';
  if (diffInHours < 24) return `${diffInHours}h ago`;
  const diffInDays = Math.floor(diffInHours / 24);
  if (diffInDays < 7) return `${diffInDays}d ago`;
  if (diffInDays < 30) return `${diffInDays}d ago`;
  if (diffInDays < 365) {
    const months = Math.floor(diffInDays / 30);
    return `${months}mo ago`;
  }
  const years = Math.floor(diffInDays / 365);
  return `${years}y ago`;
};

export const formatDuration = (hours: number): string => {
  if (hours < 24) return `${Math.round(hours)}h`;
  const days = Math.floor(hours / 24);
  const remainingHours = Math.round(hours % 24);
  return remainingHours > 0 ? `${days}d ${remainingHours}h` : `${days}d`;
};

export const formatTimestamp = (timestamp: string) => {
  const date = new Date(timestamp);
  return {
    date: date.toLocaleDateString(),
    time: date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
    iso: date.toISOString(),
    relative: formatRelativeTime(timestamp),
    full: date.toLocaleString(),
  };
};

// Process workflow utilities
export const getProcessStages = (currentStatus: string, timestamps: Record<string, string> = {}): ProcessStage[] => {
  const stages = [
    {
      id: 'pending',
      name: 'Sample Submitted',
      description: 'Sample received and awaiting validation',
      order: 0,
    },
    {
      id: 'validated',
      name: 'Validated',
      description: 'Sample passed validation checks',
      order: 1,
    },
    {
      id: 'instorage',
      name: 'In Storage',
      description: 'Sample stored in designated location',
      order: 2,
    },
    {
      id: 'insequencing',
      name: 'In Sequencing',
      description: 'Sample processing for sequencing',
      order: 3,
    },
    {
      id: 'completed',
      name: 'Completed',
      description: 'Sample processing finished',
      order: 4,
    },
  ];

  const statusToOrder = {
    'Pending': 0,
    'Validated': 1,
    'InStorage': 2,
    'InSequencing': 3,
    'Completed': 4,
  };

  const currentOrder = statusToOrder[currentStatus as keyof typeof statusToOrder] ?? 0;

  return stages.map(stage => ({
    ...stage,
    isCompleted: stage.order < currentOrder,
    isCurrent: stage.order === currentOrder,
    timestamp: getTimestampForStage(stage.id, timestamps),
    duration: calculateStageDuration(stage.id, stages, timestamps),
  }));
};

const getTimestampForStage = (stageId: string, timestamps: Record<string, string>): string | undefined => {
  const mapping: Record<string, string> = {
    'pending': 'created_at',
    'validated': 'validated_at',
    'instorage': 'stored_at',
    'insequencing': 'sequencing_started_at',
    'completed': 'completed_at',
  };

  return timestamps[mapping[stageId]];
};

const calculateStageDuration = (stageId: string, stages: { id: string }[], timestamps: Record<string, string>): number | undefined => {
  const stageIndex = stages.findIndex(s => s.id === stageId);
  if (stageIndex === 0) return undefined; // First stage has no previous stage

  const currentTimestamp = getTimestampForStage(stageId, timestamps);
  const previousStageId = stages[stageIndex - 1]?.id;
  const previousTimestamp = getTimestampForStage(previousStageId, timestamps);

  if (!currentTimestamp || !previousTimestamp) return undefined;

  const current = new Date(currentTimestamp);
  const previous = new Date(previousTimestamp);
  return (current.getTime() - previous.getTime()) / (1000 * 60 * 60); // hours
};

// Status utilities
export const getStatusColor = (status: string): string => {
  return statusConfig[status as keyof typeof statusConfig]?.color || 'bg-gray-100 text-gray-800 border-gray-200';
};

export const getStatusDescription = (status: string): string => {
  return statusConfig[status as keyof typeof statusConfig]?.description || 'Unknown status';
};

export const isStatusValid = (status: string): boolean => {
  return status in statusConfig;
};

export const getNextValidStatuses = (currentStatus: string): string[] => {
  const validTransitions: Record<string, string[]> = {
    'Pending': ['Validated'],
    'Validated': ['InStorage'],
    'InStorage': ['InSequencing'],
    'InSequencing': ['Completed'],
    'Completed': [],
  };

  return validTransitions[currentStatus] || [];
};

// Timeline utilities
export const groupEventsByDate = (events: TimelineEvent[]): Record<string, TimelineEvent[]> => {
  const groups: Record<string, TimelineEvent[]> = {};

  events.forEach(event => {
    const date = new Date(event.timestamp).toDateString();
    if (!groups[date]) groups[date] = [];
    groups[date].push(event);
  });

  return groups;
};

export const filterEventsByTimeRange = (events: TimelineEvent[], timeRange: string): TimelineEvent[] => {
  const now = new Date();
  let cutoffTime: Date;

  switch (timeRange) {
    case '1h':
      cutoffTime = new Date(now.getTime() - 1 * 60 * 60 * 1000);
      break;
    case '6h':
      cutoffTime = new Date(now.getTime() - 6 * 60 * 60 * 1000);
      break;
    case '24h':
      cutoffTime = new Date(now.getTime() - 24 * 60 * 60 * 1000);
      break;
    case '7d':
      cutoffTime = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
      break;
    case '30d':
      cutoffTime = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000);
      break;
    default:
      return events; // 'all' or unknown
  }

  return events.filter(event => new Date(event.timestamp) >= cutoffTime);
};

// Analytics utilities
export const calculateProcessingMetrics = (samples: { status: string; created_at: string }[]): ProcessMetrics => {
  const byStatus = samples.reduce((acc, sample) => {
    acc[sample.status] = (acc[sample.status] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  // Calculate average processing times (mock data - would be calculated from actual timestamps)
  const averageProcessingTime = {
    validation: calculateAverageStageTime(samples, 'validation'),
    storage: calculateAverageStageTime(samples, 'storage'),
    sequencing: calculateAverageStageTime(samples, 'sequencing'),
    overall: calculateAverageStageTime(samples, 'overall'),
  };

  // Calculate recent throughput
  const now = new Date();
  const last24h = samples.filter(sample => {
    const created = new Date(sample.created_at);
    return (now.getTime() - created.getTime()) <= 24 * 60 * 60 * 1000;
  }).length;

  const last7d = samples.filter(sample => {
    const created = new Date(sample.created_at);
    return (now.getTime() - created.getTime()) <= 7 * 24 * 60 * 60 * 1000;
  }).length;

  const last30d = samples.filter(sample => {
    const created = new Date(sample.created_at);
    return (now.getTime() - created.getTime()) <= 30 * 24 * 60 * 60 * 1000;
  }).length;

  // Identify bottlenecks (simplified logic)
  const bottlenecks = identifyBottlenecks(byStatus);

  return {
    totalSamples: samples.length,
    byStatus,
    averageProcessingTime,
    recentThroughput: {
      last24h,
      last7d,
      last30d,
    },
    bottlenecks,
  };
};

const calculateAverageStageTime = (_samples: any[], stage: string): number => {
  // Mock implementation - would calculate from actual timestamp data
  const mockTimes: Record<string, number> = {
    validation: 4, // 4 hours
    storage: 2,    // 2 hours
    sequencing: 48, // 48 hours
    overall: 72,   // 72 hours total
  };

  return mockTimes[stage] || 0;
};

const identifyBottlenecks = (byStatus: Record<string, number>): ProcessMetrics['bottlenecks'] => {
  const bottlenecks: ProcessMetrics['bottlenecks'] = [];

  // Simple heuristic: stages with high counts might be bottlenecks
  Object.entries(byStatus).forEach(([status, count]) => {
    if (count > 10 && status !== 'Completed') { // Arbitrary threshold
      bottlenecks.push({
        stage: status,
        count,
        avgWaitTime: Math.random() * 48 + 12, // Mock wait time
      });
    }
  });

  return bottlenecks.sort((a, b) => b.avgWaitTime - a.avgWaitTime);
};

// Export utility functions for common use cases
export const processUtils = {
  formatRelativeTime,
  formatDuration,
  formatTimestamp,
  getProcessStages,
  getStatusColor,
  getStatusDescription,
  isStatusValid,
  getNextValidStatuses,
  groupEventsByDate,
  filterEventsByTimeRange,
  calculateProcessingMetrics,
};
