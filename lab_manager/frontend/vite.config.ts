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
        target: 'http://localhost:8000',
        changeOrigin: true,
        secure: false,
        configure: (_proxy, _options) => {
          _proxy.on('error', (_err, _req, _res) => {
            console.log('proxy error', _err);
          });
          _proxy.on('proxyReq', (_proxyReq, req, _res) => {
            console.log('Sending Request to the Target:', req.method, req.url);
          });
          _proxy.on('proxyRes', (_proxyRes, req, _res) => {
            console.log('Received Response from the Target:', _proxyRes.statusCode, req.url);
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
