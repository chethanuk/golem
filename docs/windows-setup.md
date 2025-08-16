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

On machines with limited RAM you may need to build with lower parallelism to
avoid linker or LLVM out-of-memory errors:

```powershell
cargo build --workspace -j 2
cargo test  --workspace --lib -j 2 -- --nocapture
```

If you run into linker errors for SQLite on Windows, ensure the VS Build Tools step completed successfully; the project uses SQLx with SQLite support and compiles C code as needed via MSVC.


## Install Nginx, Redis, Memurai

Ensure you have Chocolatey installed:

```powershell
choco install nginx redis memurai-developer -y
```

Verify Redis is working 
```powershell
$  Get-Process | Where-Object {$_.ProcessName -match "golem|redis|memurai"} | Select-Object Id, ProcessName, CPU

  Id ProcessName      CPU
  -- -----------      ---
4684 memurai     0.359375
7228 memurai      0.40625
```

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
* Out-of-memory (OOM) or `STATUS_STACK_BUFFER_OVERRUN` during compilation:  
  • Close other heavy applications.  
  • Build with fewer jobs, e.g. `cargo build -j 2`.  
  • The repository already provides a `.cargo/config.toml` with a safe default:  

    ```toml
    [build]
    jobs = 2
    ```  
    Increase or decrease this to match your hardware.  
  • You can also reduce per-crate code-generation memory:  

    ```powershell
    set RUSTFLAGS=-Ccodegen-units=8
    ```
   

  • After running `cargo make run`, you can check the status of the services with:   

    ```powershell
    Get-Process -Id 6472,5856,7324,7932,4172,904,8944,3060 -ErrorAction SilentlyContinue | Select-Object Id, ProcessName, CPU, StartTime
    ```

  • After running `cargo make run`, you can check the ports of the services with:   

    ```powershell
    netstat -an | findstr "8080\|8085"
    ```

  • After running `cargo make run`, you can check the help of the services with:   

    ```powershell
    PS C:\Users\Administrator\golem> cd .\cloud-service\ 
    PS C:\Users\Administrator\golem\cloud-service> $env:RUST_LOG="info"; $env:GOLEM__HTTP_PORT="8080"; $env:GOLEM__GRPC_PORT="9090"; $env:GOLEM__LOGIN__TYPE="Disabled"; $env:GOLEM__DB__TYPE="Sqlite"; $env:GOLEM__DB__CONFIG__DATABASE="../local-run/data/golem_cloud_service.db"; $env:GOLEM__ACCOUNTS__ROOT__TOKEN="5c832d93-ff85-4a8f-9803-513950fdfdb1"; ..\target\debug\cloud-service.exe --help
    ```

    if SqLlite is corrupted, remove using 

    ```powershell
    Remove-Item -Path ".\local-run\data\*.db*" -Force -ErrorAction SilentlyContinue; Remove-Item -Path ".\local-run\data\*.sqlite*" -Force -ErrorAction SilentlyContinue
    ```


  ## Golem CLI

  
  Invoke-WebRequest -Uri "https://github.com/golemcloud/golem-cli/releases/download/v1.3.0-dev.3/golem-x86_64-pc-windows-gnu.exe" -OutFile "golem.exe"

  .\golem.exe --version
golem 1.3.0-dev.3

.\golem.exe profile new --component-url http://localhost:8080/ --set-active cloud-local

.\golem.exe component list

# Misc

Add Windows Defender Exclusions (Recommended)


start ms-settings:windowsdefender

Windows Defender Exclusion Fix:
Manual Steps:

Windows Security should now be open
Click "Virus & threat protection"
Click "Manage settings" under Virus & threat protection settings
Scroll down to "Exclusions"
Click "Add or remove exclusions"
Add these 3 folders:
C:\Users\Administrator\golem (your project folder)
C:\Users\Administrator\.cargo (Cargo cache)
C:\Users\Administrator\.rustup (Rustup toolchain)