import React, { useState, useEffect } from 'react';
import { Menu } from '@headlessui/react';
import { 
  MagnifyingGlassIcon, 
  WifiIcon, 
  Battery100Icon,
  SpeakerWaveIcon,
  Squares2X2Icon,
  BellIcon
} from '@heroicons/react/24/outline';

interface MenuBarProps {
  onShowLaunchpad: () => void;
  onShowSpotlight?: () => void;
  onShowNotifications?: () => void;
  notificationCount?: number;
  activeAppName?: string;
}

export const MenuBar: React.FC<MenuBarProps> = ({ onShowLaunchpad, onShowSpotlight, onShowNotifications, notificationCount, activeAppName }) => {
  const [currentTime, setCurrentTime] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date());
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('en-US', { 
      hour: 'numeric', 
      minute: '2-digit',
      hour12: true 
    });
  };

  const formatDate = (date: Date) => {
    return date.toLocaleDateString('en-US', { 
      weekday: 'short',
      month: 'short',
      day: 'numeric'
    });
  };

  return (
    <div className="h-8 bg-white/80 dark:bg-gray-900/80 backdrop-blur-xl border-b border-gray-200/50 dark:border-gray-700/50 flex items-center justify-between px-4 select-none">
      {/* Left Side - App Menu */}
      <div className="flex items-center space-x-4">
        {/* App Icon and Name */}
        <div className="flex items-center space-x-2">
          <div className="w-4 h-4 bg-gradient-to-br from-indigo-400 to-indigo-600 rounded" />
          <span className="text-sm font-semibold text-gray-900 dark:text-white">
            {activeAppName || 'TracSeq OS'}
          </span>
        </div>

        {/* File Menu */}
        <Menu as="div" className="relative">
          <Menu.Button className="text-sm text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white">
            File
          </Menu.Button>
          <Menu.Items className="absolute left-0 mt-1 w-48 bg-white dark:bg-gray-800 rounded-lg shadow-lg py-1 z-50">
            <Menu.Item>
              {({ active }) => (
                <button className={`${active ? 'bg-blue-500 text-white' : 'text-gray-700 dark:text-gray-300'} block w-full text-left px-4 py-2 text-sm`}>
                  New Window
                </button>
              )}
            </Menu.Item>
            <Menu.Item>
              {({ active }) => (
                <button className={`${active ? 'bg-blue-500 text-white' : 'text-gray-700 dark:text-gray-300'} block w-full text-left px-4 py-2 text-sm`}>
                  Close Window
                </button>
              )}
            </Menu.Item>
          </Menu.Items>
        </Menu>

        {/* Edit Menu */}
        <Menu as="div" className="relative">
          <Menu.Button className="text-sm text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white">
            Edit
          </Menu.Button>
        </Menu>

        {/* View Menu */}
        <Menu as="div" className="relative">
          <Menu.Button className="text-sm text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white">
            View
          </Menu.Button>
        </Menu>

        {/* Window Menu */}
        <Menu as="div" className="relative">
          <Menu.Button className="text-sm text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white">
            Window
          </Menu.Button>
        </Menu>
      </div>

      {/* Right Side - System Controls */}
      <div className="flex items-center space-x-3">
        {/* Spotlight Search */}
        <button 
          onClick={onShowSpotlight}
          className="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded"
        >
          <MagnifyingGlassIcon className="w-4 h-4 text-gray-600 dark:text-gray-400" />
        </button>

        {/* Mission Control */}
        <button 
          onClick={() => window.dispatchEvent(new KeyboardEvent('keydown', { key: 'F3' }))}
          className="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded"
          title="Mission Control (F3)"
        >
          <svg className="w-4 h-4 text-gray-600 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
          </svg>
        </button>

        {/* Launchpad */}
        <button 
          onClick={onShowLaunchpad}
          className="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded"
        >
          <Squares2X2Icon className="w-4 h-4 text-gray-600 dark:text-gray-400" />
        </button>

        {/* Notifications */}
        <button
          onClick={onShowNotifications}
          className="relative p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded"
        >
          <BellIcon className="w-4 h-4 text-gray-600 dark:text-gray-400" />
          {notificationCount && notificationCount > 0 && (
            <span className="absolute -top-1 -right-1 w-4 h-4 bg-red-500 text-white text-xs rounded-full flex items-center justify-center">
              {notificationCount}
            </span>
          )}
        </button>

        {/* System Icons */}
        <WifiIcon className="w-4 h-4 text-gray-600 dark:text-gray-400" />
        <SpeakerWaveIcon className="w-4 h-4 text-gray-600 dark:text-gray-400" />
        <Battery100Icon className="w-4 h-4 text-gray-600 dark:text-gray-400" />

        {/* Date and Time */}
        <div className="text-sm text-gray-700 dark:text-gray-300 ml-2">
          <span>{formatDate(currentTime)}</span>
          <span className="ml-2">{formatTime(currentTime)}</span>
        </div>
      </div>
    </div>
  );
};