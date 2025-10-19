//! Identity Provider specific implementations
//!
//! This module contains provider-specific logic for different IdPs like
//! Keycloak, Zitadel, Cognito, etc.

pub mod keycloak;
pub mod zitadel;

pub use keycloak::KeycloakProvider;
pub use zitadel::ZitadelProvider;

use crate::error::Result;
use crate::jwt::{ClaimsMappingConfig, ValidatedClaims};

/// Trait for IdP-specific claim processing
pub trait IdentityProvider {
    /// Get the provider name
    fn name(&self) -> &'static str;
    
    /// Detect if this provider handles the given issuer
    fn matches_issuer(&self, issuer: &str) -> bool;
    
    /// Create a claims mapping configuration for this provider
    fn create_claims_config(&self) -> ClaimsMappingConfig;
    
    /// Validate provider-specific requirements
    fn validate_claims(&self, claims: &ValidatedClaims) -> Result<()> {
        // Default implementation: no additional validation
        let _ = claims;
        Ok(())
    }
}

/// Auto-detect the identity provider from issuer URL
pub fn detect_provider(issuer: &str) -> Option<Box<dyn IdentityProvider>> {
    // Try Zitadel first (more specific pattern)
    if ZitadelProvider::default().matches_issuer(issuer) {
        return Some(Box::new(ZitadelProvider::default()));
    }
    
    // Try Keycloak
    if KeycloakProvider::default().matches_issuer(issuer) {
        return Some(Box::new(KeycloakProvider::default()));
    }
    
    // Add more providers here as they are implemented
    // if CognitoProvider::default().matches_issuer(issuer) { ... }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_keycloak() {
        let issuer = "https://keycloak.example.com/realms/myapp";
        let provider = detect_provider(issuer);
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().name(), "Keycloak");
    }

    #[test]
    fn test_detect_zitadel() {
        let issuer = "https://myinstance.zitadel.cloud";
        let provider = detect_provider(issuer);
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().name(), "Zitadel");
    }

    #[test]
    fn test_detect_unknown() {
        let issuer = "https://unknown.example.com";
        let provider = detect_provider(issuer);
        assert!(provider.is_none());
    }
}
