// API configuration for different environments
const API_CONFIG = {
  // Main Lab Manager API
  labManager: {
    baseUrl: import.meta.env.PROD ? 'http://localhost:3001' : '',
  },
  // RAG Service API
  rag: {
    baseUrl: import.meta.env.PROD ? 'http://localhost:8000' : '',
  },
};

export default API_CONFIG; 
