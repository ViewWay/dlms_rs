//! Batch Operations Example
//!
//! This example demonstrates how to use BatchReader and BatchWriter
//! for efficient multi-attribute operations.

use dlms_client::{
    ConnectionBuilder, BatchReader, BatchWriter, AttributeReference, AttributeValue
};
use dlms_core::{ObisCode, DataObject};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("DLMS/COSEM Batch Operations Example");
    println!("===================================\n");

    // Build connection
    let mut connection = ConnectionBuilder::new()
        .tcp()
        .wrapper()
        .logical_name("127.0.0.1:4059")
        .with_authentication("low", "MD5_PASSWORD_HERE")
        .connect()
        .await?;

    println!("Connected to meter");

    // === Batch Read Example ===
    println!("\n--- Batch Read ---");
    let mut batch_reader = BatchReader::new(&mut connection)
        .with_max_per_request(10);

    // Define attributes to read
    let attributes = vec![
        // Active energy
        AttributeReference::new(ObisCode::new(1, 1, 1, 8, 0, 255), 3, 2),
        // Reactive energy
        AttributeReference::new(ObisCode::new(1, 1, 1, 9, 0, 255), 3, 2),
        // Voltage
        AttributeReference::new(ObisCode::new(1, 0, 32, 7, 0, 255), 3, 2),
        // Current
        AttributeReference::new(ObisCode::new(1, 0, 31, 7, 0, 255), 3, 2),
        // Power factor
        AttributeReference::new(ObisCode::new(1, 0, 33, 7, 0, 255), 3, 2),
    ];

    match batch_reader.read_attributes(attributes).await {
        Ok(result) => {
            println!("Read {} attributes successfully:", result.successful.len());
            for success in &result.successful {
                println!("  {}:{} = {}", success.obis_code, success.attribute_id, success.value);
            }

            if !result.failed.is_empty() {
                println!("\nFailed to read {} attributes:", result.failed.len());
                for error in &result.failed {
                    println!("  {}:{} - {}", error.obis_code, error.attribute_id, error.error);
                }
            }
        }
        Err(e) => {
            eprintln!("Batch read error: {}", e);
        }
    }

    // === Batch Write Example ===
    println!("\n--- Batch Write ---");

    // Define attributes to write (careful with real meters!)
    let write_attributes = vec![
        // Example: Write a display attribute (safe to modify)
        AttributeValue::new(
            ObisCode::new(0, 0, 1, 0, 0, 255),  // Data object
            1,                                    // Class ID
            2,                                    // Attribute ID (value)
            DataObject::OctetString(b"Test Message".to_vec()),
        ),
    ];

    let mut batch_writer = BatchWriter::new(&mut connection)
        .with_max_per_request(10)
        .with_stop_on_error(false);

    match batch_writer.write_attributes(write_attributes).await {
        Ok(result) => {
            println!("Write operation complete:");
            println!("  Successful: {}", result.successful.len());
            println!("  Failed: {}", result.failed.len());
        }
        Err(e) => {
            eprintln!("Batch write error: {}", e);
        }
    }

    println!("\nExample complete!");

    Ok(())
}
