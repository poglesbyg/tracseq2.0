import { BrowserRouter as Router, Routes, Route, Navigate, useLocation } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider } from './auth/AuthContext';
import { ProtectedRoute } from './auth/ProtectedRoute';
import { Desktop } from './components/Desktop/Desktop';
import { apps } from './config/apps-fixed';
import Login from './pages/Login';
import ForgotPassword from './pages/ForgotPassword';

const queryClient = new QueryClient();

// Custom redirect component that preserves URL parameters
function DesktopRedirect() {
  const location = useLocation();
  
  // If we're on a specific app route, redirect to desktop but store the original route
  // This allows the Desktop component to detect and handle the original URL
  console.log('🔀 DesktopRedirect from:', location.pathname + location.search);
  
  // Store the original URL in sessionStorage so Desktop can access it
  if (location.pathname !== '/desktop' && location.pathname !== '/') {
    sessionStorage.setItem('originalRoute', location.pathname + location.search);
    console.log('💾 Stored original route:', location.pathname + location.search);
  }
  
  return <Navigate to="/desktop" replace />;
}

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
            
            {/* Catch all route - redirect to desktop with original URL preserved */}
            <Route path="*" element={<DesktopRedirect />} />
          </Routes>
        </Router>
      </AuthProvider>
    </QueryClientProvider>
  );
}
