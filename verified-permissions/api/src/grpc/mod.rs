//! gRPC service implementations

pub mod control_plane;
pub mod data_plane;

pub use control_plane::AuthorizationControlService;
pub use data_plane::AuthorizationDataService;
