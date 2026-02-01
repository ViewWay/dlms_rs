//! Event Handling Example
//!
//! This example demonstrates how to use the event handling
//! functionality in DLMS/COSEM client and server.

use dlms_client::{ConnectionBuilder, EventHandler, EventFilter, EventCallback};
use dlms_client::event_handler::EventNotification;
use dlms_core::ObisCode;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("DLMS/COSEM Event Handling Example");
    println!("=================================\n");

    // Build connection
    let mut connection = ConnectionBuilder::new()
        .tcp()
        .wrapper()
        .logical_name("127.0.0.1:4059")
        .with_authentication("low", "MD5_PASSWORD_HERE")
        .connect()
        .await?;

    println!("Connected to meter");

    // Create event handler
    let mut event_handler = EventHandler::new(&mut connection);

    // === Example 1: Subscribe to all events ===
    println!("\n--- Example 1: Subscribe to All Events ---");

    let all_filter = EventFilter::all();
    event_handler.subscribe(all_filter).await?;
    println!("Subscribed to all events");

    // === Example 2: Subscribe to specific OBIS codes ===
    println!("\n--- Example 2: Subscribe to Specific OBIS Codes ---");

    let mut obis_filter = EventFilter::new()
        .with_obis_code(ObisCode::new(1, 1, 1, 8, 0, 255))  // Active energy
        .with_obis_code(ObisCode::new(1, 0, 32, 7, 0, 255));  // Voltage

    // Add severity filter
    obis_filter = obis_filter.with_min_severity(1);  // Info and above

    event_handler.subscribe(obis_filter).await?;
    println!("Subscribed to specific OBIS codes");

    // === Example 3: Register callback for events ===
    println!("\n--- Example 3: Register Event Callback ---");

    let callback: EventCallback = Arc::new(|notification: EventNotification| {
        Box::pin(async move {
            match notification.severity {
                0 => println!("[DEBUG] {}", notification.message),
                1 => println!("[INFO] {}", notification.message),
                2 => println!("[WARNING] {}", notification.message),
                3 => println!("[ERROR] {}", notification.message),
                _ => println!("[UNKNOWN] {}", notification.message),
            }
            Ok(())
        })
    });

    event_handler.set_callback(callback).await;
    println!("Event callback registered");

    // === Example 4: Listen for events ===
    println!("\n--- Example 4: Listen for Events ---");

    let listener = event_handler.create_listener().await?;

    // Spawn a task to listen for events
    let listener_handle = tokio::spawn(async move {
        let mut listener = listener;
        loop {
            match listener.next_event().await {
                Ok(Some(event)) => {
                    println!("Event received:");
                    println!("  Severity: {}", event.severity);
                    println!("  OBIS: {}", event.obis_code);
                    println!("  Message: {}", event.message);
                    println!("  Timestamp: {:?}", event.timestamp);
                }
                Ok(None) => {
                    println!("Event stream ended");
                    break;
                }
                Err(e) => {
                    eprintln!("Error receiving event: {}", e);
                    break;
                }
            }
        }
    });

    // === Example 5: Event statistics ===
    println!("\n--- Example 5: Event Statistics ---");

    // Let some time pass for events to be collected
    tokio::time::sleep(Duration::from_millis(100)).await;

    let stats = event_handler.get_statistics().await;
    println!("Events received: {}", stats.total_events);
    println!("Events by severity:");
    println!("  Debug: {}", stats.debug_count);
    println!("  Info: {}", stats.info_count);
    println!("  Warning: {}", stats.warning_count);
    println!("  Error: {}", stats.error_count);

    // === Example 6: Unsubscribe from events ===
    println!("\n--- Example 6: Unsubscribe ---");

    // Unsubscribe from specific OBIS code
    let voltage_filter = EventFilter::new()
        .with_obis_code(ObisCode::new(1, 0, 32, 7, 0, 255));

    event_handler.unsubscribe(voltage_filter).await?;
    println!("Unsubscribed from voltage events");

    // === Example 7: Clear all subscriptions ===
    println!("\n--- Example 7: Clear All Subscriptions ---");

    // To clear all subscriptions:
    // event_handler.clear_subscriptions().await?;
    println!("(Skipped for this example)");

    // Wait a bit for the listener task
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Cancel the listener task
    listener_handle.abort();

    println!("\nExample complete!");

    Ok(())
}
