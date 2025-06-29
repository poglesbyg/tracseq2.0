# TracSeq 2.0 OpenShift Deployment Guide

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Pre-deployment Checklist](#pre-deployment-checklist)
3. [Deployment Steps](#deployment-steps)
4. [Post-deployment Configuration](#post-deployment-configuration)
5. [Verification](#verification)
6. [Troubleshooting](#troubleshooting)
7. [Maintenance](#maintenance)

## Prerequisites

### OpenShift Cluster Requirements
- OpenShift 4.12 or higher
- Minimum cluster resources:
  - 16 vCPUs
  - 64GB RAM
  - 500GB storage
- Required operators:
  - OpenShift Pipelines (Tekton)
  - OpenShift Service Mesh (optional)
  - OpenShift Monitoring

### Client Tools
```bash
# Install oc CLI
wget https://mirror.openshift.com/pub/openshift-v4/clients/ocp/latest/openshift-client-linux.tar.gz
tar xvf openshift-client-linux.tar.gz
sudo mv oc /usr/local/bin/

# Verify installation
oc version
```

### Access Requirements
- Cluster admin access (for initial setup)
- Git repository access
- Container registry credentials

## Pre-deployment Checklist

- [ ] OpenShift cluster access configured
- [ ] Required operators installed
- [ ] Git repository cloned
- [ ] Environment-specific configurations prepared
- [ ] SSL certificates ready (if using custom domains)
- [ ] External service credentials ready (SMTP, SMS, etc.)
- [ ] Database backup strategy defined
- [ ] Monitoring endpoints configured

## Deployment Steps

### 1. Login to OpenShift
```bash
oc login <your-openshift-api-url> -u <username>
```

### 2. Create Project and Configure RBAC
```bash
cd openshift/scripts
./create-project.sh prod
```

### 3. Configure Secrets
Edit the secret files before deployment:

```bash
# Database credentials
vi ../base/secrets/database-credentials.yaml

# JWT and API keys
vi ../base/secrets/jwt-secret.yaml

# Generate secure passwords
openssl rand -base64 32  # For database passwords
openssl rand -base64 64  # For JWT secret
```

### 4. Configure Git Repository
Update BuildConfigs with your repository:
```bash
find ../templates -name "*.yaml" -exec sed -i 's|https://github.com/your-org/tracseq-2.0.git|YOUR_REPO_URL|g' {} \;
```

### 5. Deploy the Platform
```bash
./deploy.sh prod
```

This script will:
- Deploy infrastructure components (PostgreSQL, Redis, ChromaDB)
- Build all service images
- Deploy microservices
- Configure routes and networking
- Run database migrations

### 6. Configure Domain Names
Update routes with your domain:
```bash
oc get routes -n tracseq-prod
oc patch route api-gateway -p '{"spec":{"host":"api.your-domain.com"}}' -n tracseq-prod
```

## Post-deployment Configuration

### 1. Configure External Services

#### Email Service (SMTP)
```bash
oc set env deployment/notification-service \
  SMTP_HOST=smtp.your-provider.com \
  SMTP_PORT=587 \
  SMTP_USERNAME=your-username \
  -n tracseq-prod
```

#### SMS Service (Twilio)
```bash
oc create secret generic sms-credentials \
  --from-literal=TWILIO_ACCOUNT_SID=your-sid \
  --from-literal=TWILIO_AUTH_TOKEN=your-token \
  -n tracseq-prod
```

### 2. Configure Monitoring

#### Prometheus ServiceMonitors
```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: tracseq-services
  namespace: tracseq-prod
spec:
  selector:
    matchLabels:
      app.kubernetes.io/part-of: tracseq
  endpoints:
  - port: metrics
    interval: 30s
```

#### Grafana Dashboards
Import dashboards from `monitoring/grafana/dashboards/`:
- `tracseq-overview.json`
- `microservices.json`
- `database-performance.json`

### 3. Configure Backup

Create a backup CronJob:
```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
  namespace: tracseq-prod
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: postgres-backup
            image: registry.redhat.io/rhel9/postgresql-15
            command:
            - /bin/bash
            - -c
            - |
              DATE=$(date +%Y%m%d_%H%M%S)
              pg_dump -h postgres-service -U $POSTGRES_USER -d tracseq > /backup/tracseq_${DATE}.sql
              # Upload to S3 or other backup storage
            env:
            - name: POSTGRES_USER
              valueFrom:
                secretKeyRef:
                  name: tracseq-database-credentials
                  key: POSTGRES_USER
            volumeMounts:
            - name: backup
              mountPath: /backup
          volumes:
          - name: backup
            persistentVolumeClaim:
              claimName: backup-pvc
          restartPolicy: OnFailure
```

## Verification

### 1. Check Pod Status
```bash
oc get pods -n tracseq-prod
# All pods should be Running or Completed
```

### 2. Check Service Health
```bash
# Get API Gateway URL
API_URL=$(oc get route api-gateway -o jsonpath='{.spec.host}' -n tracseq-prod)

# Test health endpoints
curl https://${API_URL}/health
curl https://${API_URL}/api/v1/auth/health
curl https://${API_URL}/api/v1/samples/health
```

### 3. Run Integration Tests
```bash
cd ../../tests
./run-integration-tests.sh https://${API_URL}
```

### 4. Check Logs
```bash
# View logs for a specific service
oc logs -f deployment/auth-service -n tracseq-prod

# View all logs
oc logs -l app.kubernetes.io/part-of=tracseq -n tracseq-prod
```

## Troubleshooting

### Common Issues

#### 1. Pods Not Starting
```bash
# Check pod events
oc describe pod <pod-name> -n tracseq-prod

# Check resource quotas
oc describe resourcequota -n tracseq-prod
```

#### 2. Database Connection Issues
```bash
# Test database connectivity
oc run -it --rm debug --image=registry.redhat.io/rhel9/postgresql-15 --restart=Never -- psql -h postgres-service -U tracseq_admin -d tracseq

# Check secrets
oc get secret tracseq-database-credentials -o yaml -n tracseq-prod
```

#### 3. Build Failures
```bash
# Check build logs
oc logs -f bc/auth-service -n tracseq-prod

# Manually trigger build
oc start-build auth-service --follow -n tracseq-prod
```

#### 4. Route Not Working
```bash
# Check route status
oc describe route api-gateway -n tracseq-prod

# Test internal service
oc run -it --rm curl --image=curlimages/curl --restart=Never -- curl http://api-gateway:8089/health
```

### Debug Commands
```bash
# Execute commands in a running pod
oc exec -it deployment/auth-service -n tracseq-prod -- /bin/bash

# Port forward for local debugging
oc port-forward service/api-gateway 8089:8089 -n tracseq-prod

# View resource usage
oc adm top pods -n tracseq-prod
```

## Maintenance

### 1. Scaling Services
```bash
# Manual scaling
oc scale deployment/auth-service --replicas=5 -n tracseq-prod

# Autoscaling
oc autoscale deployment/api-gateway --min=3 --max=10 --cpu-percent=80 -n tracseq-prod
```

### 2. Updating Services
```bash
# Update image
oc set image deployment/auth-service auth-service=auth-service:v2.0.1 -n tracseq-prod

# Rolling restart
oc rollout restart deployment/auth-service -n tracseq-prod

# Check rollout status
oc rollout status deployment/auth-service -n tracseq-prod
```

### 3. Database Maintenance
```bash
# Run VACUUM
oc exec -it deployment/postgres -- psql -U tracseq_admin -d tracseq -c "VACUUM ANALYZE;"

# Check database size
oc exec -it deployment/postgres -- psql -U tracseq_admin -d tracseq -c "SELECT pg_database_size('tracseq');"
```

### 4. Certificate Renewal
```bash
# Check certificate expiration
oc get route api-gateway -o jsonpath='{.spec.tls.certificate}' | openssl x509 -noout -dates

# Update certificate
oc create secret tls tracseq-tls --cert=cert.pem --key=key.pem --dry-run=client -o yaml | oc apply -f -
```

## Security Considerations

1. **Network Policies**: Ensure network policies are enforced
2. **Secret Rotation**: Implement regular secret rotation
3. **RBAC**: Follow principle of least privilege
4. **Image Scanning**: Enable image vulnerability scanning
5. **Audit Logging**: Configure audit logging for compliance

## Backup and Recovery

### Backup Strategy
- Database: Daily automated backups with 30-day retention
- Persistent Volumes: Weekly snapshots
- Configuration: Version controlled in Git

### Recovery Procedures
1. Database recovery from backup
2. Service restoration from images
3. Configuration restoration from Git

## Support

For issues or questions:
1. Check logs and events
2. Review troubleshooting section
3. Contact platform team
4. Submit GitHub issue

---

*Last updated: [Current Date]*
*Version: 1.0*