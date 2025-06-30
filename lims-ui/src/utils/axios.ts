import axios from 'axios';

// CACHE-BUSTING: New axios instance to fix double /api issue
// This file replaces utils/axios.ts with correct Vite proxy configuration
const api = axios.create({
  // Use the API URL from environment or default to empty for Vite proxy
  baseURL: import.meta.env.VITE_API_URL || '',
  headers: {
    'Content-Type': 'application/json',
  },
});

console.log('🔧 FIXED Axios: Using Vite proxy (no baseURL)');

// Request interceptor to add Authorization header
api.interceptors.request.use(
  (config) => {
    const fullUrl = (config.baseURL || '') + (config.url || '');
    console.log('📤 FIXED Axios request:', config.method?.toUpperCase(), fullUrl);
    const token = localStorage.getItem('auth_token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    console.error('❌ FIXED Axios request error:', error);
    return Promise.reject(error);
  }
);

// Response interceptor to handle common errors
api.interceptors.response.use(
  (response) => {
    console.log('📥 FIXED Axios response:', response.status, response.config.url);
    return response;
  },
  (error) => {
    console.error('❌ FIXED Axios response error:', error.message, error.config?.url);
    if (error.response?.status === 401) {
      // Token expired or invalid - remove it and redirect to login
      localStorage.removeItem('auth_token');
      window.location.reload();
    }
    return Promise.reject(error);
  }
);

export default api; 