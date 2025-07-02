import React, { useState, useCallback, useEffect } from 'react';
import { Window } from './Window';
import { Dock } from './Dock';
import { MenuBar } from './MenuBar';
import { Spotlight } from './Spotlight';
import { useWindowManager } from '../../hooks/useWindowManager';
import { AppDefinition } from '../../types/apps';

interface DesktopProps {
  apps: AppDefinition[];
}

export const Desktop: React.FC<DesktopProps> = ({ apps }) => {
  const { windows, openWindow, closeWindow, focusWindow, updateWindow } = useWindowManager();
  const [showLaunchpad, setShowLaunchpad] = useState(false);
  const [showSpotlight, setShowSpotlight] = useState(false);

  const handleAppLaunch = useCallback((appId: string) => {
    const app = apps.find(a => a.id === appId);
    if (app) {
      openWindow({
        id: `${app.id}-${Date.now()}`,
        appId: app.id,
        title: app.name,
        icon: app.icon,
        component: app.component,
        position: { x: 100 + (windows.length * 30), y: 100 + (windows.length * 30) },
        size: app.defaultSize || { width: 800, height: 600 },
        isMinimized: false,
        isMaximized: false,
        zIndex: 1000 + windows.length,
      });
    }
  }, [apps, openWindow, windows.length]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+K or Cmd+Space for Spotlight
      if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === ' ')) {
        e.preventDefault();
        setShowSpotlight(true);
      }
      // Escape to close overlays
      if (e.key === 'Escape') {
        if (showSpotlight) setShowSpotlight(false);
        if (showLaunchpad) setShowLaunchpad(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [showSpotlight, showLaunchpad]);

  return (
    <div className="fixed inset-0 bg-gradient-to-br from-blue-100 via-purple-50 to-pink-100 overflow-hidden">
      {/* Desktop wallpaper with subtle animation */}
      <div className="absolute inset-0 bg-gradient-to-br from-blue-500/10 via-purple-500/10 to-pink-500/10 animate-gradient" />
      
      {/* Menu Bar */}
      <MenuBar 
        onShowLaunchpad={() => setShowLaunchpad(true)}
        onShowSpotlight={() => setShowSpotlight(true)}
        activeAppName={windows.find(w => w.isFocused)?.title}
      />

      {/* Desktop Area */}
      <div className="absolute inset-0 mt-8 mb-20">
        {/* Windows */}
        {windows.map((window) => (
          <Window
            key={window.id}
            window={window}
            onClose={() => closeWindow(window.id)}
            onFocus={() => focusWindow(window.id)}
            onUpdate={(updates) => updateWindow(window.id, updates)}
          />
        ))}

        {/* Launchpad Overlay */}
        {showLaunchpad && (
          <div className="absolute inset-0 bg-black/30 backdrop-blur-xl z-[9999] flex items-center justify-center"
               onClick={() => setShowLaunchpad(false)}>
            <div className="grid grid-cols-6 gap-8 p-12">
              {apps.map((app) => (
                <button
                  key={app.id}
                  onClick={(e) => {
                    e.stopPropagation();
                    handleAppLaunch(app.id);
                    setShowLaunchpad(false);
                  }}
                  className="flex flex-col items-center gap-2 p-4 rounded-2xl hover:bg-white/20 transition-all"
                >
                  <div className="w-20 h-20 bg-white/90 rounded-2xl shadow-lg flex items-center justify-center">
                    {app.icon}
                  </div>
                  <span className="text-white text-sm font-medium">{app.name}</span>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Dock */}
      <Dock 
        apps={apps} 
        onAppClick={handleAppLaunch}
        activeWindows={windows}
      />

      {/* Spotlight Search */}
      <Spotlight
        isOpen={showSpotlight}
        onClose={() => setShowSpotlight(false)}
        apps={apps}
        onAppLaunch={(appId) => {
          handleAppLaunch(appId);
          setShowSpotlight(false);
        }}
      />
    </div>
  );
};