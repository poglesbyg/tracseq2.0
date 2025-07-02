# TracSeq 2.0 Database Seed Scripts

This directory contains seed scripts to populate the TracSeq 2.0 database with realistic test data.

## Quick Start

To seed the database with test data, run:

```bash
docker exec -i lims-postgres psql -U postgres -d lims_db < db/seeds/final_seed.sql
```

## Test Users

All test users have the password: `password123`

| Email | Name | Role |
|-------|------|------|
| admin@tracseq.lab | System Administrator | lab_administrator |
| john.smith@tracseq.lab | John Smith | principal_investigator |
| mary.johnson@tracseq.lab | Mary Johnson | lab_technician |
| david.williams@tracseq.lab | David Williams | data_analyst |
| sarah.brown@tracseq.lab | Sarah Brown | lab_technician |
| robert.jones@university.edu | Robert Jones | guest |

## Sample Data

The seed script creates:
- 8 test samples across different projects
- Various sample types: tissue, blood, swab, stool
- Different storage locations and temperatures
- Realistic metadata including concentrations and volumes

## Available Scripts

- `final_seed.sql` - The main seed script that works with the current database schema
- `01_users_and_auth.sql` - Users and authentication data (for future schema)
- `02_storage.sql` - Storage zones and locations (for future schema)
- `03_projects_and_templates.sql` - Projects and templates (for future schema)
- `04_samples_and_sequencing.sql` - Samples and sequencing data (for future schema)
- `05_reports_notifications_events.sql` - Reports and notifications (for future schema)
- `run_all_seeds.sh` - Shell script to run all seeds (requires full schema)

## Notes

- The current database schema differs from the expected schema in the numbered seed files
- Use `final_seed.sql` for the current implementation
- The numbered seed files are preserved for future use when the full schema is implemented
- Sample metadata is stored in JSONB format with project information 