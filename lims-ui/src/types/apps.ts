import { ComponentType } from 'react';

export interface AppDefinition {
  id: string;
  name: string;
  icon: React.ReactNode;
  component: ComponentType;
  defaultSize?: { width: number; height: number };
  dockIconClass?: string;
  category: 'laboratory' | 'data' | 'analysis' | 'admin' | 'system';
  description?: string;
}