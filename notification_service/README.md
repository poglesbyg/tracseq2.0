# TracSeq Notification Service

A comprehensive notification service for the TracSeq laboratory management system, providing multi-channel communication capabilities including email, SMS, Slack, Teams, and webhooks.

## Features

### Core Functionality
- **Multi-Channel Support**: Email, SMS, Slack, Teams, Discord, webhooks, push notifications, and in-app notifications
- **Template Management**: Create, manage, and version notification templates
- **Subscription Management**: User preference-based notification subscriptions
- **Scheduled Notifications**: Support for delayed and recurring notifications
- **Bulk Operations**: Send notifications to multiple recipients efficiently
- **Real-time Integration**: Handle events from other TracSeq services

### Advanced Features
- **Rate Limiting**: Configurable rate limits per channel and user
- **Delivery Tracking**: Track delivery status and retry failed notifications
- **Channel Health Monitoring**: Monitor and test communication channels
- **Statistics & Analytics**: Comprehensive metrics and reporting
- **Template Validation**: Validate templates before deployment
- **Priority-based Processing**: Handle critical notifications first

## API Endpoints

### Health & Monitoring
- `GET /health` - Service health check
- `GET /health/ready` - Readiness probe
- `GET /health/metrics` - Service metrics

### Notifications
- `POST /notifications` - Send single notification
- `POST /notifications/bulk` - Send bulk notifications
- `GET /notifications` - List notifications with filtering
- `GET /notifications/{id}` - Get specific notification
- `GET /notifications/{id}/status` - Get delivery status
- `POST /notifications/{id}/retry` - Retry failed notification

### Channels
- `GET /channels` - List available channels
- `POST /channels/{type}/test` - Test specific channel
- `GET /channels/{type}/config` - Get channel configuration
- `PUT /channels/{type}/config` - Update channel configuration

### Templates
- `POST /templates` - Create template
- `GET /templates` - List templates
- `GET /templates/{id}` - Get template
- `PUT /templates/{id}` - Update template
- `DELETE /templates/{id}` - Delete template
- `POST /templates/{id}/preview` - Preview template with data
- `POST /templates/{id}/validate` - Validate template

### Subscriptions
- `POST /subscriptions` - Create subscription
- `GET /subscriptions` - List subscriptions
- `GET /subscriptions/{id}` - Get subscription
- `PUT /subscriptions/{id}` - Update subscription
- `DELETE /subscriptions/{id}` - Delete subscription
- `GET /subscriptions/user/{id}` - Get user subscriptions
- `GET /subscriptions/event/{type}` - Get event subscriptions

### Integration Events
- `POST /integration/lab-events` - Handle lab events
- `POST /integration/sample-events` - Handle sample events
- `POST /integration/sequencing-events` - Handle sequencing events
- `POST /integration/template-events` - Handle template events
- `POST /integration/system-alerts` - Handle system alerts

### Administration
- `GET /admin/statistics` - Get notification statistics
- `GET /admin/failed-notifications` - Get failed notifications
- `POST /admin/retry-failed` - Retry all failed notifications
- `POST /admin/cleanup` - Cleanup old notifications
- `GET /admin/channels/health` - Check channel health
- `GET /admin/rate-limits` - Get rate limits
- `PUT /admin/rate-limits` - Update rate limits

## Configuration

### Environment Variables

```bash
# Server Configuration
HOST=0.0.0.0
PORT=8085

# Database
DATABASE_URL=postgresql://notification_user:notification_password@postgres:5432/notification_db

# Authentication Service
AUTH_SERVICE_URL=http://auth-service:8080

# Email Configuration
EMAIL_ENABLED=true
EMAIL_SMTP_HOST=smtp.gmail.com
EMAIL_SMTP_PORT=587
EMAIL_USERNAME=your-email@gmail.com
EMAIL_PASSWORD=your-app-password

# SMS Configuration (Twilio)
SMS_ENABLED=true
SMS_ACCOUNT_SID=your-twilio-account-sid
SMS_AUTH_TOKEN=your-twilio-auth-token
SMS_FROM_NUMBER=+1234567890

# Slack Configuration
SLACK_ENABLED=true
SLACK_BOT_TOKEN=xoxb-your-bot-token
SLACK_WEBHOOK_URLS=https://hooks.slack.com/services/...

# Teams Configuration
TEAMS_ENABLED=true
TEAMS_WEBHOOK_URL=https://outlook.office.com/webhook/...

# Rate Limiting
RATE_LIMIT_REQUESTS_PER_MINUTE=100
RATE_LIMIT_REQUESTS_PER_HOUR=1000
RATE_LIMIT_REQUESTS_PER_DAY=10000
```

## Data Models

### Notification
```rust
pub struct Notification {
    pub id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType, // Alert, Info, Warning, Error, Success, etc.
    pub priority: Priority, // Low, Medium, High, Critical, Urgent
    pub status: NotificationStatus, // Pending, Sent, Failed, etc.
    pub channels: Vec<Channel>,
    pub recipients: Vec<String>,
    pub template_id: Option<Uuid>,
    pub template_data: Option<serde_json::Value>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivery_attempts: i32,
    pub metadata: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Template
```rust
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_type: TemplateType,
    pub subject: Option<String>,
    pub body_html: Option<String>,
    pub body_text: String,
    pub variables: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Subscription
```rust
pub struct NotificationSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_type: String,
    pub channels: Vec<Channel>,
    pub enabled: bool,
    pub filters: serde_json::Value,
    pub preferences: NotificationPreferences,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## Integration with TracSeq Services

The notification service integrates with other TracSeq microservices:

- **Auth Service**: User authentication and authorization
- **Sample Service**: Sample lifecycle notifications
- **Sequencing Service**: Job status updates
- **Template Service**: Template change notifications
- **Storage Service**: Storage alerts and warnings
- **Event Service**: Event-driven notification triggers

## Usage Examples

### Send Simple Notification
```bash
curl -X POST http://localhost:8085/notifications \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "title": "Sample Processing Complete",
    "message": "Your sample XYZ123 has finished processing",
    "notification_type": "Success",
    "priority": "Medium",
    "channels": ["email", "slack"],
    "recipients": ["user@example.com", "#lab-notifications"]
  }'
```

### Create Template
```bash
curl -X POST http://localhost:8085/templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "name": "Sample Completion",
    "template_type": "Email",
    "subject": "Sample {{sample_id}} Processing Complete",
    "body_text": "Your sample {{sample_id}} has completed {{status}}",
    "variables": ["sample_id", "status"]
  }'
```

### Subscribe to Events
```bash
curl -X POST http://localhost:8085/subscriptions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "user_id": "user-uuid",
    "event_type": "sample.completed",
    "channels": ["email"],
    "enabled": true,
    "preferences": {
      "priority_threshold": "Medium",
      "quiet_hours": {
        "start_time": "22:00",
        "end_time": "08:00",
        "timezone": "UTC"
      }
    }
  }'
```

## Development

### Running Locally
```bash
# Start database
docker run -d \
  --name notification-postgres \
  -e POSTGRES_DB=notification_db \
  -e POSTGRES_USER=notification_user \
  -e POSTGRES_PASSWORD=notification_password \
  -p 5432:5432 \
  postgres:15

# Run service
cargo run
```

### Testing
```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration

# Test with coverage
cargo tarpaulin --out html
```

## Security

- JWT-based authentication
- Role-based access control
- Input validation and sanitization
- SQL injection prevention
- Rate limiting protection
- Secure credential management

## Monitoring

The service provides comprehensive monitoring:

- Health checks and readiness probes
- Prometheus-compatible metrics
- Structured logging with tracing
- Channel health monitoring
- Delivery success/failure tracking
- Performance analytics

## Error Handling

Robust error handling with:
- Automatic retry mechanisms
- Circuit breaker patterns
- Graceful degradation
- Detailed error reporting
- Dead letter queues for failed notifications

*Context improved by Giga AI* 
