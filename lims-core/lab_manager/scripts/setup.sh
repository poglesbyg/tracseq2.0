#!/bin/bash

# Create PostgreSQL database
sudo -u postgres psql -c "CREATE DATABASE lab_manager;"
sudo -u postgres psql -c "CREATE USER lab_manager WITH PASSWORD 'lab_manager';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE lab_manager TO lab_manager;"

# Create storage directory
mkdir -p storage

# Create .env file
cat > .env << EOL
DATABASE_URL=postgres://lab_manager:lab_manager@localhost:5432/lab_manager
STORAGE_PATH=./storage
RUST_LOG=info
EOL

# Run migrations
cargo sqlx prepare 
