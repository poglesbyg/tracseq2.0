import { useState, useCallback, useEffect, RefObject } from 'react';

interface Size {
  width: number;
  height: number;
}

type ResizeDirection = 'n' | 's' | 'e' | 'w' | 'ne' | 'nw' | 'se' | 'sw';

export const useResizable = (
  _elementRef: RefObject<HTMLElement>,
  initialSize: Size,
  onSizeChange: (size: Size) => void,
  minSize: Size = { width: 300, height: 200 }
) => {
  const [isResizing, setIsResizing] = useState(false);
  const [resizeDirection, setResizeDirection] = useState<ResizeDirection>('se');
  const [resizeStart, setResizeStart] = useState({ x: 0, y: 0, width: 0, height: 0 });

  const handleResizeStart = useCallback((e: React.MouseEvent, direction: ResizeDirection) => {
    e.preventDefault();
    e.stopPropagation();
    setIsResizing(true);
    setResizeDirection(direction);
    setResizeStart({
      x: e.clientX,
      y: e.clientY,
      width: initialSize.width,
      height: initialSize.height
    });
  }, [initialSize]);

  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      const deltaX = e.clientX - resizeStart.x;
      const deltaY = e.clientY - resizeStart.y;
      
      let newWidth = resizeStart.width;
      let newHeight = resizeStart.height;

      switch (resizeDirection) {
        case 'e':
          newWidth = Math.max(minSize.width, resizeStart.width + deltaX);
          break;
        case 's':
          newHeight = Math.max(minSize.height, resizeStart.height + deltaY);
          break;
        case 'se':
          newWidth = Math.max(minSize.width, resizeStart.width + deltaX);
          newHeight = Math.max(minSize.height, resizeStart.height + deltaY);
          break;
        // Add more directions as needed
      }

      onSizeChange({ width: newWidth, height: newHeight });
    };

    const handleMouseUp = () => {
      setIsResizing(false);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isResizing, resizeDirection, resizeStart, minSize, onSizeChange]);

  return { isResizing, handleResizeStart };
};