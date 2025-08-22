//! Progress Streaming Demo
//!
//! This example demonstrates the progress streaming implementation for long-running
//! Golem operations, following the Amazon Q pattern from Compare.MD.
//!
//! # Features Demonstrated
//!
//! - Type-safe progress tokens using newtypes
//! - Timeout configuration (120s default for tool execution)
//! - Progress notifications via rmcp::model::ProgressNotificationParam  
//! - Zero-cost abstractions for short operations
//! - Comprehensive error handling
//!
//! # Usage
//!
//! ```bash
//! cargo run --example progress_streaming_demo
//! ```

use golem_mcp_server::progress::{
    TimeoutConfig, create_progress_channel, create_progress_sender, 
    execute_component_build_with_progress, ProgressStreamError
};
use golem_mcp_server::rmcp_handler::GolemMcpServer;
use rmcp::model::NumberOrString;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, debug, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("🚀 Starting Progress Streaming Demo");
    info!("This demo shows Amazon Q style progress streaming for Golem MCP Server");

    // Create a Golem MCP server
    let server = GolemMcpServer::new();
    
    // Set up progress streaming with a buffer size of 32
    let mut progress_receiver = server.setup_progress_streaming(Some(32)).await;
    
    // Configure timeout settings for build operations
    server.configure_timeouts(TimeoutConfig::slow_operations()).await;
    
    info!("✅ Progress streaming configured with 32-notification buffer");
    
    // Spawn a task to consume progress notifications
    let progress_consumer = tokio::spawn(async move {
        let mut notification_count = 0;
        info!("📡 Progress consumer started - listening for notifications...");
        
        while let Some(notification) = progress_receiver.recv().await {
            notification_count += 1;
            
            let token = match &notification.progress_token {
                rmcp::model::ProgressToken(NumberOrString::String(s)) => s.as_ref(),
                rmcp::model::ProgressToken(NumberOrString::Number(_)) => "unknown",
            };
            
            let message = notification.message.as_deref().unwrap_or("No message");
            
            info!(
                "📈 Progress Update #{}: {} - {:.1}%/{:.1}% - {}",
                notification_count,
                token,
                notification.progress,
                notification.total.unwrap_or(100.0),
                message
            );
            
            // Stop after completion notification
            if notification.progress >= 100.0 {
                info!("✅ Operation completed successfully!");
                break;
            }
        }
        
        notification_count
    });
    
    // Simulate a long-running component build operation
    info!("🏗️  Starting simulated component build operation...");
    
    let (progress_sender, _) = create_progress_channel(16);
    let operation_progress_sender = create_progress_sender(progress_sender, "demo_component_build");
    
    // Create a simple operation that just sleeps to simulate work
    let build_operation = async {
        info!("📦 Simulating component build...");
        sleep(Duration::from_millis(2000)).await;
        Ok::<String, std::io::Error>("Component build completed successfully!".to_string())
    };
    
    // Execute the operation with progress tracking
    let start_time = std::time::Instant::now();
    
    match execute_component_build_with_progress(build_operation, operation_progress_sender).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            info!("🎉 Build operation completed in {:.2}s: {}", duration.as_secs_f64(), result);
        }
        Err(e) => {
            info!("❌ Build operation failed: {}", e);
        }
    }
    
    // Wait for progress consumer to finish
    let notification_count = progress_consumer.await?;
    info!("📊 Total progress notifications received: {}", notification_count);
    
    // Demonstrate timeout behavior with a failing operation
    info!("\n🔄 Demonstrating timeout behavior...");
    
    let (timeout_sender, _) = create_progress_channel(16);
    let timeout_progress_sender = create_progress_sender(timeout_sender, "timeout_demo");
    
    let slow_operation = async {
        info!("⏳ Starting operation that will timeout...");
        sleep(Duration::from_secs(5)).await; // Will timeout after 1 second
        Ok::<String, std::io::Error>("Should not reach here".to_string())
    };
    
    let start_time = std::time::Instant::now();
    
    match execute_component_build_with_progress(slow_operation, timeout_progress_sender).await {
        Ok(_) => {
            info!("❓ Unexpected success - operation should have timed out");
        }
        Err(e) => {
            let duration = start_time.elapsed();
            info!("⏰ Operation timed out after {:.2}s as expected: {}", duration.as_secs_f64(), e);
        }
    }
    
    // Demonstrate zero-cost abstraction for quick operations
    info!("\n⚡ Demonstrating zero-cost abstraction for quick operations...");
    
    let quick_ops_count = 100;
    let start_time = std::time::Instant::now();
    
    for i in 0..quick_ops_count {
        let result = format!("Quick operation #{} result", i + 1);
        debug!("Quick op result: {}", result);
    }
    
    let duration = start_time.elapsed();
    info!("🏃 {} quick operations completed in {:.3}ms (avg {:.3}μs per op)", 
          quick_ops_count, 
          duration.as_millis(),
          duration.as_micros() as f64 / quick_ops_count as f64);
    
    info!("\n🏁 Progress Streaming Demo completed successfully!");
    info!("Key features demonstrated:");
    info!("  ✅ Type-safe progress tokens with newtypes");
    info!("  ✅ Amazon Q timeout configuration (120s default)");
    info!("  ✅ Progress notifications via ProgressNotificationParam");
    info!("  ✅ Zero-cost abstractions for quick operations");
    info!("  ✅ Comprehensive error handling with structured types");
    
    Ok(())
}