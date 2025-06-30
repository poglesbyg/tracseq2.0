# Phase 2 Quick Reference Guide

## 🚀 Week 1: Safe Services (NOW)

```bash
# Start migration
./scripts/phase2-migration.sh

# Check status
curl http://localhost:8089/routing-status

# Monitor
docker-compose -f docker-compose.microservices.yml logs -f
```

**Services**: Notification (3016), Storage (3014), RAG (3019)

---

## 📋 Week 2: Template Service

```bash
# Enable template service
./scripts/phase2-week2-template.sh

# Rollback if needed
./scripts/phase2-week2-rollback.sh
```

**Service**: Template (3013)

---

## 🔐 Week 3: Auth Service (HIGH RISK!)

```bash
# CRITICAL: Complete checklist first!
./scripts/phase2-week3-auth.sh

# Monitor constantly
./scripts/monitor-auth-migration.sh

# EMERGENCY ROLLBACK
./scripts/phase2-week3-rollback.sh
```

**Service**: Auth (3010) - **AFFECTS ALL USERS**

---

## 🧬 Week 4: Core Services

```bash
# Enable core business logic
./scripts/phase2-week4-core.sh
```

**Services**: Sample (3011), Sequencing (3012)

---

## 🏗️ Week 5: New Services

Create Dashboard, Reports, and Spreadsheet services

---

## ✅ Week 6: Complete Migration

Remove monolith, celebrate! 🎉

---

## 🚨 Emergency Contacts

- **Rollback Any Service**: Restore `.env.backup` files
- **Stop Everything**: `docker-compose down`
- **Check Logs**: `docker logs <service-name>`
- **API Gateway Config**: `api_gateway/.env`

## 📊 Health Checks

```bash
# Quick health check all services
for port in 8089 3010 3011 3012 3013 3014 3016 3019; do
  echo -n "Port $port: "
  curl -s http://localhost:$port/health > /dev/null && echo "✅" || echo "❌"
done
```