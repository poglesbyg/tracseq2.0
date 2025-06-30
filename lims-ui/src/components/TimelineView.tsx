import { CalendarIcon, ClockIcon, FunnelIcon } from '@heroicons/react/24/outline';
import { useState, useMemo } from 'react';

interface TimelineEvent {
  id: string;
  title: string;
  description: string;
  timestamp: string;
  type: string;
  entity: {
    id: string;
    name: string;
    type: 'sample' | 'job' | 'template' | 'user';
  };
  metadata?: Record<string, unknown>;
}

interface TimelineViewProps {
  events: TimelineEvent[];
  title?: string;
  className?: string;
  showFilters?: boolean;
  onEventClick?: (event: TimelineEvent) => void;
}

const eventTypeColors = {
  created: 'bg-blue-100 text-blue-800 border-blue-200',
  validated: 'bg-green-100 text-green-800 border-green-200',
  stored: 'bg-purple-100 text-purple-800 border-purple-200',
  sequencing_started: 'bg-indigo-100 text-indigo-800 border-indigo-200',
  completed: 'bg-emerald-100 text-emerald-800 border-emerald-200',
  failed: 'bg-red-100 text-red-800 border-red-200',
  status_change: 'bg-yellow-100 text-yellow-800 border-yellow-200',
};

const timeRanges = [
  { label: 'Last Hour', value: 1 },
  { label: 'Last 6 Hours', value: 6 },
  { label: 'Last 24 Hours', value: 24 },
  { label: 'Last 7 Days', value: 168 },
  { label: 'Last 30 Days', value: 720 },
  { label: 'All Time', value: Infinity },
];

export default function TimelineView({ events, title = 'Timeline', className = '', showFilters = true }: Omit<TimelineViewProps, 'onEventClick'>) {
  const [selectedTimeRange, setSelectedTimeRange] = useState(24); // Default to last 24 hours
  const [selectedEventTypes, setSelectedEventTypes] = useState<string[]>([]);
  const [selectedEntityTypes, setSelectedEntityTypes] = useState<string[]>([]);

  const filteredEvents = useMemo(() => {
    const now = new Date();
    const cutoffTime = selectedTimeRange === Infinity 
      ? new Date(0) 
      : new Date(now.getTime() - selectedTimeRange * 60 * 60 * 1000);

    return events.filter(event => {
      const eventTime = new Date(event.timestamp);
      
      // Time filter
      if (eventTime < cutoffTime) return false;
      
      // Event type filter
      if (selectedEventTypes.length > 0 && !selectedEventTypes.includes(event.type)) return false;
      
      // Entity type filter
      if (selectedEntityTypes.length > 0 && !selectedEntityTypes.includes(event.entity.type)) return false;
      
      return true;
    }).sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
  }, [events, selectedTimeRange, selectedEventTypes, selectedEntityTypes]);

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));
    
    const relative = (() => {
      if (diffInHours < 1) return 'Just now';
      if (diffInHours < 24) return `${diffInHours}h ago`;
      const diffInDays = Math.floor(diffInHours / 24);
      if (diffInDays < 7) return `${diffInDays}d ago`;
      return date.toLocaleDateString();
    })();

    return {
      relative,
      absolute: date.toLocaleString(),
      date: date.toLocaleDateString(),
      time: date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
    };
  };

  const getEventIcon = () => {
    return ClockIcon; // Could be expanded with more specific icons
  };

  const groupEventsByDate = (events: TimelineEvent[]) => {
    const groups: Record<string, TimelineEvent[]> = {};
    
    events.forEach(event => {
      const date = new Date(event.timestamp).toDateString();
      if (!groups[date]) groups[date] = [];
      groups[date].push(event);
    });
    
    return groups;
  };

  const eventGroups = groupEventsByDate(filteredEvents);
  const availableEventTypes = Array.from(new Set(events.map(e => e.type)));
  const availableEntityTypes = Array.from(new Set(events.map(e => e.entity.type)));

  return (
    <div className={`timeline-view ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-3">
          <CalendarIcon className="h-6 w-6 text-gray-400" />
          <h2 className="text-lg font-semibold text-gray-900">{title}</h2>
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
            {filteredEvents.length} events
          </span>
        </div>
      </div>

      {/* Filters */}
      {showFilters && (
        <div className="bg-gray-50 rounded-lg p-4 mb-6">
          <div className="flex items-center space-x-2 mb-3">
            <FunnelIcon className="h-4 w-4 text-gray-500" />
            <span className="text-sm font-medium text-gray-700">Filters</span>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {/* Time Range Filter */}
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">Time Range</label>
              <select
                value={selectedTimeRange}
                onChange={(e) => setSelectedTimeRange(Number(e.target.value))}
                className="w-full text-sm border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
              >
                {timeRanges.map(range => (
                  <option key={range.value} value={range.value}>
                    {range.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Event Type Filter */}
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">Event Types</label>
              <select
                multiple
                value={selectedEventTypes}
                onChange={(e) => setSelectedEventTypes(Array.from(e.target.selectedOptions, option => option.value))}
                className="w-full text-sm border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
                size={Math.min(availableEventTypes.length, 4)}
              >
                {availableEventTypes.map(type => (
                  <option key={type} value={type}>
                    {type.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                  </option>
                ))}
              </select>
            </div>

            {/* Entity Type Filter */}
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">Entity Types</label>
              <select
                multiple
                value={selectedEntityTypes}
                onChange={(e) => setSelectedEntityTypes(Array.from(e.target.selectedOptions, option => option.value))}
                className="w-full text-sm border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
                size={Math.min(availableEntityTypes.length, 4)}
              >
                {availableEntityTypes.map(type => (
                  <option key={type} value={type}>
                    {type.charAt(0).toUpperCase() + type.slice(1)}s
                  </option>
                ))}
              </select>
            </div>
          </div>
          
          {(selectedEventTypes.length > 0 || selectedEntityTypes.length > 0) && (
            <div className="mt-3 pt-3 border-t border-gray-200">
              <button
                onClick={() => {
                  setSelectedEventTypes([]);
                  setSelectedEntityTypes([]);
                }}
                className="text-xs text-blue-600 hover:text-blue-500"
              >
                Clear Filters
              </button>
            </div>
          )}
        </div>
      )}

      {/* Timeline */}
      <div className="timeline-container">
        {Object.keys(eventGroups).length === 0 ? (
          <div className="text-center py-12">
            <ClockIcon className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-4 text-sm font-medium text-gray-900">No events found</h3>
            <p className="mt-2 text-sm text-gray-500">
              Try adjusting your filters to see more events.
            </p>
          </div>
        ) : (
          Object.entries(eventGroups).map(([date, dayEvents]) => (
            <div key={date} className="timeline-day mb-8">
              {/* Date Header */}
              <div className="flex items-center mb-4">
                <div className="flex-shrink-0 w-24 text-right">
                  <span className="text-sm font-medium text-gray-900">
                    {new Date(date).toLocaleDateString([], { 
                      month: 'short', 
                      day: 'numeric' 
                    })}
                  </span>
                </div>
                <div className="flex-1 ml-4 border-t border-gray-200"></div>
              </div>

              {/* Events for this day */}
              <div className="relative">
                {dayEvents.map((event, index) => {
                  const timestamp = formatTimestamp(event.timestamp);
                  const Icon = getEventIcon();
                  const isLast = index === dayEvents.length - 1;
                  
                  return (
                    <div key={event.id} className="relative">
                      {/* Connector line */}
                      {!isLast && (
                        <div className="absolute left-[7.5rem] top-12 w-px h-16 bg-gray-200" />
                      )}
                      
                      <div className="flex items-start space-x-4 pb-8">
                        {/* Time */}
                        <div className="flex-shrink-0 w-24 text-right">
                          <span className="text-xs text-gray-500">
                            {timestamp.time}
                          </span>
                        </div>
                        
                        {/* Icon */}
                        <div className="flex-shrink-0 w-6 h-6 bg-white border-2 border-gray-300 rounded-full flex items-center justify-center relative z-10">
                          <Icon className="w-3 h-3 text-gray-500" />
                        </div>
                        
                        {/* Content */}
                        <div className="flex-1 min-w-0">
                          <div className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
                            <div className="flex items-start justify-between">
                              <div className="flex-1">
                                <div className="flex items-center space-x-2 mb-1">
                                  <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium border ${eventTypeColors[event.type as keyof typeof eventTypeColors] || eventTypeColors.status_change}`}>
                                    {event.type.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                                  </span>
                                  <span className="text-xs text-gray-500">
                                    {event.entity.type}: {event.entity.name}
                                  </span>
                                </div>
                                <h4 className="text-sm font-medium text-gray-900 mb-1">
                                  {event.title}
                                </h4>
                                <p className="text-sm text-gray-600">
                                  {event.description}
                                </p>
                              </div>
                              <div className="text-right text-xs text-gray-500 ml-4">
                                <div title={timestamp.absolute}>
                                  {timestamp.relative}
                                </div>
                              </div>
                            </div>
                            
                            {/* Metadata */}
                            {event.metadata && Object.keys(event.metadata).length > 0 && (
                              <div className="mt-3 pt-3 border-t border-gray-100">
                                <div className="grid grid-cols-2 gap-2 text-xs">
                                  {Object.entries(event.metadata).slice(0, 4).map(([key, value]) => (
                                    <div key={key}>
                                      <span className="text-gray-500">{key}:</span>
                                      <span className="ml-1 text-gray-900">{String(value)}</span>
                                    </div>
                                  ))}
                                </div>
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
          ))
        )}
      </div>
    </div>
  );
}
