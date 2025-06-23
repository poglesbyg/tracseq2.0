import React from 'react';

interface Step {
  id: string;
  title: string;
  description?: string;
}

interface ProcessFlowProps {
  steps: Step[];
  currentStepIndex: number;
  className?: string;
}

type StepStatus = 'pending' | 'current' | 'completed';

const ProcessFlow: React.FC<ProcessFlowProps> = ({
  steps,
  currentStepIndex,
  className = ''
}) => {
  const getStepStatus = (stepIndex: number): StepStatus => {
    if (currentStepIndex > stepIndex) {
      return 'completed';
    } else if (currentStepIndex === stepIndex) {
      return 'current';
    } else {
      return 'pending';
    }
  };

  const getStepClasses = (status: StepStatus): string => {
    const baseClasses = 'flex items-center justify-center w-8 h-8 rounded-full border-2 text-sm font-medium';
    
    switch (status) {
      case 'completed':
        return `${baseClasses} bg-green-500 border-green-500 text-white`;
      case 'current':
        return `${baseClasses} bg-blue-500 border-blue-500 text-white`;
      case 'pending':
        return `${baseClasses} bg-gray-100 border-gray-300 text-gray-500`;
      default:
        return baseClasses;
    }
  };

  const getConnectorClasses = (stepIndex: number): string => {
    const baseClasses = 'flex-1 h-0.5 mx-2';
    
    if (stepIndex < steps.length - 1) {
      // Connector is completed if both current step and next step are completed
      if (currentStepIndex > stepIndex) {
        return `${baseClasses} bg-green-500`;
      } else {
        return `${baseClasses} bg-gray-300`;
      }
    }
    
    return '';
  };

  const getStepIcon = (status: StepStatus, stepIndex: number): React.ReactNode => {
    switch (status) {
      case 'completed':
        return (
          <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
          </svg>
        );
      case 'current':
        return (
          <div className="w-2 h-2 bg-white rounded-full animate-pulse" />
        );
      case 'pending':
        return stepIndex + 1;
      default:
        return stepIndex + 1;
    }
  };

  return (
    <div className={`process-flow ${className}`}>
      <div className="flex items-center justify-between w-full">
        {steps.map((step, index) => {
          const status = getStepStatus(index);
          
          return (
            <React.Fragment key={step.id}>
              <div className="flex flex-col items-center">
                <div className={getStepClasses(status)}>
                  {getStepIcon(status, index)}
                </div>
                <div className="mt-2 text-center">
                  <div className={`text-sm font-medium ${
                    status === 'current' ? 'text-blue-600' : 
                    status === 'completed' ? 'text-green-600' : 'text-gray-500'
                  }`}>
                    {step.title}
                  </div>
                  {step.description && (
                    <div className="text-xs text-gray-400 mt-1">
                      {step.description}
                    </div>
                  )}
                </div>
              </div>
              
              {index < steps.length - 1 && (
                <div className={getConnectorClasses(index)} />
              )}
            </React.Fragment>
          );
        })}
      </div>
    </div>
  );
};

export default ProcessFlow;