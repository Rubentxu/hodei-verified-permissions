//! JWKS Cache with auto-refresh and TTL

use crate::error::{AuthorizationError, Result};
use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// OIDC Discovery document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcDiscovery {
    pub issuer: String,
    pub jwks_uri: String,
    #[serde(default)]
    pub authorization_endpoint: Option<String>,
    #[serde(default)]
    pub token_endpoint: Option<String>,
}

/// JWKS response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

/// JSON Web Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    pub kty: String,
    pub kid: String,
    #[serde(rename = "use")]
    pub key_use: Option<String>,
    pub alg: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
}

/// Cached JWKS entry with metadata
#[derive(Debug, Clone)]
struct CachedJwks {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Instant,
    jwks_uri: String,
}

/// Configuration for JWKS cache
#[derive(Debug, Clone)]
pub struct JwksCacheConfig {
    /// Time-to-live for cached keys (default: 1 hour)
    pub ttl: Duration,
    /// Refresh interval for background updates (default: 30 minutes)
    pub refresh_interval: Duration,
    /// HTTP request timeout (default: 10 seconds)
    pub request_timeout: Duration,
}

impl Default for JwksCacheConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(3600),           // 1 hour
            refresh_interval: Duration::from_secs(1800), // 30 minutes
            request_timeout: Duration::from_secs(10),
        }
    }
}

/// JWKS Cache with auto-discovery and refresh
pub struct JwksCache {
    /// HTTP client for fetching JWKS
    client: reqwest::Client,
    /// Cache of JWKS by issuer
    cache: Arc<RwLock<HashMap<String, CachedJwks>>>,
    /// Configuration
    config: JwksCacheConfig,
    /// Metrics
    metrics: Arc<RwLock<CacheMetrics>>,
}

/// Cache metrics for monitoring
#[derive(Debug, Default, Clone)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub refreshes: u64,
    pub errors: u64,
}

impl CacheMetrics {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl JwksCache {
    /// Create a new JWKS cache with default configuration
    pub fn new() -> Self {
        Self::with_config(JwksCacheConfig::default())
    }

    /// Create a new JWKS cache with custom configuration
    pub fn with_config(config: JwksCacheConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.request_timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    /// Get a decoding key for a specific key ID and issuer
    pub async fn get_key(&self, kid: &str, issuer: &str) -> Result<DecodingKey> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(issuer) {
                // Check if cache is still valid
                if cached.fetched_at.elapsed() < self.config.ttl {
                    if let Some(key) = cached.keys.get(kid) {
                        debug!("JWKS cache hit for issuer: {}, kid: {}", issuer, kid);
                        let mut metrics = self.metrics.write().await;
                        metrics.hits += 1;
                        return Ok(key.clone());
                    }
                }
            }
        }

        debug!("JWKS cache miss for issuer: {}, kid: {}", issuer, kid);
        let mut metrics = self.metrics.write().await;
        metrics.misses += 1;
        drop(metrics);

        // Cache miss or expired - fetch and update
        self.refresh_keys(issuer).await?;

        // Try again from cache
        let cache = self.cache.read().await;
        cache
            .get(issuer)
            .and_then(|cached| cached.keys.get(kid))
            .cloned()
            .ok_or_else(|| {
                AuthorizationError::Internal(format!(
                    "Key with kid '{}' not found for issuer '{}'",
                    kid, issuer
                ))
            })
    }

    /// Discover JWKS URI from OIDC issuer
    pub async fn discover_jwks_uri(&self, issuer: &str) -> Result<String> {
        let discovery_url = format!("{}/.well-known/openid-configuration", issuer.trim_end_matches('/'));
        
        debug!("Discovering OIDC configuration from: {}", discovery_url);

        let response = self
            .client
            .get(&discovery_url)
            .send()
            .await
            .map_err(|e| {
                AuthorizationError::Internal(format!(
                    "Failed to fetch OIDC discovery document: {}",
                    e
                ))
            })?;

        if !response.status().is_success() {
            return Err(AuthorizationError::Internal(format!(
                "OIDC discovery failed with status: {}",
                response.status()
            )));
        }

        let discovery: OidcDiscovery = response.json().await.map_err(|e| {
            AuthorizationError::Internal(format!("Failed to parse OIDC discovery document: {}", e))
        })?;

        info!("Discovered JWKS URI: {} for issuer: {}", discovery.jwks_uri, issuer);
        Ok(discovery.jwks_uri)
    }

    /// Refresh keys for a specific issuer
    pub async fn refresh_keys(&self, issuer: &str) -> Result<()> {
        info!("Refreshing JWKS for issuer: {}", issuer);

        // Discover JWKS URI
        let jwks_uri = self.discover_jwks_uri(issuer).await?;

        // Fetch JWKS
        let jwks = self.fetch_jwks(&jwks_uri).await?;

        // Convert to decoding keys
        let mut keys = HashMap::new();
        for jwk in jwks.keys {
            if let (Some(n), Some(e)) = (jwk.n, jwk.e) {
                match DecodingKey::from_rsa_components(&n, &e) {
                    Ok(key) => {
                        keys.insert(jwk.kid.clone(), key);
                        debug!("Cached key: {} for issuer: {}", jwk.kid, issuer);
                    }
                    Err(e) => {
                        warn!("Failed to create decoding key for kid {}: {}", jwk.kid, e);
                        let mut metrics = self.metrics.write().await;
                        metrics.errors += 1;
                    }
                }
            }
        }

        // Update cache
        let cached = CachedJwks {
            keys,
            fetched_at: Instant::now(),
            jwks_uri,
        };

        let mut cache = self.cache.write().await;
        cache.insert(issuer.to_string(), cached);

        let mut metrics = self.metrics.write().await;
        metrics.refreshes += 1;

        info!("Successfully refreshed {} keys for issuer: {}", cache.get(issuer).map(|c| c.keys.len()).unwrap_or(0), issuer);
        Ok(())
    }

    /// Fetch JWKS from endpoint
    async fn fetch_jwks(&self, jwks_uri: &str) -> Result<Jwks> {
        debug!("Fetching JWKS from: {}", jwks_uri);

        let response = self
            .client
            .get(jwks_uri)
            .send()
            .await
            .map_err(|e| {
                AuthorizationError::Internal(format!("Failed to fetch JWKS: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(AuthorizationError::Internal(format!(
                "JWKS fetch failed with status: {}",
                response.status()
            )));
        }

        let jwks = response.json::<Jwks>().await.map_err(|e| {
            AuthorizationError::Internal(format!("Failed to parse JWKS: {}", e))
        })?;

        Ok(jwks)
    }

    /// Start background refresh task for all cached issuers
    pub fn start_background_refresh(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let interval = self.config.refresh_interval;
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            
            loop {
                ticker.tick().await;
                
                let issuers: Vec<String> = {
                    let cache = self.cache.read().await;
                    cache.keys().cloned().collect()
                };

                for issuer in issuers {
                    if let Err(e) = self.refresh_keys(&issuer).await {
                        warn!("Background refresh failed for issuer {}: {}", issuer, e);
                        let mut metrics = self.metrics.write().await;
                        metrics.errors += 1;
                    }
                }
            }
        })
    }

    /// Get current cache metrics
    pub async fn metrics(&self) -> CacheMetrics {
        self.metrics.read().await.clone()
    }

    /// Clear the cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("JWKS cache cleared");
    }

    /// Get number of cached issuers
    pub async fn cached_issuers_count(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

impl Default for JwksCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_defaults() {
        let config = JwksCacheConfig::default();
        assert_eq!(config.ttl, Duration::from_secs(3600));
        assert_eq!(config.refresh_interval, Duration::from_secs(1800));
        assert_eq!(config.request_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_cache_metrics() {
        let mut metrics = CacheMetrics::default();
        assert_eq!(metrics.hit_rate(), 0.0);

        metrics.hits = 8;
        metrics.misses = 2;
        assert_eq!(metrics.hit_rate(), 0.8);
    }

    #[tokio::test]
    async fn test_jwks_cache_creation() {
        let cache = JwksCache::new();
        assert_eq!(cache.cached_issuers_count().await, 0);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = JwksCache::new();
        cache.clear().await;
        assert_eq!(cache.cached_issuers_count().await, 0);
    }

    #[tokio::test]
    async fn test_metrics_tracking() {
        let cache = JwksCache::new();
        let metrics = cache.metrics().await;
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.misses, 0);
    }
}
