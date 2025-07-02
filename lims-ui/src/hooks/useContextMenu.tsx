import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';

interface MenuItem {
  label?: string;
  icon?: ReactNode;
  action?: () => void;
  divider?: boolean;
  disabled?: boolean;
  submenu?: MenuItem[];
}

interface ContextMenuState {
  isOpen: boolean;
  position: { x: number; y: number };
  items: MenuItem[];
}

interface ContextMenuContextType {
  showContextMenu: (e: React.MouseEvent, items: MenuItem[]) => void;
  hideContextMenu: () => void;
}

const ContextMenuContext = createContext<ContextMenuContextType | null>(null);

export const useContextMenu = () => {
  const context = useContext(ContextMenuContext);
  if (!context) {
    throw new Error('useContextMenu must be used within ContextMenuProvider');
  }
  return context;
};

interface ContextMenuProviderProps {
  children: ReactNode;
}

export const ContextMenuProvider: React.FC<ContextMenuProviderProps> = ({ children }) => {
  const [menuState, setMenuState] = useState<ContextMenuState>({
    isOpen: false,
    position: { x: 0, y: 0 },
    items: []
  });

  const showContextMenu = useCallback((e: React.MouseEvent, items: MenuItem[]) => {
    e.preventDefault();
    setMenuState({
      isOpen: true,
      position: { x: e.clientX, y: e.clientY },
      items
    });
  }, []);

  const hideContextMenu = useCallback(() => {
    setMenuState(prev => ({ ...prev, isOpen: false }));
  }, []);

  return (
    <ContextMenuContext.Provider value={{ showContextMenu, hideContextMenu }}>
      {children}
      {menuState.isOpen && (
        <ContextMenu
          position={menuState.position}
          items={menuState.items}
          onClose={hideContextMenu}
        />
      )}
    </ContextMenuContext.Provider>
  );
};

interface ContextMenuProps {
  position: { x: number; y: number };
  items: MenuItem[];
  onClose: () => void;
}

const ContextMenu: React.FC<ContextMenuProps> = ({ position, items, onClose }) => {
  const renderMenuItem = (item: MenuItem, index: number) => {
    if (item.divider) {
      return <div key={index} className="h-px bg-gray-200 dark:bg-gray-700 my-1" />;
    }

    return (
      <button
        key={index}
        onClick={() => {
          if (item.action && !item.disabled) {
            item.action();
            onClose();
          }
        }}
        disabled={item.disabled}
        className={`w-full px-4 py-2 text-sm text-left flex items-center gap-3 ${
          item.disabled 
            ? 'text-gray-400 cursor-not-allowed' 
            : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
        }`}
      >
        {item.icon && <span className="w-4 h-4">{item.icon}</span>}
        <span className="flex-1">{item.label || ''}</span>
        {item.submenu && (
          <svg className="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        )}
      </button>
    );
  };

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-[9998] bg-transparent"
        onContextMenu={(e) => e.preventDefault()}
        onClick={onClose}
      />
      
      {/* Menu */}
      <div
        className="fixed z-[9999] min-w-[200px] bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 py-1 animate-in fade-in zoom-in-95 duration-100"
        style={{
          left: Math.min(position.x, window.innerWidth - 220),
          top: Math.min(position.y, window.innerHeight - 300)
        }}
      >
        {items.map((item, index) => renderMenuItem(item, index))}
      </div>
    </>
  );
};