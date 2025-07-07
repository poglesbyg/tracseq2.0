import React, { useRef } from 'react';
import { useDraggable } from '../../hooks/useDraggable';
import { useResizable } from '../../hooks/useResizable';

export interface WindowContext {
  windowId: string;
  openApp?: (appId: string, context?: Record<string, unknown>) => void;
  closeWindow: () => void;
  updateWindow: (updates: Partial<WindowState>) => void;
}

export interface WindowComponentProps {
  windowContext?: WindowContext;
}

export interface WindowState {
  id: string;
  appId: string;
  title: string;
  icon?: React.ReactNode;
  component: React.ComponentType<WindowComponentProps>;
  position: { x: number; y: number };
  size: { width: number; height: number };
  isMinimized: boolean;
  isMaximized: boolean;
  isFocused?: boolean;
  zIndex: number;
}

interface WindowProps {
  window: WindowState;
  onClose: () => void;
  onFocus: () => void;
  onUpdate: (updates: Partial<WindowState>) => void;
  onOpenApp?: (appId: string, context?: Record<string, unknown>) => void;
}

export const Window: React.FC<WindowProps> = ({ window, onClose, onFocus, onUpdate, onOpenApp }) => {
  const windowRef = useRef<HTMLDivElement>(null);

  const { isDragging, handleMouseDown } = useDraggable(
    windowRef,
    window.position,
    (newPosition) => onUpdate({ position: newPosition })
  );

  const { handleResizeStart } = useResizable(
    windowRef,
    window.size,
    (newSize) => onUpdate({ size: newSize })
  );

  const handleMinimize = () => {
    onUpdate({ isMinimized: true });
  };

  const handleMaximize = () => {
    if (window.isMaximized) {
      onUpdate({ 
        isMaximized: false,
        position: { x: 100, y: 100 },
        size: { width: 800, height: 600 }
      });
    } else {
      onUpdate({ 
        isMaximized: true,
        position: { x: 0, y: 32 }, // Account for menu bar
        size: { width: globalThis.window.innerWidth, height: globalThis.window.innerHeight - 112 } // Menu bar + dock
      });
    }
  };

  const Component = window.component;

  // Create window context to pass to the component
  const windowContext = {
    windowId: window.id,
    openApp: onOpenApp,
    closeWindow: onClose,
    updateWindow: (updates: Partial<WindowState>) => onUpdate(updates),
  };

  if (window.isMinimized) {
    return null;
  }

  return (
    <div
      ref={windowRef}
      className={`absolute bg-white/95 backdrop-blur-xl rounded-xl shadow-2xl overflow-hidden transition-all ${
        window.isFocused ? 'ring-2 ring-blue-500/50' : ''
      } ${isDragging ? 'cursor-grabbing' : ''}`}
      style={{
        left: window.position.x,
        top: window.position.y,
        width: window.size.width,
        height: window.size.height,
        zIndex: window.zIndex,
      }}
      onMouseDown={onFocus}
    >
      {/* Window Header */}
      <div
        className="h-8 bg-gradient-to-b from-gray-100 to-gray-200 flex items-center px-3 select-none cursor-grab"
        onMouseDown={handleMouseDown}
      >
        {/* Traffic Light Buttons */}
        <div className="flex items-center gap-2">
          <button
            onClick={onClose}
            className="w-3 h-3 bg-red-500 rounded-full hover:bg-red-600 transition-colors"
            title="Close"
          />
          <button
            onClick={handleMinimize}
            className="w-3 h-3 bg-yellow-500 rounded-full hover:bg-yellow-600 transition-colors"
            title="Minimize"
          />
          <button
            onClick={handleMaximize}
            className="w-3 h-3 bg-green-500 rounded-full hover:bg-green-600 transition-colors"
            title="Maximize"
          />
        </div>

        {/* Window Title */}
        <div className="flex-1 text-center">
          <span className="text-sm font-medium text-gray-700">{window.title}</span>
        </div>
      </div>

      {/* Window Content */}
      <div className="flex-1 overflow-auto" style={{ height: 'calc(100% - 2rem)' }}>
        <Component windowContext={windowContext} />
      </div>

      {/* Resize Handles */}
      {!window.isMaximized && (
        <>
          <div
            className="absolute bottom-0 right-0 w-4 h-4 cursor-nwse-resize"
            onMouseDown={(e) => handleResizeStart(e, 'se')}
          />
          <div
            className="absolute bottom-0 left-0 right-4 h-1 cursor-ns-resize"
            onMouseDown={(e) => handleResizeStart(e, 's')}
          />
          <div
            className="absolute top-8 bottom-4 right-0 w-1 cursor-ew-resize"
            onMouseDown={(e) => handleResizeStart(e, 'e')}
          />
        </>
      )}
    </div>
  );
};