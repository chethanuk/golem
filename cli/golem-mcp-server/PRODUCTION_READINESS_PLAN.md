# Production Readiness Plan - Golem MCP Server

## Phase 1: Critical Fixes (Week 1)

### 1.1 Remove All Fake Implementations
```rust
// DELETE these tools until real implementations exist:
- update_worker (returns "pending_implementation")
- interrupt_worker (fake success)
- Any other placeholder tools
```

### 1.2 Fix HTTP Client Pooling

Create `/src/http_client.rs`:
```rust
use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

/// Global HTTP client with connection pooling
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    ClientBuilder::new()
        .pool_max_idle_per_host(20)
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .tcp_keepalive(Duration::from_secs(60))
        .tcp_nodelay(true)
        .user_agent(concat!("golem-mcp-server/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("Failed to initialize HTTP client")
});
```

### 1.3 Implement Proper Error Hierarchy

Create `/src/errors.rs`:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpOperationError {
    #[error("Component operation failed: {context}")]
    ComponentOperation {
        operation: OperationType,
        component_id: Option<String>,
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Worker operation failed: {context}")]
    WorkerOperation {
        operation: OperationType,
        worker_id: Option<String>,
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("API not available: {feature}")]
    FeatureNotAvailable {
        feature: String,
        available_in: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub enum OperationType {
    Create,
    Update,
    Delete,
    Invoke,
    Interrupt,
    Resume,
}
```

## Phase 2: Tool Implementation Strategy (Week 2-3)

### 2.1 Tool Categorization

**Tier 1 - Core Operations (Implement First)**
- list_components ✅ (already working)
- deploy_component ✅ (already working)
- create_worker ✅ (already working)
- invoke_worker ✅ (already working)
- list_workers ✅ (already working)

**Tier 2 - Essential Operations (Implement Second)**
- delete_component ❌ (needs upstream API)
- delete_worker ⚠️ (verify API exists)
- get_component ⚠️ (verify implementation)
- get_worker ⚠️ (verify implementation)
- get_worker_oplog ⚠️ (verify implementation)

**Tier 3 - Advanced Operations (Implement Last)**
- update_worker ❌ (currently fake)
- interrupt_worker ❌ (currently fake)
- resume_worker ❌ (needs implementation)
- simulated_crash_worker ⚠️ (dangerous - needs safety checks)
- update_workers_batch ❌ (needs real batch API)

### 2.2 Implementation Template

For each unimplemented tool:
```rust
#[tool(description = "Update a worker to a new version")]
#[instrument(skip(self))]
pub async fn update_worker(
    &self,
    Parameters(params): Parameters<UpdateWorkerParams>,
) -> Result<CallToolResult, McpError> {
    // OPTION 1: If API doesn't exist yet
    return Err(McpError::FeatureNotAvailable {
        feature: "worker_update".to_string(),
        available_in: Some("v2.1.0".to_string()),
    });

    // OPTION 2: Once API is available
    let worker_id = self.parse_worker_id(&params)?;

    match self.api_client.update_worker(&worker_id, params.target_version).await {
        Ok(updated_worker) => {
            Ok(CallToolResult::success(vec![
                Content::json(updated_worker)?
            ]))
        }
        Err(e) => Err(self.convert_api_error(e))
    }
}
```

## Phase 3: Testing Strategy (Week 3-4)

### 3.1 Delete Invalid Tests
Remove tests for fake implementations:
```bash
# Delete these test files
rm tests/test_fake_tools.rs
rm tests_python/test_placeholder_responses.py
```

### 3.2 Implement Contract Tests
```rust
#[cfg(test)]
mod contract_tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};

    #[tokio::test]
    async fn test_update_worker_real_api() {
        let mock_server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path_regex(r"/v1/workers/.+"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "worker_id": "test-worker",
                    "version": "2.0.0",
                    "status": "updated"
                })))
            .mount(&mock_server)
            .await;

        // Test real behavior, not fake responses
    }
}
```

## Phase 4: Performance Optimization (Week 4)

### 4.1 Connection Pool Metrics
```rust
use prometheus::{IntGauge, register_int_gauge};

lazy_static! {
    static ref ACTIVE_CONNECTIONS: IntGauge = register_int_gauge!(
        "http_client_active_connections",
        "Number of active HTTP connections"
    ).unwrap();

    static ref IDLE_CONNECTIONS: IntGauge = register_int_gauge!(
        "http_client_idle_connections",
        "Number of idle HTTP connections"
    ).unwrap();
}
```

### 4.2 Request Deduplication
```rust
use moka::future::Cache;

pub struct RequestCache {
    cache: Cache<String, Value>,
}

impl RequestCache {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_secs(5))
                .build(),
        }
    }

    pub async fn get_or_fetch<F, Fut>(&self, key: String, fetch: F) -> Result<Value, Error>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Value, Error>>,
    {
        if let Some(cached) = self.cache.get(&key).await {
            return Ok(cached);
        }

        let value = fetch().await?;
        self.cache.insert(key, value.clone()).await;
        Ok(value)
    }
}
```

## Phase 5: Production Deployment (Week 5)

### 5.1 Feature Flags
```toml
[features]
default = ["tier1-tools"]
tier1-tools = []
tier2-tools = ["tier1-tools"]
tier3-tools = ["tier2-tools"]
experimental = ["tier3-tools"]
```

### 5.2 Health Checks
```rust
#[derive(Serialize)]
pub struct HealthStatus {
    pub status: &'static str,
    pub version: &'static str,
    pub tools_available: usize,
    pub tools_enabled: Vec<String>,
    pub connection_pool_status: PoolStatus,
}

pub async fn health_check(&self) -> HealthStatus {
    HealthStatus {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
        tools_available: self.get_enabled_tools().len(),
        tools_enabled: self.get_enabled_tools(),
        connection_pool_status: HTTP_CLIENT.pool_status(),
    }
}
```

## Success Metrics

- **Zero** fake/placeholder responses in production
- **100%** of enabled tools have real implementations
- HTTP connection reuse rate > 80%
- P99 latency < 200ms for all operations
- Zero panics in production
- All Clippy warnings resolved
- Test coverage > 80% for real implementations

## Timeline

- **Week 1**: Critical fixes (HTTP pooling, remove fakes)
- **Week 2-3**: Implement Tier 1 & 2 tools
- **Week 3-4**: Testing and validation
- **Week 4**: Performance optimization
- **Week 5**: Production deployment with monitoring

## Risk Mitigation

1. **Upstream API Gaps**: Document clearly which operations need upstream work
2. **Breaking Changes**: Use versioned APIs and feature flags
3. **Performance**: Implement circuit breakers and rate limiting
4. **Security**: Add request signing and validation

## Conclusion

This plan transforms the MCP server from a prototype with fake implementations into a production-ready service following Rust best practices. The phased approach ensures we deliver value incrementally while maintaining system stability.