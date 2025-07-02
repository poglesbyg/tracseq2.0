import { useState, useCallback } from 'react';
import { Notification } from '../components/Desktop/NotificationCenter';

export const useNotifications = () => {
  const [notifications, setNotifications] = useState<Notification[]>([
    // Sample notifications
    {
      id: '1',
      type: 'success',
      title: 'Sample Analysis Complete',
      message: 'Sample PCR-001 has been successfully analyzed. Results are ready for review.',
      timestamp: new Date(Date.now() - 1000 * 60 * 5), // 5 minutes ago
      read: false,
      actionLabel: 'View Results',
      onAction: () => console.log('View results')
    },
    {
      id: '2',
      type: 'warning',
      title: 'Low Storage Space',
      message: 'Freezer Unit 3 is at 85% capacity. Consider transferring samples.',
      timestamp: new Date(Date.now() - 1000 * 60 * 30), // 30 minutes ago
      read: false
    },
    {
      id: '3',
      type: 'info',
      title: 'System Update',
      message: 'TracSeq OS has been updated to version 2.1.0 with new features.',
      timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2), // 2 hours ago
      read: true
    }
  ]);

  const addNotification = useCallback((notification: Omit<Notification, 'id' | 'timestamp' | 'read'>) => {
    const newNotification: Notification = {
      ...notification,
      id: `notif-${Date.now()}`,
      timestamp: new Date(),
      read: false
    };
    setNotifications(prev => [newNotification, ...prev]);
  }, []);

  const markAsRead = useCallback((id: string) => {
    setNotifications(prev => prev.map(n => 
      n.id === id ? { ...n, read: true } : n
    ));
  }, []);

  const dismissNotification = useCallback((id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id));
  }, []);

  const clearAll = useCallback(() => {
    setNotifications([]);
  }, []);

  const markAllAsRead = useCallback(() => {
    setNotifications(prev => prev.map(n => ({ ...n, read: true })));
  }, []);

  const unreadCount = notifications.filter(n => !n.read).length;

  return {
    notifications,
    addNotification,
    markAsRead,
    dismissNotification,
    clearAll,
    markAllAsRead,
    unreadCount
  };
};