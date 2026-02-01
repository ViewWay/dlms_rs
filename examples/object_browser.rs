//! Object Browser Example
//!
//! This example demonstrates how to use ObjectBrowser
//! to discover and navigate COSEM objects on a meter.

use dlms_client::{ConnectionBuilder, ObjectBrowser};
use dlms_core::ObisCode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("DLMS/COSEM Object Browser Example");
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

    let mut browser = ObjectBrowser::new(&mut connection);

    // === Browse all objects ===
    println!("\n--- Browsing All Objects ---");

    match browser.browse_all().await {
        Ok(objects) => {
            println!("Found {} COSEM objects:\n", objects.len());

            for obj in &objects {
                println!("Class: {} ({})", obj.class_id, obj.class_name);
                println!("  OBIS:  {}", obj.obis_code);
                println!("  Name: {}", obj.name);
                println!("  Description: {}", obj.description);
                println!("  Attributes: {}", obj.attribute_count);
                println!("  Methods: {}", obj.method_count);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error browsing objects: {}", e);
        }
    }

    // === Browse by class ===
    println!("\n--- Browsing Register Objects ---");

    match browser.browse_class(3).await {  // Class 3 = Register
        Ok(objects) => {
            println!("Found {} Register objects:\n", objects.len());

            for obj in &objects {
                println!("  OBIS: {}", obj.obis_code);
                println!("  Name: {}", obj.name);
            }
        }
        Err(e) => {
            eprintln!("Error browsing register objects: {}", e);
        }
    }

    // === Get object attributes ===
    println!("\n--- Getting Object Attributes ---");

    let obis_code = ObisCode::new(1, 1, 1, 8, 0, 255);  // Active energy

    match browser.get_object_attributes(obis_code, 3).await {
        Ok(attributes) => {
            println!("Attributes for {} (Register):\n", obis_code);

            for attr in &attributes {
                println!("  Attribute {}: {}", attr.id, attr.name);
                println!("    Type: {:?}", attr.data_type);
                println!("    Access: {:?}", attr.access_mode);
                println!("    Description: {}", attr.description);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error getting object attributes: {}", e);
        }
    }

    // === Search by name pattern ===
    println!("\n--- Searching for 'Energy' Objects ---");

    match browser.search_by_name("energy").await {
        Ok(objects) => {
            println!("Found {} objects matching 'energy':\n", objects.len());

            for obj in &objects {
                println!("  {} - {} ({})", obj.obis_code, obj.name, obj.class_name);
            }
        }
        Err(e) => {
            eprintln!("Error searching objects: {}", e);
        }
    }

    // === Find by OBIS pattern ===
    println!("\n--- Finding Objects by OBIS Pattern ---");

    match browser.find_by_obis_pattern("1.1.1.*.0.255").await {
        Ok(objects) => {
            println!("Found {} objects matching OBIS pattern:\n", objects.len());

            for obj in &objects {
                println!("  {} - {} ({})", obj.obis_code, obj.name, obj.class_name);
            }
        }
        Err(e) => {
            eprintln!("Error finding by OBIS pattern: {}", e);
        }
    }

    println!("\nExample complete!");

    Ok(())
}
