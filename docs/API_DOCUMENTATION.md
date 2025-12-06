# GenXLink API Documentation

## Overview

The GenXLink API provides a comprehensive RESTful interface for managing remote desktop connections, user authentication, device management, and session handling. This documentation covers all available endpoints, request/response formats, and usage examples.

## Base URL

```
Development: http://localhost:8080
Production: https://api.genxlink.com
```

## Authentication

The API uses JWT (JSON Web Token) authentication. Include the token in the Authorization header:

```
Authorization: Bearer <your-jwt-token>
```

## Rate Limiting

- **Public endpoints**: 10 requests per second per IP
- **Authentication endpoints**: 5 requests per second per IP
- **Protected endpoints**: 100 requests per minute per authenticated user

## API Endpoints

### Health Check

#### GET /health

Check if the API server is running.

**Response:**
```json
{
  "status": "ok",
  "service": "genxlink-api",
  "timestamp": "2023-12-06T12:00:00Z",
  "version": "0.1.0"
}
```

---

### Authentication

#### POST /auth/register

Register a new user account.

**Request Body:**
```json
{
  "email": "user@example.com",
  "username": "username",
  "password": "SecurePassword123!",
  "display_name": "Display Name"
}
```

**Response:**
```json
{
  "success": true,
  "message": "User registered successfully",
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "username",
    "display_name": "Display Name",
    "is_active": true,
    "is_verified": false,
    "subscription_type": "free",
    "created_at": "2023-12-06T12:00:00Z",
    "updated_at": "2023-12-06T12:00:00Z",
    "last_login": null,
    "preferences": {}
  },
  "token": "jwt-token",
  "expires_at": "2023-12-13T12:00:00Z"
}
```

#### POST /auth/login

Authenticate a user and receive a JWT token.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Login successful",
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "username": "username",
    "display_name": "Display Name",
    "is_active": true,
    "is_verified": true,
    "subscription_type": "free",
    "created_at": "2023-12-06T12:00:00Z",
    "updated_at": "2023-12-06T12:00:00Z",
    "last_login": "2023-12-06T12:00:00Z",
    "preferences": {}
  },
  "token": "jwt-token",
  "expires_at": "2023-12-13T12:00:00Z"
}
```

#### POST /api/auth/refresh

Refresh an existing JWT token.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
{
  "success": true,
  "message": "Token refreshed successfully",
  "user": { ... },
  "token": "new-jwt-token",
  "expires_at": "2023-12-13T12:00:00Z"
}
```

#### POST /api/auth/change-password

Change user password.

**Headers:** `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "current_password": "OldPassword123!",
  "new_password": "NewPassword123!"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Password changed successfully"
}
```

---

### User Profile

#### GET /api/profile

Get current user profile.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
{
  "id": "uuid",
  "email": "user@example.com",
  "username": "username",
  "display_name": "Display Name",
  "avatar_url": "https://example.com/avatar.jpg",
  "is_active": true,
  "is_verified": true,
  "subscription_type": "premium",
  "created_at": "2023-12-06T12:00:00Z",
  "updated_at": "2023-12-06T12:00:00Z",
  "last_login": "2023-12-06T12:00:00Z",
  "preferences": {
    "theme": "dark",
    "notifications": true
  }
}
```

#### POST /api/profile

Update user profile.

**Headers:** `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "display_name": "New Display Name",
  "avatar_url": "https://example.com/new-avatar.jpg",
  "preferences": {
    "theme": "light",
    "notifications": false
  }
}
```

**Response:**
```json
{
  "success": true,
  "message": "Profile updated successfully"
}
```

---

### Device Management

#### GET /api/devices

Get all devices for the authenticated user.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
[
  {
    "id": "uuid",
    "user_id": "uuid",
    "device_id": "device-unique-id",
    "device_name": "My Laptop",
    "device_type": "laptop",
    "os_version": "Windows 11",
    "ip_address": "192.168.1.100",
    "mac_address": "00:11:22:33:44:55",
    "last_seen": "2023-12-06T12:00:00Z",
    "is_online": true,
    "capabilities": {
      "screen_capture": true,
      "file_transfer": true,
      "remote_control": true
    },
    "metadata": {},
    "created_at": "2023-12-06T12:00:00Z",
    "updated_at": "2023-12-06T12:00:00Z"
  }
]
```

#### POST /api/devices

Register a new device.

**Headers:** `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "user_id": "uuid",
  "device_id": "device-unique-id",
  "device_name": "My Computer",
  "device_type": "desktop",
  "os_version": "Windows 10",
  "ip_address": "192.168.1.101",
  "mac_address": "00:11:22:33:44:56",
  "capabilities": {
    "screen_capture": true,
    "file_transfer": true,
    "remote_control": true
  },
  "metadata": {}
}
```

**Response:**
```json
{
  "id": "uuid",
  "user_id": "uuid",
  "device_id": "device-unique-id",
  "device_name": "My Computer",
  "device_type": "desktop",
  "os_version": "Windows 10",
  "ip_address": "192.168.1.101",
  "mac_address": "00:11:22:33:44:56",
  "last_seen": "2023-12-06T12:00:00Z",
  "is_online": true,
  "capabilities": {
    "screen_capture": true,
    "file_transfer": true,
    "remote_control": true
  },
  "metadata": {},
  "created_at": "2023-12-06T12:00:00Z",
  "updated_at": "2023-12-06T12:00:00Z"
}
```

#### POST /api/devices/{device_id}/status

Update device online status.

**Headers:** `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "is_online": true
}
```

**Response:**
```json
{
  "success": true,
  "message": "Device status updated"
}
```

---

### Session Management

#### GET /api/sessions

Get all sessions for the authenticated user.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
[
  {
    "id": "uuid",
    "user_id": "uuid",
    "device_id": "uuid",
    "remote_device_id": "uuid",
    "session_type": "remote_control",
    "started_at": "2023-12-06T12:00:00Z",
    "ended_at": null,
    "duration_seconds": null,
    "status": "active",
    "connection_quality": {
      "latency": 50,
      "bandwidth": 1000000
    },
    "metadata": {}
  }
]
```

#### POST /api/sessions

Create a new session.

**Headers:** `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "user_id": "uuid",
  "device_id": "uuid",
  "remote_device_id": "uuid",
  "session_type": "remote_control",
  "status": "active",
  "metadata": {}
}
```

**Response:**
```json
{
  "id": "uuid",
  "user_id": "uuid",
  "device_id": "uuid",
  "remote_device_id": "uuid",
  "session_type": "remote_control",
  "started_at": "2023-12-06T12:00:00Z",
  "ended_at": null,
  "duration_seconds": null,
  "status": "active",
  "connection_quality": null,
  "metadata": {}
}
```

#### POST /api/sessions/{session_id}/end

End a session.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
{
  "success": true,
  "message": "Session ended"
}
```

---

### License Management

#### POST /api/license/activate

Activate a license key.

**Headers:** `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "license_key": "GENX-XXXX-XXXX-XXXX"
}
```

**Response:**
```json
{
  "success": true,
  "message": "License activated successfully",
  "license": {
    "id": "uuid",
    "user_id": "uuid",
    "license_key": "GENX-XXXX-XXXX-XXXX",
    "license_type": "pro",
    "expires_at": "2024-12-06T12:00:00Z",
    "is_active": true,
    "max_devices": 5,
    "max_concurrent_sessions": 3,
    "features": {
      "remote_control": true,
      "file_transfer": true,
      "multi_monitor": true,
      "audio_streaming": true
    },
    "created_at": "2023-12-06T12:00:00Z",
    "updated_at": "2023-12-06T12:00:00Z"
  }
}
```

#### GET /api/license/status

Get license status for the authenticated user.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
[
  {
    "id": "uuid",
    "user_id": "uuid",
    "license_key": "GENX-XXXX-XXXX-XXXX",
    "license_type": "pro",
    "expires_at": "2024-12-06T12:00:00Z",
    "is_active": true,
    "max_devices": 5,
    "max_concurrent_sessions": 3,
    "features": {
      "remote_control": true,
      "file_transfer": true,
      "multi_monitor": true,
      "audio_streaming": true
    },
    "created_at": "2023-12-06T12:00:00Z",
    "updated_at": "2023-12-06T12:00:00Z"
  }
]
```

---

### Connection Management

#### POST /api/connection/start

Start a new connection between devices.

**Headers:** `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "from_device_id": "uuid",
  "to_device_id": "uuid",
  "connection_type": "remote_control"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Connection started successfully",
  "session_id": "uuid"
}
```

#### POST /api/connection/end

End a connection.

**Headers:** `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "session_id": "uuid"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Connection ended successfully"
}
```

---

### Statistics

#### GET /api/stats/system

Get system statistics (admin only).

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
{
  "total_users": 1000,
  "total_devices": 2500,
  "active_sessions": 150,
  "total_connections": 50000
}
```

#### GET /api/stats/usage

Get usage statistics for the authenticated user.

**Headers:** `Authorization: Bearer <token>`

**Response:**
```json
{
  "api_calls": 1500,
  "bandwidth_used": 1073741824,
  "session_duration": 36000
}
```

---

## Error Responses

All endpoints may return error responses with the following format:

```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "details": {}
}
```

### Common HTTP Status Codes

- `200 OK` - Request successful
- `201 Created` - Resource created successfully
- `400 Bad Request` - Invalid request data
- `401 Unauthorized` - Authentication required or invalid
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server error

---

## WebSocket API

### Connection

Connect to the WebSocket endpoint for real-time signaling:

```
ws://localhost:8081/ws
```

### Message Format

All WebSocket messages use the following format:

```json
{
  "type": "message_type",
  "data": {},
  "timestamp": "2023-12-06T12:00:00Z"
}
```

### Message Types

#### offer

Send WebRTC offer:

```json
{
  "type": "offer",
  "data": {
    "session_id": "uuid",
    "sdp": "webrtc-sdp-offer"
  }
}
```

#### answer

Send WebRTC answer:

```json
{
  "type": "answer",
  "data": {
    "session_id": "uuid",
    "sdp": "webrtc-sdp-answer"
  }
}
```

#### ice_candidate

Send ICE candidate:

```json
{
  "type": "ice_candidate",
  "data": {
    "session_id": "uuid",
    "candidate": "ice-candidate-data"
  }
}
```

---

## SDK Examples

### JavaScript/TypeScript

```javascript
import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:8080',
});

// Login
const login = async (email, password) => {
  try {
    const response = await api.post('/auth/login', { email, password });
    const { token } = response.data;
    api.defaults.headers.common['Authorization'] = `Bearer ${token}`;
    return response.data;
  } catch (error) {
    console.error('Login failed:', error.response.data);
    throw error;
  }
};

// Get devices
const getDevices = async () => {
  try {
    const response = await api.get('/api/devices');
    return response.data;
  } catch (error) {
    console.error('Failed to get devices:', error.response.data);
    throw error;
  }
};

// Start connection
const startConnection = async (fromDevice, toDevice) => {
  try {
    const response = await api.post('/api/connection/start', {
      from_device_id: fromDevice,
      to_device_id: toDevice,
      connection_type: 'remote_control'
    });
    return response.data;
  } catch (error) {
    console.error('Failed to start connection:', error.response.data);
    throw error;
  }
};
```

### Python

```python
import requests
import json

class GenXLinkAPI:
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url
        self.token = None
        self.session = requests.Session()
    
    def login(self, email, password):
        response = self.session.post(f"{self.base_url}/auth/login", json={
            "email": email,
            "password": password
        })
        data = response.json()
        if data.get("success"):
            self.token = data.get("token")
            self.session.headers.update({
                "Authorization": f"Bearer {self.token}"
            })
        return data
    
    def get_devices(self):
        response = self.session.get(f"{self.base_url}/api/devices")
        return response.json()
    
    def start_connection(self, from_device_id, to_device_id):
        response = self.session.post(f"{self.base_url}/api/connection/start", json={
            "from_device_id": from_device_id,
            "to_device_id": to_device_id,
            "connection_type": "remote_control"
        })
        return response.json()

# Usage
api = GenXLinkAPI()
api.login("user@example.com", "password")
devices = api.get_devices()
print(devices)
```

---

## Deployment

### Docker

```bash
# Build the API server
docker build -t genxlink-api ./server/api

# Run with docker-compose
docker-compose up -d
```

### Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `JWT_SECRET` - Secret key for JWT signing
- `RUST_LOG` - Logging level (info, debug, warn, error)
- `API_PORT` - Port for the API server (default: 8080)

---

## Support

For API support and questions:

- Documentation: https://docs.genxlink.com
- GitHub Issues: https://github.com/genxlink/genxlink/issues
- Email: support@genxlink.com

---

## Changelog

### v0.1.0 (2023-12-06)

- Initial API release
- User authentication and registration
- Device management
- Session management
- License management
- Connection management
- Statistics endpoints
- WebSocket signaling support
- Rate limiting and security features
