# Golem MCP Server - VERIFIED File Cleanup & Fix Plan

## Executive Summary
After thorough analysis using GitHub grep, context7, and rust-staff-engineer validation, here's the **FINAL VERDICT** for this million-dollar project.

## 🔴 FILES TO DELETE (Confirmed by Analysis)

### 1. Misleading Documentation (2 files)
```bash
rm Compare.MD              # Claims V2 was "just created" - misleading
rm FINAL_ASSESSMENT.MD     # Claims 85% complete when only 20% works
```
**Reason**: These documents misrepresent the actual state. In production, false documentation is worse than no documentation.

### 2. Test Files Testing Fake Implementations (7 files)
```bash
# Rust tests that validate fake behavior - technical debt
rm tests/test_v2_tools.rs          # Line 29: "Should return placeholder response"
rm tests/test_v2_parameters.rs     # Tests parameters for fake tools
rm tests/test_v2_basic.rs          # Tests basic fake operations
rm tests/test_direct_v2.rs         # Direct testing of fake implementations
rm tests/v2_validation_tests.rs    # Validates fake responses
rm tests/test_v2_integration.rs    # Integration tests for fake tools
rm tests/test_e2e_v2_comprehensive.rs  # Comprehensive test of fake implementations
```
**Reason**: Per rust-staff-engineer: "Tests that validate fake behavior are technical debt"

### 3. Python Tests (DELETE entire folder)
```bash
rm -rf tests_python/
```
**Reason**:
- Line 57 in conftest.py expects only 10 tools when we have 46
- Tests fake implementations
- Better to rewrite when tools actually work

### 4. Broken Example (1 file)
```bash
rm examples/basic_usage.rs  # Uses old rmcp_handler, not v2
```
**Reason**: Example code must work or it misleads users

## 🟡 FILES TO FIX (High Priority)

### 1. **src/rmcp_handler_v2.rs** - CRITICAL
**Problem**: 60% of tools return fake data
**Evidence**:
- Line 1077: `update_worker` returns `"pending_implementation"`
- Line 1092: `interrupt_worker` returns fake success
- Line 1128: `resume_worker` returns fake success
- Line 1244: `revert_worker` returns scripted response
- Line 1277: `connect_worker` returns scripted response

**FIX**: Delete all fake tools OR implement real APIs
```rust
// DELETE this pattern:
Ok(CallToolResult::success(vec![
    Content::json(json!({
        "status": "pending_implementation"
    }))
]))

// REPLACE with:
let response = self.api_client.worker
    .update_worker(&component_id, &worker_name, &request)
    .await
    .map_err(|e| McpError::from_golem_error(e))?;
Ok(CallToolResult::success(vec![Content::json(response)?]))
```

### 2. **src/client_manager.rs** - CRITICAL (Performance Bug)
**Problem**: Creates new HTTP client for each API client
**Evidence**: Line 79: `client: reqwest::Client::new()`

**FIX**: Use shared connection pool
```rust
use once_cell::sync::Lazy;

pub static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .pool_max_idle_per_host(20)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(60))
        .tcp_nodelay(true)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
});

// Then use: client: HTTP_CLIENT.clone()
```

### 3. **src/api_client_extensions.rs** - CRITICAL
**Problem**: `delete_component` always fails
**Evidence**: Line 139: Returns error "not yet available"

**FIX**: Implement actual DELETE
```rust
async fn delete_component(&self, component_id: &ComponentId) -> Result<(), ErrorData> {
    let url = format!("{}/v1/components/{}", self.context.base_url, component_id.0);
    let response = self.context.client
        .delete(&url)
        .bearer_auth(self.context.bearer_token().unwrap_or_default())
        .send()
        .await
        .map_err(|e| ErrorData::internal_error(&e.to_string(), None))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(ErrorData::from_response(response).await)
    }
}
```

### 4. **src/http.rs** - CRITICAL (Security)
**Problem**: Permissive CORS
**Evidence**: Line 82: `.layer(CorsLayer::permissive())`

**FIX**: Proper CORS configuration
```rust
use tower_http::cors::{AllowOrigin, AllowMethods};

.layer(
    CorsLayer::new()
        .allow_origin(AllowOrigin::exact("http://localhost:3000".parse().unwrap()))
        .allow_methods([Method::POST, Method::GET, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true)
)
```

### 5. **src/handlers.rs** - KEEP BUT FIX
**Problem**: References workspace crates that may not be configured
**Evidence**: Lines 24, 26: `golem_mcp_tools::ToolRegistry`

**FIX**: Either:
1. Configure workspace crates properly, OR
2. Replace with rmcp patterns from v2 handler

## 🟢 FILES TO KEEP (Working Correctly)

### Core Infrastructure
- `src/main.rs` - Entry point
- `src/lib.rs` - Library interface
- `src/server.rs` - Server setup
- `src/config.rs` - Configuration
- `src/auth.rs` - Authentication (verified working)
- `src/errors.rs` - Error handling
- `src/session.rs` - Session management
- `src/progress.rs` - Progress tracking
- `src/api_client.rs` - Direct API client

### Build & Config
- `Cargo.toml` - Build configuration
- `README.md` - Documentation
- `API.md` - API documentation
- `.gitignore` - Git ignore rules

### Valid Tests (Keep and update)
- `tests/integration.rs` - Update to test real APIs
- `tests/mcp_inspector_compatibility.rs` - Important for MCP protocol
- `tests/test_tool_registration.rs` - Verify tool registration
- `tests/test_mcp_protocol.rs` - Protocol tests

## Implementation Priority (Million Dollar Project)

### Week 1: CRITICAL FIXES
1. **Day 1**: Delete all files marked for deletion
2. **Day 2**: Fix connection pooling bug
3. **Day 3**: Fix CORS security vulnerability
4. **Day 4**: Fix delete_component
5. **Day 5**: Remove or fix ALL fake tool implementations

### Week 2: REAL IMPLEMENTATIONS
1. Implement worker management tools using real APIs
2. Implement profile management using config files
3. Add proper error handling

### Week 3: TESTING
1. Write contract tests using wiremock
2. Integration tests against real server
3. Performance testing

## Success Metrics
- [ ] 0 tools return "pending_implementation"
- [ ] 0 tools return fake/hardcoded data
- [ ] All tests test real API behavior
- [ ] Connection pooling uses single HTTP client
- [ ] CORS properly configured
- [ ] All workspace crates resolve

## Final Count
- **Files to DELETE**: 11 files + 1 folder
- **Files to FIX**: 5 files (all critical)
- **Files to KEEP**: 14+ files

## Rust Staff Engineer Verdict
> "This is a million-dollar system - no room for 'pending_implementation' in production. Every line must be production-grade or not exist at all."

## Commands to Execute
```bash
# Step 1: Clean up immediately
rm Compare.MD FINAL_ASSESSMENT.MD
rm -rf tests_python/
rm tests/test_v2_{tools,parameters,basic,integration}.rs
rm tests/{test_direct_v2,v2_validation_tests,test_e2e_v2_comprehensive}.rs
rm examples/basic_usage.rs

# Step 2: Verify build still works
cargo build

# Step 3: Run remaining tests
cargo test

# Step 4: Begin fixing the 5 critical files
```