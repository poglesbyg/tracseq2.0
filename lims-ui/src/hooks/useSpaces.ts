import { useState, useCallback } from 'react';
import { Space } from '../components/Desktop/MissionControl';

export const useSpaces = () => {
  const [spaces, setSpaces] = useState<Space[]>([
    {
      id: 'primary',
      name: 'Desktop 1',
      windowIds: []
    }
  ]);
  const [currentSpaceId, setCurrentSpaceId] = useState('primary');

  const createSpace = useCallback(() => {
    const newSpace: Space = {
      id: `space-${Date.now()}`,
      name: `Desktop ${spaces.length + 1}`,
      windowIds: []
    };
    setSpaces(prev => [...prev, newSpace]);
  }, [spaces.length]);

  const deleteSpace = useCallback((spaceId: string) => {
    if (spaceId === 'primary') return; // Can't delete primary space
    
    setSpaces(prev => {
      const updatedSpaces = prev.filter(s => s.id !== spaceId);
      
      // Move windows from deleted space to primary
      const deletedSpace = prev.find(s => s.id === spaceId);
      if (deletedSpace && deletedSpace.windowIds.length > 0) {
        const primarySpaceIndex = updatedSpaces.findIndex(s => s.id === 'primary');
        if (primarySpaceIndex !== -1) {
          // Create a new object instead of mutating the existing one
          updatedSpaces[primarySpaceIndex] = {
            ...updatedSpaces[primarySpaceIndex],
            windowIds: [...updatedSpaces[primarySpaceIndex].windowIds, ...deletedSpace.windowIds]
          };
        }
      }
      
      return updatedSpaces;
    });
    
    // If current space was deleted, switch to primary
    if (currentSpaceId === spaceId) {
      setCurrentSpaceId('primary');
    }
  }, [currentSpaceId]);

  const changeSpace = useCallback((spaceId: string) => {
    setCurrentSpaceId(spaceId);
  }, []);

  const addWindowToSpace = useCallback((windowId: string, spaceId?: string) => {
    const targetSpaceId = spaceId || currentSpaceId;
    setSpaces(prev => prev.map(space => {
      if (space.id === targetSpaceId) {
        return {
          ...space,
          windowIds: [...space.windowIds, windowId]
        };
      }
      return space;
    }));
  }, [currentSpaceId]);

  const removeWindowFromSpace = useCallback((windowId: string) => {
    setSpaces(prev => prev.map(space => ({
      ...space,
      windowIds: space.windowIds.filter(id => id !== windowId)
    })));
  }, []);

  const moveWindowToSpace = useCallback((windowId: string, targetSpaceId: string) => {
    setSpaces(prev => prev.map(space => {
      // Remove from all spaces
      const filteredIds = space.windowIds.filter(id => id !== windowId);
      
      // Add to target space
      if (space.id === targetSpaceId) {
        return {
          ...space,
          windowIds: [...filteredIds, windowId]
        };
      }
      
      return {
        ...space,
        windowIds: filteredIds
      };
    }));
  }, []);

  const getWindowsForCurrentSpace = useCallback((allWindowIds: string[]) => {
    const currentSpace = spaces.find(s => s.id === currentSpaceId);
    if (!currentSpace) return [];
    return allWindowIds.filter(id => currentSpace.windowIds.includes(id));
  }, [spaces, currentSpaceId]);

  return {
    spaces,
    currentSpaceId,
    createSpace,
    deleteSpace,
    changeSpace,
    addWindowToSpace,
    removeWindowFromSpace,
    moveWindowToSpace,
    getWindowsForCurrentSpace
  };
};