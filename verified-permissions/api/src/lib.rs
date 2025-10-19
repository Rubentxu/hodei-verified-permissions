//! Hodei API - External interfaces (gRPC, CLI)
//!
//! This layer contains the external interfaces that expose the application
//! functionality through gRPC services and CLI commands.

// TODO: Update grpc handlers to use hodei_application use cases
// Temporarily commented out to allow compilation
// pub mod grpc;
pub mod cli;

// Re-export generated protobuf types
pub mod proto {
    tonic::include_proto!("authorization");
}

// pub use grpc::*;
pub use cli::*;
