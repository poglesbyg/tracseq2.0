import React, { useState, useEffect } from 'react';
import {
  ServerIcon,
  ExclamationTriangleIcon,
  ArrowPathIcon,
  ClockIcon,
  AdjustmentsHorizontalIcon
} from '@heroicons/react/24/outline';
import axios from '../../utils/axios';

interface EventStats {
  events_published: number;
  events_consumed: number;
  events_failed: number;
  handlers_registered: number;
}

interface CircuitBreakerStatus {
  [service: string]: {
    state: 'closed' | 'open' | 'half_open';
    failure_count: number;
    last_failure?: string;
    next_retry?: string;
  };
}

interface EventStream {
  id: string;
  event_type: string;
  source_service: string;
  timestamp: string;
  priority: number;
  status: 'processed' | 'failed' | 'pending';
}

export const EventMonitorDashboard: React.FC = () => {
  const [stats, setStats] = useState<EventStats>({
    events_published: 0,
    events_consumed: 0,
    events_failed: 0,
    handlers_registered: 0
  });
  const [circuitBreakers, setCircuitBreakers] = useState<CircuitBreakerStatus>({});
  const [recentEvents, setRecentEvents] = useState<EventStream[]>([]);
  const [selectedTab, setSelectedTab] = useState<'overview' | 'events' | 'circuit-breakers'>('overview');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        // Fetch event statistics
        const statsResponse = await axios.get('/api/events/stats');
        setStats(statsResponse.data);

        // Fetch circuit breaker status
        const circuitResponse = await axios.get('/api/gateway/circuit-breakers');
        setCircuitBreakers(circuitResponse.data);

        // Fetch recent events
        const eventsResponse = await axios.get('/api/events/recent');
        setRecentEvents(eventsResponse.data.events || []);
      } catch (error) {
        console.error('Failed to fetch event data:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
    // Refresh data every 10 seconds
    const interval = setInterval(fetchData, 10000);
    return () => clearInterval(interval);
  }, []);

  const getCircuitBreakerColor = (state: string) => {
    switch (state) {
      case 'closed':
        return 'text-green-600 bg-green-100';
      case 'open':
        return 'text-red-600 bg-red-100';
      case 'half_open':
        return 'text-yellow-600 bg-yellow-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  };

  const formatNumber = (num: number) => {
    return new Intl.NumberFormat().format(num);
  };

  const getEventRate = () => {
    const total = stats.events_published + stats.events_consumed;
    if (total === 0) return 0;
    return ((stats.events_consumed / stats.events_published) * 100).toFixed(1);
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
        <h1 className="text-2xl font-bold text-gray-900">Event System Monitor</h1>
        <p className="text-sm text-gray-500">Real-time event processing and circuit breaker status</p>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200 mb-6">
        <nav className="-mb-px flex space-x-8">
          <button
            onClick={() => setSelectedTab('overview')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              selectedTab === 'overview'
                ? 'border-indigo-500 text-indigo-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Overview
          </button>
          <button
            onClick={() => setSelectedTab('events')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              selectedTab === 'events'
                ? 'border-indigo-500 text-indigo-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Event Stream
          </button>
          <button
            onClick={() => setSelectedTab('circuit-breakers')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              selectedTab === 'circuit-breakers'
                ? 'border-indigo-500 text-indigo-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            Circuit Breakers
          </button>
        </nav>
      </div>

      {/* Overview Tab */}
      {selectedTab === 'overview' && (
        <div>
          {/* Stats Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <div className="bg-white rounded-lg shadow p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Events Published</p>
                  <p className="text-2xl font-semibold text-gray-900">{formatNumber(stats.events_published)}</p>
                </div>
                <div className="p-3 bg-blue-100 rounded-full">
                  <ArrowPathIcon className="h-6 w-6 text-blue-600" />
                </div>
              </div>
            </div>

            <div className="bg-white rounded-lg shadow p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Events Consumed</p>
                  <p className="text-2xl font-semibold text-gray-900">{formatNumber(stats.events_consumed)}</p>
                </div>
                <div className="p-3 bg-green-100 rounded-full">
                  <ServerIcon className="h-6 w-6 text-green-600" />
                </div>
              </div>
            </div>

            <div className="bg-white rounded-lg shadow p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Failed Events</p>
                  <p className="text-2xl font-semibold text-gray-900">{formatNumber(stats.events_failed)}</p>
                </div>
                <div className="p-3 bg-red-100 rounded-full">
                  <ExclamationTriangleIcon className="h-6 w-6 text-red-600" />
                </div>
              </div>
            </div>

            <div className="bg-white rounded-lg shadow p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-gray-500">Active Handlers</p>
                  <p className="text-2xl font-semibold text-gray-900">{stats.handlers_registered}</p>
                </div>
                <div className="p-3 bg-purple-100 rounded-full">
                  <AdjustmentsHorizontalIcon className="h-6 w-6 text-purple-600" />
                </div>
              </div>
            </div>
          </div>

          {/* Performance Metrics */}
          <div className="bg-white rounded-lg shadow p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-4">System Performance</h3>
            <div className="space-y-4">
              <div>
                <div className="flex justify-between text-sm mb-1">
                  <span className="text-gray-600">Processing Rate</span>
                  <span className="font-medium">{getEventRate()}%</span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div
                    className="bg-indigo-600 h-2 rounded-full"
                    style={{ width: `${getEventRate()}%` }}
                  ></div>
                </div>
              </div>

              <div>
                <div className="flex justify-between text-sm mb-1">
                  <span className="text-gray-600">Error Rate</span>
                  <span className="font-medium">
                    {stats.events_published > 0
                      ? ((stats.events_failed / stats.events_published) * 100).toFixed(1)
                      : 0}%
                  </span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div
                    className="bg-red-600 h-2 rounded-full"
                    style={{
                      width: `${
                        stats.events_published > 0
                          ? Math.min(((stats.events_failed / stats.events_published) * 100), 100)
                          : 0
                      }%`
                    }}
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Event Stream Tab */}
      {selectedTab === 'events' && (
        <div className="bg-white rounded-lg shadow">
          <div className="px-6 py-4 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">Recent Events</h3>
          </div>
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Event Type
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Source
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Priority
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Timestamp
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {recentEvents.map((event) => (
                  <tr key={event.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                      {event.event_type}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {event.source_service}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        event.priority <= 2 ? 'bg-red-100 text-red-800' :
                        event.priority === 3 ? 'bg-yellow-100 text-yellow-800' :
                        'bg-green-100 text-green-800'
                      }`}>
                        P{event.priority}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        event.status === 'processed' ? 'bg-green-100 text-green-800' :
                        event.status === 'failed' ? 'bg-red-100 text-red-800' :
                        'bg-gray-100 text-gray-800'
                      }`}>
                        {event.status}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      <div className="flex items-center">
                        <ClockIcon className="h-4 w-4 mr-1" />
                        {new Date(event.timestamp).toLocaleString()}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Circuit Breakers Tab */}
      {selectedTab === 'circuit-breakers' && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {Object.entries(circuitBreakers).map(([service, status]) => (
            <div key={service} className="bg-white rounded-lg shadow p-6">
              <div className="flex items-center justify-between mb-4">
                <h4 className="text-lg font-medium text-gray-900 capitalize">{service}</h4>
                <span className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${
                  getCircuitBreakerColor(status.state)
                }`}>
                  {status.state.replace('_', ' ').toUpperCase()}
                </span>
              </div>
              
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-500">Failure Count:</span>
                  <span className="font-medium">{status.failure_count}</span>
                </div>
                {status.last_failure && (
                  <div className="flex justify-between">
                    <span className="text-gray-500">Last Failure:</span>
                    <span className="font-medium">
                      {new Date(status.last_failure).toLocaleTimeString()}
                    </span>
                  </div>
                )}
                {status.next_retry && status.state === 'open' && (
                  <div className="flex justify-between">
                    <span className="text-gray-500">Next Retry:</span>
                    <span className="font-medium">
                      {new Date(status.next_retry).toLocaleTimeString()}
                    </span>
                  </div>
                )}
              </div>

              {status.state === 'open' && (
                <div className="mt-4 p-3 bg-red-50 rounded-md">
                  <p className="text-xs text-red-800">
                    Service is temporarily unavailable. Circuit will attempt to close at next retry.
                  </p>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};