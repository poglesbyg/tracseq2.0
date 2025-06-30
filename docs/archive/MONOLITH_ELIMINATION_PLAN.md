# TracSeq 2.0: Complete Monolith Elimination Plan

## 🎯 **Current State: 95% Ready for Migration**

You've already built the infrastructure! The API Gateway has feature flags that can route traffic from the monolith to microservices. Currently all flags are disabled, but the services exist.

### **What's Still in Monolith** (`lab_manager:3000`):
- Authentication & User Management (`/api/auth/*`, `/api/users/*`)
- Sample Management (`/api/samples/*`) - Complex business logic
- Template Management (`/api/templates/*`)
- Sequencing Management (`/api/sequencing/*`)  
- Spreadsheet Processing (`/api/spreadsheets/*`)
- Reports & Analytics (`/api/reports/*`)
- Dashboard & Health (`/api/dashboard/*`)

### **Microservices Built & Ready** (flags disabled):
✅ `auth_service:8080` - Authentication & user management  
✅ `sample_service:8081` - Sample CRUD & business logic  
✅ `enhanced_storage_service:8082` - Storage location management  
✅ `template_service:8083` - Template management  
✅ `sequencing_service:8084` - Sequencing job management  
✅ `notification_service:8085` - Multi-channel notifications  
✅ `enhanced_rag_service:8086` - Document processing & AI  

---

## 🚀 **Phase 1: Enable Existing Services (Week 1)**

### **Step 1A: Enable Notification Service** (Zero Risk)
```bash
# Update API Gateway environment
export USE_NOTIFICATION_SERVICE=true

# Test notification endpoints
curl http://localhost:8089/api/notifications/health
```

**Impact**: Routes notification calls away from monolith  
**Risk**: Minimal - independent service  
**Rollback**: Set flag to `false`

### **Step 1B: Enable Storage Service** (Low Risk)
```bash
export USE_STORAGE_SERVICE=true

# Test storage endpoints
curl http://localhost:8089/api/storage/locations
curl http://localhost:8089/api/storage/capacity
```

**Impact**: Storage location management → `enhanced_storage_service`  
**Risk**: Low - mostly CRUD operations  
**Rollback**: Set flag to `false`

### **Step 1C: Enable RAG Service** (Low Risk)
```bash
export USE_RAG_SERVICE=true

# Test RAG endpoints
curl http://localhost:8089/api/rag/health
curl http://localhost:8089/api/rag/submissions
```

**Impact**: Document processing → `enhanced_rag_service`  
**Risk**: Low - isolated AI functionality  
**Rollback**: Set flag to `false`

---

## 🔧 **Phase 2: Enable Business Logic Services (Week 2-3)**

### **Step 2A: Enable Template Service** (Medium Risk)
```bash
export USE_TEMPLATE_SERVICE=true

# Test template endpoints
curl http://localhost:8089/api/templates
curl -X POST http://localhost:8089/api/templates/upload
```

**Impact**: Template CRUD → `template_service`  
**Risk**: Medium - core functionality  
**Validation Required**:
- Template creation/editing works
- File uploads function properly
- Template validation rules maintained

### **Step 2B: Enable Auth Service** (High Risk - Critical)
```bash
export USE_AUTH_SERVICE=true

# Test authentication flow
curl -X POST http://localhost:8089/api/auth/login -d '{"email":"test@lab.com","password":"test"}'
curl http://localhost:8089/api/auth/me -H "Authorization: Bearer $TOKEN"
```

**Impact**: Authentication → `auth_service`  
**Risk**: High - affects all users  
**Preparation Required**:
- Migrate user database to auth service
- Test JWT token compatibility
- Verify session management
- Test all authentication flows
- Prepare rollback plan

---

## 💾 **Phase 3: Complex Services Migration (Week 3-4)**

### **Step 3A: Enable Sample Service** (High Risk - Core Business)
```bash
export USE_SAMPLE_SERVICE=true

# Test sample management
curl http://localhost:8089/api/samples
curl -X POST http://localhost:8089/api/samples/batch
```

**Impact**: Sample management → `sample_service`  
**Risk**: High - core laboratory functionality  
**Preparation Required**:
- Migrate sample data to dedicated database
- Test complex business logic
- Verify barcode generation
- Test validation rules
- Verify RAG integration still works

### **Step 3B: Enable Sequencing Service** (High Risk)
```bash
export USE_SEQUENCING_SERVICE=true

# Test sequencing functionality
curl http://localhost:8089/api/sequencing/jobs
curl -X POST http://localhost:8089/api/sequencing/jobs
```

**Impact**: Sequencing workflows → `sequencing_service`  
**Risk**: High - critical laboratory workflow  
**Preparation Required**:
- Migrate sequencing job data
- Test workflow state management
- Verify integration with sample service

---

## 🗂️ **Phase 4: Create Missing Services (Week 4-5)**

### **Services Still Needed**:

#### **4A: Dashboard Service**
```rust
// Create new service: dashboard_service
cargo new dashboard_service
```
**Responsibilities**:
- System metrics aggregation
- Health status consolidation  
- Performance analytics
- Laboratory KPIs

#### **4B: Reports Service**
```rust
// Create new service: reports_service  
cargo new reports_service
```
**Responsibilities**:
- Report template management
- Query execution
- Data export functionality
- Custom report generation

#### **4C: Spreadsheet Service**
```rust
// Create new service: spreadsheet_service
cargo new spreadsheet_service
```
**Responsibilities**:
- File upload processing
- Data parsing and validation
- Search functionality
- Export capabilities

---

## 🎯 **Phase 5: Complete Migration (Week 5-6)**

### **Step 5A: Enable All Services**
Update API Gateway configuration:
```bash
export USE_AUTH_SERVICE=true
export USE_SAMPLE_SERVICE=true  
export USE_TEMPLATE_SERVICE=true
export USE_STORAGE_SERVICE=true
export USE_SEQUENCING_SERVICE=true
export USE_NOTIFICATION_SERVICE=true
export USE_RAG_SERVICE=true
export USE_DASHBOARD_SERVICE=true
export USE_REPORTS_SERVICE=true
export USE_SPREADSHEET_SERVICE=true
```

### **Step 5B: Monitor and Validate**
1. **Health Checks**: All services responding
2. **Data Integrity**: No data loss during migration  
3. **Performance**: Response times within acceptable limits
4. **User Experience**: All functionality working
5. **Error Handling**: Proper fallback mechanisms

### **Step 5C: Remove Monolith**
Once all traffic is routed to microservices:
```bash
# Stop the monolith
docker stop lab-manager

# Remove from docker-compose files
# Update API Gateway to remove monolith routing
```

---

## 🗄️ **Database Migration Strategy**

### **Current State**: Single PostgreSQL database used by monolith

### **Target State**: Service-specific databases

#### **Migration Approach**:
1. **Dual-Write Pattern**: Write to both old and new databases
2. **Background Sync**: Migrate existing data gradually  
3. **Read-Verify**: Compare data consistency
4. **Switch-Over**: Point services to new databases
5. **Cleanup**: Remove old database dependencies

#### **Service Database Assignments**:
```yaml
auth_service: tracseq_auth_db
sample_service: tracseq_samples_db  
template_service: tracseq_templates_db
sequencing_service: tracseq_sequencing_db
storage_service: tracseq_storage_db
notification_service: tracseq_notifications_db
rag_service: tracseq_rag_db
dashboard_service: tracseq_metrics_db (read replicas)
reports_service: tracseq_reports_db  
spreadsheet_service: tracseq_spreadsheets_db
```

---

## 🔄 **Implementation Commands**

### **Quick Start: Enable Safe Services Now**
```bash
cd api_gateway

# Create .env file for feature flags
cat > .env << EOF
USE_NOTIFICATION_SERVICE=true
USE_STORAGE_SERVICE=true
USE_RAG_SERVICE=true
USE_TEMPLATE_SERVICE=false
USE_AUTH_SERVICE=false
USE_SAMPLE_SERVICE=false
USE_SEQUENCING_SERVICE=false
EOF

# Restart API Gateway with new flags
docker-compose -f docker-compose.minimal.yml restart api-gateway

# Test the enabled services
curl http://localhost:8089/routing-status
```

### **Start All Services for Testing**
```bash
# Start all microservices
docker-compose -f docker-compose.yml up -d

# Monitor health
curl http://localhost:8089/health
curl http://localhost:8089/routing-status
```

### **Gradual Flag Enablement Script**
```bash
#!/bin/bash
# scripts/enable-service.sh

SERVICE=$1
if [ -z "$SERVICE" ]; then
    echo "Usage: $0 <service_name>"
    echo "Available: auth, sample, template, storage, sequencing, notification, rag"
    exit 1
fi

# Enable the service flag
export USE_${SERVICE^^}_SERVICE=true

# Restart gateway
docker-compose restart api-gateway

# Test the service
curl http://localhost:8089/api/${SERVICE}/health

echo "✅ ${SERVICE} service enabled"
```

---

## 📊 **Success Metrics**

### **Migration Completion Indicators**:
- ✅ All feature flags enabled (`USE_*_SERVICE=true`)
- ✅ Zero traffic to monolith (port 3000)  
- ✅ All health checks passing
- ✅ Database migration complete
- ✅ Monolith container stopped/removed
- ✅ User functionality unchanged

### **Performance Targets**:
- Response times ≤ current monolith performance
- 99.9% uptime during migration
- Zero data loss
- Seamless user experience

---

## 🚨 **Risk Mitigation**

### **Rollback Strategy**:
1. **Feature Flag Rollback**: Set `USE_*_SERVICE=false`
2. **Database Rollback**: Point services back to monolith DB
3. **Container Rollback**: Stop microservices, restart monolith
4. **Health Monitoring**: Continuous health checks during migration

### **Testing Strategy**:
1. **Automated Tests**: Run full test suite after each flag enablement
2. **Load Testing**: Verify performance under load
3. **User Acceptance**: Validate all user workflows  
4. **Canary Deployment**: Enable flags for subset of users first

---

## 🎉 **Expected Timeline: 6 Weeks to Zero Monolith**

- **Week 1**: Enable notification, storage, RAG services (3 flags ✅)
- **Week 2**: Enable template service (1 flag ✅)  
- **Week 3**: Enable auth service (1 flag ✅)
- **Week 4**: Enable sample & sequencing services (2 flags ✅)
- **Week 5**: Create & deploy missing services (dashboard, reports, spreadsheet)
- **Week 6**: Final validation & monolith removal

**Result**: 100% microservices architecture, zero monolith dependencies

The infrastructure is already there - you just need to flip the switches! 🚀 