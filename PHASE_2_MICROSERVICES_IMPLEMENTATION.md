# Phase 2: Complete Microservices Implementation

## 🎯 Overview

Phase 2 transforms TracSeq 2.0 from a monolithic architecture to a fully distributed microservices architecture over 6 weeks, using progressive feature flag enablement to minimize risk.

## 📊 Current State

- ✅ **Phase 1 Complete**: Frontend liberated, infrastructure ready
- ✅ **API Gateway**: Configured with feature flags
- ✅ **Microservices**: Built and containerized
- ✅ **Migration Scripts**: Progressive enablement tools ready
- ⏳ **Monolith**: Still handling all business logic

## 🚀 Implementation Timeline

### Week 1: Enable Safe Services ✅ Ready to Deploy
**Risk Level: Low** | **Impact: Minimal**

#### Services to Enable:
1. **Notification Service** → `/api/notifications/*`
2. **Storage Service** → `/api/storage/*`
3. **RAG Service** → `/api/rag/*`

#### Implementation:
```bash
# Execute Week 1 migration
./scripts/phase2-migration.sh

# This will:
# 1. Create API Gateway .env with safe services enabled
# 2. Start notification, storage, and RAG microservices
# 3. Configure API Gateway routing
# 4. Test service health
# 5. Create migration log
```

#### Verification:
```bash
# Check service health
curl http://localhost:8089/health
curl http://localhost:8089/routing-status

# Test individual services
curl http://localhost:8089/api/notifications
curl http://localhost:8089/api/storage/locations
curl http://localhost:8089/api/rag/health
```

---

### Week 2: Enable Template Service 📋
**Risk Level: Medium** | **Impact: Template Management**

#### Pre-requisites:
- Week 1 services stable for 24-48 hours
- No degradation in performance

#### Implementation:
```bash
# Execute Week 2 migration
./scripts/phase2-week2-template.sh

# This will:
# 1. Start template microservice
# 2. Update API Gateway to route template traffic
# 3. Create rollback script
# 4. Test template endpoints
```

#### Rollback Plan:
```bash
# If issues occur
./scripts/phase2-week2-rollback.sh
```

---

### Week 3: Enable Auth Service 🔐
**Risk Level: HIGH** | **Impact: All User Authentication**

#### Critical Pre-requisites:
- [ ] Database backup completed
- [ ] User data migrated to auth DB
- [ ] JWT compatibility verified
- [ ] Support team on standby
- [ ] Users notified

#### Database Migration:
```sql
-- Run before enabling auth service
psql -h localhost -U postgres < scripts/auth-data-migration.sql
```

#### Implementation:
```bash
# Execute Week 3 migration (HIGH RISK)
./scripts/phase2-week3-auth.sh

# Monitor closely
./scripts/monitor-auth-migration.sh
```

#### Emergency Rollback:
```bash
# IMMEDIATE ROLLBACK if issues
./scripts/phase2-week3-rollback.sh
```

---

### Week 4: Enable Core Business Services 🧬
**Risk Level: HIGH** | **Impact: Core Laboratory Functions**

#### Services to Enable:
1. **Sample Service** → `/api/samples/*`
2. **Sequencing Service** → `/api/sequencing/*`

#### Implementation:
```bash
# Execute Week 4 migration
./scripts/phase2-week4-core.sh

# This enables the heart of laboratory operations
```

---

### Week 5: Create Missing Services 🏗️
**Risk Level: Low** | **New Development**

#### Services to Create:
1. **Dashboard Service** - System metrics & KPIs
2. **Reports Service** - Report generation
3. **Spreadsheet Service** - File processing

#### Implementation:
```bash
# Create new services
./scripts/phase2-week5-create-services.sh

# Deploy and test each service
```

---

### Week 6: Complete Migration 🎉
**Risk Level: Medium** | **Final Cutover**

#### Final Steps:
1. Enable all remaining feature flags
2. Monitor for 72 hours
3. Stop monolith container
4. Remove monolith routing

#### Implementation:
```bash
# Final migration
./scripts/phase2-week6-complete.sh

# Verify no traffic to monolith
curl http://localhost:3000/health # Should fail
```

## 📋 Feature Flag Configuration

### API Gateway Environment Variables
```env
# Week 1 (Safe Services)
USE_NOTIFICATION_SERVICE=true
USE_STORAGE_SERVICE=true
USE_RAG_SERVICE=true

# Week 2 (Template)
USE_TEMPLATE_SERVICE=true

# Week 3 (Auth - HIGH RISK)
USE_AUTH_SERVICE=true

# Week 4 (Core Services)
USE_SAMPLE_SERVICE=true
USE_SEQUENCING_SERVICE=true

# Week 5 (New Services)
USE_DASHBOARD_SERVICE=true
USE_REPORTS_SERVICE=true
USE_SPREADSHEET_SERVICE=true
```

## 🔍 Monitoring & Validation

### Health Check Endpoints
```bash
# API Gateway
curl http://localhost:8089/health
curl http://localhost:8089/routing-status

# Individual Services
curl http://localhost:3010/health  # Auth
curl http://localhost:3011/health  # Sample
curl http://localhost:3012/health  # Sequencing
curl http://localhost:3013/health  # Template
curl http://localhost:3014/health  # Storage
curl http://localhost:3016/health  # Notification
curl http://localhost:3019/health  # RAG
```

### Monitoring Commands
```bash
# View all service logs
docker-compose -f docker-compose.microservices.yml logs -f

# View specific service
docker logs -f auth-service

# Check resource usage
docker stats

# Database connections
psql -h localhost -U postgres -c "SELECT datname, numbackends FROM pg_stat_database;"
```

## 🚨 Risk Mitigation

### Rollback Procedures
Each week has a dedicated rollback script:
- Week 1: Minimal risk, disable flags
- Week 2: `scripts/phase2-week2-rollback.sh`
- Week 3: `scripts/phase2-week3-rollback.sh` (CRITICAL)
- Week 4: `scripts/phase2-week4-rollback.sh`

### Database Backups
```bash
# Before each major migration
pg_dump -h localhost -U postgres tracseq_main > backup_$(date +%Y%m%d).sql

# Service-specific backups
pg_dump -h localhost -U postgres tracseq_auth > auth_backup.sql
pg_dump -h localhost -U postgres tracseq_samples > samples_backup.sql
```

### Testing Strategy
1. **Unit Tests**: Run for each service before enabling
2. **Integration Tests**: Verify service communication
3. **End-to-End Tests**: Full user workflows
4. **Load Tests**: Ensure performance parity

## 📊 Success Metrics

### Week-by-Week Targets
- **Week 1**: 3 services migrated, 0% user impact
- **Week 2**: Template functionality migrated
- **Week 3**: Authentication migrated, <5 min downtime
- **Week 4**: Core business logic migrated
- **Week 5**: New services deployed
- **Week 6**: 100% microservices, monolith removed

### Performance Targets
- Response time: ≤ current monolith
- Error rate: < 0.1%
- Availability: 99.9%

## 🛠️ Troubleshooting

### Common Issues

#### Service Won't Start
```bash
# Check logs
docker logs <service-name>

# Verify database
psql -h localhost -U postgres -l

# Check port conflicts
netstat -tulpn | grep <port>
```

#### Routing Not Working
```bash
# Check API Gateway config
cat api_gateway/.env

# Verify service registration
curl http://localhost:8089/routing-status

# Test direct service access
curl http://localhost:<service-port>/health
```

#### Database Connection Issues
```bash
# Test connection
psql -h localhost -U postgres -d <database>

# Check migrations
docker exec -it postgres psql -U postgres -c "\dt"
```

## 📚 Additional Resources

### Scripts Created
- `scripts/phase2-migration.sh` - Week 1 implementation
- `scripts/phase2-week2-template.sh` - Template service
- `scripts/phase2-week3-auth.sh` - Auth service (HIGH RISK)
- `scripts/phase2-week4-core.sh` - Core services
- `scripts/phase2-week5-create-services.sh` - New services
- `scripts/phase2-week6-complete.sh` - Final migration

### Documentation
- `phase2_migration_log.md` - Real-time migration log
- `api_gateway/.env` - Current feature flags
- Service-specific READMEs in each directory

## 🎯 Current Status: Ready for Week 1

**Next Action**: Execute `./scripts/phase2-migration.sh` to begin Phase 2 implementation.

The infrastructure is ready, scripts are prepared, and the progressive migration can begin immediately with minimal risk.

---

*Phase 2 Implementation Guide - TracSeq 2.0 Microservices Migration*