-- Database initialization script for Spreadsheet Versioning Service
-- This script creates the necessary extensions and initial configuration

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Create initial user if not exists (development only)
DO
$do$
BEGIN
   IF NOT EXISTS (
      SELECT FROM pg_catalog.pg_roles
      WHERE  rolname = 'versioning_user') THEN

      CREATE ROLE versioning_user LOGIN PASSWORD 'versioning_password';
   END IF;
END
$do$;

-- Grant necessary permissions
GRANT ALL PRIVILEGES ON DATABASE tracseq_versioning TO versioning_user;
GRANT ALL ON SCHEMA public TO versioning_user;

-- Set default privileges for future objects
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO versioning_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO versioning_user;

-- Initial configuration
INSERT INTO pg_catalog.pg_settings_history (name, setting, source) 
VALUES ('log_statement', 'all', 'configuration file')
ON CONFLICT DO NOTHING; 
