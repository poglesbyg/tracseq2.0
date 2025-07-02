import { ComponentType } from 'react';

export interface WindowContext {
  windowId: string;
  openApp?: (appId: string, context?: any) => void;
  closeWindow: () => void;
  updateWindow: (updates: any) => void;
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