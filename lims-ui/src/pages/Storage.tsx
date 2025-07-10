import { useState } from 'react';
import StorageManagement from '../components/StorageManagement';
import HierarchicalStorageNavigator from '../components/HierarchicalStorageNavigator';
import { 
  MapPinIcon, 
  CubeIcon,
  ChartBarIcon 
} from '@heroicons/react/24/outline';

export default function Storage() {
  const [activeTab, setActiveTab] = useState<'hierarchical' | 'management' | 'analytics'>('hierarchical');

  const tabs = [
    { 
      id: 'hierarchical', 
      name: 'Hierarchical Storage', 
      icon: CubeIcon,
      description: 'Navigate through freezers, racks, boxes, and positions'
    },
    { 
      id: 'management', 
      name: 'Storage Management', 
      icon: MapPinIcon,
      description: 'Manage storage locations and sample assignments'
    },
    { 
      id: 'analytics', 
      name: 'Analytics', 
      icon: ChartBarIcon,
      description: 'Storage utilization and capacity analytics'
    },
  ];

  const renderTabContent = () => {
    switch (activeTab) {
      case 'hierarchical':
        return <HierarchicalStorageNavigator />;
      case 'management':
        return <StorageManagement />;
      case 'analytics':
        return (
          <div className="bg-white shadow rounded-lg p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-4">Storage Analytics</h3>
            <p className="text-gray-500">
              Storage analytics dashboard will be implemented here, showing utilization trends, 
              capacity forecasting, and performance metrics.
            </p>
          </div>
        );
      default:
        return <HierarchicalStorageNavigator />;
    }
  };

  return (
    <div className="px-4 sm:px-6 lg:px-8">
      <div className="space-y-6">
        <div className="sm:flex sm:items-center">
          <div className="sm:flex-auto">
            <h1 className="text-2xl font-semibold text-gray-900">Storage System</h1>
            <p className="mt-2 text-sm text-gray-700">
              Comprehensive storage management for laboratory samples and materials.
            </p>
          </div>
        </div>

        {/* Tab Navigation */}
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex space-x-8">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as 'hierarchical' | 'management' | 'analytics')}
                className={`${
                  activeTab === tab.id
                    ? 'border-indigo-500 text-indigo-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                } whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm flex items-center group`}
              >
                <tab.icon className="h-5 w-5 mr-2" />
                <div className="text-left">
                  <div>{tab.name}</div>
                  <div className="text-xs text-gray-400 group-hover:text-gray-500">
                    {tab.description}
                  </div>
                </div>
              </button>
            ))}
          </nav>
        </div>

        {/* Tab Content */}
        <div className="mt-6">
          {renderTabContent()}
        </div>
      </div>
    </div>
  );
} 
