//! Block Transfer Example
//!
//! This example demonstrates how to use BlockTransferWriter
//! for writing large attribute values.

use dlms_client::{ConnectionBuilder, BlockTransferWriter, BlockTransferConfig};
use dlms_core::{ObisCode, DataObject};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("DLMS/COSEM Block Transfer Example");
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

    // Create block transfer writer with custom configuration
    let config = BlockTransferConfig::new()
        .with_max_block_size(512)        // 512 bytes per block
        .with_timeout(Duration::from_secs(30))
        .with_max_retries(3);

    let mut writer = BlockTransferWriter::with_config(&mut connection, config);

    // === Write a large octet string ===
    println!("\n--- Writing Large Octet String ---");

    // Create 5KB of data (larger than default block size)
    let large_data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
    println!("Data size: {} bytes", large_data.len());

    let obis_code = ObisCode::new(0, 0, 1, 0, 0, 255);  // Data object

    match writer.write_attribute_raw(
        obis_code,
        1,  // Class ID (Data)
        2,  // Attribute ID (value)
        &large_data,
    ).await {
        Ok(_) => {
            println!("Successfully wrote large data using block transfer");
        }
        Err(e) => {
            eprintln!("Error writing large data: {}", e);
        }
    }

    // === Write multiple large values ===
    println!("\n--- Writing Multiple Large Values ---");

    let attributes = vec![
        (
            ObisCode::new(0, 0, 1, 0, 0, 255),
            1,
            2,
            DataObject::OctetString(vec![0u8; 1000]),
        ),
        (
            ObisCode::new(0, 0, 2, 0, 0, 255),
            1,
            2,
            DataObject::OctetString(vec![1u8; 1000]),
        ),
    ];

    match writer.write_attributes(attributes).await {
        Ok(_) => {
            println!("Successfully wrote multiple large values");
        }
        Err(e) => {
            eprintln!("Error writing multiple values: {}", e);
        }
    }

    // === Write typed data ===
    println!("\n--- Writing Typed Data ---");

    let typed_data = DataObject::OctetString(b"Hello, DLMS/COSEM!".to_vec());

    match writer.write_attribute(
        ObisCode::new(0, 0, 1, 0, 0, 255),
        1,
        2,
        typed_data,
    ).await {
        Ok(_) => {
            println!("Successfully wrote typed data");
        }
        Err(e) => {
            eprintln!("Error writing typed data: {}", e);
        }
    }

    println!("\nExample complete!");

    Ok(())
}
