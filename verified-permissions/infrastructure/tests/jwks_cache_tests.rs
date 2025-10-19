//! Integration tests for JWKS Cache

use hodei_verified_permissions::jwt::{JwksCache, JwksCacheConfig};
use std::time::Duration;

#[tokio::test]
async fn test_jwks_cache_creation() {
    let cache = JwksCache::new();
    assert_eq!(cache.cached_issuers_count().await, 0);
}

#[tokio::test]
async fn test_jwks_cache_with_custom_config() {
    let config = JwksCacheConfig {
        ttl: Duration::from_secs(600),
        refresh_interval: Duration::from_secs(300),
        request_timeout: Duration::from_secs(5),
    };
    
    let cache = JwksCache::with_config(config);
    assert_eq!(cache.cached_issuers_count().await, 0);
}

#[tokio::test]
async fn test_cache_metrics_initial_state() {
    let cache = JwksCache::new();
    let metrics = cache.metrics().await;
    
    assert_eq!(metrics.hits, 0);
    assert_eq!(metrics.misses, 0);
    assert_eq!(metrics.refreshes, 0);
    assert_eq!(metrics.errors, 0);
    assert_eq!(metrics.hit_rate(), 0.0);
}

#[tokio::test]
async fn test_cache_clear() {
    let cache = JwksCache::new();
    cache.clear().await;
    assert_eq!(cache.cached_issuers_count().await, 0);
    
    let metrics = cache.metrics().await;
    assert_eq!(metrics.hits, 0);
}

#[tokio::test]
#[ignore] // Requires real OIDC provider
async fn test_discover_jwks_uri_with_real_provider() {
    let cache = JwksCache::new();
    
    // Example with Google's OIDC provider
    let issuer = "https://accounts.google.com";
    let result = cache.discover_jwks_uri(issuer).await;
    
    assert!(result.is_ok());
    let jwks_uri = result.unwrap();
    assert!(jwks_uri.contains("googleapis.com"));
    assert!(jwks_uri.contains("jwks"));
}

#[tokio::test]
async fn test_discover_jwks_uri_invalid_issuer() {
    let cache = JwksCache::new();
    
    let issuer = "https://invalid-issuer-that-does-not-exist.example.com";
    let result = cache.discover_jwks_uri(issuer).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_key_with_invalid_issuer() {
    let cache = JwksCache::new();
    
    let result = cache.get_key("test-kid", "https://invalid.example.com").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_metrics_hit_rate_calculation() {
    use hodei_verified_permissions::jwt::CacheMetrics;
    
    let mut metrics = CacheMetrics::default();
    assert_eq!(metrics.hit_rate(), 0.0);
    
    metrics.hits = 7;
    metrics.misses = 3;
    assert_eq!(metrics.hit_rate(), 0.7);
    
    metrics.hits = 10;
    metrics.misses = 0;
    assert_eq!(metrics.hit_rate(), 1.0);
}

#[tokio::test]
#[ignore] // Requires real OIDC provider
async fn test_full_flow_with_google() {
    let cache = JwksCache::new();
    let issuer = "https://accounts.google.com";
    
    // First call should miss and fetch
    let result = cache.get_key("test-kid", issuer).await;
    // Will fail because test-kid doesn't exist, but should have attempted discovery
    assert!(result.is_err());
    
    let metrics = cache.metrics().await;
    assert!(metrics.misses > 0);
}
