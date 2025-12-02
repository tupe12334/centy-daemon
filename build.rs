fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bundle protoc so users don't need to install it separately
    std::env::set_var("PROTOC", protobuf_src::protoc());

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("centy_descriptor.bin"))
        .compile_protos(&["proto/centy.proto"], &["proto"])?;
    Ok(())
}
