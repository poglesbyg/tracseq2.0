-- Create users table for UNC SSO integration
-- Migration: 005_create_users_table.sql

-- Create user roles enum
CREATE TYPE user_role AS ENUM ('admin', 'researcher', 'lab_manager', 'viewer');

-- Create user status enum  
CREATE TYPE user_status AS ENUM ('active', 'inactive', 'pending', 'suspended');

-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- UNC SSO Identity fields
    unc_pid VARCHAR(50) UNIQUE NOT NULL,           -- UNC Person ID (from Shibboleth)
    email VARCHAR(255) UNIQUE NOT NULL,            -- Email address
    eppn VARCHAR(255),                             -- eduPersonPrincipalName
    given_name VARCHAR(100),                       -- First name
    family_name VARCHAR(100),                      -- Last name
    display_name VARCHAR(200),                     -- Full display name
    
    -- UNC Affiliation fields
    affiliation VARCHAR(100),                      -- Primary affiliation (student, faculty, staff)
    department VARCHAR(200),                       -- Department/unit
    title VARCHAR(200),                            -- Job title
    
    -- Application fields
    role user_role NOT NULL DEFAULT 'viewer',     -- Application role
    status user_status NOT NULL DEFAULT 'pending', -- Account status
    permissions JSONB DEFAULT '{}',               -- Additional permissions
    preferences JSONB DEFAULT '{}',               -- User preferences
    
    -- Audit fields
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE,
    created_by UUID REFERENCES users(id),
    
    -- Metadata
    metadata JSONB DEFAULT '{}'
);

-- Create user sessions table for SAML session management
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- SAML session information
    saml_session_id VARCHAR(255) UNIQUE,          -- SAML SessionIndex
    assertion_id VARCHAR(255),                    -- SAML AssertionID
    
    -- Session details
    ip_address INET,
    user_agent TEXT,
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_accessed TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    
    -- Metadata
    metadata JSONB DEFAULT '{}'
);

-- Create audit log table for user actions
CREATE TABLE user_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Action details
    action VARCHAR(100) NOT NULL,                 -- login, logout, create_sample, etc.
    resource_type VARCHAR(50),                    -- samples, templates, reports, etc.
    resource_id UUID,                             -- ID of affected resource
    
    -- Request details
    ip_address INET,
    user_agent TEXT,
    
    -- Additional data
    details JSONB DEFAULT '{}',
    
    -- Timestamp
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_users_unc_pid ON users(unc_pid);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_created_at ON users(created_at);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_saml_session_id ON user_sessions(saml_session_id);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);

CREATE INDEX idx_user_audit_log_user_id ON user_audit_log(user_id);
CREATE INDEX idx_user_audit_log_action ON user_audit_log(action);
CREATE INDEX idx_user_audit_log_created_at ON user_audit_log(created_at);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for updated_at
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create function to clean up expired sessions
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM user_sessions 
    WHERE expires_at < NOW();
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Insert default admin user (will be updated on first SSO login)
INSERT INTO users (
    unc_pid,
    email,
    given_name,
    family_name,
    display_name,
    role,
    status,
    affiliation
) VALUES (
    'admin',
    'admin@unc.edu',
    'System',
    'Administrator',
    'System Administrator',
    'admin',
    'active',
    'staff'
);

-- Create view for user summary
CREATE VIEW user_summary AS
SELECT 
    u.id,
    u.unc_pid,
    u.email,
    u.display_name,
    u.role,
    u.status,
    u.department,
    u.affiliation,
    u.last_login,
    u.created_at,
    COUNT(DISTINCT s.id) as active_sessions
FROM users u
LEFT JOIN user_sessions s ON u.id = s.user_id AND s.expires_at > NOW()
GROUP BY u.id, u.unc_pid, u.email, u.display_name, u.role, u.status, 
         u.department, u.affiliation, u.last_login, u.created_at; 
