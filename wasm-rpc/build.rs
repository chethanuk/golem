use std::io::Result;

#[cfg(feature = "protobuf")]
fn main() -> Result<()> {
    use std::env;

    let wasm_ast_root =
        env::var("GOLEM_WASM_AST_ROOT").unwrap_or_else(|_| find_package_root("golem-wasm-ast"));

    let mut config = prost_build::Config::new();
    config.extern_path(".wasm.ast", "::golem_wasm_ast::analysis::protobuf");
    config.type_attribute(".", "#[cfg(feature = \"protobuf\")]");
    config.type_attribute(
        ".",
        "#[cfg_attr(feature=\"bincode\", derive(bincode::Encode, bincode::Decode))]",
    );

    // Use vendored `protoc` so that an external installation is not required
    // (important for Windows and CI environments).
    let protoc_path = protoc_bin_vendored::protoc_bin_path().expect("protoc not found");
    std::env::set_var("PROTOC", protoc_path);

    config.compile_protos(
        &[
            "proto/wasm/rpc/val.proto",
            "proto/wasm/rpc/witvalue.proto",
            "proto/wasm/rpc/value_and_type.proto",
        ],
        &[&format!("{wasm_ast_root}/proto"), &"proto".to_string()],
    )?;
    Ok(())
}

#[cfg(feature = "protobuf")]
fn find_package_root(name: &str) -> String {
    use cargo_metadata::MetadataCommand;

    let metadata = MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()
        .unwrap();
    let package = metadata.packages.iter().find(|p| p.name == name).unwrap();
    package.manifest_path.parent().unwrap().to_string()
}

#[cfg(not(feature = "protobuf"))]
fn main() -> Result<()> {
    Ok(())
}
