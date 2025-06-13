// API configuration for different environments
const API_CONFIG = {
  // Main Lab Manager API
  labManager: {
    baseUrl: import.meta.env.PROD ? '' : '',
  },
  // RAG Service API - Use relative URLs in production, proxy in development
  rag: {
    baseUrl: '',
  },
};

export default API_CONFIG; 
