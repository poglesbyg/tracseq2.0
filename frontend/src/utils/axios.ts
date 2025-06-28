import axios from 'axios';

// Create axios instance with base configuration
const baseURL = import.meta.env.VITE_API_BASE_URL || (import.meta.env.PROD ? '/api' : 'http://localhost:8089/api');

console.log('🔧 Axios baseURL:', baseURL);

const api = axios.create({
  baseURL: baseURL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor to add Authorization header
api.interceptors.request.use(
  (config) => {
    const fullUrl = (config.baseURL || '') + (config.url || '');
    console.log('📤 Axios request:', config.method?.toUpperCase(), fullUrl);
    const token = localStorage.getItem('auth_token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    console.error('❌ Axios request error:', error);
    return Promise.reject(error);
  }
);

// Response interceptor to handle common errors
api.interceptors.response.use(
  (response) => {
    console.log('📥 Axios response:', response.status, response.config.url);
    return response;
  },
  (error) => {
    console.error('❌ Axios response error:', error.message, error.config?.url);
    if (error.response?.status === 401) {
      // Token expired or invalid - remove it and redirect to login
      localStorage.removeItem('auth_token');
      window.location.reload();
    }
    return Promise.reject(error);
  }
);

export default api; 
