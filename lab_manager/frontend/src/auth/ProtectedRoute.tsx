import React from 'react';
import { useAuth, UserRole } from './AuthContext';

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
        <div className="min-h-screen flex items-center justify-center bg-gray-50">
          <div className="max-w-md w-full space-y-8">
            <div>
              <h2 className="mt-6 text-center text-3xl font-extrabold text-gray-900">
                Authentication Required
              </h2>
              <p className="mt-2 text-center text-sm text-gray-600">
                Please log in to access this page
              </p>
            </div>
          </div>
        </div>
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
