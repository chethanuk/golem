# Windows setup (no WSL required)

This guide sets up a native Windows development environment for Golem. It avoids Docker/WSL for regular build/test; Docker remains optional for integration tests.

## Prerequisites

* Windows 10/11 x64  
* Administrator PowerShell session  
* winget v1.6 or newer (check with `winget --version`)

## 1) Install build tools

Run in an **elevated PowerShell**:

```powershell
# Visual Studio Build Tools (C++ toolchain) — passive install, no restart
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --norestart"

# CMake
winget install --id Kitware.CMake -e
```

Notes  
• The VS Build Tools install is large and may take several minutes.  
• A reboot may be required after first install if PATHs are missing.

## 2) Install Rust via mise

If you do not have mise:

```powershell
winget install jdx.mise
```

From the repository root (contains `.mise.toml`):

```powershell
mise install
# verify
rustup --version
rustc  --version
cargo  --version
```

`.mise.toml` pins Rust to **stable** and installs the `wasm32-wasip1` target automatically.

## 3) Developer tools

```powershell
# Cargo task runner used by this repo
cargo install cargo-make
```

## 4) Build and test

```powershell
# Build entire workspace
cargo build --workspace

# Run unit tests only (no Docker/Redis required)
cargo test --workspace --lib -- --nocapture
```

If you run into linker errors for SQLite on Windows, ensure the VS Build Tools step completed successfully; the project uses SQLx with SQLite support and compiles C code as needed via MSVC.

## 5) Optional: Docker Desktop and act

Some integration tests use Docker and Redis. Docker is optional for regular dev.

```powershell
# Docker Desktop (requires virtualization; WSL2 backend recommended)
winget install --id Docker.DockerDesktop -e

# act (runs GitHub Actions locally; Linux jobs only)
winget install --id nektos.act -e
```

Notes  
• The Windows CI job runs on GitHub (`windows-2022`). `act` does **not** emulate Windows runners; use it only for Linux jobs.  

## 6) Troubleshooting

* Open a **new terminal** after tool installs to refresh PATH.  
* If `cargo` is not found, check `%USERPROFILE%\.cargo\bin` is in PATH.  
* If `protoc` errors appear, this repo vendors `protoc`; no system install is required.  
* For slow builds, limit jobs in `%USERPROFILE%\.cargo\config.toml`:

  ```toml
  [build]
  jobs = 4
  ```
