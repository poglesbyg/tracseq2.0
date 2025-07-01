#!/bin/bash

# TracSeq 2.0 Initial Data Setup Script
# Sets up initial users, workflows, templates, and other required data

set +e  # Don't exit on error

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "================================================"
echo " TracSeq 2.0 - Initial Data Setup"
echo "================================================"

# Base URLs
AUTH_SERVICE="http://localhost:8080"
TEMPLATE_SERVICE="http://localhost:8083"
SEQUENCING_SERVICE="http://localhost:8084"
NOTIFICATION_SERVICE="http://localhost:8085"
API_GATEWAY="http://localhost:18089"

# Function to check service availability
check_service() {
    local service_name=$1
    local url=$2
    
    echo -n "Checking $service_name... "
    response=$(curl -s -o /dev/null -w "%{http_code}" "$url/health" || echo "000")
    
    if [ "$response" == "200" ]; then
        echo -e "${GREEN}✓ Available${NC}"
        return 0
    else
        echo -e "${RED}✗ Unavailable${NC} (HTTP $response)"
        return 1
    fi
}

echo ""
echo "=== Checking Service Availability ==="
echo ""

# Check all services
SERVICES_OK=true
check_service "Auth Service" "$AUTH_SERVICE" || SERVICES_OK=false
check_service "Template Service" "$TEMPLATE_SERVICE" || SERVICES_OK=false
check_service "Sequencing Service" "$SEQUENCING_SERVICE" || SERVICES_OK=false
check_service "Notification Service" "$NOTIFICATION_SERVICE" || SERVICES_OK=false
check_service "API Gateway" "$API_GATEWAY" || SERVICES_OK=false

if [ "$SERVICES_OK" = false ]; then
    echo -e "\n${YELLOW}⚠ Warning: Some services are unavailable. Setup may be incomplete.${NC}"
    echo -n "Continue anyway? (y/N): "
    read -r continue_choice
    if [ "$continue_choice" != "y" ] && [ "$continue_choice" != "Y" ]; then
        echo "Setup cancelled."
        exit 1
    fi
fi

echo ""
echo "=== Setting Up Database ==="
echo ""

# Create database schema if needed
echo -n "Creating database extensions... "
docker exec tracseq-postgres-primary psql -U tracseq_admin -d tracseq_prod -c "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";" 2>/dev/null
echo -e "${GREEN}✓ Done${NC}"

echo ""
echo "=== Creating Initial Users ==="
echo ""

# Create admin user
echo -n "Creating admin user... "
ADMIN_RESPONSE=$(curl -s -X POST "$AUTH_SERVICE/auth/register" \
    -H "Content-Type: application/json" \
    -d '{
        "email": "admin@tracseq.com",
        "password": "Admin123!",
        "first_name": "System",
        "last_name": "Administrator"
    }' \
    -w "\n%{http_code}" | tail -1)

if [ "$ADMIN_RESPONSE" == "200" ] || [ "$ADMIN_RESPONSE" == "201" ] || [ "$ADMIN_RESPONSE" == "409" ]; then
    echo -e "${GREEN}✓ Created/Exists${NC}"
else
    echo -e "${YELLOW}⚠ Failed${NC} (HTTP $ADMIN_RESPONSE)"
fi

# Create lab manager user
echo -n "Creating lab manager user... "
LAB_MANAGER_RESPONSE=$(curl -s -X POST "$AUTH_SERVICE/auth/register" \
    -H "Content-Type: application/json" \
    -d '{
        "email": "lab.manager@tracseq.com",
        "password": "LabManager123!",
        "first_name": "Lab",
        "last_name": "Manager"
    }' \
    -w "\n%{http_code}" | tail -1)

if [ "$LAB_MANAGER_RESPONSE" == "200" ] || [ "$LAB_MANAGER_RESPONSE" == "201" ] || [ "$LAB_MANAGER_RESPONSE" == "409" ]; then
    echo -e "${GREEN}✓ Created/Exists${NC}"
else
    echo -e "${YELLOW}⚠ Failed${NC} (HTTP $LAB_MANAGER_RESPONSE)"
fi

# Create technician user
echo -n "Creating technician user... "
TECH_RESPONSE=$(curl -s -X POST "$AUTH_SERVICE/auth/register" \
    -H "Content-Type: application/json" \
    -d '{
        "email": "tech@tracseq.com",
        "password": "Tech123!",
        "first_name": "Lab",
        "last_name": "Technician"
    }' \
    -w "\n%{http_code}" | tail -1)

if [ "$TECH_RESPONSE" == "200" ] || [ "$TECH_RESPONSE" == "201" ] || [ "$TECH_RESPONSE" == "409" ]; then
    echo -e "${GREEN}✓ Created/Exists${NC}"
else
    echo -e "${YELLOW}⚠ Failed${NC} (HTTP $TECH_RESPONSE)"
fi

echo ""
echo "=== Setting Up Workflow Templates ==="
echo ""

# Create DNA extraction workflow
echo -n "Creating DNA extraction workflow... "
DNA_WORKFLOW=$(curl -s -X POST "$TEMPLATE_SERVICE/api/templates" \
    -H "Content-Type: application/json" \
    -d '{
        "name": "Standard DNA Extraction",
        "description": "Standard workflow for DNA extraction from biological samples",
        "category": "extraction",
        "version": "1.0",
        "steps": [
            {"order": 1, "name": "Sample Preparation", "duration_minutes": 30},
            {"order": 2, "name": "Cell Lysis", "duration_minutes": 45},
            {"order": 3, "name": "DNA Binding", "duration_minutes": 20},
            {"order": 4, "name": "Washing", "duration_minutes": 15},
            {"order": 5, "name": "Elution", "duration_minutes": 10},
            {"order": 6, "name": "Quality Check", "duration_minutes": 30}
        ],
        "is_active": true
    }' \
    -w "\n%{http_code}" | tail -1)

if [ "$DNA_WORKFLOW" == "200" ] || [ "$DNA_WORKFLOW" == "201" ] || [ "$DNA_WORKFLOW" == "409" ]; then
    echo -e "${GREEN}✓ Created${NC}"
else
    echo -e "${YELLOW}⚠ Failed${NC} (HTTP $DNA_WORKFLOW)"
fi

# Create RNA extraction workflow
echo -n "Creating RNA extraction workflow... "
RNA_WORKFLOW=$(curl -s -X POST "$TEMPLATE_SERVICE/api/templates" \
    -H "Content-Type: application/json" \
    -d '{
        "name": "Standard RNA Extraction",
        "description": "Standard workflow for RNA extraction with RNase-free conditions",
        "category": "extraction",
        "version": "1.0",
        "steps": [
            {"order": 1, "name": "Sample Preparation (RNase-free)", "duration_minutes": 30},
            {"order": 2, "name": "TRIzol Addition", "duration_minutes": 15},
            {"order": 3, "name": "Phase Separation", "duration_minutes": 20},
            {"order": 4, "name": "RNA Precipitation", "duration_minutes": 60},
            {"order": 5, "name": "RNA Washing", "duration_minutes": 20},
            {"order": 6, "name": "RNA Resuspension", "duration_minutes": 15},
            {"order": 7, "name": "Quality Assessment", "duration_minutes": 30}
        ],
        "is_active": true
    }' \
    -w "\n%{http_code}" | tail -1)

if [ "$RNA_WORKFLOW" == "200" ] || [ "$RNA_WORKFLOW" == "201" ] || [ "$RNA_WORKFLOW" == "409" ]; then
    echo -e "${GREEN}✓ Created${NC}"
else
    echo -e "${YELLOW}⚠ Failed${NC} (HTTP $RNA_WORKFLOW)"
fi

echo ""
echo "=== Setting Up Sequencing Workflows ==="
echo ""

# Create Illumina sequencing workflow
echo -n "Creating Illumina sequencing workflow... "
ILLUMINA_WORKFLOW=$(curl -s -X POST "$SEQUENCING_SERVICE/workflows" \
    -H "Content-Type: application/json" \
    -d '{
        "id": "illumina_standard",
        "name": "Illumina Standard Sequencing",
        "description": "Standard Illumina sequencing workflow for NovaSeq/MiSeq",
        "version": "1.0",
        "platform_ids": ["illumina_novaseq", "illumina_miseq"],
        "workflow_type": "whole_genome",
        "steps": {
            "library_prep": {"duration_hours": 4},
            "quality_control": {"duration_hours": 1},
            "cluster_generation": {"duration_hours": 2},
            "sequencing": {"duration_hours": 24},
            "base_calling": {"duration_hours": 2},
            "demultiplexing": {"duration_hours": 1}
        },
        "default_parameters": {
            "read_length": 150,
            "paired_end": true,
            "coverage": 30
        },
        "estimated_duration_hours": 34,
        "is_active": true
    }' \
    -w "\n%{http_code}" | tail -1)

if [ "$ILLUMINA_WORKFLOW" == "200" ] || [ "$ILLUMINA_WORKFLOW" == "201" ] || [ "$ILLUMINA_WORKFLOW" == "409" ]; then
    echo -e "${GREEN}✓ Created${NC}"
else
    echo -e "${YELLOW}⚠ Failed${NC} (HTTP $ILLUMINA_WORKFLOW)"
fi

echo ""
echo "=== Setting Up Notification Channels ==="
echo ""

# Configure email channel
echo -n "Configuring email notification channel... "
EMAIL_CHANNEL=$(curl -s -X PUT "$NOTIFICATION_SERVICE/channels/email/config" \
    -H "Content-Type: application/json" \
    -d '{
        "enabled": true,
        "from_address": "noreply@tracseq.com",
        "from_name": "TracSeq Laboratory System"
    }' \
    -w "\n%{http_code}" | tail -1)

if [ "$EMAIL_CHANNEL" == "201" ] || [ "$EMAIL_CHANNEL" == "200" ]; then
    echo -e "${GREEN}✓ Configured${NC}"
else
    echo -e "${YELLOW}⚠ Failed${NC} (HTTP $EMAIL_CHANNEL)"
fi

# Configure Slack channel
echo -n "Configuring Slack notification channel... "
SLACK_CHANNEL=$(curl -s -X PUT "$NOTIFICATION_SERVICE/channels/slack/config" \
    -H "Content-Type: application/json" \
    -d '{
        "enabled": false,
        "webhook_url": "",
        "default_channel": "#lab-alerts"
    }' \
    -w "\n%{http_code}" | tail -1)

if [ "$SLACK_CHANNEL" == "201" ] || [ "$SLACK_CHANNEL" == "200" ]; then
    echo -e "${GREEN}✓ Configured${NC}"
else
    echo -e "${YELLOW}⚠ Failed${NC} (HTTP $SLACK_CHANNEL)"
fi

echo ""
echo "=== Creating Sample Storage Locations ==="
echo ""

# Note: Storage location creation would typically be done through the storage service
# For now, we'll just output instructions
echo -e "${BLUE}ℹ Storage locations should be configured through the storage service UI${NC}"
echo "  Recommended initial locations:"
echo "  - Freezer A1 (-80°C) - For long-term DNA/RNA storage"
echo "  - Freezer B1 (-20°C) - For protein samples"
echo "  - Refrigerator C1 (4°C) - For short-term storage"
echo "  - Room Temperature D1 (23°C) - For dry samples"

echo ""
echo "=== Setup Summary ==="
echo ""

echo -e "${GREEN}Initial data setup complete!${NC}"
echo ""
echo "Created users:"
echo "  - admin@tracseq.com (password: Admin123!)"
echo "  - lab.manager@tracseq.com (password: LabManager123!)"
echo "  - tech@tracseq.com (password: Tech123!)"
echo ""
echo "Created workflows:"
echo "  - Standard DNA Extraction"
echo "  - Standard RNA Extraction"
echo "  - Illumina Standard Sequencing"
echo ""
echo "Configured notification channels:"
echo "  - Email (enabled)"
echo "  - Slack (disabled - needs webhook URL)"
echo ""
echo -e "${YELLOW}⚠ Important: Change all default passwords before production use!${NC}"
echo ""

# Save setup information
SETUP_INFO_FILE="./initial-setup-info.txt"
echo "TracSeq 2.0 Initial Setup Information" > "$SETUP_INFO_FILE"
echo "=====================================" >> "$SETUP_INFO_FILE"
echo "Setup Date: $(date)" >> "$SETUP_INFO_FILE"
echo "" >> "$SETUP_INFO_FILE"
echo "Default Users:" >> "$SETUP_INFO_FILE"
echo "  admin@tracseq.com / Admin123!" >> "$SETUP_INFO_FILE"
echo "  lab.manager@tracseq.com / LabManager123!" >> "$SETUP_INFO_FILE"
echo "  tech@tracseq.com / Tech123!" >> "$SETUP_INFO_FILE"
echo "" >> "$SETUP_INFO_FILE"
echo "Services:" >> "$SETUP_INFO_FILE"
echo "  API Gateway: http://localhost:18089" >> "$SETUP_INFO_FILE"
echo "  Auth Service: http://localhost:8080" >> "$SETUP_INFO_FILE"
echo "  Sample Service: http://localhost:8081" >> "$SETUP_INFO_FILE"
echo "  Sequencing Service: http://localhost:8084" >> "$SETUP_INFO_FILE"
echo "  Notification Service: http://localhost:8085" >> "$SETUP_INFO_FILE"
echo "" >> "$SETUP_INFO_FILE"

echo -e "${GREEN}Setup information saved to: $SETUP_INFO_FILE${NC}" 