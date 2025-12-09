# Docker Setup Guide

This project is fully containerized using Docker Compose. All services run in separate containers with nginx handling routing.

## Prerequisites

- Docker and Docker Compose installed
- A `.env` file in the project root (see `.env.example` for reference)

## Quick Start

1. **Create `.env` file** (copy from `.env.example` if it exists):
```bash
DB_NAME=tiny_racing
DB_USER=tiny_racing
DB_PASSWORD=tiny_racing_password
DB_PORT=5432
DB_DATA_DIR=./data
DATABASE_URL=postgresql://tiny_racing:tiny_racing_password@postgres:5432/tiny_racing
```

2. **Build and start all services**:
```bash
docker-compose up -d --build
```

3. **Access the application**:
- Frontend: http://localhost:8080 (or the port you configured)
- Backend API: http://localhost:8080/api (or the port you configured)
- WebSocket: ws://localhost:8080/ws (or the port you configured)

## Services

- **postgres**: PostgreSQL 16 database
- **backend**: Rust API server (ports 3000, 3030 internally)
- **frontend**: Vue.js application (port 80 internally)
- **nginx**: Reverse proxy routing requests (port 8080 externally by default)

## Environment Variables

The `.env` file should contain:
- `DB_NAME`: PostgreSQL database name
- `DB_USER`: PostgreSQL username
- `DB_PASSWORD`: PostgreSQL password
- `DB_PORT`: PostgreSQL port (host port mapping)
- `DB_DATA_DIR`: Directory for PostgreSQL data persistence
- `DATABASE_URL`: Full database connection URL (used by backend)
- `NGINX_PORT`: External port for nginx (default: 8080)
- `BACKEND_API_PORT`: External port for backend API (default: 3000)
- `BACKEND_WS_PORT`: External port for backend WebSocket (default: 3030)

## Port Configuration

By default, the application uses:
- **Port 8080** for nginx (to avoid conflicts with existing services on port 80)
- **Port 3000** for direct backend API access (optional)
- **Port 3030** for direct WebSocket access (optional)

To change these ports, add them to your `.env` file:
```bash
NGINX_PORT=8080
BACKEND_API_PORT=3000
BACKEND_WS_PORT=3030
```

### Running on a Server with Existing Services

If you're running on a server that already has nginx on ports 80/443:

1. **Use the default configuration** (port 8080) - the application will be accessible at `http://your-server:8080`

2. **Or configure custom ports** in your `.env` file:
```bash
NGINX_PORT=9000
BACKEND_API_PORT=9001
BACKEND_WS_PORT=9002
```

3. **Or proxy through your existing nginx** - Add a location block to your main nginx configuration:
```nginx
location /tiny-racing/ {
    proxy_pass http://localhost:8080/;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    
    # WebSocket support
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
}

location /tiny-racing/ws {
    proxy_pass http://localhost:8080/ws;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_read_timeout 86400;
}
```
Then update the frontend build args in `docker-compose.yml` to use `/tiny-racing/api` and `/tiny-racing/ws` as the API/WS URLs.

## Database Migrations

Database migrations run automatically when the backend starts. The backend waits for the database to be ready before running migrations.

## Stopping Services

```bash
docker-compose down
```

To also remove volumes (database data):
```bash
docker-compose down -v
```

## Viewing Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend
docker-compose logs -f frontend
docker-compose logs -f postgres
docker-compose logs -f nginx
```

## Rebuilding After Changes

```bash
# Rebuild specific service
docker-compose build backend
docker-compose up -d backend

# Rebuild all services
docker-compose up -d --build
```




