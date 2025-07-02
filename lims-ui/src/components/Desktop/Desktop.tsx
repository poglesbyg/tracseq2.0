import React, { useState, useCallback, useEffect } from 'react';
import { Window } from './Window';
import { Dock } from './Dock';
import { MenuBar } from './MenuBar';
import { Spotlight } from './Spotlight';
import { MissionControl } from './MissionControl';
import { NotificationCenter } from './NotificationCenter';
import { useWindowManager } from '../../hooks/useWindowManager';
import { useSpaces } from '../../hooks/useSpaces';
import { useNotifications } from '../../hooks/useNotifications';
import { useContextMenu, ContextMenuProvider } from '../../hooks/useContextMenu';
import { AppDefinition } from '../../types/apps';
import { 
  FolderOpenIcon
} from '@heroicons/react/24/outline';

interface DesktopProps {
  apps: AppDefinition[];
}

const DesktopContent: React.FC<DesktopProps> = ({ apps }) => {
  const { windows, openWindow, closeWindow, focusWindow, updateWindow } = useWindowManager();
  const { 
    spaces, 
    currentSpaceId, 
    createSpace, 
    deleteSpace, 
    changeSpace, 
    addWindowToSpace, 
    removeWindowFromSpace, 
    moveWindowToSpace
  } = useSpaces();
  const {
    notifications,
    markAsRead,
    dismissNotification,
    clearAll,
    unreadCount
  } = useNotifications();
  const [showLaunchpad, setShowLaunchpad] = useState(false);
  const [showSpotlight, setShowSpotlight] = useState(false);
  const [showMissionControl, setShowMissionControl] = useState(false);
  const [showNotifications, setShowNotifications] = useState(false);
  const { showContextMenu } = useContextMenu();

  const handleAppLaunch = useCallback((appId: string) => {
    const app = apps.find(a => a.id === appId);
    if (app) {
      const windowId = `${app.id}-${Date.now()}`;
      openWindow({
        id: windowId,
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
      addWindowToSpace(windowId);
    }
  }, [apps, openWindow, windows.length, addWindowToSpace]);

  const handleCloseWindow = useCallback((windowId: string) => {
    closeWindow(windowId);
    removeWindowFromSpace(windowId);
  }, [closeWindow, removeWindowFromSpace]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+K or Cmd+Space for Spotlight
      if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === ' ')) {
        e.preventDefault();
        setShowSpotlight(true);
      }
      // F3 or Ctrl+Up for Mission Control
      if (e.key === 'F3' || ((e.ctrlKey || e.metaKey) && e.key === 'ArrowUp')) {
        e.preventDefault();
        setShowMissionControl(true);
      }
      // Escape to close overlays
      if (e.key === 'Escape') {
        if (showSpotlight) setShowSpotlight(false);
        if (showLaunchpad) setShowLaunchpad(false);
        if (showMissionControl) setShowMissionControl(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [showSpotlight, showLaunchpad, showMissionControl]);

  // Get windows for current space
  const visibleWindows = windows.filter(w => {
    const space = spaces.find(s => s.id === currentSpaceId);
    return space && space.windowIds.includes(w.id);
  });

  const handleDesktopContextMenu = (e: React.MouseEvent) => {
    showContextMenu(e, [
      {
        label: 'New Folder',
        icon: <FolderOpenIcon className="w-4 h-4" />,
        action: () => console.log('New folder')
      },
      { divider: true },
      {
        label: 'Change Desktop Background',
        action: () => console.log('Change background')
      },
      { divider: true },
      {
        label: 'Clean Up',
        action: () => console.log('Clean up')
      },
      {
        label: 'Clean Up By',
        submenu: [
          { label: 'Name' },
          { label: 'Kind' },
          { label: 'Date Modified' },
          { label: 'Size' }
        ]
      }
    ]);
  };

  return (
    <div 
      className="fixed inset-0 bg-gradient-to-br from-blue-100 via-purple-50 to-pink-100 overflow-hidden"
      onContextMenu={handleDesktopContextMenu}
    >
      {/* Desktop wallpaper with subtle animation */}
      <div className="absolute inset-0 bg-gradient-to-br from-blue-500/10 via-purple-500/10 to-pink-500/10 animate-gradient" />
      
      {/* Menu Bar */}
      <MenuBar 
        onShowLaunchpad={() => setShowLaunchpad(true)}
        onShowSpotlight={() => setShowSpotlight(true)}
        onShowNotifications={() => setShowNotifications(!showNotifications)}
        notificationCount={unreadCount}
        activeAppName={windows.find(w => w.isFocused)?.title}
      />

      {/* Desktop Area */}
      <div className="absolute inset-0 mt-8 mb-20">
        {/* Windows */}
        {visibleWindows.map((window) => (
          <Window
            key={window.id}
            window={window}
            onClose={() => handleCloseWindow(window.id)}
            onFocus={() => focusWindow(window.id)}
            onUpdate={(updates) => updateWindow(window.id, updates)}
            onOpenApp={handleAppLaunch}
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

      {/* Mission Control */}
      <MissionControl
        isOpen={showMissionControl}
        onClose={() => setShowMissionControl(false)}
        windows={windows}
        spaces={spaces}
        currentSpaceId={currentSpaceId}
        onWindowClick={focusWindow}
        onSpaceChange={(spaceId) => {
          changeSpace(spaceId);
          setShowMissionControl(false);
        }}
        onCreateSpace={createSpace}
        onDeleteSpace={deleteSpace}
        onMoveWindowToSpace={moveWindowToSpace}
      />

      {/* Notification Center */}
      <NotificationCenter
        isOpen={showNotifications}
        onClose={() => setShowNotifications(false)}
        notifications={notifications}
        onNotificationRead={markAsRead}
        onNotificationDismiss={dismissNotification}
        onClearAll={clearAll}
      />
    </div>
  );
};

export const Desktop: React.FC<DesktopProps> = (props) => {
  return (
    <ContextMenuProvider>
      <DesktopContent {...props} />
    </ContextMenuProvider>
  );
};