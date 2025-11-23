# 🗄️ Supabase Database Setup Guide

## 📋 Overview

This guide walks you through setting up Supabase as the database backend for GenXLink.

**What you'll get:**
- ✅ PostgreSQL database (500MB free)
- ✅ Real-time subscriptions
- ✅ User authentication
- ✅ Row-level security
- ✅ Auto-generated REST API

---

## 🚀 Step 1: Create Supabase Account

1. Go to [supabase.com](https://supabase.com)
2. Click **"Start your project"**
3. Sign up with GitHub (recommended)

---

## 🏗️ Step 2: Create New Project

1. Click **"New Project"**
2. Fill in details:
   - **Name:** `genxlink`
   - **Database Password:** (generate strong password - save it!)
   - **Region:** `Southeast Asia (Singapore)` (closest to India)
   - **Pricing Plan:** Free

3. Click **"Create new project"**
4. Wait 2-3 minutes for provisioning

---

## 🔑 Step 3: Get Connection Details

1. Go to **Settings** → **Database**
2. Copy these values:

```
Host: db.[project-id].supabase.co
Database name: postgres
Port: 5432
User: postgres
Password: [your-password]
```

3. Go to **Settings** → **API**
4. Copy these values:

```
Project URL: https://[project-id].supabase.co
anon public key: eyJhbGc...
service_role key: eyJhbGc... (keep secret!)
```

---

## 📊 Step 4: Create Database Schema

1. Go to **SQL Editor**
2. Click **"New query"**
3. Paste this SQL:

```sql
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Devices table
CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id TEXT UNIQUE NOT NULL,
    device_name TEXT NOT NULL,
    device_type TEXT NOT NULL, -- 'windows', 'macos', 'linux', 'android', 'ios'
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    ip_address TEXT,
    last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Connections table
CREATE TABLE connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    from_device_id TEXT NOT NULL,
    to_device_id TEXT NOT NULL,
    connection_type TEXT NOT NULL, -- 'screen_share', 'remote_control', 'file_transfer'
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    ended_at TIMESTAMP WITH TIME ZONE,
    duration_seconds INTEGER,
    status TEXT NOT NULL, -- 'active', 'completed', 'failed'
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Connection logs table
CREATE TABLE connection_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    connection_id UUID REFERENCES connections(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL, -- 'started', 'ended', 'error', 'quality_change'
    event_data JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User preferences table
CREATE TABLE user_preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE UNIQUE,
    video_quality TEXT DEFAULT 'high', -- 'low', 'medium', 'high', 'ultra'
    audio_enabled BOOLEAN DEFAULT true,
    clipboard_sync BOOLEAN DEFAULT true,
    file_transfer_enabled BOOLEAN DEFAULT true,
    theme TEXT DEFAULT 'dark', -- 'light', 'dark'
    language TEXT DEFAULT 'en', -- 'en', 'es', 'fr', 'de', 'hi'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- File transfers table
CREATE TABLE file_transfers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    connection_id UUID REFERENCES connections(id) ON DELETE CASCADE,
    file_name TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    file_type TEXT,
    direction TEXT NOT NULL, -- 'upload', 'download'
    status TEXT NOT NULL, -- 'pending', 'in_progress', 'completed', 'failed'
    progress INTEGER DEFAULT 0, -- 0-100
    bytes_transferred BIGINT DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX idx_devices_user_id ON devices(user_id);
CREATE INDEX idx_devices_device_id ON devices(device_id);
CREATE INDEX idx_devices_last_seen ON devices(last_seen);
CREATE INDEX idx_connections_from_device ON connections(from_device_id);
CREATE INDEX idx_connections_to_device ON connections(to_device_id);
CREATE INDEX idx_connections_status ON connections(status);
CREATE INDEX idx_connections_started_at ON connections(started_at);
CREATE INDEX idx_connection_logs_connection_id ON connection_logs(connection_id);
CREATE INDEX idx_file_transfers_connection_id ON file_transfers(connection_id);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add triggers for updated_at
CREATE TRIGGER update_devices_updated_at
    BEFORE UPDATE ON devices
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_preferences_updated_at
    BEFORE UPDATE ON user_preferences
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create views for easy querying
CREATE VIEW connection_history AS
SELECT 
    c.id,
    c.from_device_id,
    c.to_device_id,
    d1.device_name as from_device_name,
    d2.device_name as to_device_name,
    c.connection_type,
    c.started_at,
    c.ended_at,
    c.duration_seconds,
    c.status
FROM connections c
LEFT JOIN devices d1 ON c.from_device_id = d1.device_id
LEFT JOIN devices d2 ON c.to_device_id = d2.device_id
ORDER BY c.started_at DESC;

CREATE VIEW active_devices AS
SELECT 
    device_id,
    device_name,
    device_type,
    last_seen,
    EXTRACT(EPOCH FROM (NOW() - last_seen)) as seconds_since_seen
FROM devices
WHERE last_seen > NOW() - INTERVAL '5 minutes'
ORDER BY last_seen DESC;
```

4. Click **"Run"**
5. Verify all tables created successfully

---

## 🔒 Step 5: Enable Row-Level Security

1. In SQL Editor, run:

```sql
-- Enable RLS on all tables
ALTER TABLE devices ENABLE ROW LEVEL SECURITY;
ALTER TABLE connections ENABLE ROW LEVEL SECURITY;
ALTER TABLE connection_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_preferences ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_transfers ENABLE ROW LEVEL SECURITY;

-- Devices policies
CREATE POLICY "Users can view their own devices"
    ON devices FOR SELECT
    USING (auth.uid() = user_id);

CREATE POLICY "Users can insert their own devices"
    ON devices FOR INSERT
    WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own devices"
    ON devices FOR UPDATE
    USING (auth.uid() = user_id);

CREATE POLICY "Users can delete their own devices"
    ON devices FOR DELETE
    USING (auth.uid() = user_id);

-- Connections policies
CREATE POLICY "Users can view their connections"
    ON connections FOR SELECT
    USING (
        EXISTS (
            SELECT 1 FROM devices 
            WHERE (device_id = connections.from_device_id OR device_id = connections.to_device_id)
            AND user_id = auth.uid()
        )
    );

CREATE POLICY "Users can insert connections"
    ON connections FOR INSERT
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM devices 
            WHERE device_id = connections.from_device_id
            AND user_id = auth.uid()
        )
    );

-- User preferences policies
CREATE POLICY "Users can view their own preferences"
    ON user_preferences FOR SELECT
    USING (auth.uid() = user_id);

CREATE POLICY "Users can insert their own preferences"
    ON user_preferences FOR INSERT
    WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own preferences"
    ON user_preferences FOR UPDATE
    USING (auth.uid() = user_id);

-- File transfers policies
CREATE POLICY "Users can view their file transfers"
    ON file_transfers FOR SELECT
    USING (
        EXISTS (
            SELECT 1 FROM connections c
            JOIN devices d ON (c.from_device_id = d.device_id OR c.to_device_id = d.device_id)
            WHERE c.id = file_transfers.connection_id
            AND d.user_id = auth.uid()
        )
    );
```

---

## 🔌 Step 6: Add to Railway

1. Go to Railway dashboard
2. Click your GenXLink service
3. Go to **Variables** tab
4. Add these variables:

```bash
DATABASE_URL=postgresql://postgres:[password]@db.[project-id].supabase.co:5432/postgres
SUPABASE_URL=https://[project-id].supabase.co
SUPABASE_ANON_KEY=[your-anon-key]
SUPABASE_SERVICE_KEY=[your-service-key]
```

5. Click **"Add"**
6. Railway will automatically redeploy

---

## 🧪 Step 7: Test the Connection

1. In Supabase SQL Editor, run:

```sql
-- Insert test device
INSERT INTO devices (device_id, device_name, device_type)
VALUES ('test-123', 'Test Device', 'windows');

-- Query devices
SELECT * FROM devices;

-- Query active devices view
SELECT * FROM active_devices;
```

2. Test from your server:

```bash
curl https://genxlink-production.up.railway.app/devices
```

---

## 📊 Step 8: Enable Real-time (Optional)

1. Go to **Database** → **Replication**
2. Enable replication for tables:
   - `devices`
   - `connections`
   - `file_transfers`

3. This allows real-time updates in your client!

---

## 🔐 Step 9: Set Up Authentication

1. Go to **Authentication** → **Providers**
2. Enable providers you want:
   - ✅ Email (enabled by default)
   - ✅ Google
   - ✅ GitHub
   - ✅ Microsoft

3. Configure redirect URLs:
   - `http://localhost:3000/auth/callback` (development)
   - `https://genxlink-production.up.railway.app/auth/callback` (production)

---

## 📝 Step 10: Update Server Code

Add Supabase client to your server:

**File:** `server/Cargo.toml`

```toml
[dependencies]
postgrest = "1.4"
tokio-postgres = "0.7"
```

**File:** `server/src/database.rs`

```rust
use postgrest::Postgrest;

pub struct Database {
    client: Postgrest,
}

impl Database {
    pub fn new(url: &str, api_key: &str) -> Self {
        let client = Postgrest::new(url)
            .insert_header("apikey", api_key);
        
        Self { client }
    }

    pub async fn register_device(
        &self,
        device_id: &str,
        device_name: &str,
        device_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .from("devices")
            .insert(format!(
                r#"{{"device_id": "{}", "device_name": "{}", "device_type": "{}"}}"#,
                device_id, device_name, device_type
            ))
            .execute()
            .await?;
        
        Ok(())
    }

    pub async fn get_active_devices(&self) -> Result<Vec<Device>, Box<dyn std::error::Error>> {
        let response = self.client
            .from("active_devices")
            .select("*")
            .execute()
            .await?;
        
        let devices: Vec<Device> = response.json().await?;
        Ok(devices)
    }
}
```

---

## 📈 Monitoring

### View Database Stats

1. Go to **Database** → **Database**
2. Monitor:
   - Connection count
   - Database size
   - Query performance

### View API Usage

1. Go to **Settings** → **API**
2. Monitor:
   - Request count
   - Bandwidth usage
   - Error rate

---

## 💰 Free Tier Limits

- **Database:** 500 MB
- **Bandwidth:** 2 GB
- **API Requests:** Unlimited
- **Auth Users:** Unlimited
- **Storage:** 1 GB

**Upgrade when needed:**
- Pro: $25/month (8 GB database, 50 GB bandwidth)

---

## 🔧 Troubleshooting

### Connection Failed

```bash
# Test connection
psql "postgresql://postgres:[password]@db.[project-id].supabase.co:5432/postgres"
```

### RLS Blocking Queries

```sql
-- Temporarily disable RLS for testing
ALTER TABLE devices DISABLE ROW LEVEL SECURITY;

-- Re-enable after fixing
ALTER TABLE devices ENABLE ROW LEVEL SECURITY;
```

### Slow Queries

```sql
-- Check slow queries
SELECT * FROM pg_stat_statements 
ORDER BY total_exec_time DESC 
LIMIT 10;
```

---

## 📚 Next Steps

1. ✅ Implement authentication in client
2. ✅ Add connection history UI
3. ✅ Implement file transfer tracking
4. ✅ Add real-time device status updates

---

## 🔗 Useful Links

- [Supabase Documentation](https://supabase.com/docs)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Row Level Security Guide](https://supabase.com/docs/guides/auth/row-level-security)
- [Real-time Guide](https://supabase.com/docs/guides/realtime)

---

**Your database is now ready for GenXLink!** 🎉
