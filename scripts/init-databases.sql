-- Initialize multiple databases for microservices
-- This script creates separate databases for each service

-- Auth Service Database
CREATE DATABASE auth_db;
GRANT ALL PRIVILEGES ON DATABASE auth_db TO postgres;

-- Sample Service Database  
CREATE DATABASE sample_db;
GRANT ALL PRIVILEGES ON DATABASE sample_db TO postgres;

-- Storage Service Database
CREATE DATABASE storage_db;
GRANT ALL PRIVILEGES ON DATABASE storage_db TO postgres;

-- Template Service Database
CREATE DATABASE template_db;
GRANT ALL PRIVILEGES ON DATABASE template_db TO postgres;

-- Sequencing Service Database
CREATE DATABASE sequencing_db;
GRANT ALL PRIVILEGES ON DATABASE sequencing_db TO postgres;

-- Notification Service Database
CREATE DATABASE notification_db;
GRANT ALL PRIVILEGES ON DATABASE notification_db TO postgres;

-- RAG Service Database
CREATE DATABASE rag_db;
GRANT ALL PRIVILEGES ON DATABASE rag_db TO postgres;

-- Transaction Service Database
CREATE DATABASE transaction_db;
GRANT ALL PRIVILEGES ON DATABASE transaction_db TO postgres;

-- Create extensions that might be needed
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
