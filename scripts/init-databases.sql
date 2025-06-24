-- TracSeq 2.0 Database Initialization Script
-- Creates multiple databases for microservices

-- Create additional databases
CREATE DATABASE IF NOT EXISTS lab_manager;
CREATE DATABASE IF NOT EXISTS enhanced_storage;
CREATE DATABASE IF NOT EXISTS auth_db;
CREATE DATABASE IF NOT EXISTS samples_db;

-- Create users for services (optional - for better security)
-- CREATE USER lab_manager_user WITH PASSWORD 'lab_manager_pass';
-- CREATE USER storage_user WITH PASSWORD 'storage_pass';
-- CREATE USER auth_user WITH PASSWORD 'auth_pass';

-- Grant permissions
-- GRANT ALL PRIVILEGES ON DATABASE lab_manager TO lab_manager_user;
-- GRANT ALL PRIVILEGES ON DATABASE enhanced_storage TO storage_user;
-- GRANT ALL PRIVILEGES ON DATABASE auth_db TO auth_user;

-- Enable extensions that might be needed
\c tracseq_db;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c lab_manager;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c enhanced_storage;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c auth_db;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto"; 
