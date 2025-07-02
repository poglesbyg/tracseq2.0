#!/bin/bash

# TracSeq 2.0 - Run All Seed Scripts
# This script loads all seed data into the database

echo "🌱 Starting TracSeq 2.0 database seeding..."

# Database connection parameters
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5433}"
DB_NAME="${DB_NAME:-lims_db}"
DB_USER="${DB_USER:-postgres}"
DB_PASS="${DB_PASS:-postgres}"

# Export for psql
export PGPASSWORD=$DB_PASS

# Function to run a seed file
run_seed() {
    local seed_file=$1
    echo "📝 Running $seed_file..."
    psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f "$seed_file"
    if [ $? -eq 0 ]; then
        echo "✅ $seed_file completed successfully"
    else
        echo "❌ Error running $seed_file"
        exit 1
    fi
}

# Run seed files in order
echo "🔄 Starting seed process..."

# Clear existing data (optional - comment out if you want to keep existing data)
echo "🗑️  Clearing existing data..."
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "
    -- Clear in reverse order of dependencies
    TRUNCATE TABLE audit_logs CASCADE;
    TRUNCATE TABLE storage_temperature_logs CASCADE;
    TRUNCATE TABLE storage_movement_history CASCADE;
    TRUNCATE TABLE spreadsheet_collaborators CASCADE;
    TRUNCATE TABLE spreadsheet_versions CASCADE;
    TRUNCATE TABLE spreadsheets CASCADE;
    TRUNCATE TABLE sequencing_quality_metrics CASCADE;
    TRUNCATE TABLE sequencing_jobs CASCADE;
    TRUNCATE TABLE sample_quality_control CASCADE;
    TRUNCATE TABLE samples CASCADE;
    TRUNCATE TABLE storage_locations CASCADE;
    TRUNCATE TABLE storage_zones CASCADE;
    TRUNCATE TABLE user_sessions CASCADE;
    TRUNCATE TABLE user_roles CASCADE;
    TRUNCATE TABLE users CASCADE;
    TRUNCATE TABLE roles CASCADE;
    TRUNCATE TABLE permissions CASCADE;
    TRUNCATE TABLE role_permissions CASCADE;
    TRUNCATE TABLE templates CASCADE;
    TRUNCATE TABLE reports CASCADE;
    TRUNCATE TABLE notifications CASCADE;
    TRUNCATE TABLE events CASCADE;
    TRUNCATE TABLE projects CASCADE;
    TRUNCATE TABLE batches CASCADE;
    TRUNCATE TABLE library_preparations CASCADE;
    TRUNCATE TABLE qc_reviews CASCADE;
    TRUNCATE TABLE flow_cells CASCADE;
"

# Run seed files
run_seed "db/seeds/01_users_and_auth.sql"
run_seed "db/seeds/02_storage.sql"
run_seed "db/seeds/03_projects_and_templates.sql"
run_seed "db/seeds/04_samples_and_sequencing.sql"
run_seed "db/seeds/05_reports_notifications_events.sql"

echo "🎉 Database seeding completed successfully!"
echo ""
echo "📊 Summary of seeded data:"
echo "  - 6 Users with different roles"
echo "  - 5 Active projects"
echo "  - 10 Samples across different projects"
echo "  - 6 Storage zones with locations"
echo "  - 7 Templates for various workflows"
echo "  - 5 Sequencing jobs"
echo "  - 6 Reports"
echo "  - 8 Notifications"
echo "  - 10 System events"
echo ""
echo "🔐 Login credentials:"
echo "  Admin: admin / password123"
echo "  Lab Supervisor: jsmith / password123"
echo "  Lab Technician: mjohnson / password123"
echo "  Quality Analyst: dwilliams / password123"
echo "  Researcher: rjones / password123"
