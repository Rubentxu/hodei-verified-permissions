//! Server container setup for E2E tests

use std::time::Duration;
use testcontainers::{core::ContainerPort, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt};
use tokio::time::sleep;

/// Configuration for the test server container
pub struct ServerContainer {
    pub container: ContainerAsync<GenericImage>,
    pub grpc_port: u16,
}

impl ServerContainer {
    /// Start a new server container for testing
    pub async fn start() -> Self {
        // Build the server image (assumes Dockerfile exists)
        let image = GenericImage::new("hodei-server", "test")
            .with_exposed_port(ContainerPort::Tcp(50051))
            .with_env_var("DATABASE_URL", "sqlite::memory:")
            .with_env_var("RUST_LOG", "info");

        let container = image.start().await.expect("Failed to start container");
        let grpc_port = container.get_host_port_ipv4(50051).await.expect("Failed to get port");

        // Wait for server to be ready
        Self::wait_for_ready(grpc_port).await;

        Self {
            container,
            grpc_port,
        }
    }

    /// Get the gRPC endpoint URL
    pub fn grpc_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.grpc_port)
    }

    /// Wait for the server to be ready
    async fn wait_for_ready(port: u16) {
        let max_attempts = 30;
        let mut attempts = 0;

        while attempts < max_attempts {
            if Self::check_health(port).await {
                return;
            }
            sleep(Duration::from_millis(100)).await;
            attempts += 1;
        }

        panic!("Server failed to start within timeout");
    }

    /// Check if server is healthy
    async fn check_health(port: u16) -> bool {
        // Try to connect to the gRPC port
        tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_server_container_starts() {
        let container = ServerContainer::start().await;
        
        assert!(container.grpc_port > 0);
        assert!(container.grpc_url().contains("http://127.0.0.1"));
    }
}
