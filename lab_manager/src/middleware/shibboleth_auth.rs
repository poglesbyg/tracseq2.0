use axum::{
    extract::{Request, State},
    http::{header::HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::collections::HashMap;

use crate::{
    assembly::AppComponents,
    models::user::{User, UserRole},
};

/// Shibboleth authentication middleware that processes headers from Shibboleth SP
pub async fn shibboleth_auth_middleware(
    State(components): State<AppComponents>,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let headers = request.headers();

    // Extract Shibboleth attributes from headers
    let shibboleth_attrs = extract_shibboleth_attributes(headers);

    // Check if this is a Shibboleth-authenticated request
    if let Some(user_id) = shibboleth_attrs
        .get("eppn")
        .or_else(|| shibboleth_attrs.get("uid"))
    {
        match authenticate_shibboleth_user(&components, user_id, &shibboleth_attrs).await {
            Ok((user, session_id)) => {
                // Add user and session to request extensions for downstream handlers
                request.extensions_mut().insert(user);
                request.extensions_mut().insert(session_id);
                return Ok(next.run(request).await);
            }
            Err(e) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "error": {
                            "code": "SHIBBOLETH_AUTH_FAILED",
                            "message": format!("Shibboleth authentication failed: {}", e)
                        }
                    })),
                ));
            }
        }
    }

    // If no Shibboleth attributes, fall back to JWT authentication
    // This allows the system to work with both authentication methods
    Err((
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({
            "error": {
                "code": "NO_AUTHENTICATION",
                "message": "Neither Shibboleth nor JWT authentication found"
            }
        })),
    ))
}

/// Extract Shibboleth attributes from HTTP headers
pub fn extract_shibboleth_attributes(headers: &HeaderMap) -> HashMap<String, String> {
    let mut attributes = HashMap::new();

    // Common Shibboleth attribute headers
    let shibboleth_headers = [
        ("eppn", "HTTP_EPPN"),
        ("uid", "HTTP_UID"),
        ("mail", "HTTP_MAIL"),
        ("displayName", "HTTP_DISPLAYNAME"),
        ("givenName", "HTTP_GIVENNAME"),
        ("surname", "HTTP_SN"),
        ("affiliation", "HTTP_AFFILIATION"),
        ("entitlement", "HTTP_ENTITLEMENT"),
        ("isMemberOf", "HTTP_ISMEMBEROF"),
        // Custom lab-specific attributes
        ("labRole", "HTTP_LAB_ROLE"),
        ("department", "HTTP_DEPARTMENT"),
        ("institution", "HTTP_INSTITUTION"),
    ];

    for (attr_name, header_name) in shibboleth_headers {
        if let Some(header_value) = headers.get(header_name) {
            if let Ok(value) = header_value.to_str() {
                attributes.insert(attr_name.to_string(), value.to_string());
            }
        }
    }

    attributes
}

/// Authenticate user based on Shibboleth attributes
async fn authenticate_shibboleth_user(
    components: &AppComponents,
    _user_id: &str,
    attributes: &HashMap<String, String>,
) -> Result<(User, uuid::Uuid), Box<dyn std::error::Error + Send + Sync>> {
    // Try to find existing user by email or create new one
    let email = attributes.get("mail").ok_or("Missing email attribute")?;

    let user = match components.user_manager.get_user_by_email(email).await {
        Ok(existing_user) => {
            // Update user attributes from Shibboleth if needed
            update_user_from_shibboleth(&components, existing_user, attributes).await?
        }
        Err(_) => {
            // Create new user from Shibboleth attributes
            create_user_from_shibboleth(&components, attributes).await?
        }
    };

    // Create or retrieve session for the user
    let session = components
        .auth_service
        .create_session(
            user.id,
            None, // IP will be extracted from headers elsewhere
            Some("Shibboleth".to_string()),
        )
        .await?;

    Ok((user, session.id))
}

/// Create a new user from Shibboleth attributes
async fn create_user_from_shibboleth(
    components: &AppComponents,
    attributes: &HashMap<String, String>,
) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
    let email = attributes.get("mail").ok_or("Missing email attribute")?;
    let given_name = attributes
        .get("givenName")
        .map_or("Unknown", |s| s.as_str());
    let surname = attributes.get("surname").map_or("User", |s| s.as_str());
    let _display_name = attributes
        .get("displayName")
        .cloned()
        .unwrap_or_else(|| format!("{} {}", given_name, surname));

    // Map Shibboleth roles/entitlements to lab roles
    let role = map_shibboleth_role_to_lab_role(attributes);

    let user_request = crate::models::user::CreateUserRequest {
        email: email.clone(),
        password: format!("shibboleth-{}", uuid::Uuid::new_v4()), // Placeholder password for Shibboleth users
        first_name: given_name.to_string(),
        last_name: surname.to_string(),
        role,
        department: attributes.get("department").cloned(),
        position: None,
        phone: None,
        office_location: None,
        lab_affiliation: attributes.get("affiliation").cloned(),
    };

    let user = components
        .user_manager
        .create_user(user_request, None)
        .await?;

    Ok(user)
}

/// Update existing user with Shibboleth attributes
async fn update_user_from_shibboleth(
    components: &AppComponents,
    mut user: User,
    attributes: &HashMap<String, String>,
) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
    let mut updated = false;

    // Update display name if different
    if let Some(display_name) = attributes.get("displayName") {
        if user.first_name != *display_name {
            // You might want to parse first/last name from display name
            updated = true;
        }
    }

    // Update department if provided
    if let Some(department) = attributes.get("department") {
        if user.department.as_ref() != Some(department) {
            user.department = Some(department.clone());
            updated = true;
        }
    }

    // Update role based on current entitlements
    let new_role = map_shibboleth_role_to_lab_role(attributes);
    if user.role != new_role {
        user.role = new_role;
        updated = true;
    }

    if updated {
        let update_request = crate::models::user::UpdateUserRequest {
            email: None,
            first_name: Some(user.first_name.clone()),
            last_name: Some(user.last_name.clone()),
            role: Some(user.role.clone()),
            status: None, // Don't change status
            lab_affiliation: user.lab_affiliation.clone(),
            department: user.department.clone(),
            position: None,
            phone: user.phone.clone(),
            office_location: None,
        };

        components
            .user_manager
            .update_user(user.id, update_request)
            .await?;
    }

    Ok(user)
}

/// Map Shibboleth entitlements/roles to lab management system roles
pub fn map_shibboleth_role_to_lab_role(attributes: &HashMap<String, String>) -> UserRole {
    // Check for explicit lab role attribute first
    if let Some(lab_role) = attributes.get("labRole") {
        match lab_role.to_lowercase().as_str() {
            "lab_administrator" | "lab_admin" => return UserRole::LabAdministrator,
            "principal_investigator" | "pi" => return UserRole::PrincipalInvestigator,
            "lab_technician" | "technician" => return UserRole::LabTechnician,
            "research_scientist" | "scientist" => return UserRole::ResearchScientist,
            "data_analyst" | "analyst" => return UserRole::DataAnalyst,
            _ => {}
        }
    }

    // Check entitlements for role mapping
    if let Some(entitlements) = attributes.get("entitlement") {
        let entitlements_lower = entitlements.to_lowercase();

        if entitlements_lower.contains("lab:admin")
            || entitlements_lower.contains("lab:administrator")
        {
            return UserRole::LabAdministrator;
        }
        if entitlements_lower.contains("lab:pi")
            || entitlements_lower.contains("lab:principal_investigator")
        {
            return UserRole::PrincipalInvestigator;
        }
        if entitlements_lower.contains("lab:technician") {
            return UserRole::LabTechnician;
        }
        if entitlements_lower.contains("lab:scientist")
            || entitlements_lower.contains("lab:researcher")
        {
            return UserRole::ResearchScientist;
        }
        if entitlements_lower.contains("lab:analyst") {
            return UserRole::DataAnalyst;
        }
    }

    // Check group memberships
    if let Some(groups) = attributes.get("isMemberOf") {
        let groups_lower = groups.to_lowercase();

        if groups_lower.contains("cn=lab-administrators") {
            return UserRole::LabAdministrator;
        }
        if groups_lower.contains("cn=principal-investigators") {
            return UserRole::PrincipalInvestigator;
        }
        if groups_lower.contains("cn=lab-technicians") {
            return UserRole::LabTechnician;
        }
        if groups_lower.contains("cn=research-scientists") {
            return UserRole::ResearchScientist;
        }
        if groups_lower.contains("cn=data-analysts") {
            return UserRole::DataAnalyst;
        }
    }

    // Default to guest role if no specific lab permissions found
    UserRole::Guest
}

/// Hybrid authentication middleware that tries Shibboleth first, then JWT
pub async fn hybrid_auth_middleware(
    State(components): State<AppComponents>,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let headers = request.headers();

    // First, try Shibboleth authentication
    let shibboleth_attrs = extract_shibboleth_attributes(headers);
    if let Some(user_id) = shibboleth_attrs
        .get("eppn")
        .or_else(|| shibboleth_attrs.get("uid"))
    {
        if let Ok((user, session_id)) =
            authenticate_shibboleth_user(&components, user_id, &shibboleth_attrs).await
        {
            request.extensions_mut().insert(user);
            request.extensions_mut().insert(session_id);
            return Ok(next.run(request).await);
        }
    }

    // If Shibboleth fails, try JWT authentication
    if let Some(auth_header) = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        })
    {
        if let Ok((user, session)) = components.auth_service.verify_token(auth_header).await {
            request.extensions_mut().insert(user);
            request.extensions_mut().insert(session.id);
            return Ok(next.run(request).await);
        }
    }

    // No valid authentication found
    Err((
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({
            "error": {
                "code": "AUTHENTICATION_REQUIRED",
                "message": "Valid Shibboleth or JWT authentication required"
            }
        })),
    ))
}
