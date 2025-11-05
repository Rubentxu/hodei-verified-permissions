//! Configuration management for Hodei Verified Permissions
//!
//! This module provides a comprehensive configuration system following
//! SOLID principles and hexagonal architecture patterns.
//!
//! Configuration can be loaded from:
//! - Environment variables
//! - .env files
//! - Default values
//!
//! # Architecture
//! - `ConfigSource` trait: Abstract interface for configuration sources
//! - `ConfigLoader`: Main configuration loader
//! - `Settings`: Complete application configuration struct

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;

/// Configuration trait for dependency injection
pub trait Configuration: Send + Sync {
    fn server_port(&self) -> u16;
    fn server_host(&self) -> &str;
    fn database_url(&self) -> &str;
    fn log_level(&self) -> &str;
    fn grpc_max_frame_size(&self) -> Option<usize>;
    fn grpc_keepalive_time(&self) -> Option<u64>;
    fn shutdown_timeout(&self) -> u64;
    fn tls_enabled(&self) -> bool;
    fn tls_cert_path(&self) -> Option<&str>;
    fn tls_key_path(&self) -> Option<&str>;
    fn tls_client_ca_path(&self) -> Option<&str>;
}

/// Configuration source trait (Hexagonal Architecture - Port)
/// This trait abstracts where configuration comes from
pub trait ConfigSource: Send + Sync {
    /// Get a configuration value by key
    fn get(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>;

    /// Get all configuration values as a hashmap
    fn get_all(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Environment variables configuration source (Adapter)
pub struct EnvConfigSource {
    prefix: Option<String>,
}

impl EnvConfigSource {
    pub fn new() -> Self {
        Self { prefix: None }
    }

    pub fn with_prefix(prefix: &str) -> Self {
        Self {
            prefix: Some(prefix.to_string()),
        }
    }

    fn normalize_key(&self, key: &str) -> String {
        if let Some(prefix) = &self.prefix {
            format!("{}_{}", prefix.to_uppercase(), key.to_uppercase())
        } else {
            key.to_uppercase()
        }
    }
}

impl Default for EnvConfigSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigSource for EnvConfigSource {
    fn get(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let normalized_key = self.normalize_key(key);
        match env::var(&normalized_key) {
            Ok(value) => Ok(Some(value)),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn get_all(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut config = HashMap::new();
        for (key, value) in env::vars() {
            if let Some(prefix) = &self.prefix {
                if key.starts_with(&format!("{}_", prefix.to_uppercase())) {
                    config.insert(key, value);
                }
            } else {
                config.insert(key, value);
            }
        }
        Ok(config)
    }
}

/// .env file configuration source (Adapter)
pub struct DotEnvConfigSource {
    path: std::path::PathBuf,
}

impl DotEnvConfigSource {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn default_path() -> Self {
        let current_dir = env::current_dir().unwrap_or_else(|_| ".".into());
        Self::new(current_dir.join(".env"))
    }
}

impl ConfigSource for DotEnvConfigSource {
    fn get(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        match fs::read_to_string(&self.path) {
            Ok(content) => {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    if let Some((k, v)) = line.split_once('=') {
                        let k = k.trim();
                        let v = v.trim().trim_matches('"').trim_matches('\'');

                        if k == key {
                            return Ok(Some(v.to_string()));
                        }
                    }
                }
                Ok(None)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn get_all(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut config = HashMap::new();
        let content = fs::read_to_string(&self.path)?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim();
                let v = v.trim().trim_matches('"').trim_matches('\'');
                config.insert(k.to_string(), v.to_string());
            }
        }

        Ok(config)
    }
}

/// Main configuration loader following the Strategy pattern
pub struct ConfigLoader {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// Add a configuration source (e.g., .env file)
    pub fn with_source(mut self, source: Box<dyn ConfigSource>) -> Self {
        self.sources.push(source);
        self
    }

    /// Add environment variables as a source
    pub fn with_env(self) -> Self {
        self.with_source(Box::new(EnvConfigSource::new()))
    }

    /// Add environment variables with a prefix
    pub fn with_env_prefix(self, prefix: &str) -> Self {
        self.with_source(Box::new(EnvConfigSource::with_prefix(prefix)))
    }

    /// Add .env file
    pub fn with_dotenv<P: AsRef<Path>>(self, path: P) -> Self {
        self.with_source(Box::new(DotEnvConfigSource::new(path)))
    }

    /// Add default .env file in current directory
    pub fn with_default_dotenv(self) -> Self {
        self.with_source(Box::new(DotEnvConfigSource::default_path()))
    }

    /// Load configuration from all sources
    pub fn load(&self) -> Result<Settings, Box<dyn std::error::Error + Send + Sync>> {
        let mut settings = Settings::default();

        // Load from all sources (first source has priority)
        for source in &self.sources {
            let config = source.get_all()?;

            // Server configuration
            if let Some(val) = config.get("SERVER_PORT") {
                settings.server.port = val.parse().unwrap_or(settings.server.port);
            }
            if let Some(val) = config.get("SERVER_HOST") {
                settings.server.host = val.clone();
            }

            // Database configuration
            if let Some(val) = config.get("DATABASE_URL") {
                settings.database.url = val.clone();
            }

            // Logging configuration
            if let Some(val) = config.get("LOG_LEVEL") {
                settings.log_level = val.clone();
            }

            // gRPC configuration
            if let Some(val) = config.get("GRPC_MAX_FRAME_SIZE") {
                settings.grpc.max_frame_size = Some(val.parse().unwrap_or(4 * 1024 * 1024));
            }
            if let Some(val) = config.get("GRPC_KEEPALIVE_TIME") {
                settings.grpc.keepalive_time = Some(val.parse().unwrap_or(30));
            }

            // Shutdown configuration
            if let Some(val) = config.get("SHUTDOWN_TIMEOUT") {
                settings.shutdown_timeout = val.parse().unwrap_or(30);
            }

            // TLS configuration
            if let Some(val) = config.get("TLS_ENABLED") {
                settings.server.tls.enabled = val.parse().unwrap_or(false);
            }
            if let Some(val) = config.get("TLS_CERT_PATH") {
                settings.server.tls.cert_path = Some(val.clone());
            }
            if let Some(val) = config.get("TLS_KEY_PATH") {
                settings.server.tls.key_path = Some(val.clone());
            }
            if let Some(val) = config.get("TLS_CLIENT_CA_PATH") {
                settings.server.tls.client_ca_path = Some(val.clone());
            }
        }

        Ok(settings)
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new().with_default_dotenv().with_env()
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls: TlsConfig,
}

#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub client_ca_path: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
            tls: TlsConfig {
                enabled: false,
                cert_path: None,
                key_path: None,
                client_ca_path: None,
            },
        }
    }
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:///home/rubentxu/hodei-data/hodei.db".to_string(),
        }
    }
}

/// gRPC configuration
#[derive(Debug, Clone)]
pub struct GrpcConfig {
    pub max_frame_size: Option<usize>,
    pub keepalive_time: Option<u64>,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            max_frame_size: Some(4 * 1024 * 1024), // 4MB
            keepalive_time: Some(30),              // 30 seconds
        }
    }
}

/// Complete application settings
#[derive(Debug, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub grpc: GrpcConfig,
    pub log_level: String,
    pub shutdown_timeout: u64,
}

impl Settings {
    /// Create new settings with custom configuration loader
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        ConfigLoader::default().load()
    }

    /// Create new settings with custom sources
    pub fn with_sources<F>(configure: F) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(ConfigLoader) -> ConfigLoader,
    {
        let loader = configure(ConfigLoader::default());
        loader.load()
    }

    /// Get server address as string
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            grpc: GrpcConfig::default(),
            log_level: "info".to_string(),
            shutdown_timeout: 30,
        }
    }
}

/// Settings implementation of Configuration trait
impl Configuration for Settings {
    fn server_port(&self) -> u16 {
        self.server.port
    }

    fn server_host(&self) -> &str {
        &self.server.host
    }

    fn database_url(&self) -> &str {
        &self.database.url
    }

    fn log_level(&self) -> &str {
        &self.log_level
    }

    fn grpc_max_frame_size(&self) -> Option<usize> {
        self.grpc.max_frame_size
    }

    fn grpc_keepalive_time(&self) -> Option<u64> {
        self.grpc.keepalive_time
    }

    fn shutdown_timeout(&self) -> u64 {
        self.shutdown_timeout
    }

    fn tls_enabled(&self) -> bool {
        self.server.tls.enabled
    }

    fn tls_cert_path(&self) -> Option<&str> {
        self.server.tls.cert_path.as_deref()
    }

    fn tls_key_path(&self) -> Option<&str> {
        self.server.tls.key_path.as_deref()
    }

    fn tls_client_ca_path(&self) -> Option<&str> {
        self.server.tls.client_ca_path.as_deref()
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Server: {}:{} (TLS: {})\nDatabase: {}\nLog Level: {}\nShutdown Timeout: {}s",
            self.server.host,
            self.server.port,
            if self.server.tls.enabled {
                "enabled"
            } else {
                "disabled"
            },
            self.database.url,
            self.log_level,
            self.shutdown_timeout
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_display() {
        let settings = Settings::default();
        let output = format!("{}", settings);
        assert!(output.contains("Server:"));
        assert!(output.contains("Database:"));
        assert!(output.contains("Log Level:"));
    }
}
