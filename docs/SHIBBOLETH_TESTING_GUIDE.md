# Shibboleth Authentication Testing Guide

This document describes the comprehensive test suite implemented for Shibboleth authentication features in the lab management system.

## Test Coverage Overview

### 1. Core Functionality Tests (`shibboleth_auth_tests.rs`)

**Header Extraction Tests:**
- ✅ `test_shibboleth_attribute_extraction()` - Tests extraction of Shibboleth attributes from HTTP headers
- ✅ `test_missing_attributes_handling()` - Tests handling of missing or incomplete attribute sets
- ✅ `test_empty_attributes()` - Tests behavior with completely empty attribute sets

**Role Mapping Tests:**
- ✅ `test_direct_role_mapping()` - Tests direct lab role attribute mapping (labRole → UserRole)
- ✅ `test_entitlement_role_mapping()` - Tests role mapping from SAML entitlements
- ✅ `test_group_membership_role_mapping()` - Tests role mapping from LDAP group memberships
- ✅ `test_role_mapping_precedence()` - Tests precedence: labRole > entitlement > group > Guest
- ✅ `test_case_insensitive_role_mapping()` - Tests case-insensitive role matching

**Configuration Tests:**
- ✅ `test_shibboleth_config_defaults()` - Tests default configuration values
- ✅ `test_multiple_entitlements()` - Tests handling of multiple SAML entitlements

**User Creation Tests:**
- ✅ `test_user_creation_attributes()` - Tests user creation from Shibboleth attributes

### 2. Integration Tests (`shibboleth_integration_tests.rs`)

**Configuration Validation:**
- ✅ `test_shibboleth_config_validation()` - Tests configuration structure and validation

**Authentication Decision Logic:**
- ✅ `test_authentication_scenarios()` - Tests when Shibboleth authentication should be used
- ✅ `test_hybrid_authentication_logic()` - Tests hybrid Shibboleth + JWT authentication
- ✅ `test_comprehensive_role_mapping()` - Tests all role mapping scenarios systematically

**Edge Cases and Error Handling:**
- ✅ `test_error_handling()` - Tests handling of malformed or invalid data
- ✅ `test_header_extraction_edge_cases()` - Tests header parsing edge cases
- ✅ `test_user_creation_requirements()` - Tests required attributes for user creation

## Test Scenarios Covered

### Role Mapping Scenarios

| Input Type | Test Case | Expected Role |
|------------|-----------|---------------|
| `labRole="lab_administrator"` | Direct role attribute | `LabAdministrator` |
| `labRole="pi"` | PI shorthand | `PrincipalInvestigator` |
| `entitlement="lab:admin"` | SAML entitlement | `LabAdministrator` |
| `isMemberOf="cn=lab-technicians"` | LDAP group | `LabTechnician` |
| Multiple conflicting roles | Precedence test | Follows precedence order |
| No role information | Default case | `Guest` |

### Authentication Scenarios

| Headers Present | Expected Behavior |
|-----------------|-------------------|
| `HTTP_EPPN` + `HTTP_MAIL` | Shibboleth authentication |
| `Authorization: Bearer <token>` only | JWT authentication |
| Both Shibboleth + JWT | Shibboleth preferred |
| Neither | Authentication required error |

### Attribute Extraction Scenarios

| Test Case | Headers | Expected Result |
|-----------|---------|-----------------|
| Complete profile | All standard Shibboleth headers | Full attribute extraction |
| Minimal profile | Only EPPN + MAIL | Basic authentication possible |
| Empty headers | No Shibboleth headers | Empty attribute map |
| Special characters | Unicode/special chars in names | Proper handling |

## Running the Tests

### Prerequisites

1. **Rust Environment**: Ensure Rust and Cargo are installed and in PATH
2. **Database**: PostgreSQL running for integration tests (optional for unit tests)
3. **Environment Variables**: Set up test environment variables

### Test Execution Commands

```bash
# Run all Shibboleth-related tests
cargo test shibboleth

# Run specific test modules
cargo test shibboleth_auth_tests
cargo test shibboleth_integration_tests

# Run with output
cargo test shibboleth -- --nocapture

# Run specific test
cargo test test_role_mapping_precedence -- --nocapture
```

### Test Environment Setup

```bash
# Set test environment variables
export TEST_DATABASE_URL="postgres://lab_manager:lab_manager@localhost:5432/lab_manager_test"
export JWT_SECRET="test-jwt-secret-key"
export RUST_LOG="debug"

# Optional: Enable Shibboleth for integration tests
export SHIBBOLETH_ENABLED="true"
export SHIBBOLETH_HYBRID_MODE="true"
```

## Test Data Examples

### Valid Shibboleth Headers

```http
HTTP_EPPN: user@example.edu
HTTP_MAIL: user@example.edu
HTTP_DISPLAYNAME: John Doe
HTTP_GIVENNAME: John
HTTP_SN: Doe
HTTP_LAB_ROLE: lab_administrator
HTTP_DEPARTMENT: Biology
HTTP_AFFILIATION: faculty@example.edu
HTTP_ENTITLEMENT: lab:admin;urn:mace:dir:entitlement:common-lib-terms
HTTP_ISMEMBEROF: cn=lab-administrators,ou=groups,dc=example,dc=edu
```

### Role Mapping Examples

```rust
// Direct role attribute (highest precedence)
labRole="lab_administrator" → UserRole::LabAdministrator

// SAML entitlements (medium precedence)
entitlement="lab:pi;other:permission" → UserRole::PrincipalInvestigator

// LDAP group membership (lowest precedence)
isMemberOf="cn=lab-technicians,ou=groups,dc=example,dc=edu" → UserRole::LabTechnician
```

## Test Coverage Metrics

- **Functions Tested**: 2/2 public functions (100%)
- **Role Mappings**: 5/5 role types covered (100%)
- **Authentication Paths**: 3/3 paths covered (100%)
- **Edge Cases**: 8 major edge cases covered
- **Error Conditions**: 4 error scenarios tested

## Continuous Integration

### GitHub Actions Integration

```yaml
# Add to .github/workflows/test.yml
- name: Run Shibboleth Tests
  run: |
    cargo test shibboleth_auth_tests -- --nocapture
    cargo test shibboleth_integration_tests -- --nocapture
```

### Test Reporting

Tests generate comprehensive output including:
- Assertion failures with context
- Role mapping decision traces
- Attribute extraction debugging
- Configuration validation results

## Troubleshooting Test Issues

### Common Issues

1. **Missing Dependencies**: Ensure all Cargo dependencies are installed
   ```bash
   cargo build
   ```

2. **Database Connection**: For integration tests requiring database
   ```bash
   # Check PostgreSQL status
   pg_isready
   
   # Create test database
   createdb lab_manager_test
   ```

3. **Permission Errors**: Ensure proper file permissions
   ```bash
   chmod +x scripts/run_tests.sh
   ```

### Test Debugging

Enable debug logging for detailed test output:
```bash
RUST_LOG=debug cargo test shibboleth -- --nocapture
```

## Test Maintenance

### Adding New Tests

1. **Unit Tests**: Add to `src/tests/shibboleth_auth_tests.rs`
2. **Integration Tests**: Add to `src/tests/shibboleth_integration_tests.rs`
3. **Update Documentation**: Update this guide with new test scenarios

### Test Data Management

- Keep test data realistic but anonymized
- Use consistent test users across test suites
- Document any special test cases or edge conditions

## Security Considerations in Tests

- Tests use mock headers (never real production data)
- JWT secrets are test-only values
- Database isolation prevents data leakage
- No sensitive institutional data in test cases

---

*This testing guide ensures comprehensive coverage of Shibboleth authentication features while maintaining security and reliability standards.* 
