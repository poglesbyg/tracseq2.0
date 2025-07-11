import { useQuery } from '@tanstack/react-query';
import { CheckCircleIcon, XCircleIcon, ClockIcon } from '@heroicons/react/24/outline';
import axios from '../utils/axios';

interface ServiceStatus {
  name: string;
  status: 'healthy' | 'unhealthy' | 'unknown';
  endpoint: string;
  description: string;
  features?: string[];
}

export default function ServicesStatus() {
  const services: ServiceStatus[] = [
    {
      name: 'API Gateway',
      endpoint: '/api/health',
      description: 'Main API gateway and routing service',
      features: ['Request routing', 'Load balancing', 'Health monitoring']
    },
    {
      name: 'Database',
      endpoint: '/api/health', // Will extract database status from main health check
      description: 'PostgreSQL database server',
      features: ['Data persistence', 'Transaction support', 'Query optimization']
    },
    {
      name: 'Sample Management',
      endpoint: '/api/samples', // Test actual endpoint instead of health
      description: 'Laboratory sample tracking and management',
      features: ['Sample tracking', 'Barcode generation', 'Chain of custody']
    },
    {
      name: 'Storage System',
      endpoint: '/api/storage/locations', // Test actual endpoint
      description: 'Hierarchical storage management',
      features: ['Container hierarchy', 'Position tracking', 'Temperature monitoring']
    },
    {
      name: 'Reports & Analytics',
      endpoint: '/api/reports/templates', // Test actual endpoint
      description: 'SQL reporting and analytics platform',
      features: ['Custom queries', 'Report templates', 'Data export']
    }
  ].map(service => ({ ...service, status: 'unknown' as const }));

  const { data: serviceStatuses, isLoading } = useQuery({
    queryKey: ['service-status'],
    queryFn: async () => {
      const statusPromises = services.map(async (service) => {
        try {
          const response = await axios.get(service.endpoint);
          
          // Special handling for Database service - extract from API health response
          if (service.name === 'Database' && response.data?.database) {
            return { 
              ...service, 
              status: response.data.database.healthy ? 'healthy' : 'unhealthy' 
            } as ServiceStatus;
          }
          
          // For other services, check if we got a successful response
          return { 
            ...service, 
            status: response.status === 200 ? 'healthy' : 'unhealthy' 
          } as ServiceStatus;
        } catch (error: any) {
          // Some endpoints might return 401 but still be working
          if (error?.response?.status === 401) {
            return { 
              ...service, 
              status: 'healthy' // Auth required means service is up
            } as ServiceStatus;
          }
          
          return { 
            ...service, 
            status: 'unhealthy' 
          } as ServiceStatus;
        }
      });
      
      return Promise.all(statusPromises);
    },
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'healthy':
        return <CheckCircleIcon className="h-6 w-6 text-green-500" />;
      case 'unhealthy':
        return <XCircleIcon className="h-6 w-6 text-red-500" />;
      default:
        return <ClockIcon className="h-6 w-6 text-gray-400" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy':
        return 'bg-green-100 text-green-800';
      case 'unhealthy':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  const displayServices = serviceStatuses || services;

  return (
    <div className="px-4 sm:px-6 lg:px-8 py-8">
      <div className="sm:flex sm:items-center">
        <div className="sm:flex-auto">
          <h1 className="text-2xl font-semibold text-gray-900">Core Services Status</h1>
          <p className="mt-2 text-sm text-gray-700">
            Monitor the health and availability of TracSeq 2.0 core microservices.
          </p>
        </div>
      </div>

      <div className="mt-8 grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {displayServices.map((service) => (
          <div
            key={service.name}
            className="relative bg-white rounded-lg shadow-sm border border-gray-200 p-6 hover:shadow-md transition-shadow"
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <h3 className="text-lg font-medium text-gray-900">{service.name}</h3>
                <p className="mt-1 text-sm text-gray-500">{service.description}</p>
              </div>
              <div className="ml-4">{getStatusIcon(service.status)}</div>
            </div>

            <div className="mt-4">
              <span
                className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(
                  service.status
                )}`}
              >
                {service.status === 'healthy' ? 'Operational' : 
                 service.status === 'unhealthy' ? 'Unavailable' : 'Checking...'}
              </span>
            </div>

            {service.features && service.features.length > 0 && (
              <div className="mt-4 border-t border-gray-200 pt-4">
                <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider">Features</h4>
                <ul className="mt-2 space-y-1">
                  {service.features.map((feature, index) => (
                    <li key={index} className="text-sm text-gray-600 flex items-center">
                      <span className="w-1.5 h-1.5 bg-gray-400 rounded-full mr-2"></span>
                      {feature}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            <div className="mt-4 text-xs text-gray-400">
              Endpoint: <code className="font-mono">{service.endpoint}</code>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-8 bg-blue-50 border border-blue-200 rounded-lg p-4">
        <div className="flex">
          <div className="flex-shrink-0">
            <svg className="h-5 w-5 text-blue-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
            </svg>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-blue-800">About Core Services</h3>
            <div className="mt-2 text-sm text-blue-700">
              <p>
                These core microservices provide essential functionality for the TracSeq 2.0 laboratory management system.
                Services marked as "Unavailable" may be undergoing maintenance or require additional configuration.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
} 