import React, { useState } from 'react';
import { WindowState } from './Window';
import { PlusIcon, XMarkIcon } from '@heroicons/react/24/outline';

interface MissionControlProps {
  isOpen: boolean;
  onClose: () => void;
  windows: WindowState[];
  spaces: Space[];
  currentSpaceId: string;
  onWindowClick: (windowId: string) => void;
  onSpaceChange: (spaceId: string) => void;
  onCreateSpace: () => void;
  onDeleteSpace: (spaceId: string) => void;
  onMoveWindowToSpace: (windowId: string, spaceId: string) => void;
}

export interface Space {
  id: string;
  name: string;
  thumbnail?: string;
  windowIds: string[];
}

export const MissionControl: React.FC<MissionControlProps> = ({
  isOpen,
  onClose,
  windows,
  spaces,
  currentSpaceId,
  onWindowClick,
  onSpaceChange,
  onCreateSpace,
  onDeleteSpace,
  onMoveWindowToSpace,
}) => {
  const [draggedWindowId, setDraggedWindowId] = useState<string | null>(null);
  const [hoveredSpaceId, setHoveredSpaceId] = useState<string | null>(null);

  if (!isOpen) return null;

  const getWindowsForSpace = (spaceId: string) => {
    const space = spaces.find(s => s.id === spaceId);
    if (!space) return [];
    return windows.filter(w => space.windowIds.includes(w.id));
  };

  const handleDragStart = (e: React.DragEvent, windowId: string) => {
    setDraggedWindowId(windowId);
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  };

  const handleDrop = (e: React.DragEvent, spaceId: string) => {
    e.preventDefault();
    if (draggedWindowId) {
      onMoveWindowToSpace(draggedWindowId, spaceId);
      setDraggedWindowId(null);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    }
  };

  return (
    <div 
      className="fixed inset-0 z-[9999] bg-black/50 backdrop-blur-md"
      onClick={onClose}
      onKeyDown={handleKeyDown}
      tabIndex={-1}
    >
      <div className="h-full flex flex-col p-8" onClick={(e) => e.stopPropagation()}>
        {/* Spaces Bar */}
        <div className="flex items-center justify-center mb-8">
          <div className="flex items-center gap-4 bg-white/10 backdrop-blur-xl rounded-2xl p-2">
            {spaces.map((space) => (
              <div key={space.id} className="relative">
                <button
                  onClick={() => onSpaceChange(space.id)}
                  onDragOver={handleDragOver}
                  onDrop={(e) => handleDrop(e, space.id)}
                  onDragEnter={() => setHoveredSpaceId(space.id)}
                  onDragLeave={() => setHoveredSpaceId(null)}
                  className={`relative group px-6 py-3 rounded-xl transition-all ${
                    currentSpaceId === space.id
                      ? 'bg-white/20 text-white'
                      : 'hover:bg-white/10 text-white/70'
                  } ${hoveredSpaceId === space.id ? 'ring-2 ring-blue-500' : ''}`}
                >
                  <span className="text-sm font-medium">{space.name}</span>
                  <span className="ml-2 text-xs opacity-60">
                    ({getWindowsForSpace(space.id).length})
                  </span>
                  
                  {/* Delete button for non-primary spaces */}
                  {space.id !== 'primary' && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onDeleteSpace(space.id);
                      }}
                      className="absolute -top-2 -right-2 w-5 h-5 bg-red-500 rounded-full opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
                    >
                      <XMarkIcon className="w-3 h-3 text-white" />
                    </button>
                  )}
                </button>
              </div>
            ))}
            
            {/* Add Space Button */}
            <button
              onClick={onCreateSpace}
              className="px-4 py-3 rounded-xl hover:bg-white/10 text-white/70 transition-all flex items-center gap-2"
            >
              <PlusIcon className="w-4 h-4" />
              <span className="text-sm">New Desktop</span>
            </button>
          </div>
        </div>

        {/* Windows Grid */}
        <div className="flex-1 overflow-auto">
          {spaces.map((space) => (
            <div
              key={space.id}
              className={`mb-12 ${space.id !== currentSpaceId ? 'opacity-50' : ''}`}
            >
              <h3 className="text-white/60 text-sm font-medium mb-4 text-center">
                {space.name}
              </h3>
              
              <div className="grid grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6 max-w-7xl mx-auto">
                {getWindowsForSpace(space.id).map((window) => (
                  <div
                    key={window.id}
                    draggable
                    onDragStart={(e) => handleDragStart(e, window.id)}
                    onClick={() => {
                      onSpaceChange(space.id);
                      onWindowClick(window.id);
                      onClose();
                    }}
                    className="group cursor-pointer"
                  >
                    <div className="relative aspect-[4/3] bg-white/10 backdrop-blur-sm rounded-lg overflow-hidden border border-white/20 transition-all hover:scale-105 hover:shadow-2xl">
                      {/* Window Preview */}
                      <div className="absolute inset-0 p-2">
                        <div className="w-full h-full bg-white/90 rounded flex items-center justify-center">
                          <div className="text-gray-600 text-xs text-center">
                            <div className="mb-1">{window.icon}</div>
                            <div className="font-medium">{window.title}</div>
                          </div>
                        </div>
                      </div>
                      
                      {/* Hover Overlay */}
                      <div className="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors" />
                    </div>
                    
                    {/* Window Title */}
                    <div className="mt-2 text-white/80 text-sm font-medium text-center truncate">
                      {window.title}
                    </div>
                  </div>
                ))}
                
                {/* Empty State */}
                {getWindowsForSpace(space.id).length === 0 && (
                  <div className="col-span-full text-center py-12">
                    <div className="text-white/40 text-sm">No windows in this desktop</div>
                    {space.id !== currentSpaceId && (
                      <div className="text-white/30 text-xs mt-1">Drag windows here to move them</div>
                    )}
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>

        {/* Instructions */}
        <div className="text-center text-white/40 text-xs mt-4">
          Press <kbd className="px-1.5 py-0.5 bg-white/10 rounded">Esc</kbd> to exit Mission Control
        </div>
      </div>
    </div>
  );
};