import axios from 'axios';

// CACHE-BUSTING: New axios instance to fix double /api issue
// This file replaces utils/axios.ts with correct Vite proxy configuration
const api = axios.create({
  // Use the API URL from environment or default to empty for Vite proxy
  baseURL: import.meta.env.VITE_API_URL || '',
});

console.log('🔧 FIXED Axios: Using Vite proxy (no baseURL)');

// Request interceptor to add Authorization header
api.interceptors.request.use(
  (config) => {
    const fullUrl = (config.baseURL || '') + (config.url || '');
    console.log('📤 FIXED Axios request:', config.method?.toUpperCase(), fullUrl);
    
    // Debug logging for FormData detection
    console.log('📤 Axios config.data type:', typeof config.data);
    console.log('📤 Axios config.data instanceof FormData:', config.data instanceof FormData);
    console.log('📤 Axios config.data:', config.data);
    
    const token = localStorage.getItem('auth_token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    
    // IMPORTANT: Don't set Content-Type for FormData - let axios handle it
    if (!(config.data instanceof FormData)) {
      // Only set Content-Type for non-FormData requests
      config.headers['Content-Type'] = 'application/json';
      console.log('📤 Setting Content-Type to application/json');
    } else {
      console.log('📤 FormData detected - not setting Content-Type');
    }
    
    return config;
  },
  (error) => {
    console.error('Request error:', error);
    return Promise.reject(error);
  }
);

// Response interceptor to handle errors consistently
api.interceptors.response.use(
  (response) => {
    const fullUrl = (response.config.baseURL || '') + (response.config.url || '');
    console.log('📥 FIXED Axios response:', response.status, fullUrl);
    return response;
  },
  (error) => {
    const fullUrl = error.config ? (error.config.baseURL || '') + (error.config.url || '') : 'unknown';
    console.error('❌ FIXED Axios response error:', error.message, fullUrl);
    if (error.response?.status === 401) {
      // Optionally handle authentication errors here
      // For now, let components handle it
    }
    return Promise.reject(error);
  }
);

export default api; 