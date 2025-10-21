fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use shared proto file from repository root
    tonic_prost_build::compile_protos("../../proto/authorization.proto")?;
    Ok(())
}
