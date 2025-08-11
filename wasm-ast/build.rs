use std::io::Result;

#[cfg(feature = "protobuf")]
fn main() -> Result<()> {
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[cfg(feature = \"protobuf\")]");
    config.type_attribute(
        ".",
        "#[cfg_attr(feature=\"bincode\", derive(bincode::Encode, bincode::Decode))]",
    );

    // Use vendored `protoc` so that an external installation is not required
    // (important for Windows and CI environments).
    let protoc_path = protoc_bin_vendored::protoc_bin_path().expect("protoc not found");
    std::env::set_var("PROTOC", protoc_path);

    config.compile_protos(&["proto/wasm/ast/type.proto"], &["proto/"])?;
    Ok(())
}

#[cfg(not(feature = "protobuf"))]
fn main() -> Result<()> {
    Ok(())
}
