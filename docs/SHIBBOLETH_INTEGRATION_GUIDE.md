# Shibboleth Integration Guide for Lab Manager

This guide provides comprehensive instructions for integrating Shibboleth authentication with the Lab Manager system.

## Overview

The Lab Manager system now supports hybrid authentication, allowing users to authenticate via either:
- **Shibboleth SSO** (primary for institutional users)
- **JWT tokens** (fallback for API access and external integrations)

This implementation provides seamless integration with institutional identity providers while maintaining backward compatibility with existing authentication mechanisms.

## Architecture

### Authentication Flow

```
1. User accesses protected resource
2. Hybrid authentication middleware checks for:
   a. Shibboleth attributes in HTTP headers (priority)
   b. JWT Bearer token in Authorization header (fallback)
3. User information is extracted and validated
4. User session is created/updated
5. Request proceeds with authenticated user context
```

### Components

- **Shibboleth Authentication Middleware** (`src/middleware/shibboleth_auth.rs`)
- **Hybrid Authentication Middleware** (supports both Shibboleth and JWT)
- **Configuration System** (environment-based Shibboleth settings)
- **User Auto-Creation** (creates users from Shibboleth attributes)
- **Role Mapping** (maps institutional roles to lab system roles)

## Configuration

### Environment Variables

Add the following to your `.env` file:

```bash
# Enable Shibboleth Authentication
SHIBBOLETH_ENABLED=true

# Hybrid Mode - Allow both Shibboleth and JWT authentication
SHIBBOLETH_HYBRID_MODE=true

# User Management Settings
SHIBBOLETH_AUTO_CREATE_USERS=true
SHIBBOLETH_AUTO_UPDATE_ATTRIBUTES=true

# Default role for new users created via Shibboleth
SHIBBOLETH_DEFAULT_ROLE=Guest
```

### Role Mapping

The system automatically maps Shibboleth attributes to lab roles using multiple strategies:

#### 1. Direct Role Attribute (Preferred)
```
labRole="lab_administrator" → LabAdministrator
labRole="principal_investigator" → PrincipalInvestigator
labRole="lab_technician" → LabTechnician
labRole="research_scientist" → ResearchScientist
labRole="data_analyst" → DataAnalyst
```

#### 2. Entitlement-Based Mapping
```
entitlement contains "lab:admin" → LabAdministrator
entitlement contains "lab:pi" → PrincipalInvestigator
entitlement contains "lab:technician" → LabTechnician
entitlement contains "lab:scientist" → ResearchScientist
entitlement contains "lab:analyst" → DataAnalyst
```

#### 3. Group Membership Mapping
```
isMemberOf contains "cn=lab-administrators" → LabAdministrator
isMemberOf contains "cn=principal-investigators" → PrincipalInvestigator
isMemberOf contains "cn=lab-technicians" → LabTechnician
isMemberOf contains "cn=research-scientists" → ResearchScientist
isMemberOf contains "cn=data-analysts" → DataAnalyst
```

## Web Server Configuration

### Apache with mod_shib2 (Recommended)

1. **Install Shibboleth SP**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install libapache2-mod-shib2
   
   # CentOS/RHEL
   sudo yum install shibboleth
   ```

2. **Use the provided Apache configuration**:
   ```bash
   cp lab_manager/deploy/apache-shibboleth.conf /etc/apache2/sites-available/lab-manager.conf
   sudo a2ensite lab-manager
   sudo systemctl reload apache2
   ```

3. **Configure Shibboleth SP** (`/etc/shibboleth/shibboleth2.xml`):
   ```xml
   <ApplicationDefaults entityID="https://lab-manager.example.edu">
       <Sessions lifetime="28800" timeout="3600" relayState="ss:mem">
           <SSO entityID="https://idp.example.edu">
               SAML2 SAML1
           </SSO>
           <Logout>SAML2 Local</Logout>
       </Sessions>
   </ApplicationDefaults>
   ```

### Nginx with Auth Request

1. **Use the provided Nginx configuration**:
   ```bash
   cp lab_manager/deploy/nginx-shibboleth.conf /etc/nginx/sites-available/lab-manager
   sudo ln -s /etc/nginx/sites-available/lab-manager /etc/nginx/sites-enabled/
   sudo systemctl reload nginx
   ```

2. **Note**: Nginx requires additional setup for Shibboleth integration, typically involving FastCGI or external authentication services.

## Attribute Configuration

### Required Attributes

These attributes must be released by your IdP:

- **eppn** (eduPersonPrincipalName): Unique user identifier
- **mail**: User's email address

### Recommended Attributes

- **displayName**: Full name for display
- **givenName**: First name
- **sn**: Last name (surname)
- **affiliation**: Institution affiliation
- **entitlement**: Role/permission entitlements
- **isMemberOf**: Group memberships

### Custom Lab Attributes

Coordinate with your IdP administrator to configure:

- **labRole**: Direct role mapping for lab system
- **department**: User's department
- **institution**: User's institution

### Attribute Mapping (`/etc/shibboleth/attribute-map.xml`)

```xml
<Attributes xmlns="urn:mace:shibboleth:2.0:attribute-map">
    <!-- Required Attributes -->
    <Attribute name="urn:oid:1.3.6.1.4.1.5923.1.1.1.6" id="eppn"/>
    <Attribute name="urn:oid:0.9.2342.19200300.100.1.3" id="mail"/>
    
    <!-- Recommended Attributes -->
    <Attribute name="urn:oid:2.16.840.1.113730.3.1.241" id="displayName"/>
    <Attribute name="urn:oid:2.5.4.42" id="givenName"/>
    <Attribute name="urn:oid:2.5.4.4" id="sn"/>
    <Attribute name="urn:oid:1.3.6.1.4.1.5923.1.1.1.1" id="affiliation"/>
    <Attribute name="urn:oid:1.3.6.1.4.1.5923.1.1.1.7" id="entitlement"/>
    <Attribute name="urn:oid:1.3.6.1.4.1.5923.1.5.1.1" id="isMemberOf"/>
    
    <!-- Custom Lab Attributes (coordinate with IdP) -->
    <Attribute name="urn:oid:1.3.6.1.4.1.25178.1.2.9" id="labRole"/>
    <Attribute name="urn:oid:2.5.4.11" id="department"/>
    <Attribute name="urn:oid:2.5.4.10" id="institution"/>
</Attributes>
```

## Implementation Details

### Middleware Architecture

The system uses a layered middleware approach:

1. **Hybrid Authentication Middleware** (`hybrid_auth_middleware`):
   - Checks for Shibboleth attributes first
   - Falls back to JWT authentication
   - Injects user context into request

2. **Shibboleth-Only Middleware** (`shibboleth_auth_middleware`):
   - Only accepts Shibboleth authentication
   - Rejects requests without valid Shibboleth attributes

3. **JWT-Only Middleware** (`auth_middleware`):
   - Traditional JWT Bearer token authentication
   - Maintains backward compatibility

### User Auto-Creation

When `SHIBBOLETH_AUTO_CREATE_USERS=true`, the system:

1. Checks if user exists by email
2. If not found, creates new user with:
   - Email from `mail` attribute
   - Name from `givenName` and `sn` attributes
   - Role mapped from Shibboleth attributes
   - Email marked as verified (institutional trust)
3. Sets user status to active

### Attribute Synchronization

When `SHIBBOLETH_AUTO_UPDATE_ATTRIBUTES=true`, the system:

1. Updates user attributes on each login
2. Synchronizes role changes from IdP
3. Updates department and institution information
4. Maintains attribute freshness

## API Endpoints

### Authentication Endpoints

- **`GET /shibboleth-login`**: Initiates Shibboleth authentication
- **`GET /shibboleth-logout`**: Handles Shibboleth logout
- **`POST /api/auth/login`**: JWT-based login (still available)

### Protected Endpoints

All API endpoints under `/api/` support hybrid authentication:

- If Shibboleth headers are present, user is authenticated via Shibboleth
- If JWT Bearer token is present, user is authenticated via JWT
- Authentication method is transparent to the application logic

## Testing

### Testing Shibboleth Integration

1. **Check SP Status**:
   ```bash
   curl https://lab-manager.example.edu/Shibboleth.sso/Status
   ```

2. **Test Authentication Flow**:
   ```bash
   # Initiate Shibboleth login
   curl -L https://lab-manager.example.edu/shibboleth-login
   ```

3. **Verify Attribute Release**:
   - Access the application after authentication
   - Check server logs for attribute values
   - Use the test endpoint if configured

4. **Test Hybrid Mode**:
   ```bash
   # Test JWT authentication still works
   curl -H "Authorization: Bearer <jwt_token>" \
        https://lab-manager.example.edu/api/users/me
   ```

### Common Testing Issues

1. **No Attributes Received**:
   - Check IdP attribute release policy
   - Verify SP metadata registration
   - Review Shibboleth SP logs

2. **Role Mapping Issues**:
   - Check attribute values in logs
   - Verify role mapping configuration
   - Test with different user accounts

3. **Session Problems**:
   - Check session timeout settings
   - Verify cookie configuration
   - Test logout functionality

## Security Considerations

### Production Security

1. **HTTPS Required**: Shibboleth requires HTTPS in production
2. **Session Security**: Configure secure session timeouts
3. **Attribute Filtering**: Only release necessary attributes
4. **Regular Updates**: Keep Shibboleth SP software updated

### Monitoring and Logging

Monitor these log entries:

```bash
# Shibboleth authentication attempts
grep "Shibboleth authentication" /var/log/lab-manager/application.log

# User auto-creation events
grep "created.*from Shibboleth" /var/log/lab-manager/application.log

# Role mapping activities
grep "role.*mapped" /var/log/lab-manager/application.log

# Authentication failures
grep "SHIBBOLETH_AUTH_FAILED" /var/log/lab-manager/application.log
```

## Troubleshooting

### Common Issues

1. **Shibboleth Headers Not Present**:
   - Check web server configuration
   - Verify mod_shib2 is loaded and enabled
   - Check SP configuration

2. **User Creation Failures**:
   - Verify required attributes are present
   - Check database connectivity
   - Review user validation rules

3. **Role Mapping Problems**:
   - Check attribute values in logs
   - Verify mapping configuration
   - Test with known user accounts

4. **Hybrid Mode Not Working**:
   - Verify `SHIBBOLETH_HYBRID_MODE=true`
   - Check middleware configuration
   - Test both authentication methods

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug
```

This will provide detailed information about:
- Attribute extraction
- User creation/update processes
- Role mapping decisions
- Authentication flow

## Migration Guide

### From JWT-Only to Hybrid

1. **Enable Shibboleth** in configuration:
   ```bash
   SHIBBOLETH_ENABLED=true
   SHIBBOLETH_HYBRID_MODE=true
   ```

2. **Deploy web server configuration**
3. **Test both authentication methods**
4. **Monitor user creation and updates**

### Existing User Integration

Existing users can be linked to Shibboleth by:
1. Matching email addresses
2. Manual account linking by administrators
3. User self-service linking (if implemented)

## Support and Resources

### Documentation
- [Shibboleth SP Documentation](https://shibboleth.atlassian.net/wiki/spaces/SP3/overview)
- [SAML 2.0 Specification](https://docs.oasis-open.org/security/saml/v2.0/)

### Community
- [Shibboleth Users Mailing List](https://shibboleth.atlassian.net/wiki/spaces/users/overview)
- [InCommon Community](https://www.incommon.org/)

### Configuration Examples
- See `lab_manager/deploy/apache-shibboleth.conf`
- See `lab_manager/deploy/nginx-shibboleth.conf`
- See `lab_manager/env.shibboleth.example`

## Conclusion

The Shibboleth integration provides seamless institutional authentication while maintaining system flexibility through hybrid authentication support. The implementation handles user lifecycle management, role mapping, and session management automatically, reducing administrative overhead while enhancing security through institutional identity federation.

For additional support or custom configuration requirements, consult with your institutional identity management team and the Lab Manager system administrators.

*Context improved by Giga AI* 
