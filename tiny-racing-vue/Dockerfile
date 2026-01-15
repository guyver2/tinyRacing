# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY tiny-racing-vue/package*.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY tiny-racing-vue/src ./src
COPY tiny-racing-vue/index.html ./
COPY tiny-racing-vue/tsconfig*.json ./
COPY tiny-racing-vue/vite.config.ts ./
COPY tiny-racing-vue/env.d.ts ./

# Copy public directory, replacing symlink with actual assets
COPY tiny-racing-vue/public/favicon.ico ./public/
# Copy the actual assets directory (not the symlink)
COPY assets ./public/assets

# Build arguments for environment variables
ARG VITE_API_URL=/api
ARG VITE_WS_URL=/ws

# Set environment variables for build
ENV VITE_API_URL=${VITE_API_URL}
ENV VITE_WS_URL=${VITE_WS_URL}

# Build the application
RUN npm run build

# Production stage
FROM nginx:alpine

# Copy built files from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration (if needed for SPA routing)
RUN echo 'server { \
    listen 80; \
    server_name _; \
    root /usr/share/nginx/html; \
    index index.html; \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
}' > /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]

