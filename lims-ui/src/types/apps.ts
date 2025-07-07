import { ComponentType } from 'react';
import { WindowState } from '../components/Desktop/Window';

export interface WindowContext {
  windowId: string;
  openApp?: (appId: string, context?: Record<string, unknown>) => void;
  closeWindow: () => void;
  updateWindow: (updates: Partial<WindowState>) => void;
}

export interface AppDefinition {
  id: string;
  name: string;
  icon: React.ReactNode;
  component: ComponentType<{ windowContext?: WindowContext }>;
  defaultSize?: { width: number; height: number };
  dockIconClass?: string;
  category: 'laboratory' | 'data' | 'analysis' | 'admin' | 'system';
  description?: string;
}