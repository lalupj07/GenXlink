-- GenXlink Supabase Database Schema
-- This file defines the database structure for the GenXlink remote desktop application

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table (extends Supabase auth.users)
CREATE TABLE IF NOT EXISTS public.users (
    id UUID REFERENCES auth.users(id) PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    username TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    avatar_url TEXT,
    is_active BOOLEAN DEFAULT true,
    is_verified BOOLEAN DEFAULT false,
    subscription_type TEXT DEFAULT 'free' CHECK (subscription_type IN ('free', 'premium', 'enterprise')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    preferences JSONB DEFAULT '{}'
);

-- Device registry table
CREATE TABLE IF NOT EXISTS public.devices (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID REFERENCES public.users(id) ON DELETE CASCADE,
    device_id TEXT UNIQUE NOT NULL,
    device_name TEXT NOT NULL,
    device_type TEXT NOT NULL CHECK (device_type IN ('desktop', 'laptop', 'server', 'mobile', 'tablet', 'iot', 'unknown')),
    os_version TEXT NOT NULL,
    ip_address INET NOT NULL,
    mac_address MACADDR,
    last_seen TIMESTAMPTZ DEFAULT NOW(),
    is_online BOOLEAN DEFAULT false,
    capabilities JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Session records table
CREATE TABLE IF NOT EXISTS public.sessions (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID REFERENCES public.users(id) ON DELETE CASCADE,
    device_id UUID REFERENCES public.devices(id) ON DELETE CASCADE,
    remote_device_id UUID REFERENCES public.devices(id) ON DELETE CASCADE,
    session_type TEXT NOT NULL CHECK (session_type IN ('remote_control', 'screen_sharing', 'file_transfer', 'audio_streaming', 'multi_session')),
    started_at TIMESTAMPTZ DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    duration_seconds INTEGER,
    status TEXT DEFAULT 'active' CHECK (status IN ('active', 'completed', 'failed', 'terminated', 'timeout')),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Access control sessions table
CREATE TABLE IF NOT EXISTS public.access_sessions (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    session_id TEXT UNIQUE NOT NULL,
    user_id UUID REFERENCES public.users(id) ON DELETE CASCADE,
    device_id UUID REFERENCES public.devices(id) ON DELETE CASCADE,
    profile_type TEXT NOT NULL CHECK (profile_type IN ('default', 'screen_sharing', 'full_access', 'unattended_access')),
    status TEXT DEFAULT 'active' CHECK (status IN ('active', 'paused', 'terminated', 'expired')),
    expires_at TIMESTAMPTZ,
    last_activity TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Temporary permissions table
CREATE TABLE IF NOT EXISTS public.temporary_permissions (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    access_session_id UUID REFERENCES public.access_sessions(id) ON DELETE CASCADE,
    permission TEXT NOT NULL,
    granted_by UUID REFERENCES public.users(id),
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    reason TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Access policies table
CREATE TABLE IF NOT EXISTS public.access_policies (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    conditions JSONB NOT NULL DEFAULT '{}',
    actions JSONB NOT NULL DEFAULT '{}',
    priority INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES public.users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit log table
CREATE TABLE IF NOT EXISTS public.audit_log (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID REFERENCES public.users(id) ON DELETE CASCADE,
    session_id UUID REFERENCES public.access_sessions(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL CHECK (event_type IN ('session_created', 'session_terminated', 'permission_granted', 'permission_denied', 'policy_applied', 'device_registered', 'device_updated', 'device_deleted')),
    level TEXT DEFAULT 'info' CHECK (level IN ('info', 'warning', 'error', 'critical')),
    description TEXT NOT NULL,
    details JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Roles table
CREATE TABLE IF NOT EXISTS public.roles (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    is_system_role BOOLEAN DEFAULT false,
    parent_role_id UUID REFERENCES public.roles(id),
    permissions JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- User roles table (many-to-many relationship)
CREATE TABLE IF NOT EXISTS public.user_roles (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID REFERENCES public.users(id) ON DELETE CASCADE,
    role_id UUID REFERENCES public.roles(id) ON DELETE CASCADE,
    assigned_by UUID REFERENCES public.users(id),
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(user_id, role_id)
);

-- File transfers table
CREATE TABLE IF NOT EXISTS public.file_transfers (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID REFERENCES public.users(id) ON DELETE CASCADE,
    session_id UUID REFERENCES public.sessions(id) ON DELETE CASCADE,
    from_device_id UUID REFERENCES public.devices(id) ON DELETE CASCADE,
    to_device_id UUID REFERENCES public.devices(id) ON DELETE CASCADE,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    file_hash TEXT,
    transfer_type TEXT NOT NULL CHECK (transfer_type IN ('upload', 'download')),
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'failed', 'cancelled')),
    progress INTEGER DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Connection logs table
CREATE TABLE IF NOT EXISTS public.connection_logs (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID REFERENCES public.users(id) ON DELETE CASCADE,
    device_id UUID REFERENCES public.devices(id) ON DELETE CASCADE,
    remote_device_id UUID REFERENCES public.devices(id) ON DELETE CASCADE,
    connection_type TEXT NOT NULL CHECK (connection_type IN ('webrtc', 'websocket', 'direct')),
    status TEXT NOT NULL CHECK (status IN ('connecting', 'connected', 'disconnected', 'failed')),
    local_ip INET,
    remote_ip INET,
    connection_time_ms INTEGER,
    error_message TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_devices_user_id ON public.devices(user_id);
CREATE INDEX IF NOT EXISTS idx_devices_device_id ON public.devices(device_id);
CREATE INDEX IF NOT EXISTS idx_devices_is_online ON public.devices(is_online);
CREATE INDEX IF NOT EXISTS idx_devices_last_seen ON public.devices(last_seen);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON public.sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_device_id ON public.sessions(device_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON public.sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON public.sessions(started_at);

CREATE INDEX IF NOT EXISTS idx_access_sessions_user_id ON public.access_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_access_sessions_device_id ON public.access_sessions(device_id);
CREATE INDEX IF NOT EXISTS idx_access_sessions_status ON public.access_sessions(status);
CREATE INDEX IF NOT EXISTS idx_access_sessions_expires_at ON public.access_sessions(expires_at);

CREATE INDEX IF NOT EXISTS idx_temporary_permissions_session_id ON public.temporary_permissions(access_session_id);
CREATE INDEX IF NOT EXISTS idx_temporary_permissions_expires_at ON public.temporary_permissions(expires_at);
CREATE INDEX IF NOT EXISTS idx_temporary_permissions_is_active ON public.temporary_permissions(is_active);

CREATE INDEX IF NOT EXISTS idx_audit_log_user_id ON public.audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_event_type ON public.audit_log(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_log_level ON public.audit_log(level);
CREATE INDEX IF NOT EXISTS idx_audit_log_created_at ON public.audit_log(created_at);

CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON public.user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON public.user_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_is_active ON public.user_roles(is_active);

CREATE INDEX IF NOT EXISTS idx_file_transfers_user_id ON public.file_transfers(user_id);
CREATE INDEX IF NOT EXISTS idx_file_transfers_session_id ON public.file_transfers(session_id);
CREATE INDEX IF NOT EXISTS idx_file_transfers_status ON public.file_transfers(status);
CREATE INDEX IF NOT EXISTS idx_file_transfers_created_at ON public.file_transfers(created_at);

CREATE INDEX IF NOT EXISTS idx_connection_logs_user_id ON public.connection_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_connection_logs_device_id ON public.connection_logs(device_id);
CREATE INDEX IF NOT EXISTS idx_connection_logs_status ON public.connection_logs(status);
CREATE INDEX IF NOT EXISTS idx_connection_logs_created_at ON public.connection_logs(created_at);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION public.handle_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for updated_at
CREATE TRIGGER handle_users_updated_at
    BEFORE UPDATE ON public.users
    FOR EACH ROW
    EXECUTE FUNCTION public.handle_updated_at();

CREATE TRIGGER handle_devices_updated_at
    BEFORE UPDATE ON public.devices
    FOR EACH ROW
    EXECUTE FUNCTION public.handle_updated_at();

CREATE TRIGGER handle_sessions_updated_at
    BEFORE UPDATE ON public.sessions
    FOR EACH ROW
    EXECUTE FUNCTION public.handle_updated_at();

CREATE TRIGGER handle_access_sessions_updated_at
    BEFORE UPDATE ON public.access_sessions
    FOR EACH ROW
    EXECUTE FUNCTION public.handle_updated_at();

CREATE TRIGGER handle_access_policies_updated_at
    BEFORE UPDATE ON public.access_policies
    FOR EACH ROW
    EXECUTE FUNCTION public.handle_updated_at();

CREATE TRIGGER handle_roles_updated_at
    BEFORE UPDATE ON public.roles
    FOR EACH ROW
    EXECUTE FUNCTION public.handle_updated_at();

CREATE TRIGGER handle_file_transfers_updated_at
    BEFORE UPDATE ON public.file_transfers
    FOR EACH ROW
    EXECUTE FUNCTION public.handle_updated_at();

-- Row Level Security (RLS) policies
ALTER TABLE public.users ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.devices ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.access_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.temporary_permissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.audit_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.user_roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.file_transfers ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.connection_logs ENABLE ROW LEVEL SECURITY;

-- Users can only access their own data
CREATE POLICY "Users can view own profile" ON public.users
    FOR SELECT USING (auth.uid() = id);

CREATE POLICY "Users can update own profile" ON public.users
    FOR UPDATE USING (auth.uid() = id);

-- Devices policies
CREATE POLICY "Users can view own devices" ON public.devices
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own devices" ON public.devices
    FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update own devices" ON public.devices
    FOR UPDATE USING (auth.uid() = user_id);

CREATE POLICY "Users can delete own devices" ON public.devices
    FOR DELETE USING (auth.uid() = user_id);

-- Sessions policies
CREATE POLICY "Users can view own sessions" ON public.sessions
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own sessions" ON public.sessions
    FOR INSERT WITH CHECK (auth.uid() = user_id);

-- Access sessions policies
CREATE POLICY "Users can view own access sessions" ON public.access_sessions
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own access sessions" ON public.access_sessions
    FOR INSERT WITH CHECK (auth.uid() = user_id);

-- Temporary permissions policies
CREATE POLICY "Users can view own temporary permissions" ON public.temporary_permissions
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM public.access_sessions 
            WHERE id = access_session_id AND user_id = auth.uid()
        )
    );

-- Audit log policies
CREATE POLICY "Users can view own audit logs" ON public.audit_log
    FOR SELECT USING (auth.uid() = user_id);

-- User roles policies
CREATE POLICY "Users can view own roles" ON public.user_roles
    FOR SELECT USING (auth.uid() = user_id);

-- File transfers policies
CREATE POLICY "Users can view own file transfers" ON public.file_transfers
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own file transfers" ON public.file_transfers
    FOR INSERT WITH CHECK (auth.uid() = user_id);

-- Connection logs policies
CREATE POLICY "Users can view own connection logs" ON public.connection_logs
    FOR SELECT USING (auth.uid() = user_id);

-- Insert default system roles
INSERT INTO public.roles (name, description, is_system_role, permissions) VALUES
('admin', 'System administrator with full access', true, '{"all": true}'),
('operator', 'Operator with remote control capabilities', true, '{"remote_control": true, "screen_sharing": true, "file_transfer": true}'),
('viewer', 'Viewer with screen sharing only', true, '{"screen_sharing": true}'),
('guest', 'Guest with limited access', true, '{"screen_sharing": false, "remote_control": false, "file_transfer": false}')
ON CONFLICT (name) DO NOTHING;

-- Create function to automatically create user profile after signup
CREATE OR REPLACE FUNCTION public.handle_new_user()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO public.users (id, email, username, display_name)
    VALUES (
        NEW.id,
        NEW.email,
        COALESCE(NEW.raw_user_meta_data->>'username', split_part(NEW.email, '@', 1)),
        COALESCE(NEW.raw_user_meta_data->>'display_name', split_part(NEW.email, '@', 1))
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Create trigger for new user signup
CREATE TRIGGER on_auth_user_created
    AFTER INSERT ON auth.users
    FOR EACH ROW
    EXECUTE FUNCTION public.handle_new_user();

-- Create function to cleanup expired sessions
CREATE OR REPLACE FUNCTION public.cleanup_expired_sessions()
RETURNS void AS $$
BEGIN
    -- Update expired access sessions
    UPDATE public.access_sessions 
    SET status = 'expired', updated_at = NOW()
    WHERE expires_at < NOW() AND status = 'active';
    
    -- Update expired temporary permissions
    UPDATE public.temporary_permissions 
    SET is_active = false
    WHERE expires_at < NOW() AND is_active = true;
    
    -- Delete old audit logs (older than 90 days)
    DELETE FROM public.audit_log 
    WHERE created_at < NOW() - INTERVAL '90 days';
    
    -- Delete old connection logs (older than 30 days)
    DELETE FROM public.connection_logs 
    WHERE created_at < NOW() - INTERVAL '30 days';
END;
$$ LANGUAGE plpgsql;

-- Create a cron job to run cleanup daily (requires pg_cron extension)
-- SELECT cron.schedule('cleanup-expired-sessions', '0 2 * * *', 'SELECT public.cleanup_expired_sessions();');

-- Grant necessary permissions
GRANT ALL ON public.users TO authenticated;
GRANT ALL ON public.devices TO authenticated;
GRANT ALL ON public.sessions TO authenticated;
GRANT ALL ON public.access_sessions TO authenticated;
GRANT ALL ON public.temporary_permissions TO authenticated;
GRANT ALL ON public.audit_log TO authenticated;
GRANT ALL ON public.user_roles TO authenticated;
GRANT ALL ON public.file_transfers TO authenticated;
GRANT ALL ON public.connection_logs TO authenticated;

GRANT SELECT ON public.roles TO authenticated;
GRANT SELECT ON public.access_policies TO authenticated;

-- Grant usage on sequences
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO authenticated;
