import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider } from './auth/AuthContext';
import { ProtectedRoute } from './auth/ProtectedRoute';
import { Desktop } from './components/Desktop/Desktop';
import { apps } from './config/apps';
import Login from './pages/Login';
import ForgotPassword from './pages/ForgotPassword';

const queryClient = new QueryClient();

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <Router future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
          <Routes>
            {/* Public routes */}
            <Route path="/login" element={<Login />} />
            <Route path="/forgot-password" element={<ForgotPassword />} />
            
            {/* Protected Desktop OS */}
            <Route path="/desktop" element={
              <ProtectedRoute>
                <Desktop apps={apps} />
              </ProtectedRoute>
            } />
            
            {/* Redirect root to desktop */}
            <Route path="/" element={<Navigate to="/desktop" replace />} />
            
            {/* Catch all route - redirect to desktop */}
            <Route path="*" element={<Navigate to="/desktop" replace />} />
          </Routes>
        </Router>
      </AuthProvider>
    </QueryClientProvider>
  );
}
