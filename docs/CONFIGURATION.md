# Configuration System Documentation

## Overview

The Hodei Verified Permissions Server implements a comprehensive configuration management system following **SOLID principles** and **Hexagonal Architecture** patterns.

## Architecture

### Components

1. **`Configuration` Trait** - Defines the contract for accessing configuration values
2. **`ConfigSource` Trait** - Abstract interface for configuration sources (Port)
3. **`ConfigLoader` Strategy** - Loads and merges configuration from multiple sources
4. **`Settings` Struct** - Complete application configuration (Entity)

### Design Patterns Applied

- **Dependency Inversion Principle (DIP)**: Configuration depends on abstractions, not concretions
- **Strategy Pattern**: `ConfigLoader` can use different loading strategies
- **Adapter Pattern**: Each configuration source is an adapter (`.env`, environment variables)
- **Builder Pattern**: `ConfigLoader` uses builder API for flexible configuration

## Configuration Sources

Configuration is loaded from multiple sources in the following priority order (highest to lowest):

1. **Environment variables** (highest priority)
2. **`.env` file** in the current directory
3. **Default values** (lowest priority)

This allows for flexible deployment scenarios:
- Development: Use `.env` file
- Production: Use environment variables from orchestrator (Docker, Kubernetes, etc.)
- Testing: Override specific values as needed

## Configuration Structure

### Server Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `SERVER_HOST` | String | `0.0.0.0` | Host to bind the server |
| `SERVER_PORT` | u16 | `50051` | Port to bind the server |
| `TLS_ENABLED` | bool | `false` | Enable TLS encryption |
| `TLS_CERT_PATH` | String | None | Path to TLS certificate (optional) |
| `TLS_KEY_PATH` | String | None | Path to TLS private key (optional) |
| `TLS_CLIENT_CA_PATH` | String | None | Path to client CA certificate (optional) |

### Database Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `DATABASE_URL` | String | `sqlite:///home/rubentxu/hodei-data/hodei.db` | Database connection URL |

Supported database URLs:
- **SQLite**: `sqlite:///path/to/database.db` or `sqlite:///:memory:`
- **PostgreSQL**: `postgres://user:password@host:port/database`
- **MySQL**: `mysql://user:password@host:port/database`

### Logging Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `LOG_LEVEL` | String | `info` | Application log level |

Valid log levels: `trace`, `debug`, `info`, `warn`, `error`

### gRPC Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `GRPC_MAX_FRAME_SIZE` | usize | `4194304` (4MB) | Maximum frame size in bytes |
| `GRPC_KEEPALIVE_TIME` | u64 | `30` seconds | Keep-alive interval |

### Shutdown Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `SHUTDOWN_TIMEOUT` | u64 | `30` seconds | Graceful shutdown timeout |

## Usage Examples

### Basic Usage

The configuration system is automatically initialized in `main()`:

```rust
use hodei_shared::config::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration is automatically loaded from .env and environment variables
    let settings = Settings::new()?;
    
    // Use configuration
    println!("Server: {}:{}", settings.server_host(), settings.server_port());
    println!("Database: {}", settings.database_url());
    
    Ok(())
}
```

### Custom Configuration Loader

For advanced scenarios, you can configure the loader:

```rust
use hodei_shared::config::{ConfigLoader, Settings};

let settings = ConfigLoader::default()
    .with_env_prefix("MY_APP")  // Use MY_APP_* prefix for env vars
    .with_dotenv("/path/to/.env")
    .with_env()                 // Also load plain environment variables
    .load()?;

```

### Programmatic Configuration

You can also create settings programmatically:

```rust
use hodei_shared::config::{ServerConfig, DatabaseConfig, Settings};

let settings = Settings {
    server: ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        tls: TlsConfig {
            enabled: true,
            cert_path: Some("/path/to/cert.pem".to_string()),
            key_path: Some("/path/to/key.pem".to_string()),
            client_ca_path: None,
        },
    },
    database: DatabaseConfig {
        url: "postgres://user:pass@localhost:5432/db".to_string(),
    },
    log_level: "debug".to_string(),
    grpc: GrpcConfig {
        max_frame_size: Some(8 * 1024 * 1024), // 8MB
        keepalive_time: Some(60),
    },
    shutdown_timeout: 60,
};
```

## Environment Variable Prefixes

You can use environment variable prefixes to avoid conflicts:

```bash
export SERVER_PORT=8080
export DATABASE_URL=postgres://...
export LOG_LEVEL=debug
```

Or use a custom prefix:

```rust
ConfigLoader::default()
    .with_env_prefix("HODEI")  // Will look for HODEI_SERVER_PORT, etc.
    .load()
```

## File Format

The `.env` file uses a simple key-value format:

```bash
# Comments start with #
SERVER_HOST=0.0.0.0
SERVER_PORT=50051
DATABASE_URL=sqlite:///home/rubentxu/hodei-data/hodei.db
LOG_LEVEL=info

# Values can be quoted
APP_NAME="Hodei Verified Permissions"

# Spaces around = are trimmed
KEY = value
```

## Best Practices

### 1. Development

Use a `.env` file for local development:

```bash
cp .env.example .env
# Edit .env with your settings
cargo run --bin hodei-verified-permissions
```

### 2. Production

Use environment variables from your orchestrator:

```yaml
# Docker Compose
services:
  hodei-server:
    environment:
      - SERVER_PORT=50051
      - DATABASE_URL=postgres://user:pass@db:5432/hodei
      - LOG_LEVEL=warn
```

```bash
# Kubernetes
kubectl set env deployment/hodei-server \
  SERVER_PORT=50051 \
  DATABASE_URL=postgres://...
```

### 3. Testing

Override specific values for tests:

```rust
use hodei_shared::config::{Settings, ConfigSource};

fn test_specific_config() {
    // Use in-memory database for tests
    let settings = ConfigLoader::default()
        .with_env()  // Load env vars
        .with_source(Box::new(DotEnvConfigSource::new("test.env")))
        .load()
        .unwrap();
        
    assert_eq!(settings.database_url(), "sqlite:///:memory:");
}
```

### 4. Security

Never commit sensitive information (passwords, keys) to version control:

```bash
# .gitignore
.env
.env.local
.env.*.local
```

Instead, use:
- Secret management systems (Kubernetes Secrets, AWS Secrets Manager, etc.)
- Environment variables from orchestrator
- Encrypted `.env` files

## Advanced Topics

### Custom Configuration Sources

You can implement your own `ConfigSource`:

```rust
use hodei_shared::config::ConfigSource;

struct MyConfigSource {
    data: HashMap<String, String>,
}

impl ConfigSource for MyConfigSource {
    type Error = MyError;

    fn get(&self, key: &str) -> Result<Option<String>, Self::Error> {
        Ok(self.data.get(key).cloned())
    }

    fn get_all(&self) -> Result<HashMap<String, String>, Self::Error> {
        Ok(self.data.clone())
    }
}
```

### Configuration Validation

Add validation by implementing the `Configuration` trait with custom logic:

```rust
impl Configuration for MySettings {
    fn server_port(&self) -> u16 {
        let port = self.server.port;
        if port < 1024 {
            panic!("Server port must be >= 1024");
        }
        port
    }
}
```

### Hot Reloading

For configuration changes without restart:

```rust
use tokio::time::{interval, Duration};

async fn watch_config() {
    let mut interval = interval(Duration::from_secs(30));
    let mut last_hash = None;

    loop {
        interval.tick().await;
        
        let settings = Settings::new().unwrap();
        let current_hash = hash_config(&settings);
        
        if Some(current_hash) != last_hash {
            info!("Configuration changed, reloading...");
            reload_application(settings);
            last_hash = Some(current_hash);
        }
    }
}
```

## Troubleshooting

### Configuration Not Loading

1. Check if `.env` file exists in the current directory
2. Verify environment variables are set: `env | grep SERVER`
3. Check logs for configuration loading errors

```bash
# Check environment variables
env | grep -E "SERVER|DATABASE|LOG"

# Run with debug logging
LOG_LEVEL=debug cargo run --bin hodei-verified-permissions
```

### Invalid Configuration Values

Configuration values are validated during loading:

```bash
# Invalid port will use default
SERVER_PORT=not_a_number  # Will use default 50051

# Invalid log level will use default
LOG_LEVEL=invalid_level    # Will use default info
```

### Database Connection Issues

Verify the database URL format:

```bash
# SQLite
DATABASE_URL=sqlite:///absolute/path/to.db
DATABASE_URL=sqlite:///:memory:

# PostgreSQL
DATABASE_URL=postgres://user:password@host:port/database

# MySQL
DATABASE_URL=mysql://user:password@host:port/database
```

## See Also

- [Rust Dotenv crate](https://crates.io/crates/dotenv) for .env file handling
- [tonic](https://github.com/hyperium/tonic) for gRPC configuration
- [Tracing](https://github.com/tokio-rs/tracing) for logging configuration
- [Tokio](https://tokio.rs/) for async runtime configuration
