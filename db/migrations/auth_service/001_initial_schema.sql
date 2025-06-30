-- Initial Auth Service Database Schema
-- This migration creates all necessary tables for authentication functionality

-- User roles enum
CREATE TYPE user_role AS ENUM (
    'guest',
    'data_analyst', 
    'research_scientist',
    'lab_technician',
    'principal_investigator',
    'lab_administrator'
);

-- User status enum
CREATE TYPE user_status AS ENUM (
    'active',
    'inactive',
    'suspended',
    'deleted'
);

-- Main users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    role user_role NOT NULL DEFAULT 'guest',
    status user_status NOT NULL DEFAULT 'active',
    
    -- Profile information
    department VARCHAR(100),
    position VARCHAR(100),
    lab_affiliation VARCHAR(100),
    phone VARCHAR(50),
    
    -- Authentication fields
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    password_changed_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Indexes
    CONSTRAINT users_email_check CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- User sessions table
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Token information (hashed)
    token_hash VARCHAR(64) NOT NULL,
    refresh_token_hash VARCHAR(64),
    
    -- Session metadata
    device_info JSONB,
    ip_address INET,
    user_agent TEXT,
    
    -- Session lifecycle
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ
);

-- Password reset tokens table
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    used BOOLEAN NOT NULL DEFAULT FALSE,
    used_at TIMESTAMPTZ
);

-- Email verification tokens table
CREATE TABLE email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    used BOOLEAN NOT NULL DEFAULT FALSE,
    used_at TIMESTAMPTZ
);

-- Security audit log table
CREATE TABLE security_audit_log (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(50) NOT NULL,
    user_id UUID REFERENCES users(id),
    ip_address INET,
    user_agent TEXT,
    resource VARCHAR(255),
    action VARCHAR(100),
    outcome VARCHAR(20),
    severity VARCHAR(20) NOT NULL DEFAULT 'MEDIUM',
    details JSONB,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_users_last_login ON users(last_login_at);

CREATE INDEX idx_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_sessions_token_hash ON user_sessions(token_hash);
CREATE INDEX idx_sessions_refresh_token_hash ON user_sessions(refresh_token_hash);
CREATE INDEX idx_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX idx_sessions_revoked ON user_sessions(revoked);
CREATE INDEX idx_sessions_created_at ON user_sessions(created_at);

CREATE INDEX idx_password_reset_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_token_hash ON password_reset_tokens(token_hash);
CREATE INDEX idx_password_reset_expires_at ON password_reset_tokens(expires_at);
CREATE INDEX idx_password_reset_used ON password_reset_tokens(used);

CREATE INDEX idx_email_verification_user_id ON email_verification_tokens(user_id);
CREATE INDEX idx_email_verification_token_hash ON email_verification_tokens(token_hash);
CREATE INDEX idx_email_verification_expires_at ON email_verification_tokens(expires_at);
CREATE INDEX idx_email_verification_used ON email_verification_tokens(used);

CREATE INDEX idx_audit_log_user_id ON security_audit_log(user_id);
CREATE INDEX idx_audit_log_event_type ON security_audit_log(event_type);
CREATE INDEX idx_audit_log_timestamp ON security_audit_log(timestamp);
CREATE INDEX idx_audit_log_severity ON security_audit_log(severity);

-- Create function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at for users table
CREATE TRIGGER update_users_updated_at 
    BEFORE UPDATE ON users 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Create function to automatically lock accounts after too many failed attempts
CREATE OR REPLACE FUNCTION check_failed_login_attempts()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.failed_login_attempts >= 5 AND OLD.failed_login_attempts < 5 THEN
        NEW.locked_until = NOW() + INTERVAL '15 minutes';
    END IF;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically lock accounts
CREATE TRIGGER check_failed_login_attempts_trigger
    BEFORE UPDATE ON users
    FOR EACH ROW
    WHEN (NEW.failed_login_attempts IS DISTINCT FROM OLD.failed_login_attempts)
    EXECUTE FUNCTION check_failed_login_attempts();

-- Create function to clean up expired tokens
CREATE OR REPLACE FUNCTION cleanup_expired_tokens()
RETURNS void AS $$
BEGIN
    -- Delete expired password reset tokens
    DELETE FROM password_reset_tokens 
    WHERE expires_at < NOW() AND used = TRUE;
    
    -- Delete expired email verification tokens
    DELETE FROM email_verification_tokens 
    WHERE expires_at < NOW() AND used = TRUE;
    
    -- Delete expired and revoked sessions older than 30 days
    DELETE FROM user_sessions 
    WHERE (expires_at < NOW() OR revoked = TRUE) 
    AND created_at < NOW() - INTERVAL '30 days';
    
    -- Delete old audit log entries older than 1 year
    DELETE FROM security_audit_log 
    WHERE timestamp < NOW() - INTERVAL '1 year';
END;
$$ language 'plpgsql';

-- Insert default admin user (password: admin123)
-- Password hash generated using Argon2 for "admin123"
INSERT INTO users (
    id,
    email, 
    password_hash, 
    first_name, 
    last_name, 
    role, 
    status,
    email_verified,
    department,
    position
) VALUES (
    '00000000-0000-0000-0000-000000000001',
    'admin@lab.local',
    '$argon2id$v=19$m=19456,t=2,p=1$VhepxIzMB2nSzs4fPZ/Hfw$qgxOOjqB6t2x5v7Y1ALBbgw7DxHQF8vGS7P/2LyM8Ss',
    'System',
    'Administrator',
    'lab_administrator',
    'active',
    TRUE,
    'IT',
    'System Administrator'
);

-- Create view for user statistics
CREATE VIEW user_stats AS
SELECT 
    role,
    status,
    COUNT(*) as user_count,
    COUNT(CASE WHEN last_login_at > NOW() - INTERVAL '30 days' THEN 1 END) as active_last_30_days,
    COUNT(CASE WHEN created_at > NOW() - INTERVAL '30 days' THEN 1 END) as new_last_30_days
FROM users 
GROUP BY role, status;

-- Create view for session statistics  
CREATE VIEW session_stats AS
SELECT 
    DATE_TRUNC('day', created_at) as date,
    COUNT(*) as total_sessions,
    COUNT(CASE WHEN revoked = FALSE AND expires_at > NOW() THEN 1 END) as active_sessions,
    COUNT(DISTINCT user_id) as unique_users
FROM user_sessions
WHERE created_at > NOW() - INTERVAL '30 days'
GROUP BY DATE_TRUNC('day', created_at)
ORDER BY date DESC;

-- Create view for security events
CREATE VIEW security_events_summary AS
SELECT 
    DATE_TRUNC('hour', timestamp) as hour,
    event_type,
    severity,
    COUNT(*) as event_count
FROM security_audit_log
WHERE timestamp > NOW() - INTERVAL '24 hours'
GROUP BY DATE_TRUNC('hour', timestamp), event_type, severity
ORDER BY hour DESC, event_count DESC; 
