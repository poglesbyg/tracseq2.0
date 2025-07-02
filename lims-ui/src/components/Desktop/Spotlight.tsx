import React, { useState, useEffect, useRef } from 'react';
import { MagnifyingGlassIcon, CommandLineIcon } from '@heroicons/react/24/outline';
import { AppDefinition } from '../../types/apps';

interface SpotlightProps {
  isOpen: boolean;
  onClose: () => void;
  apps: AppDefinition[];
  onAppLaunch: (appId: string) => void;
}

export const Spotlight: React.FC<SpotlightProps> = ({ isOpen, onClose, apps, onAppLaunch }) => {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const filteredApps = apps.filter(app => 
    app.name.toLowerCase().includes(query.toLowerCase()) ||
    app.description?.toLowerCase().includes(query.toLowerCase())
  );

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
      setQuery('');
      setSelectedIndex(0);
    }
  }, [isOpen]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev => Math.min(prev + 1, filteredApps.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredApps[selectedIndex]) {
          onAppLaunch(filteredApps[selectedIndex].id);
          onClose();
        }
        break;
      case 'Escape':
        onClose();
        break;
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-[10000] flex items-start justify-center pt-32">
      {/* Backdrop */}
      <div 
        className="absolute inset-0 bg-black/30 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Spotlight Window */}
      <div className="relative w-full max-w-2xl bg-white/95 dark:bg-gray-900/95 backdrop-blur-2xl rounded-2xl shadow-2xl overflow-hidden">
        {/* Search Input */}
        <div className="flex items-center px-6 py-4 border-b border-gray-200/50 dark:border-gray-700/50">
          <MagnifyingGlassIcon className="w-6 h-6 text-gray-400 mr-3" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search for apps, files, or actions..."
            className="flex-1 bg-transparent outline-none text-lg text-gray-900 dark:text-white placeholder-gray-500"
          />
          <div className="flex items-center gap-1 text-xs text-gray-500">
            <kbd className="px-1.5 py-0.5 bg-gray-200 dark:bg-gray-700 rounded">⌘</kbd>
            <span>K</span>
          </div>
        </div>

        {/* Results */}
        <div className="max-h-96 overflow-y-auto">
          {filteredApps.length > 0 ? (
            <div className="py-2">
              <div className="px-6 py-2 text-xs font-medium text-gray-500 uppercase">
                Applications
              </div>
              {filteredApps.map((app, index) => (
                <button
                  key={app.id}
                  onClick={() => {
                    onAppLaunch(app.id);
                    onClose();
                  }}
                  className={`w-full px-6 py-3 flex items-center gap-3 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors ${
                    index === selectedIndex ? 'bg-blue-50 dark:bg-blue-900/30' : ''
                  }`}
                >
                  <div className={`w-10 h-10 ${app.dockIconClass} rounded-xl flex items-center justify-center text-white shadow-sm`}>
                    {app.icon}
                  </div>
                  <div className="flex-1 text-left">
                    <div className="font-medium text-gray-900 dark:text-white">{app.name}</div>
                    {app.description && (
                      <div className="text-sm text-gray-500 dark:text-gray-400">{app.description}</div>
                    )}
                  </div>
                  {index === selectedIndex && (
                    <div className="text-xs text-gray-500">
                      <kbd className="px-1.5 py-0.5 bg-gray-200 dark:bg-gray-700 rounded">↵</kbd>
                    </div>
                  )}
                </button>
              ))}
            </div>
          ) : query ? (
            <div className="px-6 py-12 text-center text-gray-500">
              No results found for "{query}"
            </div>
          ) : (
            <div className="px-6 py-8">
              <div className="text-sm text-gray-500 mb-4">Recent</div>
              <div className="space-y-2">
                {apps.slice(0, 5).map((app) => (
                  <button
                    key={app.id}
                    onClick={() => {
                      onAppLaunch(app.id);
                      onClose();
                    }}
                    className="w-full px-3 py-2 flex items-center gap-3 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors text-left"
                  >
                    <div className={`w-8 h-8 ${app.dockIconClass} rounded-lg flex items-center justify-center text-white shadow-sm`}>
                      {React.cloneElement(app.icon as React.ReactElement, { className: 'w-4 h-4' })}
                    </div>
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">{app.name}</span>
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-3 border-t border-gray-200/50 dark:border-gray-700/50 flex items-center justify-between text-xs text-gray-500">
          <div className="flex items-center gap-4">
            <span className="flex items-center gap-1">
              <kbd className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded">↑</kbd>
              <kbd className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded">↓</kbd>
              Navigate
            </span>
            <span className="flex items-center gap-1">
              <kbd className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded">↵</kbd>
              Open
            </span>
            <span className="flex items-center gap-1">
              <kbd className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded">esc</kbd>
              Close
            </span>
          </div>
          <div className="flex items-center gap-1">
            <CommandLineIcon className="w-3 h-3" />
            <span>AI Commands Coming Soon</span>
          </div>
        </div>
      </div>
    </div>
  );
};