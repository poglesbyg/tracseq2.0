import React from 'react';
import { CheckCircleIcon, ClockIcon, BeakerIcon, CircleStackIcon, CogIcon } from '@heroicons/react/24/outline';
import { CheckCircleIcon as CheckCircleIconSolid } from '@heroicons/react/24/solid';

interface ProcessStep {
  id: string;
  name: string;
  description: string;
  icon: React.ComponentType<React.SVGProps<SVGSVGElement>>;
  status: 'completed' | 'current' | 'upcoming';
  timestamp?: string;
  duration?: string;
}

interface ProcessFlowProps {
  currentStatus: string;
  timestamps?: {
    created_at?: string;
    validated_at?: string;
    stored_at?: string;
    sequencing_started_at?: string;
    completed_at?: string;
  };
  className?: string;
}

const statusToStepMap: Record<string, number> = {
  'Pending': 0,
  'Validated': 1,
  'InStorage': 2,
  'InSequencing': 3,
  'Completed': 4,
};

export default function ProcessFlow({ currentStatus, timestamps, className = '' }: ProcessFlowProps) {
  const currentStepIndex = statusToStepMap[currentStatus] ?? 0;

  const steps: ProcessStep[] = [
    {
      id: 'pending',
      name: 'Sample Submitted',
      description: 'Sample received and awaiting validation',
      icon: ClockIcon,
      status: currentStepIndex >= 0 ? 'completed' : 'upcoming',
      timestamp: timestamps?.created_at,
    },
    {
      id: 'validated',
      name: 'Validated',
      description: 'Sample passed validation checks',
      icon: CheckCircleIcon,
      status: currentStepIndex >= 1 ? 'completed' : currentStepIndex === 0 ? 'current' : 'upcoming',
      timestamp: timestamps?.validated_at,
    },
    {
      id: 'instorage',
      name: 'In Storage',
      description: 'Sample stored in designated location',
      icon: CircleStackIcon,
      status: currentStepIndex >= 2 ? 'completed' : currentStepIndex === 1 ? 'current' : 'upcoming',
      timestamp: timestamps?.stored_at,
    },
    {
      id: 'insequencing',
      name: 'In Sequencing',
      description: 'Sample processing for sequencing',
      icon: CogIcon,
      status: currentStepIndex >= 3 ? 'completed' : currentStepIndex === 2 ? 'current' : 'upcoming',
      timestamp: timestamps?.sequencing_started_at,
    },
    {
      id: 'completed',
      name: 'Completed',
      description: 'Sample processing finished',
      icon: BeakerIcon,
      status: currentStepIndex >= 4 ? 'completed' : currentStepIndex === 3 ? 'current' : 'upcoming',
      timestamp: timestamps?.completed_at,
    },
  ];

  const getStepStyles = (status: string) => {
    switch (status) {
      case 'completed':
        return {
          container: 'bg-green-50 border-green-200',
          icon: 'bg-green-500 text-white',
          title: 'text-green-800',
          description: 'text-green-600',
          connector: 'bg-green-500',
        };
      case 'current':
        return {
          container: 'bg-blue-50 border-blue-200',
          icon: 'bg-blue-500 text-white',
          title: 'text-blue-800',
          description: 'text-blue-600',
          connector: 'bg-gray-300',
        };
      default:
        return {
          container: 'bg-gray-50 border-gray-200',
          icon: 'bg-gray-300 text-gray-500',
          title: 'text-gray-500',
          description: 'text-gray-400',
          connector: 'bg-gray-300',
        };
    }
  };

  const formatTimestamp = (timestamp?: string) => {
    if (!timestamp) return null;
    const date = new Date(timestamp);
    return {
      date: date.toLocaleDateString(),
      time: date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      relative: getRelativeTime(date),
    };
  };

  const getRelativeTime = (date: Date) => {
    const now = new Date();
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 1) return 'Just now';
    if (diffInHours < 24) return `${diffInHours}h ago`;
    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays < 7) return `${diffInDays}d ago`;
    return date.toLocaleDateString();
  };

  const calculateDuration = (start?: string, end?: string) => {
    if (!start || !end) return null;
    const startDate = new Date(start);
    const endDate = new Date(end);
    const diffInHours = Math.floor((endDate.getTime() - startDate.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 24) return `${diffInHours}h`;
    const diffInDays = Math.floor(diffInHours / 24);
    return `${diffInDays}d ${diffInHours % 24}h`;
  };

  return (
    <div className={`process-flow ${className}`}>
      <div className="relative">
        {steps.map((step, index) => {
          const styles = getStepStyles(step.status);
          const timestamp = formatTimestamp(step.timestamp);
          const isLast = index === steps.length - 1;
          const IconComponent = step.icon;
          
          return (
            <div key={step.id} className="relative">
              {/* Connector Line */}
              {!isLast && (
                <div
                  className={`absolute left-6 top-12 w-0.5 h-16 ${styles.connector}`}
                  aria-hidden="true"
                />
              )}
              
              {/* Step Container */}
              <div className={`relative flex items-start space-x-4 pb-8 last:pb-0`}>
                {/* Icon */}
                <div className={`flex-shrink-0 w-12 h-12 rounded-full flex items-center justify-center ${styles.icon} relative z-10`}>
                  {step.status === 'completed' ? (
                    <CheckCircleIconSolid className="w-6 h-6" />
                  ) : (
                    <IconComponent className="w-6 h-6" />
                  )}
                </div>
                
                {/* Content */}
                <div className="flex-1 min-w-0">
                  <div className={`p-4 rounded-lg border ${styles.container}`}>
                    <div className="flex items-center justify-between">
                      <h3 className={`text-sm font-semibold ${styles.title}`}>
                        {step.name}
                      </h3>
                      {timestamp && (
                        <div className="text-right">
                          <div className={`text-xs font-medium ${styles.title}`}>
                            {timestamp.relative}
                          </div>
                          <div className={`text-xs ${styles.description}`}>
                            {timestamp.date} {timestamp.time}
                          </div>
                        </div>
                      )}
                    </div>
                    <p className={`text-sm mt-1 ${styles.description}`}>
                      {step.description}
                    </p>
                    
                    {/* Duration */}
                    {index > 0 && step.timestamp && steps[index - 1].timestamp && (
                      <div className={`text-xs mt-2 ${styles.description}`}>
                        Duration: {calculateDuration(steps[index - 1].timestamp, step.timestamp)}
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
