# TracSeq 2.0 Microservices Migration Plan

## Overview
This document outlines the strategy for completing the migration from the monolithic lab_manager to a fully microservices-based architecture.

## Current State Analysis

### Services with Duplicates (Exist in both lab_manager and as microservices)
1. **auth_service** - Authentication and authorization
2. **sample_service** - Sample management
3. **sequencing_service** - Sequencing workflow management
4. **template_service** - Template management
5. **spreadsheet_service** - Spreadsheet processing (as spreadsheet_versioning_service)
6. **storage_service** - Storage management (as enhanced_storage_service)

### Services Only in lab_manager
1. **barcode_service** - Barcode generation and management
2. **rag_integration_service** - RAG integration (partially migrated to enhanced_rag_service)
3. **storage_management_service** - Advanced storage features

### Standalone Microservices (Already extracted)
1. **qaqc_service** - Quality control
2. **library_details_service** - Library management
3. **notification_service** - Notifications
4. **event_service** - Event handling
5. **transaction_service** - Transaction management
6. **config_service** - Configuration management

## Migration Strategy

### Phase 1: API Gateway Implementation (Week 1)
- [ ] Implement API Gateway using the existing api_gateway service
- [ ] Configure routing rules for all microservices
- [ ] Set up service discovery mechanism
- [ ] Implement circuit breakers and retry logic

### Phase 2: Service Proxy Implementation (Week 2)
- [ ] Update lab_manager to proxy auth requests to auth_service
- [ ] Update lab_manager to proxy sample requests to sample_service
- [ ] Update lab_manager to proxy sequencing requests to sequencing_service
- [ ] Update lab_manager to proxy template requests to template_service
- [ ] Implement health check aggregation

### Phase 3: Frontend Migration (Week 3)
- [ ] Update frontend API configuration to use API Gateway
- [ ] Implement service-specific API clients
- [ ] Add fallback mechanisms for gradual rollout
- [ ] Update environment configurations

### Phase 4: Service Extraction (Week 4-5)
- [ ] Extract barcode_service as standalone microservice
- [ ] Complete rag_integration_service migration to enhanced_rag_service
- [ ] Merge storage_management_service into enhanced_storage_service
- [ ] Remove duplicate service implementations from lab_manager

### Phase 5: Cleanup and Optimization (Week 6)
- [ ] Remove all service implementations from lab_manager
- [ ] Convert lab_manager to a lightweight orchestration service
- [ ] Update deployment configurations
- [ ] Optimize inter-service communication

## Service Communication Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│   Frontend  │────▶│  API Gateway │────▶│  Microservices  │
└─────────────┘     └──────────────┘     └─────────────────┘
                            │                      │
                            ▼                      ▼
                    ┌──────────────┐      ┌──────────────┐
                    │ Auth Service │      │Sample Service│
                    └──────────────┘      └──────────────┘
                            │                      │
                            ▼                      ▼
                    ┌──────────────┐      ┌──────────────┐
                    │  Event Bus   │      │   Database   │
                    └──────────────┘      └──────────────┘
```

## Implementation Guidelines

### 1. Service Proxy Pattern
```rust
// Example proxy implementation in lab_manager
pub async fn proxy_to_service(
    service_url: &str,
    path: &str,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, Error> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", service_url, path);
    
    let mut request = client.request(method, &url);
    
    // Forward headers
    for (key, value) in headers.iter() {
        request = request.header(key, value);
    }
    
    // Forward body
    request = request.body(body);
    
    // Execute request with circuit breaker
    circuit_breaker.call(async move {
        request.send().await
    }).await
}
```

### 2. Service Discovery
```yaml
# services.yaml
services:
  auth_service:
    url: http://auth-service:3010
    health_check: /health
    timeout: 30s
    
  sample_service:
    url: http://sample-service:3011
    health_check: /health
    timeout: 30s
    
  sequencing_service:
    url: http://sequencing-service:3012
    health_check: /health
    timeout: 30s
```

### 3. Database Migration
- Each service maintains its own database schema
- Use event sourcing for data synchronization
- Implement saga pattern for distributed transactions

## Risk Mitigation

### 1. Zero-Downtime Migration
- Use feature flags for gradual rollout
- Implement parallel running of old and new systems
- Monitor performance metrics during transition

### 2. Data Consistency
- Implement distributed transaction patterns
- Use event sourcing for audit trails
- Ensure backward compatibility

### 3. Rollback Strategy
- Maintain version compatibility
- Implement database migration rollback scripts
- Keep monolith functional during transition

## Success Metrics

1. **Service Availability**: 99.9% uptime for all services
2. **Response Time**: <100ms p95 latency
3. **Error Rate**: <0.1% error rate
4. **Test Coverage**: >80% for all services
5. **Documentation**: Complete API documentation for all services

## Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| 1 | API Gateway | Gateway implementation, routing configuration |
| 2 | Service Proxy | Proxy implementations, health checks |
| 3 | Frontend Migration | Updated API clients, configuration |
| 4-5 | Service Extraction | Standalone barcode service, completed migrations |
| 6 | Cleanup | Removed duplicates, optimized architecture |

## Next Steps

1. Review and approve migration plan
2. Set up development environment for parallel testing
3. Begin Phase 1 implementation
4. Schedule daily standup meetings for migration team
5. Create monitoring dashboards for migration metrics