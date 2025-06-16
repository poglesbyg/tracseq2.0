import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';

export interface User {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  role: UserRole;
  status: UserStatus;
  lab_affiliation?: string;
  department?: string;
  position?: string;
  phone?: string;
  office_location?: string;
  email_verified: boolean;
  last_login?: string;
  created_at: string;
}

export type UserRole = 
  | 'lab_administrator'
  | 'principal_investigator' 
  | 'lab_technician'
  | 'research_scientist'
  | 'data_analyst'
  | 'guest';

export type UserStatus = 'active' | 'inactive' | 'locked' | 'pending_verification';

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  user: User;
  token: string;
  expires_at: string;
}

interface AuthContextType {
  user: User | null;
  login: (credentials: LoginRequest) => Promise<void>;
  logout: () => Promise<void>;
  isLoading: boolean;
  isAuthenticated: boolean;
  hasRole: (role: UserRole | UserRole[]) => boolean;
  hasPermission: (resource: string, action: string) => boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

interface AuthProviderProps {
  children: ReactNode;
}

// Use relative URLs to go through Vite proxy
const API_BASE_URL = '';

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Initialize auth state from localStorage or auto-login as admin
  useEffect(() => {
    const initializeAuth = async () => {
      const token = localStorage.getItem('auth_token');
      if (token && token !== 'mock-admin-token') {
        try {
          const response = await fetch(`${API_BASE_URL}/api/users/me`, {
            headers: {
              'Authorization': `Bearer ${token}`,
              'Content-Type': 'application/json',
            },
          });

          if (response.ok) {
            const data = await response.json();
            // Handle both possible response formats
            const userData = data.data ? data.data : data;
            setUser(userData);
            setIsLoading(false);
            return;
          } else {
            localStorage.removeItem('auth_token');
          }
        } catch (error) {
          console.error('Failed to verify token:', error);
          localStorage.removeItem('auth_token');
        }
      }
      
      // Auto-login as admin if no valid token exists
      try {
        console.log('Auto-logging in as admin...');
        const response = await fetch(`${API_BASE_URL}/api/auth/login`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ email: 'admin@lab.local', password: 'admin123' }),
        });

        if (response.ok) {
          const data = await response.json();
          // Handle both possible response formats
          const responseData = data.data || data;
          const { user: userData, token } = responseData;
          setUser(userData);
          localStorage.setItem('auth_token', token);
          console.log('Auto-login successful');
        } else {
          throw new Error('Login request failed');
        }
      } catch (error) {
        console.log('Auto-login failed, creating mock admin user for development');
        // If login fails, create a mock admin user for development
        const mockAdminUser: User = {
          id: 'mock-admin-id',
          email: 'admin@lab.local',
          first_name: 'Admin',
          last_name: 'User',
          role: 'lab_administrator',
          status: 'active',
          email_verified: true,
          created_at: new Date().toISOString(),
        };
        setUser(mockAdminUser);
        localStorage.setItem('auth_token', 'mock-admin-token');
      }
      
      setIsLoading(false);
    };

    initializeAuth();
  }, []);

  const login = async (credentials: LoginRequest): Promise<void> => {
    try {
      const response = await fetch(`${API_BASE_URL}/api/auth/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(credentials),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error?.message || 'Login failed');
      }

      const data = await response.json();
      const { user: userData, token } = data.data;
      setUser(userData);
      localStorage.setItem('auth_token', token);
    } catch (error) {
      console.error('Login error:', error);
      throw error;
    }
  };

  const logout = async (): Promise<void> => {
    try {
      const token = localStorage.getItem('auth_token');
      if (token) {
        await fetch(`${API_BASE_URL}/api/auth/logout`, {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json',
          },
        });
      }
    } catch (error) {
      console.error('Logout error:', error);
    } finally {
      setUser(null);
      localStorage.removeItem('auth_token');
    }
  };

  const hasRole = (roles: UserRole | UserRole[]): boolean => {
    if (!user) return false;
    const roleArray = Array.isArray(roles) ? roles : [roles];
    return roleArray.includes(user.role);
  };

  const hasPermission = (resource: string, action: string): boolean => {
    if (!user) return false;
    
    // Define role permissions
    const permissions: Record<UserRole, Record<string, string[]>> = {
      lab_administrator: {
        users: ['create', 'read', 'update', 'delete'],
        samples: ['create', 'read', 'update', 'delete'],
        templates: ['create', 'read', 'update', 'delete'],
        reports: ['create', 'read', 'update', 'delete'],
        storage: ['create', 'read', 'update', 'delete'],
        sequencing: ['create', 'read', 'update', 'delete'],
      },
      principal_investigator: {
        users: ['read'],
        samples: ['create', 'read', 'update', 'delete'],
        templates: ['create', 'read', 'update', 'delete'],
        reports: ['create', 'read', 'update'],
        storage: ['create', 'read', 'update'],
        sequencing: ['create', 'read', 'update'],
      },
      research_scientist: {
        samples: ['create', 'read', 'update'],
        templates: ['read', 'update'],
        reports: ['create', 'read'],
        storage: ['read', 'update'],
        sequencing: ['create', 'read'],
      },
      lab_technician: {
        samples: ['create', 'read', 'update'],
        templates: ['read'],
        storage: ['read', 'update'],
        sequencing: ['create', 'read', 'update'],
      },
      data_analyst: {
        samples: ['read'],
        reports: ['create', 'read', 'update'],
        storage: ['read'],
        sequencing: ['read'],
      },
      guest: {
        samples: ['read'],
        reports: ['read'],
        storage: ['read'],
        sequencing: ['read'],
      },
    };

    const userPermissions = permissions[user.role];
    return userPermissions?.[resource]?.includes(action) || false;
  };

  const value: AuthContextType = {
    user,
    login,
    logout,
    isLoading,
    isAuthenticated: !!user,
    hasRole,
    hasPermission,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}; 
