import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import {
  HomeIcon,
  DocumentIcon,
  BeakerIcon,
  QueueListIcon,
  MapPinIcon,
  ChartBarIcon,
  Bars3Icon,
  XMarkIcon,
  TableCellsIcon,
  UsersIcon,
  SparklesIcon,
  CpuChipIcon,
} from '@heroicons/react/24/outline';
import { UserMenu } from './UserMenu';
import { useAuth } from '../auth/AuthContext';

interface LayoutProps {
  children: React.ReactNode;
}

export default function Layout({ children }: LayoutProps) {
  const location = useLocation();
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const { hasRole } = useAuth();

  const navigation = [
    { name: 'Dashboard', href: '/dashboard', icon: HomeIcon, testId: 'nav-dashboard' },
    { name: 'AI Submissions', href: '/rag-submissions', icon: SparklesIcon, testId: 'nav-rag-submissions' },
    { name: 'AI Samples', href: '/rag-samples', icon: CpuChipIcon, testId: 'nav-rag-samples' },
    { name: 'Templates', href: '/templates', icon: DocumentIcon, testId: 'nav-templates' },
    { name: 'Samples', href: '/samples', icon: BeakerIcon, testId: 'nav-samples' },
    { name: 'Sequencing', href: '/sequencing', icon: QueueListIcon, testId: 'nav-sequencing' },
    { name: 'Spreadsheets', href: '/spreadsheets', icon: TableCellsIcon, testId: 'nav-spreadsheets' },
    { name: 'Storage', href: '/storage', icon: MapPinIcon, testId: 'nav-storage' },
    { name: 'Reports', href: '/reports', icon: ChartBarIcon, testId: 'nav-reports' },
    // Only show Users for administrators
    ...(hasRole('lab_administrator') ? [{ name: 'Users', href: '/users', icon: UsersIcon, testId: 'nav-users' }] : []),
  ];

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Mobile sidebar overlay */}
      {sidebarOpen && (
        <div className="fixed inset-0 z-40 flex md:hidden">
          <div className="fixed inset-0 bg-gray-600 bg-opacity-75" onClick={() => setSidebarOpen(false)} />
          <div className="relative flex w-full max-w-xs flex-1 flex-col bg-white">
            <div className="absolute top-0 right-0 -mr-12 pt-2">
              <button
                type="button"
                className="ml-1 flex h-10 w-10 items-center justify-center rounded-full focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
                onClick={() => setSidebarOpen(false)}
              >
                <XMarkIcon className="h-6 w-6 text-white" aria-hidden="true" />
              </button>
            </div>
            <div className="h-0 flex-1 overflow-y-auto pt-5 pb-4">
              <div className="flex flex-shrink-0 items-center px-4">
                <h1 className="text-xl font-bold text-indigo-600">Lab Manager</h1>
              </div>
              <nav className="mt-5 space-y-1 px-2">
                {navigation.map((item) => {
                  const isActive = location.pathname === item.href;
                  return (
                    <Link
                      key={item.name}
                      to={item.href}
                      className={`${
                        isActive
                          ? 'bg-indigo-50 text-indigo-700 border-r-4 border-indigo-700'
                          : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
                      } sidebar-nav-item`}
                      onClick={() => setSidebarOpen(false)}
                      data-testid={item.testId}
                    >
                      <item.icon
                        className={`${
                          isActive ? 'text-indigo-500' : 'text-gray-400 group-hover:text-gray-500'
                        } mr-3 h-6 w-6 flex-shrink-0`}
                        aria-hidden="true"
                      />
                      {item.name}
                    </Link>
                  );
                })}
              </nav>
            </div>
          </div>
        </div>
      )}

      {/* Desktop sidebar */}
      <div className="hidden md:fixed md:inset-y-0 md:flex md:w-64 md:flex-col">
        <div className="flex min-h-0 flex-1 flex-col border-r border-gray-200 bg-white">
          <div className="flex flex-1 flex-col overflow-y-auto pt-5 pb-4">
            <div className="flex flex-shrink-0 items-center px-4">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <div className="h-8 w-8 bg-indigo-600 rounded-lg flex items-center justify-center">
                    <BeakerIcon className="h-5 w-5 text-white" />
                  </div>
                </div>
                <div className="ml-3">
                  <h1 className="text-xl font-bold text-indigo-600">Lab Manager</h1>
                  <p className="text-xs text-gray-500">v1.0.0</p>
                </div>
              </div>
            </div>
            <nav className="mt-8 flex-1 space-y-1 px-2">
              {navigation.map((item) => {
                const isActive = location.pathname === item.href;
                return (
                  <Link
                    key={item.name}
                    to={item.href}
                    className={`${
                      isActive
                        ? 'bg-indigo-50 text-indigo-700 border-r-4 border-indigo-700'
                        : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
                    } sidebar-nav-item group`}
                    data-testid={item.testId}
                  >
                    <item.icon
                      className={`${
                        isActive ? 'text-indigo-500' : 'text-gray-400 group-hover:text-gray-500'
                      } mr-3 h-6 w-6 flex-shrink-0`}
                      aria-hidden="true"
                    />
                    {item.name}
                  </Link>
                );
              })}
            </nav>
          </div>
          
          {/* Sidebar footer */}
          <div className="flex flex-shrink-0 border-t border-gray-200 p-4">
            <div className="flex w-full items-center">
              <div className="ml-3">
                <p className="text-sm font-medium text-gray-700">Lab Manager</p>
                <p className="text-xs text-gray-500">Laboratory Management System</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Main content area */}
      <div className="md:pl-64 flex flex-col flex-1">
        {/* Mobile header */}
        <div className="sticky top-0 z-10 bg-white pl-1 pt-1 sm:pl-3 sm:pt-3 md:hidden border-b border-gray-200">
          <div className="flex items-center justify-between px-4 py-2">
            <button
              type="button"
              className="-ml-0.5 -mt-0.5 inline-flex h-12 w-12 items-center justify-center rounded-md text-gray-500 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-500"
              onClick={() => setSidebarOpen(true)}
              data-testid="mobile-menu-button"
            >
              <Bars3Icon className="h-6 w-6" aria-hidden="true" />
            </button>
            <h1 className="text-lg font-semibold text-gray-900">Lab Manager</h1>
            <div className="flex-shrink-0">
              <UserMenu />
            </div>
          </div>
        </div>

        {/* Desktop header */}
        <div className="hidden md:flex md:items-center md:justify-end md:px-6 md:py-4 md:bg-white md:border-b md:border-gray-200">
          <UserMenu />
        </div>

        {/* Main content */}
        <main className="flex-1">
          <div className="min-h-screen bg-gray-50">
            {children}
          </div>
        </main>
      </div>
    </div>
  );
} 
