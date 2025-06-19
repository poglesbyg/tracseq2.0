# TracSeq API Gateway

Intelligent routing and management for TracSeq microservices ecosystem.

## Features

- **Monolith Router**: Initially routes all requests to existing monolith
- **Feature Flags**: Gradual service extraction with zero downtime
- **Load Balancing**: Intelligent request distribution
- **Rate Limiting**: Configurable rate limiting per service
- **Monitoring**: Comprehensive logging and metrics
- **Health Checks**: Service health monitoring

## Quick Start

```bash
# Start with monolith routing
docker-compose -f docker-compose.minimal.yml up -d

# Check status
curl http://localhost:8000/health
curl http://localhost:8000/routing-status
```

## Migration Strategy

1. Start with all traffic to monolith
2. Deploy individual microservices
3. Enable feature flags gradually
4. Monitor and validate each service
5. Complete migration 
