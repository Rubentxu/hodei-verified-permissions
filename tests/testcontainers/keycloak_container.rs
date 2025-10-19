//! Keycloak testcontainer for E2E testing
//!
//! This module provides a wrapper around the Keycloak Docker container
//! with automatic configuration of realms, clients, users, and roles.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use testcontainers::{
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

/// Keycloak container configuration
#[derive(Debug, Clone)]
pub struct KeycloakConfig {
    /// Realm name
    pub realm: String,
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Admin username
    pub admin_username: String,
    /// Admin password
    pub admin_password: String,
    /// Users to create (username, password, roles)
    pub users: Vec<(String, String, Vec<String>)>,
}

impl Default for KeycloakConfig {
    fn default() -> Self {
        Self {
            realm: "test-realm".to_string(),
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
            users: vec![
                ("testuser".to_string(), "password".to_string(), vec!["user".to_string()]),
                ("testadmin".to_string(), "password".to_string(), vec!["admin".to_string()]),
            ],
        }
    }
}

/// Keycloak container wrapper
pub struct KeycloakContainer {
    container: ContainerAsync<GenericImage>,
    config: KeycloakConfig,
    base_url: String,
    realm_url: String,
}

/// Token response from Keycloak
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

impl KeycloakContainer {
    /// Start a new Keycloak container with default configuration
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        Self::start_with_config(KeycloakConfig::default()).await
    }

    /// Start a new Keycloak container with custom configuration
    pub async fn start_with_config(config: KeycloakConfig) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🚀 Starting Keycloak container...");

        // Create Keycloak image
        let image = GenericImage::new("quay.io/keycloak/keycloak", "latest")
            .with_exposed_port(ContainerPort::Tcp(8080))
            .with_env_var("KEYCLOAK_ADMIN", &config.admin_username)
            .with_env_var("KEYCLOAK_ADMIN_PASSWORD", &config.admin_password)
            .with_wait_for(WaitFor::message_on_stdout("Listening on:"))
            .with_cmd(vec!["start-dev"]);

        // Start container
        let container = image.start().await?;

        // Get host and port
        let host = container.get_host().await?;
        let port = container.get_host_port_ipv4(8080).await?;

        let base_url = format!("http://{}:{}", host, port);
        let realm_url = format!("{}/realms/{}", base_url, config.realm);

        println!("✅ Keycloak started at: {}", base_url);

        let mut kc = Self {
            container,
            config,
            base_url,
            realm_url,
        };

        // Wait for Keycloak to be ready
        kc.wait_for_ready().await?;

        // Configure Keycloak
        kc.configure().await?;

        Ok(kc)
    }

    /// Wait for Keycloak to be ready
    async fn wait_for_ready(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⏳ Waiting for Keycloak to be ready...");

        let client = reqwest::Client::new();
        let health_url = format!("{}/health/ready", self.base_url);

        for i in 1..=30 {
            match client.get(&health_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    println!("✅ Keycloak is ready!");
                    return Ok(());
                }
                _ => {
                    if i == 30 {
                        return Err("Keycloak failed to start within 30 seconds".into());
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }

        Ok(())
    }

    /// Configure Keycloak (create realm, client, users, roles)
    async fn configure(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Configuring Keycloak...");

        // Get admin token
        let admin_token = self.get_admin_token().await?;

        // Create realm
        self.create_realm(&admin_token).await?;

        // Create client
        self.create_client(&admin_token).await?;

        // Create roles
        self.create_roles(&admin_token).await?;

        // Create users
        self.create_users(&admin_token).await?;

        println!("✅ Keycloak configured successfully!");

        Ok(())
    }

    /// Get admin access token
    async fn get_admin_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let token_url = format!("{}/realms/master/protocol/openid-connect/token", self.base_url);

        let params = [
            ("grant_type", "password"),
            ("client_id", "admin-cli"),
            ("username", &self.config.admin_username),
            ("password", &self.config.admin_password),
        ];

        let response = client
            .post(&token_url)
            .form(&params)
            .send()
            .await?;

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response.access_token)
    }

    /// Create realm
    async fn create_realm(&self, admin_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!("{}/admin/realms", self.base_url);

        let realm_data = json!({
            "realm": self.config.realm,
            "enabled": true,
            "displayName": self.config.realm,
        });

        let response = client
            .post(&url)
            .bearer_auth(admin_token)
            .json(&realm_data)
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 409 {
            println!("  ✓ Realm '{}' created", self.config.realm);
            Ok(())
        } else {
            Err(format!("Failed to create realm: {}", response.status()).into())
        }
    }

    /// Create client
    async fn create_client(&self, admin_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!("{}/admin/realms/{}/clients", self.base_url, self.config.realm);

        let client_data = json!({
            "clientId": self.config.client_id,
            "enabled": true,
            "publicClient": false,
            "secret": self.config.client_secret,
            "directAccessGrantsEnabled": true,
            "serviceAccountsEnabled": true,
            "standardFlowEnabled": true,
            "redirectUris": ["*"],
            "webOrigins": ["*"],
        });

        let response = client
            .post(&url)
            .bearer_auth(admin_token)
            .json(&client_data)
            .send()
            .await?;

        if response.status().is_success() || response.status().as_u16() == 409 {
            println!("  ✓ Client '{}' created", self.config.client_id);
            Ok(())
        } else {
            Err(format!("Failed to create client: {}", response.status()).into())
        }
    }

    /// Create roles
    async fn create_roles(&self, admin_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        // Collect all unique roles from users
        let mut roles: Vec<String> = self.config.users
            .iter()
            .flat_map(|(_, _, roles)| roles.clone())
            .collect();
        roles.sort();
        roles.dedup();

        for role in roles {
            let url = format!("{}/admin/realms/{}/roles", self.base_url, self.config.realm);

            let role_data = json!({
                "name": role,
                "description": format!("Role: {}", role),
            });

            let response = client
                .post(&url)
                .bearer_auth(admin_token)
                .json(&role_data)
                .send()
                .await?;

            if response.status().is_success() || response.status().as_u16() == 409 {
                println!("  ✓ Role '{}' created", role);
            }
        }

        Ok(())
    }

    /// Create users
    async fn create_users(&self, admin_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let http_client = reqwest::Client::new();

        for (username, password, roles) in &self.config.users {
            // Create user
            let url = format!("{}/admin/realms/{}/users", self.base_url, self.config.realm);

            let user_data = json!({
                "username": username,
                "enabled": true,
                "credentials": [{
                    "type": "password",
                    "value": password,
                    "temporary": false,
                }],
            });

            let response = http_client
                .post(&url)
                .bearer_auth(admin_token)
                .json(&user_data)
                .send()
                .await?;

            if !response.status().is_success() && response.status().as_u16() != 409 {
                continue;
            }

            println!("  ✓ User '{}' created", username);

            // Get user ID
            let search_url = format!("{}/admin/realms/{}/users?username={}", self.base_url, self.config.realm, username);
            let users: Vec<serde_json::Value> = http_client
                .get(&search_url)
                .bearer_auth(admin_token)
                .send()
                .await?
                .json()
                .await?;

            if let Some(user) = users.first() {
                if let Some(user_id) = user.get("id").and_then(|v| v.as_str()) {
                    // Assign roles
                    for role in roles {
                        self.assign_role_to_user(admin_token, user_id, role).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Assign role to user
    async fn assign_role_to_user(
        &self,
        admin_token: &str,
        user_id: &str,
        role_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        // Get role
        let role_url = format!("{}/admin/realms/{}/roles/{}", self.base_url, self.config.realm, role_name);
        let role: serde_json::Value = client
            .get(&role_url)
            .bearer_auth(admin_token)
            .send()
            .await?
            .json()
            .await?;

        // Assign role
        let assign_url = format!(
            "{}/admin/realms/{}/users/{}/role-mappings/realm",
            self.base_url, self.config.realm, user_id
        );

        client
            .post(&assign_url)
            .bearer_auth(admin_token)
            .json(&vec![role])
            .send()
            .await?;

        println!("    → Role '{}' assigned to user", role_name);

        Ok(())
    }

    /// Get access token for a user
    pub async fn get_user_token(
        &self,
        username: &str,
        password: &str,
    ) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let token_url = format!("{}/protocol/openid-connect/token", self.realm_url);

        let params = [
            ("grant_type", "password"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("username", username),
            ("password", password),
        ];

        let response = client
            .post(&token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to get token: {}", error_text).into());
        }

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response)
    }

    /// Get the issuer URL for this Keycloak instance
    pub fn issuer(&self) -> String {
        self.realm_url.clone()
    }

    /// Get the JWKS URI for this Keycloak instance
    pub fn jwks_uri(&self) -> String {
        format!("{}/protocol/openid-connect/certs", self.realm_url)
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the realm name
    pub fn realm(&self) -> &str {
        &self.config.realm
    }

    /// Get the client ID
    pub fn client_id(&self) -> &str {
        &self.config.client_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_keycloak_container_starts() {
        let kc = KeycloakContainer::start().await;
        assert!(kc.is_ok());

        let kc = kc.unwrap();
        assert!(!kc.issuer().is_empty());
        assert!(!kc.jwks_uri().is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_get_user_token() {
        let kc = KeycloakContainer::start().await.unwrap();

        let token = kc.get_user_token("testuser", "password").await;
        assert!(token.is_ok());

        let token = token.unwrap();
        assert!(!token.access_token.is_empty());
    }
}
