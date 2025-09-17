# Golem MCP Server V2

Model Context Protocol server for Golem Cloud with 100% direct API integration.

## ✅ What's New in V2

- **NO subprocess execution** - Direct API calls only
- **Type-safe operations** - Strongly typed throughout
- **13 granular tools** - Better LLM control
- **Performance** - Async/await, no process overhead

## 🚀 Quick Start

```bash
# Build
cargo build --release --bin golem-mcp-server

# Configure
export GOLEM_API_URL=https://api.golem.cloud
export GOLEM_API_TOKEN=your-token-here

# Run
../../target/release/golem-mcp-server --stdio
```

## 🛠 Available Tools

| Tool | Description |
|------|-------------|
| `list_components` | List all components |
| `deploy_component` | Deploy WASM component |
| `update_component` | Update component |
| `delete_component` | Delete component |
| `list_workers` | List workers |
| `create_worker` | Create worker |
| `invoke_worker` | Invoke function |
| `delete_worker` | Delete worker |
| `list_profiles` | List profiles |
| `switch_profile` | Switch profile |
| `new_app` | Create app |
| `build_app` | Build app |
| `deploy_app` | Deploy app |

## ⚠️ Known Issue

**rmcp 0.6.0 Bug**: The `tools/list` endpoint doesn't respond due to a framework routing issue. Tools are registered and functional, but discovery requires manual configuration.

**Workaround**: Configure tools manually in your MCP client.

**Status**: Investigating fixes. See [V2_STATUS_REPORT.md](V2_STATUS_REPORT.md).

## 📁 Architecture

```
src/
├── rmcp_handler_v2.rs    # V2 implementation (direct API)
├── api_client.rs         # Golem API client
├── api_client_extensions.rs # API extensions
└── main.rs              # Entry point
```

## 🧪 Testing

```bash
# Unit tests (30/32 pass)
cargo test --lib

# Tool registration test
cargo test --test test_v2_tools

# Validation
python3 validate_v2.py
```

## 📊 Status

- ✅ **Implementation**: Complete
- ✅ **Tools**: 13/13 registered
- ✅ **Tests**: 30/32 passing
- ⚠️ **Issue**: tools/list not routing (rmcp bug)
- ✅ **API**: 100% direct, no subprocess

## 📚 Documentation

- [Development Plan](.plan/2SEPT.MD)
- [Status Report](V2_STATUS_REPORT.md)
- [Final Analysis](FINAL_ANALYSIS.md)

## License

See LICENSE in repository root.