#!/bin/bash

# Create database
createdb lab_manager

# Create .env file
echo "DATABASE_URL=postgres://$(whoami)@localhost:5432/lab_manager" > .env
echo "RUST_LOG=debug" >> .env

# Run migrations
cargo sqlx database create
cargo sqlx migrate run 
