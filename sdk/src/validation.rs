//! Validation utilities for authorization configurations

use crate::error::{Result, SdkError};
use crate::proto::OidcConfiguration;

#[cfg(feature = "schema")]
use url::Url;

/// Validator for OIDC configuration
pub struct OidcConfigValidator;

impl OidcConfigValidator {
    /// Validate OIDC configuration
    ///
    /// # Validation Rules
    ///
    /// - `issuer` must be a valid HTTPS URL (when schema feature is enabled)
    /// - `jwks_uri` must be a valid HTTPS URL (when schema feature is enabled)
    /// - `client_ids` must not be empty
    /// - At least one client ID must be provided
    ///
    /// # Arguments
    ///
    /// * `config` - The OIDC configuration to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the configuration is valid, or an error otherwise
    #[cfg(feature = "schema")]
    pub fn validate(config: &OidcConfiguration) -> Result<()> {
        // Validate issuer
        if config.issuer.is_empty() {
            return Err(SdkError::InvalidRequest(
                "OIDC issuer cannot be empty".to_string(),
            ));
        }

        // Validate issuer is a valid HTTPS URL
        match Url::parse(&config.issuer) {
            Ok(url) => {
                if url.scheme() != "https" {
                    return Err(SdkError::InvalidRequest(
                        "OIDC issuer must use HTTPS scheme".to_string(),
                    ));
                }
            }
            Err(_) => {
                return Err(SdkError::InvalidRequest(
                    "OIDC issuer must be a valid URL".to_string(),
                ));
            }
        }

        // Validate JWKS URI
        if config.jwks_uri.is_empty() {
            return Err(SdkError::InvalidRequest(
                "OIDC JWKS URI cannot be empty".to_string(),
            ));
        }

        // Validate JWKS URI is a valid HTTPS URL
        match Url::parse(&config.jwks_uri) {
            Ok(url) => {
                if url.scheme() != "https" {
                    return Err(SdkError::InvalidRequest(
                        "OIDC JWKS URI must use HTTPS scheme".to_string(),
                    ));
                }
            }
            Err(_) => {
                return Err(SdkError::InvalidRequest(
                    "OIDC JWKS URI must be a valid URL".to_string(),
                ));
            }
        }

        // Validate client IDs
        if config.client_ids.is_empty() {
            return Err(SdkError::InvalidRequest(
                "OIDC client_ids cannot be empty".to_string(),
            ));
        }

        // Validate each client ID is not empty
        for (i, client_id) in config.client_ids.iter().enumerate() {
            if client_id.is_empty() {
                return Err(SdkError::InvalidRequest(
                    format!("OIDC client_id at index {} cannot be empty", i),
                ));
            }
        }

        Ok(())
    }

    /// Validate issuer URL format
    #[cfg(feature = "schema")]
    pub fn validate_issuer(issuer: &str) -> Result<()> {
        if issuer.is_empty() {
            return Err(SdkError::InvalidRequest(
                "Issuer cannot be empty".to_string(),
            ));
        }

        match Url::parse(issuer) {
            Ok(url) => {
                if url.scheme() != "https" {
                    return Err(SdkError::InvalidRequest(
                        "Issuer must use HTTPS scheme".to_string(),
                    ));
                }
                Ok(())
            }
            Err(_) => Err(SdkError::InvalidRequest(
                "Issuer must be a valid URL".to_string(),
            )),
        }
    }

    /// Validate JWKS URI format
    #[cfg(feature = "schema")]
    pub fn validate_jwks_uri(jwks_uri: &str) -> Result<()> {
        if jwks_uri.is_empty() {
            return Err(SdkError::InvalidRequest(
                "JWKS URI cannot be empty".to_string(),
            ));
        }

        match Url::parse(jwks_uri) {
            Ok(url) => {
                if url.scheme() != "https" {
                    return Err(SdkError::InvalidRequest(
                        "JWKS URI must use HTTPS scheme".to_string(),
                    ));
                }
                Ok(())
            }
            Err(_) => Err(SdkError::InvalidRequest(
                "JWKS URI must be a valid URL".to_string(),
            )),
        }
    }

    /// Validate client IDs
    pub fn validate_client_ids(client_ids: &[String]) -> Result<()> {
        if client_ids.is_empty() {
            return Err(SdkError::InvalidRequest(
                "Client IDs cannot be empty".to_string(),
            ));
        }

        for (i, client_id) in client_ids.iter().enumerate() {
            if client_id.is_empty() {
                return Err(SdkError::InvalidRequest(
                    format!("Client ID at index {} cannot be empty", i),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "schema")]
    fn test_validate_issuer_valid() {
        assert!(OidcConfigValidator::validate_issuer("https://auth.example.com").is_ok());
    }

    #[test]
    #[cfg(feature = "schema")]
    fn test_validate_issuer_empty() {
        assert!(OidcConfigValidator::validate_issuer("").is_err());
    }

    #[test]
    #[cfg(feature = "schema")]
    fn test_validate_issuer_http() {
        assert!(OidcConfigValidator::validate_issuer("http://auth.example.com").is_err());
    }

    #[test]
    #[cfg(feature = "schema")]
    fn test_validate_issuer_invalid_url() {
        assert!(OidcConfigValidator::validate_issuer("not a url").is_err());
    }

    #[test]
    #[cfg(feature = "schema")]
    fn test_validate_jwks_uri_valid() {
        assert!(OidcConfigValidator::validate_jwks_uri("https://auth.example.com/.well-known/jwks.json").is_ok());
    }

    #[test]
    #[cfg(feature = "schema")]
    fn test_validate_jwks_uri_empty() {
        assert!(OidcConfigValidator::validate_jwks_uri("").is_err());
    }

    #[test]
    #[cfg(feature = "schema")]
    fn test_validate_jwks_uri_http() {
        assert!(OidcConfigValidator::validate_jwks_uri("http://auth.example.com/.well-known/jwks.json").is_err());
    }

    #[test]
    fn test_validate_client_ids_valid() {
        let ids = vec!["client-1".to_string(), "client-2".to_string()];
        assert!(OidcConfigValidator::validate_client_ids(&ids).is_ok());
    }

    #[test]
    fn test_validate_client_ids_empty() {
        let ids: Vec<String> = vec![];
        assert!(OidcConfigValidator::validate_client_ids(&ids).is_err());
    }

    #[test]
    fn test_validate_client_ids_with_empty_id() {
        let ids = vec!["client-1".to_string(), "".to_string()];
        assert!(OidcConfigValidator::validate_client_ids(&ids).is_err());
    }

    #[test]
    #[cfg(feature = "schema")]
    fn test_validate_oidc_config_valid() {
        let config = OidcConfiguration {
            issuer: "https://auth.example.com".to_string(),
            client_ids: vec!["client-1".to_string()],
            jwks_uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
            group_claim: "groups".to_string(),
        };
        assert!(OidcConfigValidator::validate(&config).is_ok());
    }

    #[test]
    #[cfg(feature = "schema")]
    fn test_validate_oidc_config_missing_issuer() {
        let config = OidcConfiguration {
            issuer: "".to_string(),
            client_ids: vec!["client-1".to_string()],
            jwks_uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
            group_claim: "".to_string(),
        };
        assert!(OidcConfigValidator::validate(&config).is_err());
    }

    #[test]
    #[cfg(feature = "schema")]
    fn test_validate_oidc_config_missing_client_ids() {
        let config = OidcConfiguration {
            issuer: "https://auth.example.com".to_string(),
            client_ids: vec![],
            jwks_uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
            group_claim: "".to_string(),
        };
        assert!(OidcConfigValidator::validate(&config).is_err());
    }
}
