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
  const [showTrashDialog, setShowTrashDialog] = useState(false);
  const [trashItems, setTrashItems] = useState<{id: string, name: string, deletedAt: Date}[]>([
    { id: '1', name: 'Old Report.pdf', deletedAt: new Date(Date.now() - 86400000) },
    { id: '2', name: 'Backup Sample Data.xlsx', deletedAt: new Date(Date.now() - 172800000) },
  ]);

  const getActiveWindowsForApp = (appId: string) => {
    return activeWindows.filter(w => w.appId === appId && !w.isMinimized);
  };

  const handleEmptyTrash = () => {
    if (window.confirm('Are you sure you want to permanently delete all items in the Trash?')) {
      setTrashItems([]);
      setShowTrashDialog(false);
    }
  };

  const handleRestoreItem = (itemId: string) => {
    setTrashItems(items => items.filter(item => item.id !== itemId));
  };

  // Group apps by section
  const desktopApps = apps.filter((app: AppDefinition) => app.section === 'desktop');
  const customerSupportApps = apps.filter((app: AppDefinition) => app.section === 'customer-support');
  const scienceApps = apps.filter((app: AppDefinition) => app.section === 'science');
  const dataApps = apps.filter((app: AppDefinition) => app.section === 'data');

  const renderAppIcon = (app: AppDefinition) => {
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
          <div className="absolute bottom-full mb-2 left-1/2 transform -translate-x-1/2 px-2 py-1 bg-black/80 text-white text-xs rounded whitespace-nowrap z-10">
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
  };

  const renderSection = (apps: AppDefinition[], sectionName: string, sectionColor: string) => {
    if (apps.length === 0) return null;
    
    return (
      <div key={sectionName} className="flex flex-col items-center">
        {/* Section Label */}
        <div className="mb-2 px-2 py-1 bg-white/20 dark:bg-black/20 rounded text-xs font-medium text-gray-700 dark:text-gray-300 backdrop-blur-sm">
          {sectionName}
        </div>
        {/* Apps */}
        <div className="flex items-end gap-2">
          {apps.map(renderAppIcon)}
        </div>
      </div>
    );
  };

  return (
    <>
      <div className="absolute bottom-4 left-1/2 transform -translate-x-1/2">
        <div className="bg-white/30 dark:bg-black/30 backdrop-blur-2xl rounded-2xl px-4 py-3 shadow-2xl border border-white/20">
          <div className="flex items-end gap-6">
            {/* Desktop Apps */}
            {renderSection(desktopApps, 'Desktop', 'from-blue-400 to-blue-600')}
            
            {/* Section Separator */}
            {desktopApps.length > 0 && (customerSupportApps.length > 0 || scienceApps.length > 0 || dataApps.length > 0) && (
              <div className="mx-2 w-px h-12 bg-gray-400/50" />
            )}
            
            {/* Customer Support Section */}
            {renderSection(customerSupportApps, 'Customer Support', 'from-violet-400 to-violet-600')}
            
            {/* Section Separator */}
            {customerSupportApps.length > 0 && (scienceApps.length > 0 || dataApps.length > 0) && (
              <div className="mx-2 w-px h-12 bg-gray-400/50" />
            )}
            
            {/* Science Section */}
            {renderSection(scienceApps, 'Science', 'from-pink-400 to-pink-600')}
            
            {/* Section Separator */}
            {scienceApps.length > 0 && dataApps.length > 0 && (
              <div className="mx-2 w-px h-12 bg-gray-400/50" />
            )}
            
            {/* Data Section */}
            {renderSection(dataApps, 'Data', 'from-emerald-400 to-emerald-600')}
            
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
                  Trash {trashItems.length > 0 && `(${trashItems.length} items)`}
                </div>
              )}
              <button
                onClick={() => setShowTrashDialog(true)}
                className={`relative transition-all duration-200 ${
                  hoveredApp === 'trash' ? 'transform -translate-y-2 scale-125' : ''
                }`}
              >
                <div className={`w-12 h-12 ${trashItems.length > 0 ? 'bg-gradient-to-br from-gray-400 to-gray-600' : 'bg-gradient-to-br from-gray-300 to-gray-500'} rounded-xl shadow-lg flex items-center justify-center text-white`}>
                  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </div>
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Trash Dialog */}
      {showTrashDialog && (
        <div className="fixed inset-0 bg-black/50 z-[10000] flex items-center justify-center" onClick={() => setShowTrashDialog(false)}>
          <div className="bg-white rounded-lg shadow-xl p-6 max-w-md w-full" onClick={(e) => e.stopPropagation()}>
            <h2 className="text-xl font-semibold mb-4">Trash</h2>
            
            {trashItems.length === 0 ? (
              <p className="text-gray-500 text-center py-8">Trash is empty</p>
            ) : (
              <>
                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {trashItems.map(item => (
                    <div key={item.id} className="flex items-center justify-between p-2 hover:bg-gray-50 rounded">
                      <div>
                        <p className="font-medium">{item.name}</p>
                        <p className="text-xs text-gray-500">
                          Deleted {item.deletedAt.toLocaleDateString()}
                        </p>
                      </div>
                      <button
                        onClick={() => handleRestoreItem(item.id)}
                        className="text-sm text-blue-600 hover:text-blue-700"
                      >
                        Restore
                      </button>
                    </div>
                  ))}
                </div>
                
                <div className="mt-4 pt-4 border-t flex justify-between">
                  <button
                    onClick={handleEmptyTrash}
                    className="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
                  >
                    Empty Trash
                  </button>
                  <button
                    onClick={() => setShowTrashDialog(false)}
                    className="px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300"
                  >
                    Close
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
      )}
    </>
  );
};