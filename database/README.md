# GenXlink Supabase Database Setup

This directory contains the database schema and setup instructions for GenXlink's Supabase backend.

## Overview

GenXlink uses Supabase as its primary database backend to handle:
- User authentication and profiles
- Device registry and management
- Session tracking and logging
- Access control and permissions
- File transfer records
- Audit logging and security

## Database Schema

### Core Tables

#### `users`
Extends Supabase's `auth.users` table with additional profile information:
- User preferences and settings
- Subscription tier (Free/Premium/Enterprise)
- Display name and avatar
- Account verification status

#### `devices`
Registry of all registered devices:
- Device identification and capabilities
- Online status tracking
- Device type and OS information
- Network details (IP/MAC addresses)

#### `sessions`
Records of all remote sessions:
- Session types (remote control, screen sharing, etc.)
- Duration and status tracking
- Participating devices
- Metadata for analytics

### Security & Access Control

#### `access_sessions`
Manages active access control sessions:
- Permission profiles applied
- Session expiration and status
- Activity tracking

#### `temporary_permissions`
Time-limited permission grants:
- Permission-specific expiration
- Grant tracking and audit
- Reason and approval metadata

#### `access_policies`
Policy-based access control:
- Conditional access rules
- Automated permission evaluation
- Priority-based policy application

#### `audit_log`
Comprehensive security logging:
- All permission decisions
- Session lifecycle events
- Security-relevant actions

### Role-Based Access Control

#### `roles`
Hierarchical role system:
- System-defined roles (Admin, Operator, Viewer, Guest)
- Custom role creation
- Permission inheritance

#### `user_roles`
Many-to-many user-role assignments:
- Time-based role assignments
- Assignment tracking
- Expiration handling

### Additional Features

#### `file_transfers`
Complete file transfer tracking:
- Transfer progress and status
- File metadata and hashing
- Source/destination tracking

#### `connection_logs`
Network connection monitoring:
- Connection type and performance
- Success/failure tracking
- Network diagnostics

## Setup Instructions

### 1. Create Supabase Project

1. Go to [supabase.com](https://supabase.com)
2. Create a new project
3. Note your project URL and anon key

### 2. Apply Database Schema

```bash
# Using the Supabase CLI
supabase db push

# Or via SQL Editor in Supabase Dashboard
# Copy and paste the contents of supabase_schema.sql
```

### 3. Configure Environment

Create a `.env` file in your project root:

```env
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key
```

### 4. Test Connection

Run the database tests:

```bash
cargo test --package genxlink-client-core --lib database
cargo test --package genxlink-client-core --lib auth_service
```

## Security Features

### Row Level Security (RLS)

All tables have RLS policies enabled:
- Users can only access their own data
- System roles have appropriate access
- Automatic user profile creation on signup

### Data Encryption

- All sensitive data is encrypted at rest
- API connections use HTTPS
- Authentication tokens are properly scoped

### Audit Trail

Complete audit logging for:
- Authentication events
- Permission changes
- Session lifecycle
- File transfers
- Security violations

## Performance Optimizations

### Indexes

Strategic indexes on:
- Foreign key relationships
- Frequently queried columns
- Time-based fields (created_at, expires_at)
- Status and type fields

### Cleanup Functions

Automated cleanup for:
- Expired sessions and permissions
- Old audit logs (90-day retention)
- Historical connection logs (30-day retention)

## API Integration

### Authentication Flow

1. User registers/logs in via Supabase Auth
2. JWT token is returned to client
3. Token is used for all subsequent API calls
4. Row Level Security enforces data access

### Device Registration

1. Client authenticates with user token
2. Device information is registered
3. Unique device ID is generated
4. Online status is tracked

### Session Management

1. Access session is created
2. Permission profile is applied
3. Temporary permissions can be granted
4. All actions are audited

## Monitoring and Maintenance

### Dashboard Metrics

Monitor via Supabase Dashboard:
- Active users and devices
- Session duration and success rates
- Permission grant/deny patterns
- File transfer volumes

### Regular Maintenance

- Review audit logs for security issues
- Monitor database performance
- Update role permissions as needed
- Cleanup old data retention policies

## Troubleshooting

### Common Issues

**Permission Denied Errors**
- Check RLS policies are correctly applied
- Verify user is authenticated
- Ensure JWT token is valid

**Connection Issues**
- Verify Supabase URL and keys
- Check network connectivity
- Review CORS settings

**Performance Issues**
- Monitor query performance
- Check index usage
- Review database size

### Debug Queries

Useful queries for debugging:

```sql
-- Check user devices
SELECT * FROM devices WHERE user_id = auth.uid();

-- Check active sessions
SELECT * FROM access_sessions WHERE user_id = auth.uid() AND status = 'active';

-- Check recent audit events
SELECT * FROM audit_log WHERE user_id = auth.uid() ORDER BY created_at DESC LIMIT 10;

-- Check role assignments
SELECT r.name, ur.assigned_at FROM user_roles ur
JOIN roles r ON ur.role_id = r.id
WHERE ur.user_id = auth.uid();
```

## Migration Notes

When updating the schema:

1. Always backup before migrations
2. Test migrations in development first
3. Use transactional migrations
4. Update application code accordingly
5. Monitor for performance impact

## Support

For database-related issues:
- Check Supabase documentation
- Review audit logs for errors
- Monitor database performance metrics
- Contact support with detailed logs
