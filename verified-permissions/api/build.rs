fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use shared proto file from repository root
    // Try multiple paths for flexibility (local dev vs Docker)
    let proto_paths = [
        "../../proto/authorization.proto",  // Local development
        "../proto/authorization.proto",     // Docker build
        "proto/authorization.proto",        // Alternative
    ];
    
    for proto_path in &proto_paths {
        if std::path::Path::new(proto_path).exists() {
            tonic_prost_build::compile_protos(proto_path)?;
            return Ok(());
        }
    }
    
    panic!("Could not find authorization.proto in any expected location");
}
