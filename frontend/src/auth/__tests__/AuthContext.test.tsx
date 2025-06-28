import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import { AuthProvider, useAuth } from '..';

// Mock axios
jest.mock('axios');

// Mock fetch globally
const mockFetch = jest.fn();
(window as unknown as { fetch: jest.Mock }).fetch = mockFetch;

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
    mockFetch.mockReset();
    // Default successful login response
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        data: {
          user: {
            id: '1',
            email: 'test@example.com',
            first_name: 'Test',
            last_name: 'User',
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
    it('persists authentication in localStorage', async () => {
      renderWithAuthProvider();
      
      expect(screen.getByTestId('auth-status')).toHaveTextContent('Not Authenticated');
      
      const loginButton = screen.getByText('Login');
      
      await act(async () => {
        fireEvent.click(loginButton);
        // Wait for the async operation to complete
        await waitFor(() => {
          expect(screen.getByTestId('auth-status')).toHaveTextContent('Authenticated');
        });
      });
      
      expect(localStorage.getItem('auth_token')).toBe('mock-token');
      expect(screen.getByTestId('user-info')).toHaveTextContent('test@example.com');
    });

    it('clears localStorage on logout', async () => {
      localStorage.setItem('auth_token', 'mock-token');
      
      renderWithAuthProvider();
      
      const logoutButton = screen.getByText('Logout');
      
      await act(async () => {
        fireEvent.click(logoutButton);
        // Wait for the async logout to complete
        await waitFor(() => {
          expect(screen.getByTestId('auth-status')).toHaveTextContent('Not Authenticated');
        });
      });
      
      expect(localStorage.getItem('auth_token')).toBeNull();
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