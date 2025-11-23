-- Create app_users table for GenXLink authentication
-- Run this in Supabase SQL Editor

CREATE TABLE IF NOT EXISTS app_users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE
);

-- Create index for faster email lookups
CREATE INDEX IF NOT EXISTS idx_app_users_email ON app_users(email);

-- Add trigger for updated_at
CREATE TRIGGER update_app_users_updated_at
    BEFORE UPDATE ON app_users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Enable Row Level Security (optional - for future multi-tenant support)
ALTER TABLE app_users ENABLE ROW LEVEL SECURITY;

-- Policy: Users can view their own data
CREATE POLICY "Users can view own data"
    ON app_users FOR SELECT
    USING (auth.uid()::text = id::text);

-- Policy: Users can update their own data
CREATE POLICY "Users can update own data"
    ON app_users FOR UPDATE
    USING (auth.uid()::text = id::text);

-- Test query
SELECT * FROM app_users;
