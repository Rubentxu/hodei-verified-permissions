//! Authorization client - Data Plane only
//!
//! This module provides a lightweight client focused exclusively on authorization
//! checks (Data Plane). For policy and schema management (Control Plane), use
//! the CLI tool or HodeiAdmin library.

use crate::error::{Result, SdkError};
use crate::proto::authorization_data_client::AuthorizationDataClient;
use crate::proto::*;
use tonic::transport::Channel;

/// Main client for Hodei Verified Permissions (Data Plane only)
///
/// This client provides authorization checking capabilities only.
/// For policy store, schema, and policy management, use the CLI tool or HodeiAdmin library.
///
/// # Example
///
/// ```no_run
/// use verified_permissions_sdk::AuthorizationClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = AuthorizationClient::connect("http://localhost:50051").await?;
///
///     let response = client
///         .is_authorized(
///             "policy-store-id",
///             "User::alice",
///             "Action::view",
///             "Document::doc123"
///         )
///         .await?;
///
///     println!("Decision: {:?}", response.decision());
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct AuthorizationClient {
    data_client: AuthorizationDataClient<Channel>,
}

impl AuthorizationClient {
    /// Connect to the authorization service
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to connect to (e.g., "http://localhost:50051")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use verified_permissions_sdk::AuthorizationClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AuthorizationClient::connect("http://localhost:50051").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(addr: impl Into<String>) -> Result<Self> {
        let addr = addr.into();
        let channel = Channel::from_shared(addr.clone())
            .map_err(|e| SdkError::ConnectionError(e.to_string()))?
            .connect()
            .await?;

        Ok(Self {
            data_client: AuthorizationDataClient::new(channel),
        })
    }

    // ========================================================================
    // Data Plane - Authorization
    // ========================================================================

    /// Check if an action is authorized
    ///
    /// This is the core authorization check operation. Given a principal, action,
    /// and resource, it returns whether the action is allowed or denied.
    ///
    /// # Arguments
    ///
    /// * `policy_store_id` - The ID of the policy store to evaluate against
    /// * `principal` - The principal (user, service, etc.) making the request (format: "Type::id")
    /// * `action` - The action being performed (format: "Type::id")
    /// * `resource` - The resource being accessed (format: "Type::id")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use verified_permissions_sdk::AuthorizationClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AuthorizationClient::connect("http://localhost:50051").await?;
    ///
    /// let response = client
    ///     .is_authorized(
    ///         "policy-store-id",
    ///         "User::alice",
    ///         "Action::view",
    ///         "Document::doc123"
    ///     )
    ///     .await?;
    ///
    /// match response.decision() {
    ///     Decision::Allow => println!("Access granted"),
    ///     Decision::Deny => println!("Access denied"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_authorized(
        &self,
        policy_store_id: impl Into<String>,
        principal: impl Into<String>,
        action: impl Into<String>,
        resource: impl Into<String>,
    ) -> Result<IsAuthorizedResponse> {
        let request = IsAuthorizedRequest {
            policy_store_id: policy_store_id.into(),
            principal: Some(parse_entity_id(principal.into())?),
            action: Some(parse_entity_id(action.into())?),
            resource: Some(parse_entity_id(resource.into())?),
            context: None,
            entities: vec![],
        };

        let response = self
            .data_client
            .clone()
            .is_authorized(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Check authorization with entities and context
    ///
    /// This method allows for more complex authorization checks by providing
    /// additional context and entity data that can be used in Cedar policies.
    ///
    /// # Arguments
    ///
    /// * `request` - A pre-built `IsAuthorizedRequest` with entities and context
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use verified_permissions_sdk::{AuthorizationClient, IsAuthorizedRequestBuilder};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AuthorizationClient::connect("http://localhost:50051").await?;
    ///
    /// let request = IsAuthorizedRequestBuilder::new("policy-store-id")
    ///     .principal("User", "alice")
    ///     .action("Action", "view")
    ///     .resource("Document", "doc123")
    ///     .context(r#"{"ip": "192.168.1.1"}"#)
    ///     .build();
    ///
    /// let response = client.is_authorized_with_context(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_authorized_with_context(
        &self,
        request: IsAuthorizedRequest,
    ) -> Result<IsAuthorizedResponse> {
        let response = self
            .data_client
            .clone()
            .is_authorized(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Batch authorization check
    ///
    /// Perform multiple authorization checks in a single request.
    /// This is more efficient than making individual calls when you need to
    /// check multiple permissions.
    ///
    /// # Arguments
    ///
    /// * `policy_store_id` - The ID of the policy store to evaluate against
    /// * `requests` - A vector of `IsAuthorizedRequest` to evaluate
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use verified_permissions_sdk::{AuthorizationClient, IsAuthorizedRequestBuilder};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AuthorizationClient::connect("http://localhost:50051").await?;
    ///
    /// let requests = vec![
    ///     IsAuthorizedRequestBuilder::new("policy-store-id")
    ///         .principal("User", "alice")
    ///         .action("Action", "view")
    ///         .resource("Document", "doc1")
    ///         .build(),
    ///     IsAuthorizedRequestBuilder::new("policy-store-id")
    ///         .principal("User", "alice")
    ///         .action("Action", "edit")
    ///         .resource("Document", "doc1")
    ///         .build(),
    /// ];
    ///
    /// let responses = client.batch_is_authorized("policy-store-id", requests).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn batch_is_authorized(
        &self,
        policy_store_id: impl Into<String>,
        requests: Vec<IsAuthorizedRequest>,
    ) -> Result<BatchIsAuthorizedResponse> {
        let request = BatchIsAuthorizedRequest {
            policy_store_id: policy_store_id.into(),
            requests,
        };

        let response = self
            .data_client
            .clone()
            .batch_is_authorized(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Check authorization with JWT token
    ///
    /// This method validates a JWT token against an identity source and performs
    /// authorization using the token's claims.
    ///
    /// # Arguments
    ///
    /// * `policy_store_id` - The ID of the policy store to evaluate against
    /// * `identity_source_id` - The ID of the identity source for token validation
    /// * `access_token` - The JWT access token
    /// * `action` - The action being performed (format: "Type::id")
    /// * `resource` - The resource being accessed (format: "Type::id")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use verified_permissions_sdk::AuthorizationClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AuthorizationClient::connect("http://localhost:50051").await?;
    ///
    /// let jwt_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...";
    ///
    /// let response = client
    ///     .is_authorized_with_token(
    ///         "policy-store-id",
    ///         "identity-source-id",
    ///         jwt_token,
    ///         "Action::view",
    ///         "Document::doc123"
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_authorized_with_token(
        &self,
        policy_store_id: impl Into<String>,
        identity_source_id: impl Into<String>,
        access_token: impl Into<String>,
        action: impl Into<String>,
        resource: impl Into<String>,
    ) -> Result<IsAuthorizedResponse> {
        let request = IsAuthorizedWithTokenRequest {
            policy_store_id: policy_store_id.into(),
            identity_source_id: identity_source_id.into(),
            access_token: access_token.into(),
            action: Some(parse_entity_id(action.into())?),
            resource: Some(parse_entity_id(resource.into())?),
            context: None,
            entities: vec![],
        };

        let response = self
            .data_client
            .clone()
            .is_authorized_with_token(request)
            .await?
            .into_inner();

        Ok(response)
    }

    /// Check authorization with JWT token and context
    ///
    /// This method validates a JWT token and performs authorization with additional
    /// context and entity data.
    ///
    /// # Arguments
    ///
    /// * `request` - A pre-built `IsAuthorizedWithTokenRequest` with entities and context
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use verified_permissions_sdk::{AuthorizationClient, IsAuthorizedWithTokenRequestBuilder};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AuthorizationClient::connect("http://localhost:50051").await?;
    ///
    /// let jwt_token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...";
    ///
    /// let request = IsAuthorizedWithTokenRequestBuilder::new(
    ///     "policy-store-id",
    ///     "identity-source-id",
    ///     jwt_token
    /// )
    ///     .action("Action", "view")
    ///     .resource("Document", "doc123")
    ///     .context(r#"{"ip": "192.168.1.1"}"#)
    ///     .build();
    ///
    /// let response = client.is_authorized_with_token_and_context(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_authorized_with_token_and_context(
        &self,
        request: IsAuthorizedWithTokenRequest,
    ) -> Result<IsAuthorizedResponse> {
        let response = self
            .data_client
            .clone()
            .is_authorized_with_token(request)
            .await?
            .into_inner();

        Ok(response)
    }
}

/// Parse entity ID from string format "Type::id"
///
/// # Arguments
///
/// * `s` - String in format "Type::id" (e.g., "User::alice")
///
/// # Errors
///
/// Returns `SdkError::InvalidRequest` if the format is invalid
///
/// # Example
///
/// ```
/// # use verified_permissions_sdk::client::parse_entity_id;
/// let entity = parse_entity_id("User::alice".to_string()).unwrap();
/// assert_eq!(entity.entity_type, "User");
/// assert_eq!(entity.entity_id, "alice");
/// ```
fn parse_entity_id(s: String) -> Result<EntityIdentifier> {
    let parts: Vec<&str> = s.split("::").collect();
    if parts.len() != 2 {
        return Err(SdkError::InvalidRequest(format!(
            "Invalid entity format: {}. Expected 'Type::id'",
            s
        )));
    }

    Ok(EntityIdentifier {
        entity_type: parts[0].trim_matches('"').to_string(),
        entity_id: parts[1].trim_matches('"').to_string(),
    })
}

// Implement AuthorizationClientTrait for AuthorizationClient
#[async_trait::async_trait]
impl crate::client_trait::AuthorizationClientTrait for AuthorizationClient {
    /// Check if an action is authorized
    async fn is_authorized(
        &self,
        policy_store_id: &str,
        principal: &str,
        action: &str,
        resource: &str,
    ) -> Result<IsAuthorizedResponse> {
        AuthorizationClient::is_authorized(self, policy_store_id, principal, action, resource).await
    }

    /// Check authorization with entities and context
    async fn is_authorized_with_context(
        &self,
        request: IsAuthorizedRequest,
    ) -> Result<IsAuthorizedResponse> {
        AuthorizationClient::is_authorized_with_context(self, request).await
    }

    /// Batch authorization check
    async fn batch_is_authorized(
        &self,
        policy_store_id: &str,
        requests: Vec<IsAuthorizedRequest>,
    ) -> Result<BatchIsAuthorizedResponse> {
        AuthorizationClient::batch_is_authorized(self, policy_store_id, requests).await
    }

    /// Check authorization with JWT token
    async fn is_authorized_with_token(
        &self,
        policy_store_id: &str,
        identity_source_id: &str,
        access_token: &str,
        action: &str,
        resource: &str,
    ) -> Result<IsAuthorizedResponse> {
        AuthorizationClient::is_authorized_with_token(
            self,
            policy_store_id,
            identity_source_id,
            access_token,
            action,
            resource,
        )
        .await
    }

    /// Check authorization with JWT token and context
    async fn is_authorized_with_token_and_context(
        &self,
        request: IsAuthorizedWithTokenRequest,
    ) -> Result<IsAuthorizedResponse> {
        AuthorizationClient::is_authorized_with_token_and_context(self, request).await
    }
}
