# GenXLink API Documentation

Base URL: `https://api.genxlink.com` (or your self-hosted URL)

## Authentication

Most endpoints require JWT authentication. Include the token in the Authorization header:
```
Authorization: Bearer <jwt_token>
```

## Endpoints

### Authentication

#### POST /auth/register
Register a new user account.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "secure_password"
}
```

**Response:**
```json
{
  "success": true,
  "message": "User registered successfully"
}
```

#### POST /auth/login
Login to get JWT token.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "secure_password"
}
```

**Response:**
```json
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### License Management

#### POST /license/activate
Activate a license key for a device.

**Request:**
```json
{
  "license_key": "XXXXX-XXXXX-XXXXX-XXXXX-XXXXX",
  "device_id": "ABC123...",
  "device_name": "My Windows PC"
}
```

**Response:**
```json
{
  "success": true,
  "message": "License activated successfully",
  "jwt_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

#### GET /license/status
Get current license status (requires authentication).

**Response:**
```json
{
  "active": true,
  "plan": "pro",
  "expires_at": "2027-01-01T00:00:00Z"
}
```

### Connection Management

#### POST /connection/start
Start a new remote connection session.

**Request:**
```json
{
  "device_id": "ABC123...",
  "remote_device_id": "XYZ789..."
}
```

**Response:**
```json
{
  "success": true,
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### POST /connection/end
End a remote connection session.

**Request:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response:**
```json
{
  "success": true
}
```

## WebSocket Signaling

Connect to: `wss://signal.genxlink.com/ws`

### Message Format

All messages are JSON:

```json
{
  "type": "offer|answer|ice_candidate",
  "from": "device_id",
  "to": "device_id",
  "data": {}
}
```

### Message Types

#### Offer
```json
{
  "type": "offer",
  "from": "ABC123",
  "to": "XYZ789",
  "sdp": "v=0\r\no=- ..."
}
```

#### Answer
```json
{
  "type": "answer",
  "from": "XYZ789",
  "to": "ABC123",
  "sdp": "v=0\r\no=- ..."
}
```

#### ICE Candidate
```json
{
  "type": "ice_candidate",
  "from": "ABC123",
  "to": "XYZ789",
  "candidate": "candidate:...",
  "sdp_mid": "0",
  "sdp_mline_index": 0
}
```

## Error Responses

All error responses follow this format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message"
  }
}
```

### Common Error Codes

- `INVALID_LICENSE` - License key is invalid or expired
- `DEVICE_LIMIT_REACHED` - Maximum number of devices reached
- `SESSION_LIMIT_REACHED` - Session time limit reached
- `UNAUTHORIZED` - Invalid or missing authentication token
- `NOT_FOUND` - Resource not found
- `RATE_LIMIT_EXCEEDED` - Too many requests
