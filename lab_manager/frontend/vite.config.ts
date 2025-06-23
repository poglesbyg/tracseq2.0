import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: true,
    port: 5173,
    proxy: {
      // RAG API - route to RAG service (containerized development)
      '/api/rag': {
        target: process.env.RAG_API_URL || 'http://localhost:8000',
        changeOrigin: true,
        secure: false,
        configure: (proxy) => {
          proxy.on('error', (err) => {
            console.log('RAG API proxy error', err);
          });
          proxy.on('proxyReq', (proxyReq, req) => {
            console.log('Sending RAG Request to the Target:', req.method, req.url);
          });
          proxy.on('proxyRes', (proxyRes, req) => {
            console.log('Received RAG Response from the Target:', proxyRes.statusCode, req.url);
          });
        },
      },
      // Route ALL API requests through the API Gateway
      // This enables gradual microservices migration with feature flags
      '/api': {
        target: 'http://localhost:8000',
        changeOrigin: true,
        secure: false,
        configure: (proxy) => {
          proxy.on('error', (err) => {
            console.log('proxy error', err);
          });
          proxy.on('proxyReq', (proxyReq, req) => {
            console.log('Sending Request to the Target:', req.method, req.url);
          });
          proxy.on('proxyRes', (proxyRes, req) => {
            console.log('Received Response from the Target:', proxyRes.statusCode, req.url);
          });
        },
      },
      // Health check through API Gateway
      '/health': {
        target: process.env.API_GATEWAY_URL || 'http://localhost:8000',
        changeOrigin: true,
        secure: false,
      },
      // Routing status for API Gateway
      '/routing-status': {
        target: process.env.API_GATEWAY_URL || 'http://localhost:8000',
        changeOrigin: true,
        secure: false,
      },
    },
  },
})
