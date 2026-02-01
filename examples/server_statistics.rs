//! Server Statistics Example
//!
//! This example demonstrates how to use the server statistics
//! functionality to monitor DLMS/COSEM server operations.

use dlms_server::{DlmsServer, ServerConfig};
use dlms_server::request_stats::{RequestType, ServerRequestStats};
use dlms_interface::{Register, Clock, CosemObject};
use dlms_core::{ObisCode, DataObject};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("DLMS/COSEM Server Statistics Example");
    println!("=====================================\n");

    // Create server with configuration
    let config = ServerConfig::new()
        .with_listen_address("0.0.0.0:4059")
        .with_max_clients(10)
        .with_timeout(Duration::from_secs(300))
        .with_low_level_authentication("MD5_PASSWORD_HERE");

    let server = DlmsServer::new();

    // Register COSEM objects
    let energy_register = Register::new(ObisCode::new(1, 1, 1, 8, 0, 255));
    energy_register.set_unit(dlms_interface::Unit::KilowattHour);
    energy_register.set_attribute(2, DataObject::Unsigned32(1234567)).await?;
    server.register_object(Arc::new(energy_register)).await?;

    let clock = Clock::with_default_obis();
    server.register_object(Arc::new(clock)).await?;

    println!("Registered {} COSEM objects", server.get_all_objects().await.len());
    println!("Server listening on 0.0.0.0:4059\n");

    // Initialize request statistics
    let mut stats = ServerRequestStats::new();

    // Simulate some requests for demonstration
    println!("Simulating requests for statistics demonstration...\n");

    // Simulate various request types
    for i in 0..10 {
        // Start tracking
        let tracker = stats.record_request_start(RequestType::Get);

        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Record completion
        let success = i < 9; // 9 successful, 1 failed
        stats.record_request_completion(tracker, success, 100, 50);
    }

    // Simulate SET requests
    for i in 0..5 {
        let tracker = stats.record_request_start(RequestType::Set);
        tokio::time::sleep(Duration::from_millis(15)).await;
        stats.record_request_completion(tracker, true, 50, 20);
    }

    // Simulate ACTION requests
    for i in 0..3 {
        let tracker = stats.record_request_start(RequestType::Action);
        tokio::time::sleep(Duration::from_millis(5)).await);
        stats.record_request_completion(tracker, true, 30, 15);
    }

    // Record some errors
    stats.record_error("Access denied".to_string());
    stats.record_error("Invalid attribute ID".to_string());
    stats.record_error("Access denied".to_string()); // Duplicate error

    // Display statistics
    println!("--- Server Statistics Summary ---\n");
    let summary = stats.summary();
    println!("{}", summary.format());

    println!("\n--- Request Type Breakdown ---");
    for (req_type, count) in &summary.request_type_counts {
        println!("  {}: {} requests", req_type.name(), count);
    }

    println!("\n--- Performance Metrics ---");
    println!("  Average processing time: {:?}", summary.requests_per_second);

    // Display connection statistics
    println!("\n--- Connection Statistics ---");
    let conn_stats = server.get_connection_statistics().await;
    println!("  Active connections: {}", conn_stats.active_connections);
    println!("  Max connections: {:?}", conn_stats.max_connections);
    println!("  Utilization: {:.1}%",
        conn_stats.utilization_percent().unwrap_or(0.0));

    // Demonstrate periodic statistics reporting
    println!("\n--- Starting Periodic Statistics Report ---");
    let server_for_report = server.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10));
        loop {
            ticker.tick().await;

            let stats = server_for_report.get_connection_statistics().await;
            log::info!(
                "Connections: {} active, {} max, {:.1}% utilized",
                stats.active_connections,
                stats.max_connections.map(|m| m.to_string()).unwrap_or("unlimited".to_string()),
                stats.utilization_percent().unwrap_or(0.0)
            );
        }
    });

    println!("Server running. Press Ctrl+C to stop.");

    // In a real application, you would start the server listener here
    // server.serve().await?;

    Ok(())
}
