-- Initialize multiple databases for microservices
-- This script creates separate databases for each service

-- Main TracSeq databases
CREATE DATABASE tracseq;
GRANT ALL PRIVILEGES ON DATABASE tracseq TO postgres;

CREATE DATABASE tracseq_main;
GRANT ALL PRIVILEGES ON DATABASE tracseq_main TO postgres;

-- Auth Service Database
CREATE DATABASE tracseq_auth;
GRANT ALL PRIVILEGES ON DATABASE tracseq_auth TO postgres;

-- Sample Service Database  
CREATE DATABASE tracseq_samples;
GRANT ALL PRIVILEGES ON DATABASE tracseq_samples TO postgres;

-- Storage Service Database
CREATE DATABASE tracseq_storage;
GRANT ALL PRIVILEGES ON DATABASE tracseq_storage TO postgres;

-- Template Service Database
CREATE DATABASE tracseq_templates;
GRANT ALL PRIVILEGES ON DATABASE tracseq_templates TO postgres;

-- Sequencing Service Database
CREATE DATABASE tracseq_sequencing;
GRANT ALL PRIVILEGES ON DATABASE tracseq_sequencing TO postgres;

-- Notification Service Database
CREATE DATABASE tracseq_notifications;
GRANT ALL PRIVILEGES ON DATABASE tracseq_notifications TO postgres;

-- RAG Service Database
CREATE DATABASE tracseq_rag;
GRANT ALL PRIVILEGES ON DATABASE tracseq_rag TO postgres;

-- Transaction Service Database
CREATE DATABASE tracseq_transactions;
GRANT ALL PRIVILEGES ON DATABASE tracseq_transactions TO postgres;

-- Event Service Database
CREATE DATABASE tracseq_events;
GRANT ALL PRIVILEGES ON DATABASE tracseq_events TO postgres;

-- QAQC Service Database
CREATE DATABASE tracseq_qaqc;
GRANT ALL PRIVILEGES ON DATABASE tracseq_qaqc TO postgres;

-- Library Service Database
CREATE DATABASE tracseq_library;
GRANT ALL PRIVILEGES ON DATABASE tracseq_library TO postgres;

-- Spreadsheet Versioning Service Database
CREATE DATABASE tracseq_spreadsheets;
GRANT ALL PRIVILEGES ON DATABASE tracseq_spreadsheets TO postgres;

-- API Gateway Database
CREATE DATABASE tracseq_gateway;
GRANT ALL PRIVILEGES ON DATABASE tracseq_gateway TO postgres;

-- Legacy databases for backward compatibility
CREATE DATABASE auth_db;
GRANT ALL PRIVILEGES ON DATABASE auth_db TO postgres;

CREATE DATABASE sample_db;
GRANT ALL PRIVILEGES ON DATABASE sample_db TO postgres;

CREATE DATABASE storage_db;
GRANT ALL PRIVILEGES ON DATABASE storage_db TO postgres;

CREATE DATABASE template_db;
GRANT ALL PRIVILEGES ON DATABASE template_db TO postgres;

CREATE DATABASE sequencing_db;
GRANT ALL PRIVILEGES ON DATABASE sequencing_db TO postgres;

CREATE DATABASE notification_db;
GRANT ALL PRIVILEGES ON DATABASE notification_db TO postgres;

CREATE DATABASE rag_db;
GRANT ALL PRIVILEGES ON DATABASE rag_db TO postgres;

CREATE DATABASE transaction_db;
GRANT ALL PRIVILEGES ON DATABASE transaction_db TO postgres;

-- Create extensions for TracSeq databases
\c tracseq_auth;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c tracseq_samples;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c tracseq_storage;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c tracseq_templates;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c tracseq_sequencing;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c tracseq_notifications;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c tracseq_rag;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "vector";

\c tracseq_transactions;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c tracseq_events;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c tracseq_qaqc;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c tracseq_library;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c tracseq_spreadsheets;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create extensions for legacy databases
\c auth_db;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c sample_db;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c storage_db;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c template_db;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c sequencing_db;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c notification_db;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

\c rag_db;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "vector";

\c transaction_db;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto"; 
