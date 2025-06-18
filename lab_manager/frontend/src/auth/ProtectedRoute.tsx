import React, { useState } from 'react';
import { useAuth, UserRole } from './AuthContext';
import { LoginModal } from './LoginModal';

interface ProtectedRouteProps {
  children: React.ReactNode;
  requireRoles?: UserRole | UserRole[];
  requirePermission?: {
    resource: string;
    action: string;
  };
  fallback?: React.ReactNode;
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({
  children,
  requireRoles,
  requirePermission,
  fallback,
}) => {
  const { isAuthenticated, isLoading, hasRole, hasPermission } = useAuth();
  const [showLoginModal, setShowLoginModal] = useState(false);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      fallback || (
        <>
          <div className="min-h-screen flex items-center justify-center bg-gray-50">
            <div className="max-w-md w-full space-y-8">
              <div className="text-center">
                <h2 className="mt-6 text-3xl font-extrabold text-gray-900">
                  Lab Manager
                </h2>
                <p className="mt-2 text-sm text-gray-600">
                  Laboratory Management System
                </p>
              </div>
              <div className="mt-8 space-y-6">
                <button
                  onClick={() => setShowLoginModal(true)}
                  className="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                >
                  Sign In
                </button>
              </div>
              <div className="text-center">
                <p className="text-xs text-gray-500">
                  Access the laboratory management system to manage samples, templates, and reports.
                </p>
              </div>
            </div>
          </div>
          <LoginModal 
            isOpen={showLoginModal} 
            onClose={() => setShowLoginModal(false)} 
          />
        </>
      )
    );
  }

  // Check role requirements
  if (requireRoles && !hasRole(requireRoles)) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="max-w-md w-full space-y-8">
          <div>
            <h2 className="mt-6 text-center text-3xl font-extrabold text-gray-900">
              Access Denied
            </h2>
            <p className="mt-2 text-center text-sm text-gray-600">
              You don't have permission to access this page
            </p>
          </div>
        </div>
      </div>
    );
  }

  // Check permission requirements
  if (requirePermission && !hasPermission(requirePermission.resource, requirePermission.action)) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="max-w-md w-full space-y-8">
          <div>
            <h2 className="mt-6 text-center text-3xl font-extrabold text-gray-900">
              Insufficient Permissions
            </h2>
            <p className="mt-2 text-center text-sm text-gray-600">
              You don't have the required permissions to access this resource
            </p>
          </div>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}; 
