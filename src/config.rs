//! Configuration module for the application

use serde::{Deserialize, Serialize};
use std::env;

/// Database provider type
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseProvider {
    /// SQLite database (embedded)
    Sqlite,
    /// PostgreSQL database (server)
    Postgres,
    /// SurrealDB database (multi-model)
    Surreal,
}

impl Default for DatabaseProvider {
    fn default() -> Self {
        DatabaseProvider::Sqlite
    }
}

/// Database configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// Database provider type
    pub provider: DatabaseProvider,
    
    /// Database connection URL
    pub url: String,
    
    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_max_connections() -> u32 {
    10
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Server host
    #[serde(default = "default_host")]
    pub host: String,
    
    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    50051
}

/// Cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    /// Whether cache is enabled
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    
    /// Cache reload interval in seconds
    #[serde(default = "default_reload_interval")]
    pub reload_interval_secs: u64,
}

fn default_cache_enabled() -> bool {
    true
}

fn default_reload_interval() -> u64 {
    300 // 5 minutes
}

/// Application configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
    
    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            reload_interval_secs: default_reload_interval(),
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    /// 
    /// Environment variables:
    /// - `DATABASE_PROVIDER`: sqlite, postgres, or surreal (default: sqlite)
    /// - `DATABASE_URL`: Connection URL (default: sqlite::memory:)
    /// - `DATABASE_MAX_CONNECTIONS`: Max connections (default: 10)
    /// - `SERVER_HOST`: Server host (default: 0.0.0.0)
    /// - `SERVER_PORT`: Server port (default: 50051)
    /// - `CACHE_ENABLED`: Enable cache (default: true)
    /// - `CACHE_RELOAD_INTERVAL_SECS`: Reload interval (default: 300)
    pub fn from_env() -> Self {
        let database_provider = env::var("DATABASE_PROVIDER")
            .unwrap_or_else(|_| "sqlite".to_string())
            .to_lowercase();
        
        let provider = match database_provider.as_str() {
            "postgres" | "postgresql" => DatabaseProvider::Postgres,
            "surreal" | "surrealdb" => DatabaseProvider::Surreal,
            _ => DatabaseProvider::Sqlite,
        };
        
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            match provider {
                DatabaseProvider::Sqlite => "sqlite::memory:".to_string(),
                DatabaseProvider::Postgres => "postgresql://localhost:5432/hodei".to_string(),
                DatabaseProvider::Surreal => "ws://localhost:8000".to_string(),
            }
        });
        
        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        
        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());
        
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(50051);
        
        let cache_enabled = env::var("CACHE_ENABLED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);
        
        let cache_reload_interval = env::var("CACHE_RELOAD_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);
        
        Self {
            database: DatabaseConfig {
                provider,
                url: database_url,
                max_connections,
            },
            server: ServerConfig {
                host: server_host,
                port: server_port,
            },
            cache: CacheConfig {
                enabled: cache_enabled,
                reload_interval_secs: cache_reload_interval,
            },
        }
    }
    
    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        // Ensure no DATABASE_PROVIDER env var is set for this test
        unsafe {
            std::env::remove_var("DATABASE_PROVIDER");
        }
        let config = Config::from_env();
        assert_eq!(config.database.provider, DatabaseProvider::Sqlite);
        assert_eq!(config.server.port, 50051);
        assert!(config.cache.enabled);
    }
    
    #[test]
    fn test_database_provider_parsing() {
        unsafe {
            std::env::set_var("DATABASE_PROVIDER", "postgres");
        }
        let config = Config::from_env();
        assert_eq!(config.database.provider, DatabaseProvider::Postgres);
        unsafe {
            std::env::remove_var("DATABASE_PROVIDER");
        }
    }
}
