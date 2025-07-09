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
import { ChatBotWrapper } from '../ChatBotWrapper';
import { ChatBotFloat } from '../ChatBotFloat';

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
  const [desktopBackground, setDesktopBackground] = useState('gradient-to-br from-blue-100 via-purple-50 to-pink-100');
  const { showContextMenu } = useContextMenu();
  const [showChatBot, setShowChatBot] = useState(false);

  const handleAppLaunch = useCallback((appId: string, context?: Record<string, unknown>) => {
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
      
      // Store parameters in a way that can be accessed by the component
      if (context) {
        (window as Record<string, any>).appParams = (window as Record<string, any>).appParams || {};
        (window as Record<string, any>).appParams[windowId] = context;
      }
    }
  }, [apps, openWindow, windows.length, addWindowToSpace]);

  // Handle URL parameters on initial load
  useEffect(() => {
    // Check if we have a stored original route from the redirect
    const originalRoute = sessionStorage.getItem('originalRoute');
    let pathname = window.location.pathname;
    let search = window.location.search;
    
    if (originalRoute) {
      console.log('📍 Found stored original route:', originalRoute);
      const url = new URL('http://localhost' + originalRoute); // Add dummy base for parsing
      pathname = url.pathname;
      search = url.search;
      // Clear the stored route so it doesn't interfere with future navigation
      sessionStorage.removeItem('originalRoute');
    }
    
    const urlParams = new URLSearchParams(search);
    
    console.log('🔍 Desktop mounted, checking URL:', { pathname, search, originalRoute });
    
    const launchApp = (appId: string, context?: Record<string, unknown>) => {
      const app = apps.find(a => a.id === appId);
      if (app) {
        console.log('📱 Launching app:', appId, 'with context:', context);
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
        
        // Store parameters in a way that can be accessed by the component
        if (context) {
          console.log('💾 Storing context for window:', windowId, context);
          (window as Record<string, any>).appParams = (window as Record<string, any>).appParams || {};
          (window as Record<string, any>).appParams[windowId] = context;
        }
      } else {
        console.error('❌ App not found:', appId);
      }
    };
    
    // Check if we're trying to open a specific app with parameters
    if (pathname.includes('rag-submissions')) {
      const submissionId = urlParams.get('submission');
      if (submissionId) {
        console.log('🚀 Auto-opening RAG Submissions with submission:', submissionId);
        // Use setTimeout to ensure the desktop is ready
        setTimeout(() => {
          launchApp('rag-submissions', { submission: submissionId });
        }, 200);
      } else {
        console.log('🚀 Auto-opening RAG Submissions');
        setTimeout(() => {
          launchApp('rag-submissions');
        }, 200);
      }
    } else if (pathname.includes('rag-samples')) {
      console.log('🚀 Auto-opening RAG Samples');
      setTimeout(() => {
        launchApp('rag-samples');
      }, 200);
    } else if (pathname.includes('samples')) {
      console.log('🚀 Auto-opening Samples');
      setTimeout(() => {
        launchApp('samples');
      }, 200);
    } else if (pathname.includes('templates')) {
      console.log('🚀 Auto-opening Templates');
      setTimeout(() => {
        launchApp('templates');
      }, 200);
    } else if (pathname.includes('reports')) {
      console.log('🚀 Auto-opening Reports');
      setTimeout(() => {
        launchApp('reports');
      }, 200);
    } else if (pathname.includes('dashboard')) {
      console.log('🚀 Auto-opening Dashboard');
      setTimeout(() => {
        launchApp('dashboard');
      }, 200);
    }
    
    // Only clear the URL if we're not using a stored original route
    // (The URL has already been cleared by the redirect in that case)
    if (!originalRoute && pathname !== '/desktop') {
      console.log('🔄 Clearing URL to /desktop');
      window.history.replaceState({}, '', '/desktop');
    }
  }, [apps]); // Only depend on apps to prevent infinite re-renders

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
        action: () => {
          // Open Finder and create a new folder
          const finderId = `finder-${Date.now()}`;
          const finderApp = apps.find(a => a.id === 'finder');
          if (finderApp) {
            openWindow({
              id: finderId,
              appId: 'finder',
              title: 'Finder',
              icon: finderApp.icon,
              component: finderApp.component,
              position: { x: 200, y: 100 },
              size: finderApp.defaultSize || { width: 900, height: 600 },
              isMinimized: false,
              isMaximized: false,
              zIndex: 1000 + windows.length,
            });
            addWindowToSpace(finderId);
          }
        }
      },
      { divider: true },
      {
        label: 'Change Desktop Background',
        submenu: [
          { 
            label: 'Blue Gradient', 
            action: () => setDesktopBackground('gradient-to-br from-blue-100 via-blue-50 to-blue-200')
          },
          { 
            label: 'Purple Gradient', 
            action: () => setDesktopBackground('gradient-to-br from-purple-100 via-purple-50 to-purple-200')
          },
          { 
            label: 'Green Gradient', 
            action: () => setDesktopBackground('gradient-to-br from-green-100 via-green-50 to-green-200')
          },
          { 
            label: 'Sunset Gradient', 
            action: () => setDesktopBackground('gradient-to-br from-orange-100 via-pink-50 to-purple-100')
          },
          { 
            label: 'Ocean Gradient', 
            action: () => setDesktopBackground('gradient-to-br from-cyan-100 via-blue-50 to-indigo-100')
          },
          { 
            label: 'Default', 
            action: () => setDesktopBackground('gradient-to-br from-blue-100 via-purple-50 to-pink-100')
          }
        ]
      },
      { divider: true },
      {
        label: 'Clean Up',
        action: () => {
          // Arrange windows in a grid
          const visibleWindows = windows.filter(w => {
            const space = spaces.find(s => s.id === currentSpaceId);
            return space && space.windowIds.includes(w.id) && !w.isMinimized;
          });
          
          const cols = Math.ceil(Math.sqrt(visibleWindows.length));
          const windowWidth = 600;
          const windowHeight = 400;
          const padding = 20;
          const startX = 50;
          const startY = 50;
          
          visibleWindows.forEach((window, index) => {
            const row = Math.floor(index / cols);
            const col = index % cols;
            
            updateWindow(window.id, {
              position: {
                x: startX + col * (windowWidth + padding),
                y: startY + row * (windowHeight + padding)
              },
              size: { width: windowWidth, height: windowHeight },
              isMaximized: false
            });
          });
        }
      },
      {
        label: 'Clean Up By',
        submenu: [
          { 
            label: 'Name',
            action: () => {
              const sortedWindows = [...windows].sort((a, b) => a.title.localeCompare(b.title));
              arrangeWindows(sortedWindows);
            }
          },
          { 
            label: 'Kind',
            action: () => {
              const sortedWindows = [...windows].sort((a, b) => a.appId.localeCompare(b.appId));
              arrangeWindows(sortedWindows);
            }
          },
          { 
            label: 'Date Modified',
            action: () => {
              const sortedWindows = [...windows].sort((a, b) => b.zIndex - a.zIndex);
              arrangeWindows(sortedWindows);
            }
          },
          { 
            label: 'Size',
            action: () => {
              const sortedWindows = [...windows].sort((a, b) => 
                (b.size.width * b.size.height) - (a.size.width * a.size.height)
              );
              arrangeWindows(sortedWindows);
            }
          }
        ]
      }
    ]);
  };

  const arrangeWindows = (windowsToArrange: typeof windows) => {
    const visibleWindows = windowsToArrange.filter(w => {
      const space = spaces.find(s => s.id === currentSpaceId);
      return space && space.windowIds.includes(w.id) && !w.isMinimized;
    });
    
    const cols = Math.ceil(Math.sqrt(visibleWindows.length));
    const windowWidth = 600;
    const windowHeight = 400;
    const padding = 20;
    const startX = 50;
    const startY = 50;
    
    visibleWindows.forEach((window, index) => {
      const row = Math.floor(index / cols);
      const col = index % cols;
      
      updateWindow(window.id, {
        position: {
          x: startX + col * (windowWidth + padding),
          y: startY + row * (windowHeight + padding)
        },
        size: { width: windowWidth, height: windowHeight },
        isMaximized: false
      });
    });
  };

  return (
    <div 
      className={`fixed inset-0 bg-${desktopBackground} overflow-hidden`}
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

      {/* ChatBot */}
      <ChatBotWrapper 
        isOpen={showChatBot} 
        onToggle={() => setShowChatBot(false)} 
      />
      
      {/* ChatBot Float Button - Always visible */}
      {!showChatBot && (
        <ChatBotFloat 
          onClick={() => setShowChatBot(true)} 
          hasUnread={false}
        />
      )}
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