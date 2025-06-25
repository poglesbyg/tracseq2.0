import React, { useState, useRef, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { 
  UserIcon, 
  Cog6ToothIcon, 
  ArrowRightOnRectangleIcon,
  ChevronDownIcon,
  UserCircleIcon
} from '@heroicons/react/24/outline';
import { useAuth } from '../auth/AuthContext';

export const UserMenu: React.FC = () => {
  const [isOpen, setIsOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const { user, logout, hasRole } = useAuth();

  const getRoleDisplayName = (role: string): string => {
    const roleMap: Record<string, string> = {
      'lab_administrator': 'Lab Administrator',
      'principal_investigator': 'Principal Investigator',
      'lab_technician': 'Lab Technician',
      'research_scientist': 'Research Scientist',
      'data_analyst': 'Data Analyst',
      'guest': 'Guest',
    };
    return roleMap[role] || role;
  };

  const getRoleBadgeColor = (role: string): string => {
    const colorMap: Record<string, string> = {
      'lab_administrator': 'bg-red-100 text-red-800',
      'principal_investigator': 'bg-purple-100 text-purple-800',
      'lab_technician': 'bg-blue-100 text-blue-800',
      'research_scientist': 'bg-green-100 text-green-800',
      'data_analyst': 'bg-yellow-100 text-yellow-800',
      'guest': 'bg-gray-100 text-gray-800',
    };
    return colorMap[role] || 'bg-gray-100 text-gray-800';
  };

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const handleLogout = async () => {
    try {
      await logout();
      setIsOpen(false);
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };

  if (!user) {
    return null;
  }

  return (
    <div className="relative" ref={menuRef}>
      <button
        type="button"
        className="flex items-center text-sm rounded-full bg-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
        onClick={() => setIsOpen(!isOpen)}
        aria-expanded="false"
        aria-haspopup="true"
        data-testid="user-menu"
      >
        <span className="sr-only">Open user menu</span>
        <div className="flex items-center space-x-3 px-3 py-2 rounded-lg hover:bg-gray-50 transition-colors">
          <div className="flex-shrink-0">
            <UserCircleIcon className="h-8 w-8 text-gray-400" />
          </div>
          <div className="hidden md:block text-left">
            <div className="text-sm font-medium text-gray-900">
              {user.first_name} {user.last_name}
            </div>
            <div className="text-xs text-gray-500">
              {user.department || user.lab_affiliation || getRoleDisplayName(user.role)}
            </div>
          </div>
          <ChevronDownIcon 
            className={`h-4 w-4 text-gray-400 transition-transform ${isOpen ? 'rotate-180' : ''}`} 
          />
        </div>
      </button>

      {isOpen && (
        <div className="absolute right-0 z-10 mt-2 w-80 origin-top-right rounded-md bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none">
          {/* User info section */}
          <div className="px-4 py-3 border-b border-gray-200">
            <div className="flex items-center space-x-3">
              <UserCircleIcon className="h-10 w-10 text-gray-400" />
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-gray-900 truncate">
                  {user.first_name} {user.last_name}
                </p>
                <p className="text-sm text-gray-500 truncate">
                  {user.email}
                </p>
                <div className="mt-1">
                  <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getRoleBadgeColor(user.role)}`}>
                    {getRoleDisplayName(user.role)}
                  </span>
                </div>
              </div>
            </div>
            {(user.department || user.lab_affiliation || user.position) && (
              <div className="mt-2 text-xs text-gray-500">
                {[user.position, user.department, user.lab_affiliation].filter(Boolean).join(' • ')}
              </div>
            )}
          </div>

          {/* Menu items */}
          <div className="py-1">
            <Link
              to="/profile"
              className="flex items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 transition-colors"
              onClick={() => setIsOpen(false)}
            >
              <UserIcon className="mr-3 h-4 w-4 text-gray-400" />
              Profile Settings
            </Link>

            {hasRole(['lab_administrator']) && (
              <Link
                to="/users"
                className="flex items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 transition-colors"
                onClick={() => setIsOpen(false)}
              >
                <Cog6ToothIcon className="mr-3 h-4 w-4 text-gray-400" />
                User Management
              </Link>
            )}
          </div>

          <div className="border-t border-gray-200">
            <button
              onClick={handleLogout}
              className="flex w-full items-center px-4 py-2 text-sm text-red-700 hover:bg-red-50 transition-colors"
              data-testid="logout-button"
            >
              <ArrowRightOnRectangleIcon className="mr-3 h-4 w-4 text-red-400" />
              Sign out
            </button>
          </div>
        </div>
      )}
    </div>
  );
}; 
