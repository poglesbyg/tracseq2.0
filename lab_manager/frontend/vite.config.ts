import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: true,
    port: 5173,
    proxy: {
      // Route ALL API requests through the API Gateway
      // This enables gradual microservices migration with feature flags
      '/api': {
        target: process.env.API_GATEWAY_URL || 'http://localhost:8000',
        changeOrigin: true,
        secure: false,
        configure: (proxy, _options) => {
          proxy.on('error', (err, _req, _res) => {
            console.log('API Gateway proxy error', err);
          });
          proxy.on('proxyReq', (proxyReq, req, _res) => {
            console.log('Sending Request to API Gateway:', req.method, req.url);
          });
          proxy.on('proxyRes', (proxyRes, req, _res) => {
            console.log('Received Response from API Gateway:', proxyRes.statusCode, req.url);
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
