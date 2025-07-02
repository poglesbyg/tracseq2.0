import React, { useState, useEffect } from 'react';
import {
  BellIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  InformationCircleIcon,
  XCircleIcon,
  EnvelopeIcon,
  ChatBubbleBottomCenterTextIcon,
  DevicePhoneMobileIcon
} from '@heroicons/react/24/outline';
import { BellIcon as BellSolidIcon } from '@heroicons/react/24/solid';
import axios from '../../utils/axios';

interface Notification {
  id: string;
  title: string;
  message: string;
  priority: 'low' | 'normal' | 'high' | 'critical';
  status: 'pending' | 'sent' | 'failed' | 'read';
  channels: string[];
  created_at: string;
  sent_at?: string;
  read_at?: string;
  metadata?: Record<string, any>;
}

interface NotificationTemplate {
  id: string;
  name: string;
  description: string;
  channel: string;
  subject_template: string;
  body_template: string;
  variables: string[];
}

export const NotificationCenter: React.FC = () => {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [templates, setTemplates] = useState<NotificationTemplate[]>([]);
  const [activeTab, setActiveTab] = useState<'inbox' | 'templates' | 'settings'>('inbox');
  const [filter, setFilter] = useState<'all' | 'unread' | 'critical'>('all');
  const [loading, setLoading] = useState(true);
  const [unreadCount, setUnreadCount] = useState(0);

  useEffect(() => {
    fetchNotifications();
    fetchTemplates();
  }, []);

  const fetchNotifications = async () => {
    try {
      const response = await axios.get('/api/notifications/list');
      setNotifications(response.data.notifications);
      setUnreadCount(response.data.notifications.filter((n: Notification) => n.status !== 'read').length);
    } catch (error) {
      console.error('Failed to fetch notifications:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchTemplates = async () => {
    try {
      const response = await axios.get('/api/notifications/templates');
      setTemplates(response.data.templates);
    } catch (error) {
      console.error('Failed to fetch templates:', error);
    }
  };

  const markAsRead = async (notificationId: string) => {
    try {
      await axios.post(`/api/notifications/${notificationId}/read`);
      setNotifications(prev =>
        prev.map(n =>
          n.id === notificationId ? { ...n, status: 'read', read_at: new Date().toISOString() } : n
        )
      );
      setUnreadCount(prev => Math.max(0, prev - 1));
    } catch (error) {
      console.error('Failed to mark notification as read:', error);
    }
  };

  const getPriorityIcon = (priority: string) => {
    switch (priority) {
      case 'critical':
        return <XCircleIcon className="h-5 w-5 text-red-500" />;
      case 'high':
        return <ExclamationTriangleIcon className="h-5 w-5 text-orange-500" />;
      case 'normal':
        return <InformationCircleIcon className="h-5 w-5 text-blue-500" />;
      case 'low':
        return <CheckCircleIcon className="h-5 w-5 text-green-500" />;
      default:
        return <InformationCircleIcon className="h-5 w-5 text-gray-500" />;
    }
  };

  const getChannelIcon = (channel: string) => {
    switch (channel) {
      case 'email':
        return <EnvelopeIcon className="h-4 w-4" />;
      case 'slack':
        return <ChatBubbleBottomCenterTextIcon className="h-4 w-4" />;
      case 'sms':
        return <DevicePhoneMobileIcon className="h-4 w-4" />;
      default:
        return <BellIcon className="h-4 w-4" />;
    }
  };

  const filteredNotifications = notifications.filter(n => {
    if (filter === 'unread') return n.status !== 'read';
    if (filter === 'critical') return n.priority === 'critical' || n.priority === 'high';
    return true;
  });

  return (
    <div className="flex h-full">
      {/* Sidebar */}
      <div className="w-64 bg-gray-50 border-r border-gray-200 p-4">
        <div className="mb-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-gray-900">Notifications</h2>
            <div className="relative">
              {unreadCount > 0 ? (
                <>
                  <BellSolidIcon className="h-6 w-6 text-indigo-600" />
                  <span className="absolute -top-1 -right-1 h-4 w-4 bg-red-500 text-white text-xs rounded-full flex items-center justify-center">
                    {unreadCount}
                  </span>
                </>
              ) : (
                <BellIcon className="h-6 w-6 text-gray-400" />
              )}
            </div>
          </div>
        </div>

        <nav className="space-y-1">
          <button
            onClick={() => setActiveTab('inbox')}
            className={`w-full text-left px-3 py-2 rounded-md text-sm font-medium ${
              activeTab === 'inbox'
                ? 'bg-indigo-100 text-indigo-700'
                : 'text-gray-700 hover:bg-gray-100'
            }`}
          >
            Inbox
          </button>
          <button
            onClick={() => setActiveTab('templates')}
            className={`w-full text-left px-3 py-2 rounded-md text-sm font-medium ${
              activeTab === 'templates'
                ? 'bg-indigo-100 text-indigo-700'
                : 'text-gray-700 hover:bg-gray-100'
            }`}
          >
            Templates
          </button>
          <button
            onClick={() => setActiveTab('settings')}
            className={`w-full text-left px-3 py-2 rounded-md text-sm font-medium ${
              activeTab === 'settings'
                ? 'bg-indigo-100 text-indigo-700'
                : 'text-gray-700 hover:bg-gray-100'
            }`}
          >
            Settings
          </button>
        </nav>

        {activeTab === 'inbox' && (
          <div className="mt-6">
            <h3 className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2">
              Filter
            </h3>
            <div className="space-y-1">
              <button
                onClick={() => setFilter('all')}
                className={`w-full text-left px-3 py-1 rounded text-sm ${
                  filter === 'all' ? 'bg-gray-200 text-gray-900' : 'text-gray-600 hover:bg-gray-100'
                }`}
              >
                All
              </button>
              <button
                onClick={() => setFilter('unread')}
                className={`w-full text-left px-3 py-1 rounded text-sm ${
                  filter === 'unread' ? 'bg-gray-200 text-gray-900' : 'text-gray-600 hover:bg-gray-100'
                }`}
              >
                Unread
              </button>
              <button
                onClick={() => setFilter('critical')}
                className={`w-full text-left px-3 py-1 rounded text-sm ${
                  filter === 'critical' ? 'bg-gray-200 text-gray-900' : 'text-gray-600 hover:bg-gray-100'
                }`}
              >
                Critical
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Main Content */}
      <div className="flex-1 p-6">
        {activeTab === 'inbox' && (
          <div>
            <div className="mb-4">
              <h3 className="text-lg font-medium text-gray-900">
                {filter === 'all' && 'All Notifications'}
                {filter === 'unread' && 'Unread Notifications'}
                {filter === 'critical' && 'Critical Notifications'}
              </h3>
              <p className="text-sm text-gray-500">
                {filteredNotifications.length} notification{filteredNotifications.length !== 1 ? 's' : ''}
              </p>
            </div>

            {loading ? (
              <div className="flex justify-center items-center h-64">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
              </div>
            ) : filteredNotifications.length === 0 ? (
              <div className="text-center py-12">
                <BellIcon className="mx-auto h-12 w-12 text-gray-400" />
                <h3 className="mt-2 text-sm font-medium text-gray-900">No notifications</h3>
                <p className="mt-1 text-sm text-gray-500">
                  {filter === 'unread' ? 'All caught up!' : 'No notifications to display'}
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                {filteredNotifications.map((notification) => (
                  <div
                    key={notification.id}
                    className={`bg-white rounded-lg shadow-sm border ${
                      notification.status === 'read' ? 'border-gray-200' : 'border-indigo-200'
                    } p-4 hover:shadow-md transition-shadow cursor-pointer`}
                    onClick={() => notification.status !== 'read' && markAsRead(notification.id)}
                  >
                    <div className="flex items-start space-x-3">
                      <div className="flex-shrink-0">
                        {getPriorityIcon(notification.priority)}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between">
                          <p className={`text-sm font-medium ${
                            notification.status === 'read' ? 'text-gray-600' : 'text-gray-900'
                          }`}>
                            {notification.title}
                          </p>
                          <div className="flex items-center space-x-2">
                            {notification.channels.map((channel) => (
                              <span key={channel} className="text-gray-400">
                                {getChannelIcon(channel)}
                              </span>
                            ))}
                          </div>
                        </div>
                        <p className={`mt-1 text-sm ${
                          notification.status === 'read' ? 'text-gray-400' : 'text-gray-500'
                        }`}>
                          {notification.message}
                        </p>
                        <p className="mt-1 text-xs text-gray-400">
                          {new Date(notification.created_at).toLocaleString()}
                        </p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {activeTab === 'templates' && (
          <div>
            <div className="mb-6">
              <h3 className="text-lg font-medium text-gray-900">Notification Templates</h3>
              <p className="text-sm text-gray-500">Manage reusable notification templates</p>
            </div>

            <div className="grid grid-cols-1 gap-4">
              {templates.map((template) => (
                <div key={template.id} className="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
                  <div className="flex items-center justify-between mb-2">
                    <h4 className="text-sm font-medium text-gray-900">{template.name}</h4>
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                      {template.channel}
                    </span>
                  </div>
                  <p className="text-sm text-gray-500 mb-2">{template.description}</p>
                  <div className="text-xs text-gray-400">
                    Variables: {template.variables.join(', ')}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'settings' && (
          <div>
            <h3 className="text-lg font-medium text-gray-900 mb-6">Notification Settings</h3>
            <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
              <p className="text-sm text-gray-500">Notification preferences and channel configuration</p>
              {/* Add settings form here */}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}; 