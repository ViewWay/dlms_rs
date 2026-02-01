//! Basic DLMS/COSEM Server Example
//!
//! This example demonstrates how to create a simple DLMS server
//! that registers COSEM objects and handles client connections.

use dlms_server::{DlmsServer, ServerConfig};
use dlms_interface::{Register, CosemObject, Clock};
use dlms_core::{ObisCode, DataObject};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("DLMS/COSEM Basic Server Example");
    println!("===============================\n");

    // Create server configuration
    let config = ServerConfig::new()
        .with_listen_address("0.0.0.0:4059")
        .with_max_clients(10)
        .with_timeout(Duration::from_secs(300))
        .with_low_level_authentication("MD5_PASSWORD_HERE");

    // Create server
    let server = DlmsServer::new(config);

    println!("Server configured on 0.0.0.0:4059");

    // === Register COSEM Objects ===

    // Active Energy Register (1.1.1.8.0.255)
    let energy_register = Register::new(ObisCode::new(1, 1, 1, 8, 0, 255));
    energy_register.set_unit(dlms_interface::Unit::KilowattHour);
    energy_register.set_attribute(2, DataObject::Unsigned32(1234567)).await?;
    server.register_object(Arc::new(energy_register)).await?;
    println!("Registered: Active Energy Register");

    // Voltage Register (1.0.32.7.0.255)
    let voltage_register = Register::new(ObisCode::new(1, 0, 32, 7, 0, 255));
    voltage_register.set_unit(dlms_interface::Unit::Volt);
    voltage_register.set_attribute(2, DataObject::Float32(230.5)).await?;
    server.register_object(Arc::new(voltage_register)).await?;
    println!("Registered: Voltage Register");

    // Current Register (1.0.31.7.0.255)
    let current_register = Register::new(ObisCode::new(1, 0, 31, 7, 0, 255));
    current_register.set_unit(dlms_interface::Unit::Ampere);
    current_register.set_attribute(2, DataObject::Float32(5.2)).await?;
    server.register_object(Arc::new(current_register)).await?;
    println!("Registered: Current Register");

    // Power Factor Register (1.0.33.7.0.255)
    let pf_register = Register::new(ObisCode::new(1, 0, 33, 7, 0, 255));
    pf_register.set_unit(dlms_interface::Unit::None);
    pf_register.set_attribute(2, DataObject::Float32(0.95)).await?;
    server.register_object(Arc::new(pf_register)).await?;
    println!("Registered: Power Factor Register");

    // Clock Object (0.0.1.0.0.255)
    let clock = Clock::new();
    clock.set_attribute(2, DataObject::OctetString(vec![0x07, 0xE6, 0x01, 0x18, 0x0C, 0x30, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00])).await?; // 2026-01-24 12:30:00
    server.register_object(Arc::new(clock)).await?;
    println!("Registered: Clock");

    println!("\nAll objects registered successfully");
    println!("Starting server...\n");

    // Spawn a task to simulate meter values changing
    let server_ref = server.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));
        loop {
            ticker.tick().await;

            // Simulate energy increment
            if let Some(obj) = server_ref.find_object(ObisCode::new(1, 1, 1, 8, 0, 255)) {
                if let Some(reg) = obj.as_any().downcast_ref::<Register>() {
                    if let Ok(DataObject::Unsigned32(value)) = reg.get_attribute(2).await {
                        let _ = reg.set_attribute(2, DataObject::Unsigned32(value + 1)).await;
                    }
                }
            }

            // Simulate voltage fluctuation
            if let Some(obj) = server_ref.find_object(ObisCode::new(1, 0, 32, 7, 0, 255)) {
                if let Some(reg) = obj.as_any().downcast_ref::<Register>() {
                    let voltage = 228.0 + (rand::random::<f32>() * 5.0); // 228-233V
                    let _ = reg.set_attribute(2, DataObject::Float32(voltage)).await;
                }
            }

            log::info!("Simulated meter values updated");
        }
    });

    // Start the server (this will block)
    match server.serve().await {
        Ok(_) => {
            println!("Server stopped gracefully");
        }
        Err(e) => {
            eprintln!("Server error: {}", e);
        }
    }

    Ok(())
}
