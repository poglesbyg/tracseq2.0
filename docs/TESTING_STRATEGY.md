# TracSeq 2.0 Testing Strategy

## Overview

This document outlines the comprehensive testing strategy implemented for TracSeq 2.0, including test infrastructure, patterns, and best practices.

## Test Infrastructure

### 1. Test Helpers Library (`test-helpers`)

A dedicated test utilities crate providing:

- **Database Utilities**: Test database creation, migration management, transaction isolation
- **HTTP Testing**: Enhanced test server with fluent API for testing Axum applications
- **Fixtures**: Laboratory-specific test data generators
- **Mocks**: Service mocks for inter-service communication testing

### 2. Environment Setup

#### Scripts

- `scripts/setup-test-environment.sh`: Initializes test database and environment
- `scripts/run-all-tests.sh`: Comprehensive test runner with reporting

#### Configuration

All services support test-specific configuration through:
- `TEST_DATABASE_URL` environment variable
- `Config::test_config()` methods
- SQLx offline mode for CI/CD environments

## Testing Patterns

### Unit Tests

Located alongside source code in each service:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function() {
        // Test implementation
    }
}
```

### Integration Tests

Located in `tests/` directory for each service:

```rust
use test_helpers::{TestContext, TestServer};

#[tokio::test]
async fn test_integration_flow() {
    let ctx = TestContext::with_database().await.unwrap();
    // Test implementation
}
```

### End-to-End Tests

Cross-service tests in workspace root `tests/` directory:

```rust
use test_helpers::mocks::MockServices;

#[tokio::test]
async fn test_full_workflow() {
    let mocks = MockServices::new();
    // Test complete workflows
}
```

## Service-Specific Testing

### Auth Service
- User registration and authentication flows
- Role-based access control
- Security features (lockout, password policies)
- Session management
- SSO integration (when enabled)

### Sample Service
- CRUD operations
- Barcode generation and validation
- Sample lifecycle management
- Batch operations
- QC integration

### Sequencing Service
- Run creation and management
- State transitions
- Quality metrics tracking
- Integration with sample service

### Storage Service
- Location management
- IoT sensor integration
- Temperature monitoring
- Capacity planning
- AI predictions

### Transaction Service
- Saga pattern implementation
- Distributed transaction coordination
- Compensation logic
- Workflow orchestration

## Database Testing

### Test Isolation

Three strategies available:

1. **Shared Test Database**: Fast, suitable for read-only tests
2. **Transaction Rollback**: Each test runs in a transaction that's rolled back
3. **Isolated Database**: Each test gets its own database (slowest but most isolated)

### Migration Testing

All services include migration tests:

```rust
#[tokio::test]
async fn test_migrations() {
    let pool = DatabaseTestBuilder::new()
        .isolated()
        .with_migrations("./migrations")
        .build()
        .await
        .unwrap();
}
```

## Performance Testing

### Benchmarks

Using Criterion for micro-benchmarks:

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("function_name", |b| {
        b.iter(|| {
            // Code to benchmark
        })
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

### Load Testing

- Concurrent request handling
- Database connection pooling
- Service resilience under load

## Test Data Management

### Fixtures

Pre-defined test data using the fixtures module:

```rust
use test_helpers::fixtures::{UserFixture, SampleFixture};

let user = UserFixture::admin();
let sample = SampleFixture::dna();
```

### Test Data Builder

For complex scenarios:

```rust
let data = TestDataBuilder::new()
    .with_users(5)
    .with_samples(10)
    .with_qc_results_for_samples()
    .build();
```

## Continuous Integration

### GitHub Actions Workflow

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: tracseq
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
      - run: ./scripts/setup-test-environment.sh
      - run: ./scripts/run-all-tests.sh
```

## Test Coverage

### Running Coverage

```bash
cargo tarpaulin --workspace --out Html --output-dir target/coverage
```

### Coverage Goals

- Unit test coverage: >80%
- Integration test coverage: >70%
- Critical path coverage: 100%

## Best Practices

### 1. Test Naming

Use descriptive names that explain what is being tested:

```rust
#[test]
fn test_user_registration_with_invalid_email_returns_validation_error() {
    // Test implementation
}
```

### 2. Test Organization

- Group related tests in modules
- Use shared setup functions
- Clean up test data after each test

### 3. Assertion Messages

Always include descriptive messages:

```rust
assert_eq!(
    result, 
    expected, 
    "User role should be 'admin' after promotion"
);
```

### 4. Mock Usage

Use mocks for external dependencies:

```rust
let (mock_auth, state) = create_mock_auth_service();
state.add_response("/validate/token", "POST", StatusCode::OK, json!({
    "valid": true,
    "user_id": "test-user"
})).await;
```

### 5. Error Testing

Test both success and failure paths:

```rust
// Success case
let result = function_under_test(valid_input).await;
assert!(result.is_ok());

// Error case
let result = function_under_test(invalid_input).await;
assert!(matches!(result, Err(SpecificError::InvalidInput(_))));
```

## Running Tests

### All Tests
```bash
cargo test --workspace
```

### Specific Service
```bash
cargo test -p auth_service
```

### With Features
```bash
cargo test --workspace --all-features
```

### Single Test
```bash
cargo test test_user_registration
```

### With Output
```bash
cargo test -- --nocapture
```

## Troubleshooting

### Common Issues

1. **Database Connection Errors**
   - Ensure PostgreSQL is running
   - Check DATABASE_URL is set correctly
   - Verify database exists and migrations are run

2. **Compilation Errors with SQLx**
   - Run `cargo sqlx prepare` in each service directory
   - Ensure DATABASE_URL is set during compilation

3. **Flaky Tests**
   - Use test isolation strategies
   - Add retries for network-dependent tests
   - Ensure proper cleanup between tests

4. **Slow Tests**
   - Use shared test database where possible
   - Parallelize independent tests
   - Consider test categorization (unit/integration/e2e)

## Future Improvements

1. **Property-Based Testing**: Add proptest for complex business logic
2. **Mutation Testing**: Integrate cargo-mutants
3. **Visual Regression Testing**: For frontend components
4. **Contract Testing**: For service boundaries
5. **Chaos Testing**: For distributed system resilience

## Conclusion

This testing strategy ensures high-quality, reliable code across the TracSeq 2.0 platform. By following these patterns and practices, developers can confidently make changes while maintaining system stability.

*Context improved by Giga AI*