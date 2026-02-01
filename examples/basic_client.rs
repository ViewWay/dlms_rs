//! Basic DLMS/COSEM Client Example
//!
//! This example demonstrates how to create a simple DLMS client
//! that connects to a meter and reads attributes.

use dlms_client::{ConnectionBuilder, DlmsClient, ClientConfig};
use dlms_core::ObisCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("DLMS/COSEM Basic Client Example");
    println!("================================\n");

    // Build connection to a meter
    let mut connection = ConnectionBuilder::new()
        .tcp()
        .wrapper()
        .logical_name("127.0.0.1:4059")
        .with_authentication("low", "MD5_PASSWORD_HERE")
        .connect()
        .await?;

    println!("Connected to meter");

    // Create client with configuration
    let config = ClientConfig::new()
        .with_auto_retry(true)
        .with_timeout(Duration::from_secs(30));

    let mut client = DlmsClient::with_config(connection, config);

    // Read active energy register (OBIS: 1.1.1.8.0.255)
    println!("\nReading active energy register...");
    let obis_code = ObisCode::new(1, 1, 1, 8, 0, 255);

    match client.get_attribute(obis_code, 3, 2).await {
        Ok(value) => {
            println!("  Active energy: {}", value);
        }
        Err(e) => {
            eprintln!("  Error reading register: {}", e);
        }
    }

    // Read voltage (OBIS: 1.0.32.7.0.255)
    println!("\nReading voltage...");
    let voltage_obis = ObisCode::new(1, 0, 32, 7, 0, 255);

    match client.get_attribute(voltage_obis, 3, 2).await {
        Ok(value) => {
            println!("  Voltage: {} V", value);
        }
        Err(e) => {
            eprintln!("  Error reading voltage: {}", e);
        }
    }

    // Read current (OBIS: 1.0.31.7.0.255)
    println!("\nReading current...");
    let current_obis = ObisCode::new(1, 0, 31, 7, 0, 255);

    match client.get_attribute(current_obis, 3, 2).await {
        Ok(value) => {
            println!("  Current: {} A", value);
        }
        Err(e) => {
            eprintln!("  Error reading current: {}", e);
        }
    }

    println!("\nExample complete!");

    Ok(())
}
