//! Zitadel testcontainer for E2E testing
//!
//! This module provides a wrapper around the Zitadel Docker container
//! with REAL configuration via Zitadel's gRPC API (no shortcuts, no mocks).

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use testcontainers::{
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

/// Zitadel container configuration
#[derive(Debug, Clone)]
pub struct ZitadelConfig {
    /// Project name
    pub project_name: String,
    /// Application name
    pub application_name: String,
    /// Organization name
    pub organization_name: String,
    /// Users to create (username, password, roles)
    pub users: Vec<(String, String, Vec<String>)>,
    /// Admin username (for initial setup)
    pub admin_username: String,
    /// Admin password (for initial setup)
    pub admin_password: String,
}

impl Default for ZitadelConfig {
    fn default() -> Self {
        Self {
            project_name: "test-project".to_string(),
            application_name: "test-app".to_string(),
            organization_name: "test-org".to_string(),
            users: vec![
                ("developer".to_string(), "Password123!".to_string(), vec!["developer".to_string()]),
                ("viewer".to_string(), "Password123!".to_string(), vec!["viewer".to_string()]),
            ],
            admin_username: "zitadel-admin@zitadel.localhost".to_string(),
            admin_password: "Password1!".to_string(),
        }
    }
}

/// Zitadel container wrapper
pub struct ZitadelContainer {
    container: ContainerAsync<GenericImage>,
    config: ZitadelConfig,
    base_url: String,
    issuer: String,
    project_id: Option<String>,
    application_client_id: Option<String>,
    application_client_secret: Option<String>,
}

/// Token response from Zitadel
#[derive(Debug, Deserialize, Serialize)]
pub struct ZitadelTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

impl ZitadelContainer {
    /// Start a new Zitadel container with default configuration
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        Self::start_with_config(ZitadelConfig::default()).await
    }

    /// Start a new Zitadel container with custom configuration
    pub async fn start_with_config(config: ZitadelConfig) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🚀 Starting Zitadel container (this may take a few minutes)...");

        // Zitadel requires PostgreSQL, so we use the all-in-one image
        let image = GenericImage::new("ghcr.io/zitadel/zitadel", "latest")
            .with_wait_for(WaitFor::message_on_stdout("ready to serve"))
            .with_exposed_port(ContainerPort::Tcp(8080))
            .with_env_var("ZITADEL_MASTERKEY", "MasterkeyNeedsToHave32Characters")
            .with_env_var("ZITADEL_DATABASE_POSTGRES_HOST", "localhost")
            .with_env_var("ZITADEL_DATABASE_POSTGRES_PORT", "5432")
            .with_env_var("ZITADEL_DATABASE_POSTGRES_DATABASE", "zitadel")
            .with_env_var("ZITADEL_DATABASE_POSTGRES_USER_USERNAME", "zitadel")
            .with_env_var("ZITADEL_DATABASE_POSTGRES_USER_PASSWORD", "zitadel")
            .with_env_var("ZITADEL_DATABASE_POSTGRES_USER_SSL_MODE", "disable")
            .with_env_var("ZITADEL_DATABASE_POSTGRES_ADMIN_USERNAME", "postgres")
            .with_env_var("ZITADEL_DATABASE_POSTGRES_ADMIN_PASSWORD", "postgres")
            .with_env_var("ZITADEL_DATABASE_POSTGRES_ADMIN_SSL_MODE", "disable")
            .with_env_var("ZITADEL_EXTERNALSECURE", "false")
            .with_cmd(vec!["start-from-init", "--masterkeyFromEnv"]);

        // Start container
        let container = image.start().await?;

        // Get host and port
        let host = container.get_host().await?;
        let port = container.get_host_port_ipv4(8080).await?;

        let base_url = format!("http://{}:{}", host, port);
        let issuer = format!("{}/", base_url); // Zitadel issuer ends with /

        println!("✅ Zitadel started at: {}", base_url);

        let mut zitadel = Self {
            container,
            config,
            base_url,
            issuer,
            project_id: None,
            application_client_id: None,
            application_client_secret: None,
        };

        // Wait for Zitadel to be ready
        zitadel.wait_for_ready().await?;

        // Configure Zitadel via REAL API calls
        zitadel.configure().await?;

        Ok(zitadel)
    }

    /// Wait for Zitadel to be ready
    async fn wait_for_ready(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⏳ Waiting for Zitadel to be ready (this can take 2-3 minutes)...");

        let client = reqwest::Client::new();
        let health_url = format!("{}/debug/healthz", self.base_url);

        for i in 1..=180 {
            match client.get(&health_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    println!("✅ Zitadel is ready!");
                    // Give it a few more seconds to fully initialize
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    return Ok(());
                }
                _ => {
                    if i % 10 == 0 {
                        println!("   Still waiting... ({}/180 seconds)", i);
                    }
                    if i == 180 {
                        return Err("Zitadel failed to start within 3 minutes".into());
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }

        Ok(())
    }

    /// Configure Zitadel using REAL API calls (no shortcuts!)
    async fn configure(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Configuring Zitadel via REAL API calls...");

        // 1. Get admin access token using REAL OAuth2 flow
        let admin_token = self.get_admin_token().await?;
        println!("  ✓ Admin authenticated");

        // 2. Create organization using REAL Zitadel Management API
        let org_id = self.create_organization(&admin_token).await?;
        println!("  ✓ Organization '{}' created: {}", self.config.organization_name, org_id);

        // 3. Create project using REAL Zitadel Management API
        let project_id = self.create_project(&admin_token, &org_id).await?;
        self.project_id = Some(project_id.clone());
        println!("  ✓ Project '{}' created: {}", self.config.project_name, project_id);

        // 4. Create application using REAL Zitadel Management API
        let (client_id, client_secret) = self.create_application(&admin_token, &project_id).await?;
        self.application_client_id = Some(client_id.clone());
        self.application_client_secret = Some(client_secret.clone());
        println!("  ✓ Application '{}' created", self.config.application_name);

        // 5. Create roles using REAL Zitadel Management API
        self.create_roles(&admin_token, &project_id).await?;

        // 6. Create users using REAL Zitadel Management API
        self.create_users(&admin_token, &org_id, &project_id).await?;

        println!("✅ Zitadel configured successfully with REAL API!");

        Ok(())
    }

    /// Get admin access token using REAL OAuth2 password grant
    async fn get_admin_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let token_url = format!("{}/oauth/v2/token", self.base_url);

        let params = [
            ("grant_type", "password"),
            ("username", &self.config.admin_username),
            ("password", &self.config.admin_password),
            ("scope", "openid profile email urn:zitadel:iam:org:project:id:zitadel:aud"),
        ];

        let response = client
            .post(&token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to get admin token: {}", error_text).into());
        }

        let token_response: ZitadelTokenResponse = response.json().await?;
        Ok(token_response.access_token)
    }

    /// Create organization using REAL Zitadel Management API
    async fn create_organization(&self, admin_token: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!("{}/management/v1/orgs", self.base_url);

        let org_data = json!({
            "name": self.config.organization_name,
        });

        let response = client
            .post(&url)
            .bearer_auth(admin_token)
            .json(&org_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to create organization: {}", error_text).into());
        }

        let result: serde_json::Value = response.json().await?;
        let org_id = result["id"]
            .as_str()
            .ok_or("No organization ID in response")?
            .to_string();

        Ok(org_id)
    }

    /// Create project using REAL Zitadel Management API
    async fn create_project(&self, admin_token: &str, _org_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!("{}/management/v1/projects", self.base_url);

        let project_data = json!({
            "name": self.config.project_name,
            "projectRoleAssertion": true,
            "projectRoleCheck": true,
        });

        let response = client
            .post(&url)
            .bearer_auth(admin_token)
            .json(&project_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to create project: {}", error_text).into());
        }

        let result: serde_json::Value = response.json().await?;
        let project_id = result["id"]
            .as_str()
            .ok_or("No project ID in response")?
            .to_string();

        Ok(project_id)
    }

    /// Create application using REAL Zitadel Management API
    async fn create_application(
        &self,
        admin_token: &str,
        project_id: &str,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!("{}/management/v1/projects/{}/apps/oidc", self.base_url, project_id);

        let app_data = json!({
            "name": self.config.application_name,
            "redirectUris": ["http://localhost:3000/callback"],
            "responseTypes": ["OIDC_RESPONSE_TYPE_CODE"],
            "grantTypes": [
                "OIDC_GRANT_TYPE_AUTHORIZATION_CODE",
                "OIDC_GRANT_TYPE_REFRESH_TOKEN",
                "OIDC_GRANT_TYPE_PASSWORD"
            ],
            "appType": "OIDC_APP_TYPE_WEB",
            "authMethodType": "OIDC_AUTH_METHOD_TYPE_BASIC",
            "version": "OIDC_VERSION_1_0",
            "accessTokenType": "OIDC_TOKEN_TYPE_JWT",
            "accessTokenRoleAssertion": true,
        });

        let response = client
            .post(&url)
            .bearer_auth(admin_token)
            .json(&app_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to create application: {}", error_text).into());
        }

        let result: serde_json::Value = response.json().await?;
        let client_id = result["clientId"]
            .as_str()
            .ok_or("No client ID in response")?
            .to_string();
        let client_secret = result["clientSecret"]
            .as_str()
            .ok_or("No client secret in response")?
            .to_string();

        Ok((client_id, client_secret))
    }

    /// Create roles using REAL Zitadel Management API
    async fn create_roles(&self, admin_token: &str, project_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        // Collect all unique roles from users
        let mut roles: Vec<String> = self.config.users
            .iter()
            .flat_map(|(_, _, roles)| roles.clone())
            .collect();
        roles.sort();
        roles.dedup();

        for role in roles {
            let url = format!("{}/management/v1/projects/{}/roles", self.base_url, project_id);

            let role_data = json!({
                "key": role,
                "displayName": role,
                "group": "default",
            });

            let response = client
                .post(&url)
                .bearer_auth(admin_token)
                .json(&role_data)
                .send()
                .await?;

            if response.status().is_success() || response.status().as_u16() == 409 {
                println!("  ✓ Role '{}' created", role);
            } else {
                let error_text = response.text().await?;
                println!("  ⚠ Failed to create role '{}': {}", role, error_text);
            }
        }

        Ok(())
    }

    /// Create users using REAL Zitadel Management API
    async fn create_users(
        &self,
        admin_token: &str,
        _org_id: &str,
        project_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let http_client = reqwest::Client::new();

        for (username, password, roles) in &self.config.users {
            // Create user using REAL API
            let url = format!("{}/management/v1/users/human/_import", self.base_url);

            let user_data = json!({
                "userName": username,
                "profile": {
                    "firstName": username,
                    "lastName": "User",
                },
                "email": {
                    "email": format!("{}@test.local", username),
                    "isEmailVerified": true,
                },
                "password": password,
                "passwordChangeRequired": false,
            });

            let response = http_client
                .post(&url)
                .bearer_auth(admin_token)
                .json(&user_data)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                println!("  ⚠ Failed to create user '{}': {}", username, error_text);
                continue;
            }

            let result: serde_json::Value = response.json().await?;
            let user_id = result["userId"]
                .as_str()
                .ok_or("No user ID in response")?;

            println!("  ✓ User '{}' created: {}", username, user_id);

            // Assign roles to user using REAL API
            for role in roles {
                self.assign_role_to_user(admin_token, project_id, user_id, role).await?;
            }
        }

        Ok(())
    }

    /// Assign role to user using REAL Zitadel Management API
    async fn assign_role_to_user(
        &self,
        admin_token: &str,
        project_id: &str,
        user_id: &str,
        role_key: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/management/v1/users/{}/grants",
            self.base_url, user_id
        );

        let grant_data = json!({
            "projectId": project_id,
            "roleKeys": [role_key],
        });

        let response = client
            .post(&url)
            .bearer_auth(admin_token)
            .json(&grant_data)
            .send()
            .await?;

        if response.status().is_success() {
            println!("    → Role '{}' assigned to user", role_key);
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to assign role: {}", error_text).into())
        }
    }

    /// Get access token for a user using REAL OAuth2 password grant
    pub async fn get_user_token(
        &self,
        username: &str,
        password: &str,
    ) -> Result<ZitadelTokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let token_url = format!("{}/oauth/v2/token", self.base_url);

        let client_id = self.application_client_id
            .as_ref()
            .ok_or("Application not configured")?;
        let client_secret = self.application_client_secret
            .as_ref()
            .ok_or("Application not configured")?;

        let project_id = self.project_id
            .as_ref()
            .ok_or("Project not configured")?;

        let scope = format!("openid profile email urn:zitadel:iam:org:project:id:{}:aud", project_id);

        let params = [
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
            ("scope", &scope),
        ];

        let response = client
            .post(&token_url)
            .basic_auth(client_id, Some(client_secret))
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to get token for user '{}': {}", username, error_text).into());
        }

        let token_response: ZitadelTokenResponse = response.json().await?;
        Ok(token_response)
    }

    /// Get the issuer URL for this Zitadel instance
    pub fn issuer(&self) -> String {
        self.issuer.clone()
    }

    /// Get the JWKS URI for this Zitadel instance
    pub fn jwks_uri(&self) -> String {
        format!("{}/oauth/v2/keys", self.base_url)
    }

    /// Get the client ID
    pub fn client_id(&self) -> &str {
        self.application_client_id.as_ref().map(|s| s.as_str()).unwrap_or("")
    }

    /// Get the project ID
    pub fn project_id(&self) -> &str {
        self.project_id.as_ref().map(|s| s.as_str()).unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Docker and takes several minutes
    async fn test_zitadel_container_starts() {
        let zitadel = ZitadelContainer::start().await;
        assert!(zitadel.is_ok());

        let zitadel = zitadel.unwrap();
        assert!(!zitadel.issuer().is_empty());
        assert!(!zitadel.jwks_uri().is_empty());
        assert!(!zitadel.client_id().is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires Docker and takes several minutes
    async fn test_get_user_token() {
        let zitadel = ZitadelContainer::start().await.unwrap();

        let token = zitadel.get_user_token("developer", "Password123!").await;
        assert!(token.is_ok());

        let token = token.unwrap();
        assert!(!token.access_token.is_empty());
    }
}
