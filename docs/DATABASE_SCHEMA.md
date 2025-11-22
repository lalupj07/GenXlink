# Database Schema

## PostgreSQL Tables

### users
```sql
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
```

### licenses
```sql
CREATE TABLE licenses (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    license_key VARCHAR(50) UNIQUE NOT NULL,
    plan_type VARCHAR(50) NOT NULL,
    max_devices INTEGER,
    expires_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_licenses_user_id ON licenses(user_id);
CREATE INDEX idx_licenses_license_key ON licenses(license_key);
CREATE INDEX idx_licenses_status ON licenses(status);
```

### device_links
```sql
CREATE TABLE device_links (
    id BIGSERIAL PRIMARY KEY,
    license_id BIGINT NOT NULL REFERENCES licenses(id) ON DELETE CASCADE,
    device_id VARCHAR(255) NOT NULL,
    device_name VARCHAR(255) NOT NULL,
    activated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMP WITH TIME ZONE,
    UNIQUE(license_id, device_id)
);

CREATE INDEX idx_device_links_license_id ON device_links(license_id);
CREATE INDEX idx_device_links_device_id ON device_links(device_id);
```

### sessions
```sql
CREATE TABLE sessions (
    id BIGSERIAL PRIMARY KEY,
    session_id UUID UNIQUE NOT NULL,
    local_device_id VARCHAR(255) NOT NULL,
    remote_device_id VARCHAR(255) NOT NULL,
    connection_type VARCHAR(50) NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMP WITH TIME ZONE,
    duration_seconds INTEGER,
    bytes_transferred BIGINT DEFAULT 0
);

CREATE INDEX idx_sessions_session_id ON sessions(session_id);
CREATE INDEX idx_sessions_local_device ON sessions(local_device_id);
CREATE INDEX idx_sessions_remote_device ON sessions(remote_device_id);
CREATE INDEX idx_sessions_started_at ON sessions(started_at);
```

## Redis Keys

### Session Management
- `session:{session_id}` - Session data (JSON)
- `device:{device_id}:online` - Device online status (TTL: 60s)
- `device:{device_id}:license` - Cached license data (TTL: 3600s)

### Rate Limiting
- `ratelimit:{device_id}:{endpoint}` - Request count (TTL: 60s)

### JWT Tokens
- `jwt:blacklist:{token_hash}` - Blacklisted tokens (TTL: token expiry)
