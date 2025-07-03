# Build stage
FROM node:18-alpine AS build

# Accept build arguments
ARG VITE_API_URL=""

WORKDIR /app

# Copy package files first for better caching
COPY package.json ./

# Install dependencies with legacy peer deps to handle React version conflicts
RUN npm install --legacy-peer-deps

# Copy only necessary source files
COPY src/ ./src/
COPY public/ ./public/
COPY *.config.* ./
COPY *.json ./
COPY index.html ./

# Set environment variable for Vite build
ENV VITE_API_URL=$VITE_API_URL

# Build the application
RUN npm run build

# Production stage with nginx
FROM nginx:1.25-alpine AS production

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Copy built application from build stage
COPY --from=build /app/dist /usr/share/nginx/html

# Health check endpoint
RUN echo '<!DOCTYPE html><html><body><h1>OK</h1></body></html>' > /usr/share/nginx/html/health

# Expose port
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"] 
