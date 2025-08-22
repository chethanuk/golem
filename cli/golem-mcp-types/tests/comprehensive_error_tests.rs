//! Comprehensive error handling tests for MCP types
//!
//! Tests all error categories, serialization, and edge cases following
//! Staff Engineer best practices with zero tolerance for anti-patterns.

use golem_mcp_types::error::McpError;
use rstest::*;
use serde_json;
use std::collections::HashMap;

/// Test fixture for error scenarios
#[fixture]
fn sample_context() -> String {
    "test context".to_owned()
}

/// Test fixture for complex error details
#[fixture]
fn complex_error_details() -> HashMap<String, String> {
    let mut details = HashMap::new();
    details.insert("component".to_owned(), "test".to_owned());
    details.insert("operation".to_owned(), "serialize".to_owned());
    details
}

#[rstest]
#[case::invalid_request("invalid request")]
#[case::empty_message("")]
#[case::whitespace_only("   ")]
#[case::special_chars("special chars: !@#$%")]
#[case::unicode("unicode: 🚀")]
fn test_invalid_request_error_creation(#[case] message: &str) {
    let error = McpError::invalid_request(message.to_owned());
    
    assert_eq!(error.code(), -32_600);
    assert!(error.to_string().contains("Invalid request"));
    assert!(error.to_string().contains(message));
}

#[rstest]
#[case::method_not_found("unknown_method")]
#[case::empty_method("")]
#[case::method_with_spaces("method with spaces")]
#[case::long_method_name(&"x".repeat(1000))]
fn test_authorization_error_edge_cases(#[case] method: &str) {
    let error = McpError::authorization(format!("Method '{}' not authorized", method));
    
    assert_eq!(error.code(), -32_601);
    assert!(error.to_string().contains("Authorization failed"));
}

#[rstest]
#[case::component_access("component", "test-id", "not found")]
#[case::empty_resource("", "id", "error")]
#[case::unicode_resource("🚀-resource", "test", "access denied")]
fn test_resource_access_errors(
    #[case] resource_type: &str,
    #[case] resource_id: &str,
    #[case] details: &str,
) {
    let error = McpError::resource_access(
        resource_type.to_owned(),
        resource_id.to_owned(),
        details.to_owned(),
    );
    
    assert_eq!(error.code(), -32_602);
    assert!(error.to_string().contains("Resource access error"));
    assert!(error.to_string().contains(resource_type));
    assert!(error.to_string().contains(resource_id));
}

#[rstest]
#[case::serialize_error("serialization", "failed to encode JSON")]
#[case::deserialize_error("deserialization", "invalid JSON structure")]
#[case::empty_operation("", "unknown error")]
#[case::complex_message("validation", "expected object, found array at line 42")]
fn test_serialization_errors(#[case] operation: &str, #[case] details: &str) {
    let error = McpError::serialization(operation.to_owned(), details.to_owned());
    
    assert_eq!(error.code(), -32_605);
    assert!(error.to_string().contains("Serialization error"));
}

#[rstest]
#[case::config_field("database_url", "missing required field")]
#[case::invalid_value("port", "must be between 1 and 65535")]
#[case::type_mismatch("timeout", "expected number, got string")]
fn test_configuration_errors(#[case] field: &str, #[case] message: &str) {
    let error = McpError::configuration(field.to_owned(), message.to_owned());
    
    assert_eq!(error.code(), -32_606);
    assert!(error.to_string().contains("Configuration error"));
    assert!(error.to_string().contains(field));
}

#[rstest]
#[case::version_mismatch("2025-06-18", "2024-11-05")]
#[case::empty_versions("", "1.0.0")]
#[case::malformed_versions("invalid", "also-invalid")]
fn test_protocol_version_mismatch(#[case] expected: &str, #[case] received: &str) {
    let error = McpError::protocol_version_mismatch(expected.to_owned(), received.to_owned());
    
    assert_eq!(error.code(), -32_000);
    assert!(error.to_string().contains("MCP protocol version mismatch"));
    assert!(error.to_string().contains(expected));
    assert!(error.to_string().contains(received));
}

#[rstest]
#[case::network_error("http", "connection refused")]
#[case::timeout_error("websocket", "operation timed out")]
#[case::ssl_error("https", "certificate verification failed")]
fn test_transport_errors(#[case] transport_type: &str, #[case] details: &str) {
    let error = McpError::transport(transport_type.to_owned(), details.to_owned());
    
    assert_eq!(error.code(), -32_604);
    assert!(error.to_string().contains("Transport error"));
}

#[rstest]
#[case::execution_failure("component_list", "CLI command failed")]
#[case::timeout("worker_invoke", "command timed out")]
#[case::invalid_params("invalid_tool", "missing required parameter")]
fn test_tool_execution_errors(#[case] tool_name: &str, #[case] details: &str) {
    let error = McpError::tool_execution(tool_name.to_owned(), details.to_owned());
    
    assert_eq!(error.code(), -32_603);
    assert!(error.to_string().contains("Tool execution error"));
}

#[rstest]
fn test_json_rpc_error_with_custom_code() {
    let custom_code = -50000;
    let message = "Custom application error";
    
    let error = McpError::json_rpc(custom_code, message.to_owned());
    
    assert_eq!(error.code(), custom_code);
    assert!(error.to_string().contains(message));
}

#[rstest]
#[case::boundary_negative(i32::MIN)]
#[case::boundary_positive(i32::MAX)]
#[case::zero(0)]
#[case::standard_json_rpc(-32_603)]
fn test_error_code_boundaries(#[case] code: i32) {
    let error = McpError::json_rpc(code, "test message".to_owned());
    assert_eq!(error.code(), code);
}

#[rstest]
fn test_error_serialization_roundtrip(sample_context: String) {
    let original_error = McpError::resource_access(
        "component".to_owned(),
        "test-id".to_owned(),
        sample_context.clone(),
    );
    
    // Convert to JSON and back
    let json = serde_json::to_string(&original_error).expect("should serialize");
    let deserialized: McpError = serde_json::from_str(&json).expect("should deserialize");
    
    assert_eq!(original_error.code(), deserialized.code());
    assert_eq!(original_error.to_string(), deserialized.to_string());
}

#[rstest]
fn test_error_chain_preservation() {
    // Create a chain of errors to test context preservation
    let root_cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let serialization_error = McpError::serialization(
        "read_file".to_owned(),
        format!("failed to read configuration: {}", root_cause),
    );
    
    let error_chain = format!("{:?}", serialization_error);
    assert!(error_chain.contains("file not found"));
    assert!(error_chain.contains("read_file"));
}

#[rstest]
fn test_error_display_formatting() {
    let error = McpError::tool_execution(
        "component_add".to_owned(),
        "Component 'example' already exists in project".to_owned(),
    );
    
    let display = format!("{}", error);
    let debug = format!("{:?}", error);
    
    // Display should be user-friendly
    assert!(display.contains("Tool execution error"));
    assert!(display.contains("component_add"));
    
    // Debug should contain technical details
    assert!(debug.contains("ToolExecution"));
}

#[rstest]
fn test_concurrent_error_creation() {
    use std::thread;
    
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                McpError::invalid_request(format!("concurrent error {}", i))
            })
        })
        .collect();
    
    let errors: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    assert_eq!(errors.len(), 10);
    for (i, error) in errors.iter().enumerate() {
        assert_eq!(error.code(), -32_600);
        assert!(error.to_string().contains(&format!("concurrent error {}", i)));
    }
}

#[rstest]
#[case::max_length(&"x".repeat(10_000))]
#[case::empty("")]
#[case::null_chars("test\0with\0nulls")]
fn test_error_message_edge_cases(#[case] message: &str) {
    let error = McpError::invalid_request(message.to_owned());
    
    // Should not panic or cause issues
    let serialized = serde_json::to_string(&error);
    assert!(serialized.is_ok());
    
    let display = format!("{}", error);
    assert!(!display.is_empty());
}

#[rstest]
fn test_error_source_trait_implementation() {
    let error = McpError::internal("internal error".to_owned());
    
    // Test that the error implements std::error::Error properly
    assert!(std::error::Error::source(&error).is_none());
    
    let error_display = format!("{}", error);
    assert!(!error_display.is_empty());
}

/// Property-based testing helper for error codes
#[rstest]
#[case(-32_608)]
#[case(-32_000)]
#[case(-31_999)]
fn test_json_rpc_error_code_ranges(#[case] code: i32) {
    let error = McpError::json_rpc(code, "test".to_owned());
    assert_eq!(error.code(), code);
    
    // Should be serializable regardless of code
    let json_result = serde_json::to_string(&error);
    assert!(json_result.is_ok());
}

#[rstest]
fn test_error_memory_usage() {
    use std::mem;
    
    let error = McpError::resource_access(
        "test".to_owned(),
        "id".to_owned(),
        "details".to_owned(),
    );
    
    // Ensure errors don't consume excessive memory
    let size = mem::size_of_val(&error);
    assert!(size < 1000, "Error should not consume excessive memory: {} bytes", size);
}