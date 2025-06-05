use anyhow::Result;
use saml2::{
    metadata::{EntityDescriptor, IdpSsoDescriptor},
    service_provider::ServiceProvider,
    Binding, Endpoint, EntityId, IdentityProvider, MetadataUrl, SingleSignOnService,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SamlConfig {
    pub entity_id: String,
    pub acs_url: String,
    pub idp_metadata_url: String,
}

pub struct SamlService {
    service_provider: Arc<RwLock<ServiceProvider>>,
    identity_provider: Arc<RwLock<IdentityProvider>>,
}

impl SamlService {
    pub async fn new(config: SamlConfig) -> Result<Self> {
        // Create service provider
        let service_provider = ServiceProvider::new(
            EntityId::new(config.entity_id),
            vec![Endpoint::new(Binding::HTTP_POST, config.acs_url)],
        );

        // Fetch and parse IdP metadata
        let metadata = reqwest::get(&config.idp_metadata_url).await?.text().await?;

        let entity_descriptor: EntityDescriptor = quick_xml::de::from_str(&metadata)?;

        let idp_sso = entity_descriptor
            .idp_sso_descriptors
            .first()
            .ok_or_else(|| anyhow::anyhow!("No IdP SSO descriptor found in metadata"))?;

        let sso_endpoint = idp_sso
            .single_sign_on_services
            .iter()
            .find(|sso| sso.binding == Binding::HTTP_REDIRECT)
            .ok_or_else(|| anyhow::anyhow!("No HTTP-Redirect SSO endpoint found"))?;

        let identity_provider = IdentityProvider::new(
            entity_descriptor.entity_id,
            vec![sso_endpoint.clone()],
            idp_sso.key_descriptors.clone(),
        );

        Ok(Self {
            service_provider: Arc::new(RwLock::new(service_provider)),
            identity_provider: Arc::new(RwLock::new(identity_provider)),
        })
    }

    pub async fn create_auth_request(&self) -> Result<String> {
        let sp = self.service_provider.read().await;
        let idp = self.identity_provider.read().await;

        let auth_request = sp.create_auth_request(&idp, Binding::HTTP_REDIRECT, None)?;

        Ok(auth_request.redirect_url()?)
    }

    pub async fn process_response(&self, response: &str) -> Result<SamlUserInfo> {
        let sp = self.service_provider.read().await;
        let idp = self.identity_provider.read().await;

        let response = sp.process_response(&idp, response, None)?;

        // Extract user information from SAML attributes
        let attributes = response.attributes();

        Ok(SamlUserInfo {
            unc_pid: attributes
                .get("urn:oid:2.16.840.1.113730.3.1.3") // UNC PID attribute
                .and_then(|v| v.first())
                .ok_or_else(|| anyhow::anyhow!("Missing UNC PID"))?
                .to_string(),
            email: attributes
                .get("urn:oid:0.9.2342.19200300.100.1.3") // email attribute
                .and_then(|v| v.first())
                .ok_or_else(|| anyhow::anyhow!("Missing email"))?
                .to_string(),
            eppn: attributes
                .get("urn:oid:1.3.6.1.4.1.5923.1.1.1.6") // eduPersonPrincipalName
                .and_then(|v| v.first())
                .ok_or_else(|| anyhow::anyhow!("Missing eppn"))?
                .to_string(),
            given_name: attributes
                .get("urn:oid:2.5.4.42") // givenName
                .and_then(|v| v.first())
                .ok_or_else(|| anyhow::anyhow!("Missing given name"))?
                .to_string(),
            family_name: attributes
                .get("urn:oid:2.5.4.4") // sn
                .and_then(|v| v.first())
                .ok_or_else(|| anyhow::anyhow!("Missing family name"))?
                .to_string(),
            display_name: attributes
                .get("urn:oid:2.16.840.1.113730.3.1.241") // displayName
                .and_then(|v| v.first())
                .ok_or_else(|| anyhow::anyhow!("Missing display name"))?
                .to_string(),
            affiliation: attributes
                .get("urn:oid:1.3.6.1.4.1.5923.1.1.1.9") // eduPersonAffiliation
                .and_then(|v| v.first())
                .unwrap_or("unknown")
                .to_string(),
            department: attributes
                .get("urn:oid:1.3.6.1.4.1.5923.1.1.1.4") // eduPersonOrgUnitDN
                .and_then(|v| v.first())
                .unwrap_or("unknown")
                .to_string(),
            title: attributes
                .get("urn:oid:2.5.4.12") // title
                .and_then(|v| v.first())
                .unwrap_or("unknown")
                .to_string(),
        })
    }
}

#[derive(Debug)]
pub struct SamlUserInfo {
    pub unc_pid: String,
    pub email: String,
    pub eppn: String,
    pub given_name: String,
    pub family_name: String,
    pub display_name: String,
    pub affiliation: String,
    pub department: String,
    pub title: String,
}
