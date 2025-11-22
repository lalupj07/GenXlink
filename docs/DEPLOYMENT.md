# Deployment Guide

## Prerequisites

- Docker and Docker Compose
- PostgreSQL 14+
- Redis 7+
- Domain name with SSL certificate

## Server Deployment

### Using Docker Compose

1. **Create docker-compose.yml:**

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:14
    environment:
      POSTGRES_DB: genxlink
      POSTGRES_USER: genxlink
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  api:
    build: ./server/api
    environment:
      DATABASE_URL: postgresql://genxlink:${DB_PASSWORD}@postgres:5432/genxlink
      REDIS_URL: redis://redis:6379
      JWT_SECRET: ${JWT_SECRET}
    ports:
      - "8080:8080"
    depends_on:
      - postgres
      - redis

  signaling:
    build: ./server/signaling
    ports:
      - "8081:8081"

  relay:
    build: ./server/relay
    ports:
      - "3478:3478/udp"
      - "3478:3478/tcp"

volumes:
  postgres_data:
  redis_data:
```

2. **Create .env file:**

```bash
DB_PASSWORD=your_secure_password
JWT_SECRET=your_jwt_secret_key
```

3. **Start services:**

```bash
docker-compose up -d
```

### Manual Deployment

#### API Server

```bash
cd server/api
cargo build --release
./target/release/genxlink-api
```

Environment variables:
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `JWT_SECRET` - Secret key for JWT signing
- `PORT` - Server port (default: 8080)

#### Signaling Server

```bash
cd server/signaling
cargo build --release
./target/release/genxlink-signaling
```

Environment variables:
- `PORT` - Server port (default: 8081)

#### Relay Server

```bash
cd server/relay
cargo build --release
./target/release/genxlink-relay
```

Environment variables:
- `PORT` - Server port (default: 3478)

## Client Deployment

### Windows Client

1. **Build:**

```bash
cd client/windows
cargo build --release
```

2. **Create installer (optional):**

Use tools like:
- WiX Toolset for MSI installer
- Inno Setup for EXE installer

3. **Code signing:**

Sign the executable with a valid certificate:
```bash
signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com genxlink.exe
```

### Android Client

1. **Setup Android NDK and SDK**

2. **Build:**

```bash
cd client/android
./gradlew assembleRelease
```

3. **Sign APK:**

```bash
jarsigner -keystore keystore.jks app-release.apk alias_name
```

## SSL/TLS Configuration

### Using Let's Encrypt

```bash
certbot certonly --standalone -d api.genxlink.com
certbot certonly --standalone -d signal.genxlink.com
```

### Nginx Reverse Proxy

```nginx
server {
    listen 443 ssl http2;
    server_name api.genxlink.com;

    ssl_certificate /etc/letsencrypt/live/api.genxlink.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.genxlink.com/privkey.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}

server {
    listen 443 ssl http2;
    server_name signal.genxlink.com;

    ssl_certificate /etc/letsencrypt/live/signal.genxlink.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/signal.genxlink.com/privkey.pem;

    location /ws {
        proxy_pass http://localhost:8081;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

## Database Migration

Run initial schema:

```bash
psql -U genxlink -d genxlink -f docs/schema.sql
```

## Monitoring

### Health Checks

- API: `https://api.genxlink.com/health`
- Signaling: WebSocket connection test

### Logging

All services log to stdout. Use a log aggregation service like:
- ELK Stack (Elasticsearch, Logstash, Kibana)
- Grafana Loki
- CloudWatch (AWS)

## Backup

### Database Backup

```bash
pg_dump -U genxlink genxlink > backup_$(date +%Y%m%d).sql
```

### Redis Backup

Redis automatically creates RDB snapshots in `/data`.

## Security Checklist

- [ ] Change all default passwords
- [ ] Enable firewall rules
- [ ] Use SSL/TLS for all connections
- [ ] Implement rate limiting
- [ ] Regular security updates
- [ ] Monitor for suspicious activity
- [ ] Backup encryption keys securely
