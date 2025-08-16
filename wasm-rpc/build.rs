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
    // Set `PROTOC` for crates that respect the environment variable...
    std::env::set_var("PROTOC", &protoc_path);

    // ...and also prepend the containing directory to `PATH` so that tools
    // performing their own lookup can still discover the vendored binary.
    let protoc_dir = protoc_path.parent().unwrap();
    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let mut new_path = std::ffi::OsString::from(protoc_dir);
    new_path.push(if cfg!(windows) { ";" } else { ":" });
    new_path.push(old_path);
    std::env::set_var("PATH", &new_path);

    // Build-time diagnostics.
    println!("cargo:warning=Prepended to PATH: {}", protoc_dir.display());
    println!(
        "cargo:warning=Vendored protoc path: {}",
        protoc_path.display()
    );
    println!(
        "cargo:warning=Vendored protoc exists: {}",
        protoc_path.exists()
    );
    println!(
        "cargo:warning=PROTOC env var: {:?}",
        std::env::var_os("PROTOC")
    );

    // Attempt to invoke `protoc --version` for extra diagnostics.
    if let Ok(out) = std::process::Command::new(&protoc_path)
        .arg("--version")
        .output()
    {
        println!("cargo:warning=protoc --version status: {}", out.status);
        println!(
            "cargo:warning=protoc stdout: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        println!(
            "cargo:warning=protoc stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

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
