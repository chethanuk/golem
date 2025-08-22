# Golem MCP Server API Documentation

Comprehensive API reference for the Golem CLI MCP Server, implementing the Model Context Protocol (MCP) 2025-06-18 specification.

## Table of Contents

- [Core Types](#core-types)
- [Server Interface](#server-interface)
- [Transport Layer](#transport-layer)
- [Error Handling](#error-handling)
- [Tool Operations](#tool-operations)
- [Resource Management](#resource-management)
- [Progress Streaming](#progress-streaming)
- [Configuration](#configuration)

## Core Types

### GolemMcpServer

The main server implementation providing MCP protocol compliance and Golem CLI integration.

```rust
pub struct GolemMcpServer {
    // Internal implementation details hidden
}

impl GolemMcpServer {
    /// Create a new server instance with default configuration
    pub fn new() -> Self
    
    /// Create a server with custom base directory for resource discovery
    pub fn with_base_dir(base_dir: PathBuf) -> Self
    
    /// Configure operation timeouts
    pub async fn configure_timeouts(&self, config: TimeoutConfig)
    
    /// Setup progress streaming with optional buffer size
    pub async fn setup_progress_streaming(&self, buffer_size: Option<usize>) -> ProgressReceiver
}
```

**Example Usage:**
```rust
use golem_mcp_server::rmcp_handler::GolemMcpServer;

// Basic server
let server = GolemMcpServer::new();

// Server with custom resource directory
let server = GolemMcpServer::with_base_dir("/path/to/project".into());

// Configure timeouts
server.configure_timeouts(TimeoutConfig::default()).await;
```

### GolemMcpTransport

Transport configuration for server communication modes.

```rust
#[derive(Debug, Clone)]
pub enum GolemMcpTransport {
    /// Standard input/output transport for Claude Desktop integration
    Stdio,
    /// HTTP transport with configurable host and port
    Http {
        port: u16,
        host: Option<String>,
    },
}
```

**Example Usage:**
```rust
use golem_mcp_server::rmcp_handler::GolemMcpTransport;

// Stdio transport (Claude Desktop)
let transport = GolemMcpTransport::Stdio;

// HTTP transport
let transport = GolemMcpTransport::Http {
    port: 8080,
    host: Some("127.0.0.1".to_string()),
};
```

## Server Interface

### MCP Handler Implementation

The server implements the complete MCP protocol through the `MCP_Handler` trait from rmcp v0.6.0.

#### Core MCP Methods

```rust
impl MCP_Handler for GolemMcpServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult, McpError>
    async fn list_tools(&self, params: ListToolsParams) -> Result<ListToolsResult, McpError>
    async fn call_tool(&self, params: CallToolParams) -> Result<CallToolResult, McpError>
    async fn list_resources(&self, params: ListResourcesParams) -> Result<ListResourcesResult, McpError>
    async fn read_resource(&self, params: ReadResourceParams) -> Result<ReadResourceResult, McpError>
}
```

#### Initialize Protocol Session

```rust
// Initialize with protocol capabilities
let params = InitializeParams {
    protocol_version: "2025-06-18".to_string(),
    capabilities: ClientCapabilities {
        tools: Some(ToolCapabilities::default()),
        resources: Some(ResourceCapabilities::default()),
    },
    client_info: ClientInfo {
        name: "claude-code".to_string(),
        version: "1.0.0".to_string(),
    },
};

let result = server.initialize(params).await?;
```

## Transport Layer

### HTTP Transport

Start the server with HTTP transport for direct API access.

```rust
pub async fn serve_mcp_http(
    server: GolemMcpServer,
    port: u16,
    host: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
```

**Example:**
```rust
use golem_mcp_server::{serve_mcp_http, rmcp_handler::GolemMcpServer};

let server = GolemMcpServer::new();
serve_mcp_http(server, 8080, "127.0.0.1").await?;
```

**HTTP Endpoints:**
- `POST /mcp` - Main MCP JSON-RPC endpoint
- `GET /mcp/health` - Health check endpoint
- `GET /mcp/capabilities` - Server capabilities

### Stdio Transport

Start the server with stdio transport for Claude Desktop integration.

```rust
pub async fn serve_mcp_stdio(
    server: GolemMcpServer,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
```

**Example:**
```rust
use golem_mcp_server::{serve_mcp_stdio, rmcp_handler::GolemMcpServer};

let server = GolemMcpServer::new();
serve_mcp_stdio(server).await?;
```

## Error Handling

### GolemMcpError

Comprehensive structured error hierarchy with categorization and recovery guidance.

```rust
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum GolemMcpError {
    CliCommandFailed {
        operation: String,
        message: String,
        exit_code: Option<i32>,
        stderr: Option<String>,
        recovery_hint: Option<RecoveryHint>,
    },
    ResourceNotFound {
        uri: String,
        resource_type: ResourceType,
        search_paths: Vec<String>,
    },
    ConfigurationError {
        message: String,
        config_key: Option<String>,
        expected_format: Option<String>,
    },
    NetworkError {
        transport_type: TransportType,
        message: String,
        retry_after: Option<Duration>,
    },
    OperationTimeout {
        operation_name: String,
        timeout_millis: u64,
        progress_stage: Option<ProgressStage>,
    },
    ProgressStreamError {
        stage: ProgressStage,
        message: String,
        buffer_state: BufferState,
    },
    ValidationError {
        field: String,
        message: String,
        provided_value: Option<String>,
        valid_values: Option<Vec<String>>,
    },
    InternalError {
        message: String,
        error_id: String,
    },
    ResourceContentError {
        uri: String,
        encoding_error: Option<String>,
        content_type: Option<String>,
    },
}
```

### Error Categories

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    UserError,           // User input or usage errors
    SystemError,         // OS, filesystem, or external system errors
    ConfigurationError,  // Configuration or setup issues
    NetworkError,        // Network connectivity issues
    ResourceError,       // File or resource access issues
    PerformanceError,    // Timeout or performance-related errors
    InternalError,       // Server implementation errors
}
```

### Recovery Hints

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryHint {
    RetryWithBackoff { initial_delay_ms: u64, max_retries: u32 },
    CheckConfiguration { config_key: String, expected_format: String },
    VerifyResource { resource_path: String, required_permissions: String },
    ContactSupport { error_id: String, user_context: String },
}
```

### Error Methods

```rust
impl GolemMcpError {
    /// Get error category for observability
    pub fn category(&self) -> ErrorCategory
    
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity
    
    /// Get recovery guidance
    pub fn recovery_hint(&self) -> Option<RecoveryHint>
    
    /// Get MCP protocol error code
    pub fn get_mcp_error_code(&self) -> i32
    
    /// Log structured error information
    pub fn log_error(&self)
    
    /// Create CLI command error with recovery hint
    pub fn cli_command_failed(
        operation: impl Into<String>,
        message: impl Into<String>,
        exit_code: Option<i32>,
        stderr: Option<String>,
    ) -> Self
    
    /// Create operation timeout error
    pub fn operation_timeout(
        operation_name: impl Into<String>,
        timeout_millis: u64,
    ) -> Self
    
    /// Create transport error with retry information
    pub fn transport(
        transport_type: TransportType,
        message: impl Into<String>,
    ) -> Self
}
```

## Tool Operations

### Component Management

Golem component lifecycle operations exposed as MCP tools.

```rust
pub struct ComponentOperation {
    pub operation: String,
    pub component_name: Option<String>,
    pub wasm_file: Option<String>,
    pub project_id: Option<String>,
    pub extra: serde_json::Value,
}
```

**Available Operations:**
- `component_list` - List all components
- `component_get` - Get component details
- `component_update` - Update component
- `component_add` - Add new component

**Example:**
```json
{
  "name": "component_list",
  "arguments": {
    "project_id": "my-project"
  }
}
```

### Worker Management

Golem worker instance operations.

```rust
pub struct WorkerOperation {
    pub operation: String,
    pub worker_name: Option<String>,
    pub component_name: Option<String>,
    pub extra: serde_json::Value,
}
```

**Available Operations:**
- `worker_list` - List workers
- `worker_get` - Get worker details
- `worker_start` - Start worker
- `worker_stop` - Stop worker
- `worker_delete` - Delete worker

### Application Management

Golem application project operations.

```rust
pub struct AppOperation {
    pub operation: String,
    pub app_name: Option<String>,
    pub template: Option<String>,
    pub extra: serde_json::Value,
}
```

**Available Operations:**
- `app_new` - Create new application
- `app_list` - List applications
- `app_deploy` - Deploy application

## Resource Management

### Resource Discovery

The server automatically discovers and exposes project manifests and configuration files.

```rust
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}
```

**Discovered Resource Types:**
- `golem.yaml` - Application manifests
- `component.yaml` - Component configurations
- `worker.yaml` - Worker definitions
- `*.wit` - WebAssembly Interface Type files

### Resource Access

```rust
impl GolemMcpServer {
    async fn list_resources(&self, params: ListResourcesParams) -> Result<ListResourcesResult, McpError>
    async fn read_resource(&self, params: ReadResourceParams) -> Result<ReadResourceResult, McpError>
}
```

**Example Resource URIs:**
- `file:///project/golem.yaml`
- `file:///project/components/my-component/component.yaml`
- `file:///project/wit/component.wit`

## Progress Streaming

### TimeoutConfig

Configuration for operation timeouts and progress tracking.

```rust
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub default_timeout: Duration,
    pub long_operation_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub progress_buffer_size: usize,
}

impl TimeoutConfig {
    /// Default production timeouts
    pub fn default() -> Self
    
    /// Fast timeouts for testing
    pub fn fast_operations() -> Self
    
    /// Long timeouts for heavy operations
    pub fn extended_operations() -> Self
}
```

### Progress Tracking

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressStage {
    Initializing,
    Validating,
    Processing,
    Building,
    Deploying,
    Finalizing,
    Complete,
}

pub struct ProgressReceiver {
    // Internal implementation for receiving progress updates
}
```

## Configuration

### Server Configuration

```rust
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub transport: GolemMcpTransport,
    pub enable_progress: bool,
    pub timeout_config: TimeoutConfig,
    pub enable_resources: bool,
    pub resource_base_dir: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            transport: GolemMcpTransport::Stdio,
            enable_progress: true,
            timeout_config: TimeoutConfig::default(),
            enable_resources: true,
            resource_base_dir: None,
        }
    }
}
```

### Environment Variables

The server respects standard environment variables:

- `RUST_LOG` - Logging level (debug, info, warn, error)
- `GOLEM_CLI_PATH` - Custom path to Golem CLI binary
- `MCP_TIMEOUT_MS` - Default operation timeout in milliseconds

## JSON-RPC API Examples

### Initialize Session

```json
POST /mcp
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-06-18",
    "capabilities": {
      "tools": {},
      "resources": {}
    },
    "clientInfo": {
      "name": "claude-code",
      "version": "1.0.0"
    }
  }
}
```

### List Available Tools

```json
POST /mcp
{
  "jsonrpc": "2.0",
  "id": "2",
  "method": "tools/list",
  "params": {}
}
```

### Execute Component List

```json
POST /mcp
{
  "jsonrpc": "2.0",
  "id": "3",
  "method": "tools/call",
  "params": {
    "name": "component_list",
    "arguments": {
      "project_id": "my-golem-project"
    }
  }
}
```

### List Resources

```json
POST /mcp
{
  "jsonrpc": "2.0",
  "id": "4",
  "method": "resources/list",
  "params": {}
}
```

### Read Manifest

```json
POST /mcp
{
  "jsonrpc": "2.0",
  "id": "5",
  "method": "resources/read",
  "params": {
    "uri": "file:///project/golem.yaml"
  }
}
```

## Error Response Format

```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "category": "UserError",
      "severity": "Error",
      "recovery_hint": {
        "type": "CheckConfiguration",
        "config_key": "project_id",
        "expected_format": "non-empty string"
      }
    }
  }
}
```

## Integration Patterns

### Claude Code Integration

```json
{
  "mcpServers": {
    "golem-cli": {
      "command": "golem-cli",
      "args": ["server", "mcp", "--serve-port", "1232"],
      "transport": "http",
      "url": "http://localhost:1232/mcp"
    }
  }
}
```

### Custom Client Integration

```rust
use golem_mcp_server::{serve_mcp_http, rmcp_handler::GolemMcpServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = GolemMcpServer::new();
    serve_mcp_http(server, 8080, "127.0.0.1").await
}
```

## Performance Characteristics

### Concurrent Operations

The server supports concurrent tool execution with proper resource isolation:

- **Tool calls**: Up to 10 concurrent operations
- **Resource reads**: Unlimited concurrent access
- **Progress streaming**: Per-operation isolated buffers

### Memory Usage

- **Base memory**: ~5MB for server initialization
- **Per-operation overhead**: ~100KB for progress tracking
- **Resource caching**: LRU cache with 50MB limit

### Timeout Behavior

- **Default operations**: 30 seconds
- **Long operations**: 5 minutes (build, deploy)
- **Network operations**: 10 seconds with exponential backoff

## Security Considerations

### Input Validation

All tool parameters are validated before execution:

- Path traversal protection for resource URIs
- Command injection prevention for CLI parameters
- JSON schema validation for all inputs

### Resource Access

- Read-only access to discovered resources
- Sandboxed CLI execution environment
- No shell command execution beyond Golem CLI

### Error Information

Error messages are sanitized to prevent information leakage:

- No sensitive file paths in public errors
- Structured logging for debugging without exposure
- Recovery hints provide guidance without internal details
