import { render, screen, fireEvent, act } from '@testing-library/react';
import { AuthProvider, useAuth } from '..';

// Mock axios
jest.mock('axios');

// Mock fetch globally
global.fetch = jest.fn();

// Test component to use AuthContext
const TestAuthComponent = () => {
  const { user, isAuthenticated, login, logout, isLoading } = useAuth();
  
  return (
    <div>
      <div data-testid="loading">{isLoading ? 'Loading' : 'Not Loading'}</div>
      <div data-testid="auth-status">{isAuthenticated ? 'Authenticated' : 'Not Authenticated'}</div>
      <div data-testid="user-info">{user ? user.email : 'No User'}</div>
      <button onClick={() => login({ email: 'test@example.com', password: 'password' })}>Login</button>
      <button onClick={logout}>Logout</button>
    </div>
  );
};

describe('AuthContext', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    // Clear localStorage
    localStorage.clear();
    
    // Reset fetch mock
    (global.fetch as jest.Mock).mockReset();
    // Default successful login response
    (global.fetch as jest.Mock).mockResolvedValue({
      ok: true,
      json: async () => ({
        data: {
          user: {
            id: '1',
            email: 'test@example.com',
            name: 'Test User',
            role: 'lab_technician'
          },
          token: 'mock-token'
        }
      })
    });
  });

  const renderWithAuthProvider = () => {
    return render(
      <AuthProvider>
        <TestAuthComponent />
      </AuthProvider>
    );
  };

  describe('Initial State', () => {
    it('starts with unauthenticated state', () => {
      renderWithAuthProvider();
      
      expect(screen.getByTestId('auth-status')).toHaveTextContent('Not Authenticated');
      expect(screen.getByTestId('user-info')).toHaveTextContent('No User');
      expect(screen.getByTestId('loading')).toHaveTextContent('Not Loading');
    });
  });

  describe('Authentication State Management', () => {
    it('updates state during login', async () => {
      renderWithAuthProvider();
      
      const loginButton = screen.getByText('Login');
      
      act(() => {
        fireEvent.click(loginButton);
      });
      
      // Should show loading state
      expect(screen.getByTestId('loading')).toHaveTextContent('Loading');
    });

    it('handles logout correctly', async () => {
      renderWithAuthProvider();
      
      // First login
      const loginButton = screen.getByText('Login');
      fireEvent.click(loginButton);
      
      // Then logout
      const logoutButton = screen.getByText('Logout');
      fireEvent.click(logoutButton);
      
      expect(screen.getByTestId('auth-status')).toHaveTextContent('Not Authenticated');
      expect(screen.getByTestId('user-info')).toHaveTextContent('No User');
    });
  });

  describe('Token Management', () => {
    it('persists authentication in localStorage', () => {
      // Set up localStorage with token
      localStorage.setItem('auth_token', 'mock-token');
      localStorage.setItem('user', JSON.stringify({ 
        id: '1', 
        email: 'test@example.com', 
        name: 'Test User' 
      }));
      
      renderWithAuthProvider();
      
      expect(screen.getByTestId('auth-status')).toHaveTextContent('Authenticated');
      expect(screen.getByTestId('user-info')).toHaveTextContent('test@example.com');
    });

    it('clears localStorage on logout', async () => {
      // Set up localStorage
      localStorage.setItem('auth_token', 'mock-token');
      localStorage.setItem('user', JSON.stringify({ 
        id: '1', 
        email: 'test@example.com' 
      }));
      
      renderWithAuthProvider();
      
      const logoutButton = screen.getByText('Logout');
      fireEvent.click(logoutButton);
      
      expect(localStorage.getItem('auth_token')).toBeNull();
      expect(localStorage.getItem('user')).toBeNull();
    });
  });

  describe('Error Handling', () => {
    it('handles invalid localStorage data gracefully', () => {
      // Set invalid JSON in localStorage
      localStorage.setItem('user', 'invalid-json');
      
      expect(() => renderWithAuthProvider()).not.toThrow();
      expect(screen.getByTestId('auth-status')).toHaveTextContent('Not Authenticated');
    });

    it('handles missing localStorage data gracefully', () => {
      renderWithAuthProvider();
      
      expect(screen.getByTestId('auth-status')).toHaveTextContent('Not Authenticated');
      expect(screen.getByTestId('user-info')).toHaveTextContent('No User');
    });
  });

  describe('useAuth Hook', () => {
    it('throws error when used outside AuthProvider', () => {
      // Spy on console.error to prevent error output in test
      const consoleSpy = jest.spyOn(console, 'error').mockImplementation(() => {});
      
      expect(() => {
        render(<TestAuthComponent />);
      }).toThrow('useAuth must be used within an AuthProvider');
      
      consoleSpy.mockRestore();
    });
  });
});