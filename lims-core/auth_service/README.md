# Authentication Service

A standalone microservice for authentication and authorization in the Laboratory Management System.

## Overview

This service extracts authentication concerns from the main lab manager application into a dedicated, scalable microservice. It provides JWT-based authentication, user management, session handling, and comprehensive security features.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                 Authentication Service                      │
├─────────────────────────────────────────────────────────────┤
│  HTTP API Layer                                            │
│  ├── Health Endpoints      (/health, /health/ready)        │
│  ├── Auth Endpoints        (/auth/login, /auth/logout)     │
│  ├── Validation Endpoints  (/validate/token)              │
│  └── Admin Endpoints       (/admin/users, /admin/audit)    │
├─────────────────────────────────────────────────────────────┤
│  Middleware Layer                                          │
│  ├── Authentication Middleware                             │
│  ├── Authorization Middleware                              │
│  └── Service Authentication                                │
├─────────────────────────────────────────────────────────────┤
│  Business Logic Layer                                      │
│  ├── AuthService (JWT, passwords, sessions)               │
│  ├── User Management                                       │
│  ├── Security & Audit Logging                             │
│  └── Rate Limiting                                         │
├─────────────────────────────────────────────────────────────┤
│  Data Layer                                                │
│  ├── PostgreSQL Database                                   │
│  ├── Redis (optional, for rate limiting)                  │
│  └── Database Migrations                                   │
└─────────────────────────────────────────────────────────────┘
```

## Features

### 🔐 Authentication
- JWT-based authentication with configurable expiration
- Secure password hashing using Argon2
- Session management with device tracking
- Remember-me functionality with refresh tokens
- Failed login attempt tracking and account lockout

### 👥 User Management
- User registration and profile management
- Role-based access control (Guest → Lab Administrator)
- Email verification (configurable)
- Password reset functionality
- Account status management (Active, Inactive, Suspended)

### 🛡️ Security Features
- Comprehensive audit logging
- Rate limiting protection
- Secure token storage and validation
- Password strength validation
- Session revocation and cleanup

### 📊 Admin Features
- User administration endpoints
- Session monitoring and management
- Security audit log access
- System metrics and health monitoring

### 🔗 Service Integration
- Token validation API for other services
- Permission checking endpoints
- Service-to-service authentication
- Middleware for easy integration

## Database Schema

The service uses PostgreSQL with the following main tables:

- `users` - User accounts and profiles
- `user_sessions` - Active sessions and tokens
- `security_audit_log` - Security events and audit trail
- `password_reset_tokens` - Password reset tokens
- `email_verification_tokens` - Email verification tokens
- `rate_limits` - Rate limiting data (if not using Redis)

## Configuration

The service is configured via environment variables:

```bash
# Server Configuration
AUTH_HOST=0.0.0.0
AUTH_PORT=8080
AUTH_WORKERS=4

# Database
AUTH_DATABASE_URL=postgresql://auth_user:auth_password@localhost:5432/auth_db

# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_ACCESS_TOKEN_EXPIRY_HOURS=1
JWT_REFRESH_TOKEN_EXPIRY_DAYS=30
JWT_ISSUER=auth-service
JWT_AUDIENCE=lab-management-system

# Security Settings
PASSWORD_MIN_LENGTH=8
PASSWORD_REQUIRE_UPPERCASE=true
PASSWORD_REQUIRE_LOWERCASE=true
PASSWORD_REQUIRE_NUMBERS=true
PASSWORD_REQUIRE_SYMBOLS=false
MAX_LOGIN_ATTEMPTS=5
LOCKOUT_DURATION_MINUTES=15
SESSION_TIMEOUT_HOURS=8

# Features
REGISTRATION_ENABLED=true
EMAIL_VERIFICATION_REQUIRED=false
PASSWORD_RESET_ENABLED=true
AUDIT_LOGGING_ENABLED=true

# Email (optional)
EMAIL_ENABLED=false
SMTP_HOST=localhost
SMTP_PORT=587
SMTP_USERNAME=
SMTP_PASSWORD=
EMAIL_FROM_ADDRESS=noreply@lab-management.com

# Redis (optional, for rate limiting)
REDIS_URL=redis://localhost:6379
```

## API Endpoints

### Health & Monitoring
- `GET /health` - Basic health check
- `GET /health/ready` - Readiness probe
- `GET /health/metrics` - Service metrics

### Authentication
- `POST /auth/login` - User login
- `POST /auth/logout` - User logout
- `POST /auth/register` - User registration (if enabled)
- `POST /auth/refresh` - Refresh access token
- `POST /auth/forgot-password` - Request password reset
- `POST /auth/reset-password` - Reset password with token
- `POST /auth/verify-email` - Verify email address

### User Management
- `GET /auth/me` - Get current user profile
- `PUT /auth/me` - Update current user profile
- `POST /auth/change-password` - Change user password
- `GET /auth/sessions` - Get user sessions
- `DELETE /auth/sessions/:id` - Revoke specific session

### Token Validation (for other services)
- `POST /validate/token` - Validate JWT token
- `POST /validate/permissions` - Check user permissions
- `POST /validate/extract-user` - Extract user from token

### Administration (Admin only)
- `GET /admin/users` - List all users
- `GET /admin/users/:id` - Get specific user
- `DELETE /admin/users/:id` - Delete user
- `POST /admin/users/:id/disable` - Disable user account
- `POST /admin/users/:id/enable` - Enable user account
- `GET /admin/sessions` - List all sessions
- `GET /admin/audit-log` - Get security audit log

## Setup & Installation

### Prerequisites
- Rust 1.70+
- PostgreSQL 12+
- Redis (optional)

### Development Setup

1. **Clone and build:**
```bash
cd auth_service
cargo build
```

2. **Set up database:**
```bash
# Create database
createdb auth_db

# Set database URL
export AUTH_DATABASE_URL=postgresql://username:password@localhost:5432/auth_db
```

3. **Configure environment:**
```bash
cp .env.example .env
# Edit .env with your configuration
```

4. **Run migrations and start service:**
```bash
cargo run
```

### Docker Setup

```dockerfile
FROM rust:1.70-alpine as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/auth_service .
CMD ["./auth_service"]
```

## Integration with Lab Manager

### Step 1: Update Lab Manager to use Auth Service

Replace direct authentication calls with HTTP calls to the auth service:

```rust
// Before: Direct authentication
let user = components.auth_service.login(request).await?;

// After: Auth service API call
let response = reqwest::Client::new()
    .post("http://auth-service:8080/auth/login")
    .json(&request)
    .send()
    .await?;
let auth_response: LoginResponse = response.json().await?;
```

### Step 2: Replace Auth Middleware

```rust
// New auth middleware for lab manager
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let token = extract_token_from_headers(request.headers())?;
    
    // Validate with auth service
    let validation_response = auth_client
        .validate_token(token)
        .await?;
    
    if validation_response.valid {
        // Extract user info and inject into request
        let user = auth_client.get_user(validation_response.user_id).await?;
        request.extensions_mut().insert(user);
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
```

### Step 3: Create Auth Client Library

```rust
// lib: auth_client
pub struct AuthClient {
    base_url: String,
    client: reqwest::Client,
}

impl AuthClient {
    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse> {
        let response = self.client
            .post(&format!("{}/auth/login", self.base_url))
            .json(&request)
            .send()
            .await?;
        
        Ok(response.json().await?)
    }
    
    pub async fn validate_token(&self, token: &str) -> Result<ValidateTokenResponse> {
        let response = self.client
            .post(&format!("{}/validate/token", self.base_url))
            .json(&json!({ "token": token }))
            .send()
            .await?;
        
        Ok(response.json().await?)
    }
}
```

## Security Considerations

1. **JWT Secrets**: Use strong, unique secrets in production
2. **HTTPS**: Always use HTTPS in production
3. **Rate Limiting**: Configure appropriate rate limits
4. **Session Management**: Regular cleanup of expired sessions
5. **Audit Logging**: Monitor and analyze security events
6. **Password Policies**: Enforce strong password requirements
7. **Database Security**: Use connection pooling and prepared statements

## Monitoring & Observability

The service provides comprehensive monitoring:

- Health check endpoints for container orchestration
- Prometheus-compatible metrics
- Structured logging with configurable levels
- Security audit trails
- Performance metrics (response times, connection pools)

## Next Steps

### Phase 2: Shibboleth Integration
- Add Shibboleth authentication support
- Implement attribute mapping
- Support hybrid authentication modes

### Phase 3: Advanced Features
- Multi-factor authentication (MFA)
- OAuth2/OIDC provider support
- API key management
- Advanced rate limiting
- Geo-location tracking

### Phase 4: Scalability
- Distributed session storage
- Service mesh integration
- Load balancing strategies
- Horizontal scaling support

## Testing

```bash
# Run tests
cargo test

# Run with coverage
cargo tarpaulin --out html

# Integration tests
cargo test --test integration_tests
```

## Contributing

1. Follow the established code structure
2. Add tests for new features
3. Update documentation
4. Follow security best practices
5. Ensure backward compatibility

## License

This project is part of the Laboratory Management System and follows the same licensing terms.

---

*Context improved by Giga AI* 
