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

export interface TestUser {
  email: string;
  password: string;
  firstName: string;
  lastName: string;
  role: string;
}

export interface LoginResponse {
  user: User;
  token: string;
  expires_at: string;
}

export interface AuthContextType {
  user: User | null;
  login: (credentials: LoginRequest) => Promise<void>;
  logout: () => Promise<void>;
  isLoading: boolean;
  isAuthenticated: boolean;
  hasRole: (role: UserRole | UserRole[]) => boolean;
  hasPermission: (resource: string, action: string) => boolean;
}

// Test users for E2E testing
export const getTestUsers = (): Record<string, TestUser> => {
  return {
    admin: {
      email: 'admin.test@tracseq.com',
      password: 'AdminTest123!',
      firstName: 'Admin',
      lastName: 'Test',
      role: 'lab_administrator'
    },
    researcher: {
      email: 'researcher.test@tracseq.com',
      password: 'ResearchTest123!',
      firstName: 'Research',
      lastName: 'Scientist',
      role: 'research_scientist'
    },
    technician: {
      email: 'tech.test@tracseq.com',
      password: 'TechTest123!',
      firstName: 'Lab',
      lastName: 'Technician',
      role: 'lab_technician'
    }
  };
}; 