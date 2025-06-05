use serde::Deserialize;
use std::env;

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub saml_config: SamlConfig,
}

#[derive(Debug, Clone)]
pub struct SamlConfig {
    pub entity_id: String,
    pub acs_url: String,
    pub idp_metadata_url: String,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        Self {
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            saml_config: SamlConfig {
                entity_id: env::var("SAML_ENTITY_ID").expect("SAML_ENTITY_ID must be set"),
                acs_url: env::var("SAML_ACS_URL").expect("SAML_ACS_URL must be set"),
                idp_metadata_url: env::var("SAML_IDP_METADATA_URL")
                    .expect("SAML_IDP_METADATA_URL must be set"),
            },
        }
    }
}
