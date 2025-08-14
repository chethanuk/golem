# Cargo Make Tools Documentation

## Overview
This document provides comprehensive documentation for all cargo-make tools defined in `Makefile.toml`, with actual Windows output examples, usage instructions, and cross-platform compatibility notes.

## Prerequisites
- Rust and Cargo installed
- cargo-make installed: `cargo install cargo-make`
- Git for version control operations
- Additional dependencies per tool (documented below)

---

## Core Development Flow Tools

### 1. `cargo make dev-flow` (or `cargo make`)
**Purpose**: Runs a full development flow, including fixing format and clippy, building and running tests and generating OpenAPI specs.

**Windows Usage**: 
```powershell
cargo make dev-flow
# OR simply:
cargo make
```

**Dependencies**: Runs in sequence: wit → fix → check → build
**Cross-Platform**: ✅ Compatible with Windows, Linux, Mac

**Test Output**:
```
[Testing in progress...]
```

---

### 2. `cargo make dev`
**Purpose**: Alias to the dev-flow task.

**Windows Usage**: 
```powershell
cargo make dev
```

**Cross-Platform**: ✅ Compatible with Windows, Linux, Mac

**Test Output**:
```
[Testing in progress...]
```

---

## WIT Dependencies Tools

### 3. `cargo make wit`
**Purpose**: Fetches the WIT dependencies based on wit/deps.toml.

**Windows Usage**: 
```powershell
cargo make wit
```

**Prerequisites**: 
- wit-deps-cli crate will be automatically installed
- Network access for downloading dependencies

**Cross-Platform**: ⚠️ **Requires Testing** - Uses shell scripts that may need Windows adaptations

**Test Output**:
```
[Testing in progress...]
```

---

### 4. `cargo make check-wit`
**Purpose**: Deletes then fetches the WIT dependencies based on wit/deps.toml, then checks if it's up-to-date.

**Windows Usage**: 
```powershell
cargo make check-wit
```

**Cross-Platform**: ⚠️ **Requires Testing** - May have file path and git diff compatibility issues

**Test Output**:
```
[Testing in progress...]
```

---

## Build Tools

### 5. `cargo make build`
**Purpose**: Builds everything in debug mode.

**Windows Usage**: 
```powershell
cargo make build
```

**Cross-Platform**: ✅ Should be compatible with Windows, Linux, Mac

**Test Output**:
```
[Testing in progress...]
```

---

### 6. `cargo make build-release`
**Purpose**: Builds everything in release mode. Customizable with PLATFORM_OVERRIDE env variable for docker builds.

**Windows Usage**: 
```powershell
cargo make build-release
# With platform override:
$env:PLATFORM_OVERRIDE="linux/amd64"; cargo make build-release
```

**Cross-Platform**: ⚠️ **Requires Testing** - Platform override logic may need Windows adaptations

**Test Output**:
```
[Testing in progress...]
```

---

## Code Quality Tools

### 7. `cargo make check`
**Purpose**: Runs rustfmt and clippy checks without applying any fix.

**Windows Usage**: 
```powershell
cargo make check
```

**Prerequisites**: 
- rustfmt and clippy components installed
- `rustup component add rustfmt clippy`

**Cross-Platform**: ✅ Should be compatible with Windows, Linux, Mac

**Test Output**:
```
[Testing in progress...]
```

---

### 8. `cargo make fix`
**Purpose**: Runs rustfmt and clippy checks and applies fixes.

**Windows Usage**: 
```powershell
cargo make fix
```

**Prerequisites**: Same as check command

**Cross-Platform**: ✅ Should be compatible with Windows, Linux, Mac

**Test Output**:
```
[Testing in progress...]
```

---

## Testing Tools

### 9. `cargo make unit-tests`
**Purpose**: Runs unit tests only.

**Windows Usage**: 
```powershell
cargo make unit-tests
```

**Cross-Platform**: ✅ Should be compatible with Windows, Linux, Mac

**Test Output**:
```
[Testing in progress...]
```

---

### 10. `cargo make worker-executor-tests`
**Purpose**: Runs worker executor tests only.

**Windows Usage**: 
```powershell
cargo make worker-executor-tests
```

**Environment Variables**: 
- `WASMTIME_BACKTRACE_DETAILS=1`
- `RUST_BACKTRACE=1`
- `RUST_LOG=info`

**Cross-Platform**: ✅ Should be compatible with Windows, Linux, Mac

**Test Output**:
```
[Testing in progress...]
```

---

### 11. `cargo make integration-tests`
**Purpose**: Runs integration tests only.

**Windows Usage**: 
```powershell
cargo make integration-tests
```

**Prerequisites**: Requires built binaries (depends on build-bins task)

**Cross-Platform**: ⚠️ **Requires Testing** - May have shell script compatibility issues

**Test Output**:
```
[Testing in progress...]
```

---

### 12. `cargo make sharding-tests-debug`
**Purpose**: Runs sharding integration tests with file logging enabled, also accepts test name filter arguments.

**Windows Usage**: 
```powershell
cargo make sharding-tests-debug
```

**Cross-Platform**: ⚠️ **Requires Testing** - Complex shell scripts may need Windows adaptations

**Test Output**:
```
[Testing in progress...]
```

---

### 13. `cargo make api-tests-http`
**Purpose**: Runs API integration tests using HTTP API only.

**Windows Usage**: 
```powershell
cargo make api-tests-http
```

**Environment**: Sets `GOLEM_CLIENT_PROTOCOL=http`

**Cross-Platform**: ✅ Should be compatible with Windows, Linux, Mac

**Test Output**:
```
[Testing in progress...]
```

---

### 14. `cargo make api-tests-grpc`
**Purpose**: Runs API integration tests using GRPC API only.

**Windows Usage**: 
```powershell
cargo make api-tests-grpc
```

**Environment**: Sets `GOLEM_CLIENT_PROTOCOL=grpc`

**Cross-Platform**: ✅ Should be compatible with Windows, Linux, Mac

**Test Output**:
```
[Testing in progress...]
```

---

### 15. `cargo make test`
**Purpose**: Runs all unit tests, worker executor tests and integration tests.

**Windows Usage**: 
```powershell
cargo make test
```

**Cross-Platform**: ⚠️ **Requires Testing** - Combines multiple test suites

**Test Output**:
```
[Testing in progress...]
```

---

## OpenAPI Tools

### 16. `cargo make check-openapi`
**Purpose**: Generates openapi spec from the code and checks if it is the same as the one in the openapi directory (for CI).

**Windows Usage**: 
```powershell
cargo make check-openapi
```

**Cross-Platform**: ⚠️ **Requires Testing** - File diff operations may need Windows adaptations

**Test Output**:
```
[Testing in progress...]
```

---

### 17. `cargo make generate-openapi`
**Purpose**: Generates openapi spec from the code and saves it to the openapi directory.

**Windows Usage**: 
```powershell
cargo make generate-openapi
```

**Cross-Platform**: ⚠️ **Requires Testing** - File operations and merging logic

**Test Output**:
```
[Testing in progress...]
```

---

## Configuration Tools

### 18. `cargo make check-configs`
**Purpose**: Generates configs from code defaults and checks if it's up-to-date.

**Windows Usage**: 
```powershell
cargo make check-configs
```

**Cross-Platform**: ⚠️ **Requires Testing** - Uses git diff which should work on Windows

**Test Output**:
```
[Testing in progress...]
```

---

### 19. `cargo make generate-configs`
**Purpose**: Generates configs from code defaults.

**Windows Usage**: 
```powershell
cargo make generate-configs
```

**Prerequisites**: Requires built binaries in target/debug/

**Cross-Platform**: ⚠️ **Requires Testing** - Uses shell scripts with executable paths

**Test Output**:
```
[Testing in progress...]
```

---

## Publishing & Running Tools

### 20. `cargo make publish`
**Purpose**: Publishes packages to crates.io.

**Windows Usage**: 
```powershell
cargo make publish
```

**Prerequisites**: 
- Authenticated with crates.io (`cargo login`)
- Proper package metadata in Cargo.toml files

**Cross-Platform**: ✅ Should be compatible with Windows, Linux, Mac

**Test Output**:
```
[Testing in progress...]
```

---

### 21. `cargo make run`
**Purpose**: Runs all services locally, requires redis, lnav and nginx.

**Windows Usage**: 
```powershell
cargo make run
```

**Prerequisites**: 
- Redis server installed and accessible
- nginx installed and accessible  
- lnav (Log File Navigator) installed
- Built binaries

**Cross-Platform**: ❌ **Linux/Mac Only** - Uses ./local-run/start.sh shell script

**Notes**: Needs Windows batch file equivalent or PowerShell script

**Test Output**:
```
[Testing in progress...]
```

---

### 22. `cargo make run-with-login-enabled`
**Purpose**: Runs all services locally with login enabled, requires redis, lnav and nginx. Also requires oauth2 configuration to be provided.

**Windows Usage**: 
```powershell
# Set environment variables first:
$env:GITHUB_CLIENT_ID="your_client_id"
$env:GITHUB_CLIENT_SECRET="your_client_secret"
cargo make run-with-login-enabled
```

**Prerequisites**: Same as `run` plus OAuth2 configuration

**Cross-Platform**: ❌ **Linux/Mac Only** - Uses ./local-run/start.sh shell script

**Test Output**:
```
[Testing in progress...]
```

---

## Elastic Stack Tools

### 23. `cargo make elastic-up`
**Purpose**: Starts elastic, kibana, filebeat (in detached mode) and loads logs into elastic.

**Windows Usage**: 
```powershell
cargo make elastic-up
```

**Prerequisites**: 
- Docker and Docker Compose installed
- log-tools/elastic directory with docker-compose.yml

**Cross-Platform**: ✅ Should be compatible with Windows, Linux, Mac (Docker based)

**Test Output**:
```
[Testing in progress...]
```

---

### 24. `cargo make elastic-stop`
**Purpose**: Stops the elastic env.

**Windows Usage**: 
```powershell
cargo make elastic-stop
```

**Prerequisites**: Same as elastic-up

**Cross-Platform**: ✅ Should be compatible with Windows, Linux, Mac (Docker based)

**Test Output**:
```
[Testing in progress...]
```

---

### 25. `cargo make elastic-down`
**Purpose**: Stops and removes the elastic env, including all stored data.

**Windows Usage**: 
```powershell
cargo make elastic-down
```

**Prerequisites**: Same as elastic-up

**Cross-Platform**: ✅ Should be compatible with Windows, Linux, Mac (Docker based)

**Test Output**:
```
[Testing in progress...]
```

---

## Summary

### Cross-Platform Compatibility Status:
- ✅ **Fully Compatible**: 11 tools
- ⚠️ **Requires Testing**: 12 tools  
- ❌ **Linux/Mac Only**: 2 tools (run, run-with-login-enabled)

### Next Steps:
1. Test each tool systematically on Windows
2. Fix cross-platform issues found during testing
3. Update this documentation with actual output examples
4. Create Windows-specific adaptations where needed

**Last Updated**: [Current testing session]
