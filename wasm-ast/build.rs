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
    // Keep environment variable for downstream tools, but also pass the path
    // directly to prost-build to ensure it uses the vendored binary.
    std::env::set_var("PROTOC", &protoc_path);
    // Also prepend the directory of the vendored binary to PATH so that any
    // tooling (including prost-build internals) relying on searching the PATH
    // can still locate `protoc`, even if it ignores the PROTOC env var.
    let protoc_dir = protoc_path.parent().unwrap();
    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let mut new_path = std::ffi::OsString::from(protoc_dir);
    // Use platform-appropriate separator.
    new_path.push(if cfg!(windows) { ";" } else { ":" });
    new_path.push(old_path);
    std::env::set_var("PATH", &new_path);
    println!("cargo:warning=Prepended to PATH: {}", protoc_dir.display());
    // Emit helpful build-time diagnostics so it is clear which `protoc`
    // binary is being used and whether it is actually present on disk.
    println!("cargo:warning=Vendored protoc path: {}", protoc_path.display());
    println!("cargo:warning=Vendored protoc exists: {}", protoc_path.exists());
    // Also print the value picked up by the build script for further diagnostics.
    println!(
        "cargo:warning=PROTOC env var: {:?}",
        std::env::var_os("PROTOC")
    );

    // Try invoking the vendored `protoc --version` for additional diagnostics.
    {
        use std::process::Command;
        match Command::new(&protoc_path).arg("--version").output() {
            Ok(out) => {
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
            Err(e) => {
                println!("cargo:warning=failed to spawn protoc: {}", e);
            }
        }
    }

    config.compile_protos(&["proto/wasm/ast/type.proto"], &["proto/"])?;
    Ok(())
}

#[cfg(not(feature = "protobuf"))]
fn main() -> Result<()> {
    Ok(())
}
