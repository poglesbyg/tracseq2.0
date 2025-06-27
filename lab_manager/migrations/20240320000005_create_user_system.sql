-- Migration for user management system
-- Creates tables for users, roles, sessions, and authentication

-- User roles enumeration (add new values to existing type if it exists)
DO $$ BEGIN
    CREATE TYPE user_role AS ENUM (
        'lab_administrator',
        'principal_investigator', 
        'lab_technician',
        'research_scientist',
        'data_analyst',
        'guest'
    );
EXCEPTION
    WHEN duplicate_object THEN
        -- Type already exists, add new enum values if they don't exist
        BEGIN
            ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'lab_administrator';
            ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'principal_investigator';
            ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'lab_technician';
            ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'research_scientist';
            ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'data_analyst';
            ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'guest';
        EXCEPTION WHEN OTHERS THEN
            -- Ignore if values already exist or other errors
            NULL;
        END;
END $$;

-- User account status enumeration (add new values to existing type if it exists)
DO $$ BEGIN
    CREATE TYPE user_status AS ENUM (
        'active',
        'inactive',
        'locked',
        'pending_verification'
    );
EXCEPTION
    WHEN duplicate_object THEN
        -- Type already exists, add new enum values if they don't exist
        BEGIN
            ALTER TYPE user_status ADD VALUE IF NOT EXISTS 'locked';
            ALTER TYPE user_status ADD VALUE IF NOT EXISTS 'pending_verification';
        EXCEPTION WHEN OTHERS THEN
            -- Ignore if values already exist or other errors
            NULL;
        END;
END $$;

-- Main users table (skip if already exists)
DO $$ BEGIN
    CREATE TABLE users (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        email VARCHAR(255) NOT NULL UNIQUE,
        password_hash VARCHAR(255) NOT NULL,
        first_name VARCHAR(100) NOT NULL,
        last_name VARCHAR(100) NOT NULL,
        role user_role NOT NULL DEFAULT 'guest',
        status user_status NOT NULL DEFAULT 'pending_verification',
        
        -- Laboratory affiliation
        lab_affiliation VARCHAR(255),
        department VARCHAR(255),
        position VARCHAR(255),
        
        -- Contact information
        phone VARCHAR(20),
        office_location VARCHAR(255),
        
        -- Security fields
        email_verified BOOLEAN DEFAULT FALSE,
        failed_login_attempts INTEGER DEFAULT 0,
        locked_until TIMESTAMPTZ,
        last_login TIMESTAMPTZ,
        password_changed_at TIMESTAMPTZ DEFAULT NOW(),
        
        -- Audit fields
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        created_by UUID REFERENCES users(id),
        metadata JSONB NOT NULL DEFAULT '{}'::jsonb
    );
EXCEPTION
    WHEN duplicate_table THEN
        -- Table already exists, add missing columns if needed
        BEGIN
            -- Try to add columns that might be missing
            ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255);
            ALTER TABLE users ADD COLUMN IF NOT EXISTS first_name VARCHAR(100);
            ALTER TABLE users ADD COLUMN IF NOT EXISTS last_name VARCHAR(100);
            ALTER TABLE users ADD COLUMN IF NOT EXISTS lab_affiliation VARCHAR(255);
            ALTER TABLE users ADD COLUMN IF NOT EXISTS department VARCHAR(255);
            ALTER TABLE users ADD COLUMN IF NOT EXISTS position VARCHAR(255);
            ALTER TABLE users ADD COLUMN IF NOT EXISTS phone VARCHAR(20);
            ALTER TABLE users ADD COLUMN IF NOT EXISTS office_location VARCHAR(255);
            ALTER TABLE users ADD COLUMN IF NOT EXISTS email_verified BOOLEAN DEFAULT FALSE;
            ALTER TABLE users ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER DEFAULT 0;
            ALTER TABLE users ADD COLUMN IF NOT EXISTS locked_until TIMESTAMPTZ;
            ALTER TABLE users ADD COLUMN IF NOT EXISTS password_changed_at TIMESTAMPTZ DEFAULT NOW();
            ALTER TABLE users ADD COLUMN IF NOT EXISTS created_by UUID;
            ALTER TABLE users ADD COLUMN IF NOT EXISTS metadata JSONB DEFAULT '{}'::jsonb;
        EXCEPTION WHEN OTHERS THEN
            -- Ignore any errors in adding columns
            NULL;
        END;
END $$;

-- User sessions table for JWT token management (create if not exists)
DO $$ BEGIN
    CREATE TABLE user_sessions (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        token_hash VARCHAR(255) NOT NULL UNIQUE, -- Hash of the JWT token
        device_info VARCHAR(500), -- User agent, device info
        ip_address INET,
        expires_at TIMESTAMPTZ NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        last_used_at TIMESTAMPTZ DEFAULT NOW()
    );
EXCEPTION
    WHEN duplicate_table THEN
        -- Table already exists, add missing columns if needed
        BEGIN
            ALTER TABLE user_sessions ADD COLUMN IF NOT EXISTS token_hash VARCHAR(255);
            ALTER TABLE user_sessions ADD COLUMN IF NOT EXISTS device_info VARCHAR(500);
            ALTER TABLE user_sessions ADD COLUMN IF NOT EXISTS last_used_at TIMESTAMPTZ DEFAULT NOW();
        EXCEPTION WHEN OTHERS THEN
            -- Ignore any errors in adding columns
            NULL;
        END;
END $$;

-- Password reset tokens table (create if not exists)
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Email verification tokens table (create if not exists)
CREATE TABLE IF NOT EXISTS email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User activity log for audit trail (create if not exists)
CREATE TABLE IF NOT EXISTS user_activity_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL, -- login, logout, password_change, etc.
    resource_type VARCHAR(50), -- samples, templates, etc.
    resource_id UUID,
    ip_address INET,
    user_agent VARCHAR(500),
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Role permissions table (for fine-grained permissions) (create if not exists)
CREATE TABLE IF NOT EXISTS role_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    role user_role NOT NULL,
    resource VARCHAR(50) NOT NULL, -- samples, templates, users, etc.
    action VARCHAR(50) NOT NULL, -- create, read, update, delete, manage
    granted BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(role, resource, action)
);

-- Indexes for performance (create if not exists)
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_users_lab_affiliation ON users(lab_affiliation);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_token_hash ON user_sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at);

CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_token_hash ON password_reset_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_expires_at ON password_reset_tokens(expires_at);

CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_user_id ON email_verification_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_token_hash ON email_verification_tokens(token_hash);

CREATE INDEX IF NOT EXISTS idx_user_activity_log_user_id ON user_activity_log(user_id);
CREATE INDEX IF NOT EXISTS idx_user_activity_log_action ON user_activity_log(action);
CREATE INDEX IF NOT EXISTS idx_user_activity_log_created_at ON user_activity_log(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_role_permissions_role ON role_permissions(role);
CREATE INDEX IF NOT EXISTS idx_role_permissions_resource ON role_permissions(resource, action);

-- Insert default role permissions (on conflict do nothing)
INSERT INTO role_permissions (role, resource, action, granted) VALUES
-- Lab Administrator - Full access
('lab_administrator', 'users', 'create', TRUE),
('lab_administrator', 'users', 'read', TRUE),
('lab_administrator', 'users', 'update', TRUE),
('lab_administrator', 'users', 'delete', TRUE),
('lab_administrator', 'users', 'manage', TRUE),
('lab_administrator', 'samples', 'create', TRUE),
('lab_administrator', 'samples', 'read', TRUE),
('lab_administrator', 'samples', 'update', TRUE),
('lab_administrator', 'samples', 'delete', TRUE),
('lab_administrator', 'templates', 'create', TRUE),
('lab_administrator', 'templates', 'read', TRUE),
('lab_administrator', 'templates', 'update', TRUE),
('lab_administrator', 'templates', 'delete', TRUE),
('lab_administrator', 'sequencing', 'create', TRUE),
('lab_administrator', 'sequencing', 'read', TRUE),
('lab_administrator', 'sequencing', 'update', TRUE),
('lab_administrator', 'sequencing', 'delete', TRUE),
('lab_administrator', 'storage', 'create', TRUE),
('lab_administrator', 'storage', 'read', TRUE),
('lab_administrator', 'storage', 'update', TRUE),
('lab_administrator', 'storage', 'delete', TRUE),
('lab_administrator', 'spreadsheets', 'create', TRUE),
('lab_administrator', 'spreadsheets', 'read', TRUE),
('lab_administrator', 'spreadsheets', 'update', TRUE),
('lab_administrator', 'spreadsheets', 'delete', TRUE),
('lab_administrator', 'reports', 'create', TRUE),
('lab_administrator', 'reports', 'read', TRUE),

-- Principal Investigator - Lab oversight
('principal_investigator', 'users', 'read', TRUE),
('principal_investigator', 'samples', 'create', TRUE),
('principal_investigator', 'samples', 'read', TRUE),
('principal_investigator', 'samples', 'update', TRUE),
('principal_investigator', 'templates', 'create', TRUE),
('principal_investigator', 'templates', 'read', TRUE),
('principal_investigator', 'templates', 'update', TRUE),
('principal_investigator', 'sequencing', 'create', TRUE),
('principal_investigator', 'sequencing', 'read', TRUE),
('principal_investigator', 'sequencing', 'update', TRUE),
('principal_investigator', 'storage', 'read', TRUE),
('principal_investigator', 'storage', 'update', TRUE),
('principal_investigator', 'spreadsheets', 'create', TRUE),
('principal_investigator', 'spreadsheets', 'read', TRUE),
('principal_investigator', 'reports', 'create', TRUE),
('principal_investigator', 'reports', 'read', TRUE),

-- Lab Technician - Sample processing
('lab_technician', 'samples', 'create', TRUE),
('lab_technician', 'samples', 'read', TRUE),
('lab_technician', 'samples', 'update', TRUE),
('lab_technician', 'templates', 'read', TRUE),
('lab_technician', 'sequencing', 'read', TRUE),
('lab_technician', 'sequencing', 'update', TRUE),
('lab_technician', 'storage', 'read', TRUE),
('lab_technician', 'storage', 'update', TRUE),
('lab_technician', 'spreadsheets', 'create', TRUE),
('lab_technician', 'spreadsheets', 'read', TRUE),

-- Research Scientist - Data analysis
('research_scientist', 'samples', 'read', TRUE),
('research_scientist', 'templates', 'read', TRUE),
('research_scientist', 'sequencing', 'read', TRUE),
('research_scientist', 'storage', 'read', TRUE),
('research_scientist', 'spreadsheets', 'create', TRUE),
('research_scientist', 'spreadsheets', 'read', TRUE),
('research_scientist', 'reports', 'create', TRUE),
('research_scientist', 'reports', 'read', TRUE),

-- Data Analyst - Analytics and reporting
('data_analyst', 'samples', 'read', TRUE),
('data_analyst', 'templates', 'read', TRUE),
('data_analyst', 'sequencing', 'read', TRUE),
('data_analyst', 'storage', 'read', TRUE),
('data_analyst', 'spreadsheets', 'read', TRUE),
('data_analyst', 'reports', 'create', TRUE),
('data_analyst', 'reports', 'read', TRUE),

-- Guest - Limited read access
('guest', 'samples', 'read', TRUE),
('guest', 'templates', 'read', TRUE),
('guest', 'reports', 'read', TRUE)
ON CONFLICT (role, resource, action) DO NOTHING;

-- Create a default admin user (password will need to be set on first run)
-- Password hash for 'admin123' - should be changed immediately
INSERT INTO users (
    email,
    password_hash,
    first_name,
    last_name,
    role,
    status,
    email_verified,
    lab_affiliation,
    department,
    position
) VALUES (
    'admin@lab.local',
    '$argon2id$v=19$m=65536,t=3,p=4$VGhpc0lzQVNhbHQ$rP2Y1zKGQl8fZCdZHPgZb1sR5vMQl6JsWXL1QLvE3Xo', -- admin123
    'Lab',
    'Administrator',
    'lab_administrator',
    'active',
    TRUE,
    'Default Laboratory',
    'System Administration',
    'System Administrator'
) ON CONFLICT (email) DO NOTHING; 
