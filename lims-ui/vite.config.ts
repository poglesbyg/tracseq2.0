import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Determine if we're running in Docker
const isDocker = process.env.DOCKER_ENV === 'true'
const apiTarget = isDocker ? 'http://lims-gateway:8000' : 'http://localhost:8089'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    host: '0.0.0.0', // Allow external connections (required for Docker)
    watch: {
      usePolling: true, // Enable polling for Docker file watching
    },
    proxy: {
      // Proxy API requests to the API Gateway
      '/api': {
        target: apiTarget,
        changeOrigin: true,
        secure: false,
        rewrite: (path) => path, // Don't rewrite the path
        configure: (proxy) => {
          proxy.on('error', (err) => {
            console.log('🚨 Proxy error:', err);
          });
          proxy.on('proxyReq', (proxyReq, req) => {
            console.log('📤 Proxying request:', req.method, req.url, '→', proxyReq.getHeader('host') + proxyReq.path);
          });
          proxy.on('proxyRes', (proxyRes, req) => {
            console.log('📥 Proxy response:', proxyRes.statusCode, req.url);
          });
        },
      },
      // Proxy WebSocket connections for chat
      '/ws': {
        target: apiTarget.replace('http:', 'ws:'),
        ws: true,
        changeOrigin: true,
        configure: (proxy) => {
          proxy.on('error', (err) => {
            console.log('🚨 WebSocket proxy error:', err);
          });
        },
      },
    },
  },
  build: {
    outDir: 'dist',
  },
})
