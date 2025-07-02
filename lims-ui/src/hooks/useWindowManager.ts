import { useState, useCallback } from 'react';
import { WindowState } from '../components/Desktop/Window';

export const useWindowManager = () => {
  const [windows, setWindows] = useState<WindowState[]>([]);

  const openWindow = useCallback((window: WindowState) => {
    setWindows(prev => [...prev, { ...window, isFocused: true }].map((w, _idx, arr) => ({
      ...w,
      isFocused: w.id === window.id,
      zIndex: w.id === window.id ? 1000 + arr.length : w.zIndex
    })));
  }, []);

  const closeWindow = useCallback((windowId: string) => {
    setWindows(prev => prev.filter(w => w.id !== windowId));
  }, []);

  const focusWindow = useCallback((windowId: string) => {
    setWindows(prev => {
      const maxZ = Math.max(...prev.map(w => w.zIndex), 1000);
      return prev.map(w => ({
        ...w,
        isFocused: w.id === windowId,
        zIndex: w.id === windowId ? maxZ + 1 : w.zIndex
      }));
    });
  }, []);

  const updateWindow = useCallback((windowId: string, updates: Partial<WindowState>) => {
    setWindows(prev => prev.map(w => 
      w.id === windowId ? { ...w, ...updates } : w
    ));
  }, []);

  const minimizeWindow = useCallback((windowId: string) => {
    updateWindow(windowId, { isMinimized: true });
  }, [updateWindow]);

  const restoreWindow = useCallback((windowId: string) => {
    updateWindow(windowId, { isMinimized: false });
    focusWindow(windowId);
  }, [updateWindow, focusWindow]);

  return {
    windows,
    openWindow,
    closeWindow,
    focusWindow,
    updateWindow,
    minimizeWindow,
    restoreWindow
  };
};