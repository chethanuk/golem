# Windows setup (no WSL required)

Native Windows development environment for Golem. Two setup approaches: **traditional commands** or **automated mise tasks**.

## Prerequisites

* Windows 10/11 x64, Administrator PowerShell, winget v1.6+
* Mise (recommended for automation)

```powershell
winget install jdx.mise
# Add C:\Users\Administrator\AppData\Local\mise\shims to PATH
```


## 1) Install build tools

Install VS Build Tools with C++ workload and CMake for native compilation.

```powershell
# Traditional approach
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --norestart"
winget install --id Kitware.CMake -e
```

```powershell
# Mise automation (includes verification)
mise run install-build-tools
mise run verify-build-tools
```

**Note:** VS Build Tools is large (~3GB). Reboot may be required.

## 2) Install Rust

Install Rust stable with wasm32-wasip1 target (required for WASM components).

```powershell
# Traditional approach (from repo root)
mise install
rustup --version && rustc --version && cargo --version
```

```powershell
# Mise automation (includes verification)
mise run install-rust
```

## 3) Developer tools

Install cargo-make task runner used by Golem build system.

```powershell
# Traditional approach
cargo install cargo-make
```

```powershell
# Mise automation
mise run install-dev-tools
```

## 4) Build and test

Build workspace and run unit tests (no Docker/Redis required).

```powershell
# Traditional approach
cargo build --workspace
cargo test --workspace --lib -- --nocapture
```

```powershell
# Mise automation
mise run build-workspace
mise run test-workspace
```

```powershell
# Low RAM systems (limited parallelism)
mise run build-workspace-limited
mise run test-workspace-limited
```

If you run into linker errors for SQLite on Windows, ensure the VS Build Tools step completed successfully; the project uses SQLx with SQLite support and compiles C code as needed via MSVC.


## 5) Services (Nginx, Redis, Memurai)

Install services required for local development.

```powershell
# Traditional approach
choco install nginx redis memurai-developer -y
Get-Process | Where-Object {$_.ProcessName -match "golem|redis|memurai"}
```

```powershell
# Mise automation (includes Chocolatey install + verification)
mise run install-services
mise run verify-services
```

## 6) Optional: Docker and GitHub Actions

Docker for integration tests, act for local CI testing.

```powershell
# Traditional approach
winget install --id Docker.DockerDesktop -e
winget install --id nektos.act -e
```

```powershell
# Mise automation
mise run install-docker
mise run install-act
```

**Note:** act only emulates Linux jobs, not Windows runners.  

## Task Status and Known Issues

### Working Tasks 
- `check-prerequisites` - Verifies winget is available
- `install-golem-cli` - Downloads Golem CLI binary (with idempotency check)
- All install tasks now include smart checks to avoid reinstalling existing tools

### Tasks with Issues ⚠️
**Critical Issue**: mise tasks have fundamental execution problems on Windows. Despite applying proper shell configuration (`shell = "powershell -ExecutionPolicy Bypass -Command"`), PATH refresh fixes, and idempotency checks, the tasks still hang or fail to execute commands properly.

**Affected Tasks**:
- `install-build-tools` - Visual Studio Build Tools & CMake installation
- `verify-build-tools` - cmake not found in PATH after winget install
- `install-rust` - Rust installation via mise
- `install-dev-tools` - cargo-make installation
- `install-chocolatey` - Chocolatey package manager
- `install-services` - Nginx, Redis, Memurai via Chocolatey
- `build-workspace` - Cargo build commands
- `test-workspace` - Cargo test commands

**Root Causes Identified**:
1. **PATH Issues**: winget installations don't update PATH in current session
2. **PowerShell Execution Context**: Commands hang even with proper shell configuration
3. **Terminal Session Isolation**: mise tasks run in isolated contexts that don't inherit environment changes

**Attempted Fixes**:
- ✅ Added `shell = "powershell -ExecutionPolicy Bypass -Command"` 
- ✅ Added PATH refresh: `$env:PATH = [System.Environment]::GetEnvironmentVariable('PATH', 'Machine') + ';' + [System.Environment]::GetEnvironmentVariable('PATH', 'User')`
- ✅ Added idempotency checks to prevent reinstalling existing tools
- ✅ Fixed missing template variables (`nginx_id`, `redis_id`, `memurai_id`)
- ❌ **Still not working**: Commands still hang or fail to execute

### Manual Execution Required 
**Recommended Approach**: Use the traditional PowerShell commands listed above instead of mise tasks for:
- All installation tasks requiring elevated permissions
- Build and test operations
- Service installations via Chocolatey

**Working mise tasks for basic operations**:
- `check-prerequisites`
- `install-golem-cli` (file download only)

### Troubleshooting Notes
- Tasks show dependency execution but hang on actual command execution
- PowerShell execution policy may need adjustment for mise task context
- Some tasks may require running PowerShell as Administrator
- Network connectivity required for all download/install operations

### aws-lc-sys Build Dependencies
**Critical for cargo build to work**: The aws-lc-sys crate requires three dependencies:

1. **CMake** - `winget install Kitware.CMake`
2. **NASM assembler** - `winget install NASM.NASM`  
3. **LLVM/Clang** - `winget install LLVM.LLVM`

**Important**: After installing these dependencies:
- **Restart your terminal session** to pick up PATH changes
- Set `LIBCLANG_PATH` environment variable to the clang/bin directory
- Run cargo build with **Administrator privileges**

Without these dependencies, you'll see "Missing dependency: cmake" errors during cargo build.

## 7) Complete setup

Run full automated setup excluding optional components.

```powershell
# Complete setup (recommended)
mise run full-setup
```

```powershell
# With Docker/act
mise run full-setup-with-optional
```

## 8) Troubleshooting

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
# Memory optimization
set RUSTFLAGS=-Ccodegen-units=8
```


```powershell
# Mise automation for troubleshooting
mise run troubleshoot-paths
mise run memory-optimization
mise run clean-sqlite-db
```

## 9) Golem CLI

Download and setup Golem CLI for component management.

```powershell
# Traditional approach
Invoke-WebRequest -Uri "https://github.com/golemcloud/golem-cli/releases/download/v1.3.0-dev.3/golem-x86_64-pc-windows-gnu.exe" -OutFile "golem.exe"
.\golem.exe --version
.\golem.exe profile new --component-url http://localhost:8080/ --set-active cloud-local
```

```powershell
# Mise automation
mise run install-golem-cli
mise run setup-golem-profile
```

## 10) Start services

Start all Golem services for development.

```powershell
# Traditional approach
cargo make run
```

```powershell
# Mise automation (includes service monitoring)
mise run start-services
mise run check-service-status
mise run check-service-ports
```

## 11) Windows Defender exclusions

Add exclusions for better build performance.

```powershell
# Mise automation (opens settings + shows paths)
mise run setup-defender-exclusions
```

Manually add:
- `C:\Users\Administrator\golem` (project)
- `C:\Users\Administrator\.cargo` (cache) 
- `C:\Users\Administrator\.rustup` (toolchain)