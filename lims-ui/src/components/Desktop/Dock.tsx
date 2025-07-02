import React, { useState } from 'react';
import { AppDefinition } from '../../types/apps';
import { WindowState } from './Window';

interface DockProps {
  apps: AppDefinition[];
  onAppClick: (appId: string) => void;
  activeWindows: WindowState[];
}

export const Dock: React.FC<DockProps> = ({ apps, onAppClick, activeWindows }) => {
  const [hoveredApp, setHoveredApp] = useState<string | null>(null);

  const getActiveWindowsForApp = (appId: string) => {
    return activeWindows.filter(w => w.appId === appId && !w.isMinimized);
  };

  return (
    <div className="absolute bottom-4 left-1/2 transform -translate-x-1/2">
      <div className="bg-white/30 dark:bg-black/30 backdrop-blur-2xl rounded-2xl px-3 py-2 shadow-2xl border border-white/20">
        <div className="flex items-end gap-2">
          {apps.map((app) => {
            const isHovered = hoveredApp === app.id;
            const hasActiveWindows = getActiveWindowsForApp(app.id).length > 0;
            
            return (
              <div
                key={app.id}
                className="relative"
                onMouseEnter={() => setHoveredApp(app.id)}
                onMouseLeave={() => setHoveredApp(null)}
              >
                {/* Tooltip */}
                {isHovered && (
                  <div className="absolute bottom-full mb-2 left-1/2 transform -translate-x-1/2 px-2 py-1 bg-black/80 text-white text-xs rounded whitespace-nowrap">
                    {app.name}
                  </div>
                )}
                
                {/* App Icon */}
                <button
                  onClick={() => onAppClick(app.id)}
                  className={`relative transition-all duration-200 ${
                    isHovered ? 'transform -translate-y-2 scale-125' : ''
                  }`}
                >
                  <div className={`w-12 h-12 ${app.dockIconClass || 'bg-gradient-to-br from-blue-400 to-blue-600'} rounded-xl shadow-lg flex items-center justify-center text-white transform transition-all`}>
                    {app.icon}
                  </div>
                </button>
                
                {/* Active Indicator */}
                {hasActiveWindows && (
                  <div className="absolute -bottom-1 left-1/2 transform -translate-x-1/2 w-1 h-1 bg-gray-600 dark:bg-gray-300 rounded-full" />
                )}
              </div>
            );
          })}
          
          {/* Dock Separator */}
          <div className="mx-2 w-px h-10 bg-gray-400/50" />
          
          {/* Trash */}
          <div
            className="relative"
            onMouseEnter={() => setHoveredApp('trash')}
            onMouseLeave={() => setHoveredApp(null)}
          >
            {hoveredApp === 'trash' && (
              <div className="absolute bottom-full mb-2 left-1/2 transform -translate-x-1/2 px-2 py-1 bg-black/80 text-white text-xs rounded whitespace-nowrap">
                Trash
              </div>
            )}
            <button
              className={`relative transition-all duration-200 ${
                hoveredApp === 'trash' ? 'transform -translate-y-2 scale-125' : ''
              }`}
            >
              <div className="w-12 h-12 bg-gradient-to-br from-gray-400 to-gray-600 rounded-xl shadow-lg flex items-center justify-center text-white">
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </div>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};