//! Protocol compliance tests for MCP 2025-06-18 specification
//!
//! Comprehensive testing of JSON-RPC 2.0 and MCP protocol compliance
//! following Staff Engineer best practices with systematic edge case coverage.

use golem_mcp_types::{
    error::McpError,
    protocol::{
        InitializeParams, InitializeResult, JsonRpcError, JsonRpcErrorResponse, JsonRpcId, 
        JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, McpCapabilities, McpClientInfo, 
        McpImplementation, McpServerInfo, MCP_VERSION,
    },
    serialization::{MessageBuilder, MessageSerializer},
};
use rstest::*;
use serde_json::{json, Value};
use std::collections::HashMap;

#[fixture]
fn valid_initialize_params() -> InitializeParams {
    InitializeParams {
        protocol_version: MCP_VERSION.to_owned(),
        capabilities: McpCapabilities {
            tools: None,
            resources: None,
            prompts: None,
            logging: None,
        },
        client_info: McpClientInfo {
            name: "test-client".to_owned(),
            version: "1.0.0".to_owned(),
            implementation: None,
        },
    }
}

#[fixture]
fn sample_server_info() -> McpServerInfo {
    McpServerInfo {
        name: "test-server".to_owned(),
        version: "1.0.0".to_owned(),
        implementation: Some(McpImplementation {
            name: "golem-mcp-server".to_owned(),
            version: "0.1.0".to_owned(),
            metadata: HashMap::new(),
        }),
    }
}

/// Test MCP version string compliance
#[rstest]
fn test_mcp_version_format() {
    assert_eq!(MCP_VERSION, "2025-06-18");
    
    // Version should follow YYYY-MM-DD format
    let parts: Vec<&str> = MCP_VERSION.split('-').collect();
    assert_eq!(parts.len(), 3);
    
    // Year should be reasonable
    let year: u16 = parts[0].parse().expect("year should be numeric");
    assert!(year >= 2024 && year <= 2030);
    
    // Month should be valid
    let month: u8 = parts[1].parse().expect("month should be numeric");
    assert!(month >= 1 && month <= 12);
    
    // Day should be valid
    let day: u8 = parts[2].parse().expect("day should be numeric");
    assert!(day >= 1 && day <= 31);
}

/// Test JSON-RPC 2.0 request structure compliance
#[rstest]
#[case::string_id("test-id")]
#[case::numeric_id("123")]
#[case::uuid_id("550e8400-e29b-41d4-a716-446655440000")]
fn test_json_rpc_request_structure(#[case] id: &str) {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_owned(),
        method: "initialize".to_owned(),
        params: Some(json!({"test": "value"})),
        id: JsonRpcId::String(id.to_owned()),
    };
    
    let json_str = serde_json::to_string(&request).expect("should serialize");
    let parsed: Value = serde_json::from_str(&json_str).expect("should parse");
    
    // Verify required fields
    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["method"], "initialize");
    assert_eq!(parsed["id"], id);
    assert!(parsed.get("params").is_some());
}

/// Test JSON-RPC 2.0 success response structure compliance
#[rstest]
fn test_json_rpc_success_response_structure() {
    let id = JsonRpcId::String("test".to_owned());
    let result = json!({"result": "success"});
    
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_owned(),
        result,
        id,
    };
    
    let json_str = serde_json::to_string(&response).expect("should serialize");
    let parsed: Value = serde_json::from_str(&json_str).expect("should parse");
    
    assert_eq!(parsed["jsonrpc"], "2.0");
    assert!(parsed.get("id").is_some());
    assert!(parsed.get("result").is_some());
    assert!(parsed.get("error").is_none());
}

/// Test JSON-RPC 2.0 error response structure compliance
#[rstest]
fn test_json_rpc_error_response_structure() {
    let id = JsonRpcId::String("test".to_owned());
    let error = JsonRpcError {
        code: -32600,
        message: "Invalid Request".to_owned(),
        data: None,
    };
    
    let response = JsonRpcErrorResponse {
        jsonrpc: "2.0".to_owned(),
        id: Some(id),
        error,
    };
    
    let json_str = serde_json::to_string(&response).expect("should serialize");
    let parsed: Value = serde_json::from_str(&json_str).expect("should parse");
    
    assert_eq!(parsed["jsonrpc"], "2.0");
    assert!(parsed.get("id").is_some());
    assert!(parsed.get("error").is_some());
    assert!(parsed.get("result").is_none());
}

/// Test MCP initialize request/response cycle
#[rstest]
fn test_initialize_protocol_cycle(
    valid_initialize_params: InitializeParams,
    sample_server_info: McpServerInfo,
) {
    // Create initialize request
    let request = MessageBuilder::request(
        "init-1",
        "initialize",
        Some(serde_json::to_value(&valid_initialize_params).unwrap()),
    );
    
    // Verify request structure
    if let JsonRpcMessage::Request(req) = &request {
        assert_eq!(req.method, "initialize");
        assert!(req.params.is_some());
    } else {
        panic!("Expected request message");
    }
    
    // Create initialize response
    let initialize_result = InitializeResult {
        protocol_version: MCP_VERSION.to_owned(),
        capabilities: McpCapabilities {
            tools: Some(golem_mcp_types::protocol::ToolsCapability {
                list_changed: true,
            }),
            resources: None,
            prompts: None,
            logging: None,
        },
        server_info: sample_server_info,
        instructions: Some("Test server instructions".to_owned()),
    };
    
    let response = MessageBuilder::response_with_result("init-1", &initialize_result)
        .expect("should create response");
    
    // Test serialization roundtrip
    let json_str = MessageSerializer::serialize_message(&response).expect("should serialize");
    let deserialized = MessageSerializer::deserialize_message(&json_str)
        .expect("should deserialize");
    
    assert_eq!(response, deserialized);
}

/// Test protocol version validation edge cases
#[rstest]
#[case::exact_match("2025-06-18", true)]
#[case::future_version("2026-01-01", false)]
#[case::past_version("2024-01-01", false)]
#[case::malformed_version("invalid", false)]
#[case::partial_version("2025", false)]
#[case::empty_version("", false)]
fn test_protocol_version_validation(#[case] version: &str, #[case] should_accept: bool) {
    let params = InitializeParams {
        protocol_version: version.to_owned(),
        capabilities: McpCapabilities {
            tools: None,
            resources: None,
            prompts: None,
            logging: None,
        },
        client_info: McpClientInfo {
            name: "test".to_owned(),
            version: "1.0.0".to_owned(),
            implementation: None,
        },
    };
    
    // Test version compatibility logic
    let is_compatible = params.protocol_version == MCP_VERSION;
    assert_eq!(is_compatible, should_accept);
    
    if !should_accept && version != MCP_VERSION {
        let error = McpError::protocol_version_mismatch(
            MCP_VERSION.to_owned(),
            version.to_owned(),
        );
        assert_eq!(error.code(), -32_000);
    }
}

/// Test JSON-RPC notification structure (no id field)
#[rstest]
#[case::initialized("initialized")]
#[case::cancelled("notifications/cancelled")]
fn test_json_rpc_notification_structure(#[case] method: &str) {
    let notification = MessageBuilder::notification(method, None);
    
    if let JsonRpcMessage::Notification(notif) = notification {
        assert_eq!(notif.jsonrpc, "2.0");
        assert_eq!(notif.method, method);
        
        let json_str = serde_json::to_string(&notif).expect("should serialize");
        let parsed: Value = serde_json::from_str(&json_str).expect("should parse");
        
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["method"], method);
        assert!(parsed.get("id").is_none()); // Notifications must not have id
    } else {
        panic!("Expected notification message");
    }
}

/// Test MCP capabilities structure compliance
#[rstest]
fn test_mcp_capabilities_structure() {
    let capabilities = McpCapabilities {
        tools: Some(golem_mcp_types::protocol::ToolsCapability {
            list_changed: true,
        }),
        resources: Some(golem_mcp_types::protocol::ResourcesCapability {
            subscribe: false,
            list_changed: true,
        }),
        prompts: None,
        logging: Some(golem_mcp_types::protocol::LoggingCapability {
            levels: vec![
                golem_mcp_types::protocol::LogLevel::Info,
                golem_mcp_types::protocol::LogLevel::Error,
            ],
        }),
    };
    
    let json_str = serde_json::to_string(&capabilities).expect("should serialize");
    let parsed: Value = serde_json::from_str(&json_str).expect("should parse");
    
    // Verify optional fields are handled correctly
    assert!(parsed.get("tools").is_some());
    assert!(parsed.get("resources").is_some());
    assert!(parsed.get("prompts").is_none());
    assert!(parsed.get("logging").is_some());
    
    // Verify tools capability
    let tools = &parsed["tools"];
    assert_eq!(tools["list_changed"], true);
    
    // Verify resources capability
    let resources = &parsed["resources"];
    assert_eq!(resources["subscribe"], false);
    assert_eq!(resources["list_changed"], true);
}

/// Test message serialization with various JSON-RPC ID types
#[rstest]
#[case::string_id(JsonRpcId::String("test-123".to_owned()))]
#[case::number_id(JsonRpcId::Number(42))]
fn test_json_rpc_id_serialization(#[case] id: JsonRpcId) {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_owned(),
        method: "test_method".to_owned(),
        params: None,
        id,
    };
    
    let json_str = serde_json::to_string(&request).expect("should serialize");
    let parsed: JsonRpcRequest = serde_json::from_str(&json_str).expect("should deserialize");
    
    assert_eq!(request.id, parsed.id);
}

/// Test error response format compliance
#[rstest]
#[case::parse_error(-32700, "Parse error")]
#[case::invalid_request(-32600, "Invalid Request")]
#[case::method_not_found(-32601, "Method not found")]
#[case::invalid_params(-32602, "Invalid params")]
#[case::internal_error(-32603, "Internal error")]
fn test_json_rpc_error_codes(#[case] code: i32, #[case] message: &str) {
    let error = McpError::json_rpc(code, message.to_owned());
    let error_response = MessageBuilder::error_response(
        Some(JsonRpcId::String("test".to_owned())),
        &error,
    );
    
    if let JsonRpcMessage::Error(err_resp) = error_response {
        assert_eq!(err_resp.jsonrpc, "2.0");
        assert_eq!(err_resp.error.code, code);
        assert!(err_resp.error.message.contains(message));
        
        // Test serialization
        let json_str = serde_json::to_string(&err_resp).expect("should serialize");
        let parsed: Value = serde_json::from_str(&json_str).expect("should parse");
        
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["error"]["code"], code);
    } else {
        panic!("Expected error response");
    }
}

/// Test message validation edge cases
#[rstest]
#[case::missing_jsonrpc(r#"{"method":"test","id":"1"}"#, false)]
#[case::wrong_jsonrpc_version(r#"{"jsonrpc":"1.0","method":"test","id":"1"}"#, false)]
#[case::missing_method_in_request(r#"{"jsonrpc":"2.0","id":"1"}"#, false)]
#[case::valid_request(r#"{"jsonrpc":"2.0","method":"test","id":"1"}"#, true)]
#[case::valid_notification(r#"{"jsonrpc":"2.0","method":"test"}"#, true)]
fn test_message_validation(#[case] json: &str, #[case] should_be_valid: bool) {
    let validation_result = MessageSerializer::validate_json_rpc(json);
    
    if should_be_valid {
        assert!(validation_result.is_ok(), "Should be valid: {}", json);
    } else {
        assert!(validation_result.is_err(), "Should be invalid: {}", json);
    }
}

/// Test concurrent message processing
#[rstest]
fn test_concurrent_message_handling() {
    use std::sync::Arc;
    use std::thread;
    
    let test_cases: Vec<_> = (0..100)
        .map(|i| {
            MessageBuilder::request(
                format!("req-{}", i),
                "test_method",
                Some(json!({"index": i})),
            )
        })
        .collect();
    
    let test_cases = Arc::new(test_cases);
    let handles: Vec<_> = (0..10)
        .map(|thread_id| {
            let cases = Arc::clone(&test_cases);
            thread::spawn(move || {
                let start = thread_id * 10;
                let end = start + 10;
                
                for i in start..end {
                    let message = &cases[i];
                    let serialized = MessageSerializer::serialize_message(message)
                        .expect("should serialize");
                    let deserialized = MessageSerializer::deserialize_message(&serialized)
                        .expect("should deserialize");
                    assert_eq!(*message, deserialized);
                }
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().expect("thread should complete");
    }
}

/// Test large message handling
#[rstest]
fn test_large_message_serialization() {
    // Create a large message payload
    let large_data: HashMap<String, String> = (0..1000)
        .map(|i| (format!("key_{}", i), format!("value_{}", i).repeat(100)))
        .collect();
    
    let request = MessageBuilder::request(
        "large-msg",
        "process_large_data",
        Some(serde_json::to_value(large_data).unwrap()),
    );
    
    // Should handle large messages without issues
    let serialized = MessageSerializer::serialize_message(&request)
        .expect("should serialize large message");
    
    assert!(serialized.len() > 100_000); // Verify it's actually large
    
    let deserialized = MessageSerializer::deserialize_message(&serialized)
        .expect("should deserialize large message");
    
    assert_eq!(request, deserialized);
}

/// Test malformed JSON handling
#[rstest]
#[case::incomplete_json(r#"{"jsonrpc":"2.0","method""#)]
#[case::invalid_structure(r#"{"jsonrpc":"2.0","method":123}"#)]
#[case::null_values(r#"{"jsonrpc":null,"method":"test"}"#)]
#[case::array_instead_of_object(r#"["jsonrpc","2.0"]"#)]
fn test_malformed_json_handling(#[case] malformed_json: &str) {
    let result = MessageSerializer::deserialize_message(malformed_json);
    assert!(result.is_err(), "Should reject malformed JSON: {}", malformed_json);
    
    if let Err(error) = result {
        assert_eq!(error.code(), -32_605); // Serialization error
    }
}

/// Test UTF-8 and Unicode handling in messages
#[rstest]
#[case::basic_unicode("Hello 世界")]
#[case::emoji("Testing with emojis: 🚀🎉")]
#[case::mathematical_symbols("Mathematical: ∑∏∫")]
#[case::mixed_scripts("Mixed: Привет مرحبا こんにちは")]
fn test_unicode_message_handling(#[case] unicode_text: &str) {
    let request = MessageBuilder::request(
        "unicode-test",
        "process_unicode",
        Some(json!({"text": unicode_text})),
    );
    
    let serialized = MessageSerializer::serialize_message(&request)
        .expect("should serialize unicode");
    
    let deserialized = MessageSerializer::deserialize_message(&serialized)
        .expect("should deserialize unicode");
    
    assert_eq!(request, deserialized);
    
    // Verify unicode is preserved
    if let JsonRpcMessage::Request(req) = deserialized {
        let params = req.params.unwrap();
        let params_text = params["text"].as_str().unwrap();
        assert_eq!(params_text, unicode_text);
    }
}

/// Test MCP 2025-06-18 specific features (no batching support)
#[rstest]
fn test_no_batching_support() {
    // MCP 2025-06-18 explicitly removes batching support
    // Verify that batch requests are not supported
    
    let batch_json = r#"[
        {"jsonrpc": "2.0", "method": "test1", "id": "1"},
        {"jsonrpc": "2.0", "method": "test2", "id": "2"}
    ]"#;
    
    // Should fail to parse as a single message
    let result = MessageSerializer::deserialize_message(batch_json);
    assert!(result.is_err(), "Batch requests should not be supported in MCP 2025-06-18");
}