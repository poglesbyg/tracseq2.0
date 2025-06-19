import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
// https://vitejs.dev/config/
export default defineConfig({
    plugins: [react()],
    server: {
        host: true,
        port: 5173,
        proxy: {
            // RAG API - route to RAG service (local development defaults to localhost)
            '/api/rag': {
                target: process.env.RAG_API_URL || 'http://localhost:8000',
                changeOrigin: true,
                secure: false,
                configure: function (proxy, _options) {
                    proxy.on('error', function (err, _req, _res) {
                        console.log('RAG API proxy error', err);
                    });
                    proxy.on('proxyReq', function (proxyReq, req, _res) {
                        console.log('Sending RAG Request to the Target:', req.method, req.url);
                    });
                    proxy.on('proxyRes', function (proxyRes, req, _res) {
                        console.log('Received RAG Response from the Target:', proxyRes.statusCode, req.url);
                    });
                },
            },
            // All other API calls - route through API Gateway for microservices
            '/api': {
                target: process.env.API_GATEWAY_URL || 'http://host.docker.internal:8000',
                changeOrigin: true,
                secure: false,
                configure: function (proxy, _options) {
                    proxy.on('error', function (err, _req, _res) {
                        console.log('API Gateway proxy error', err);
                    });
                    proxy.on('proxyReq', function (proxyReq, req, _res) {
                        console.log('Sending Request to API Gateway:', req.method, req.url);
                    });
                    proxy.on('proxyRes', function (proxyRes, req, _res) {
                        console.log('Received Response from API Gateway:', proxyRes.statusCode, req.url);
                    });
                },
            },
            '/health': {
                target: process.env.API_GATEWAY_URL || 'http://host.docker.internal:8000',
                changeOrigin: true,
                secure: false,
            },
            '/routing-status': {
                target: process.env.API_GATEWAY_URL || 'http://host.docker.internal:8000',
                changeOrigin: true,
                secure: false,
            },
        },
    },
});
