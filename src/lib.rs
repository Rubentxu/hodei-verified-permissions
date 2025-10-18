//! Hodei Verified Permissions - Cedar-based Authorization Service
//!
//! This service provides a Cedar policy engine implementation with gRPC APIs
//! for both data plane (authorization decisions) and control plane (policy management).

pub mod grpc;
pub mod storage;
pub mod error;
pub mod jwt;
pub mod audit;
pub mod agent;

// Re-export generated protobuf types
pub mod proto {
    tonic::include_proto!("authorization");
}
