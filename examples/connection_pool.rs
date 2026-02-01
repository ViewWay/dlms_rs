//! Connection Pool Example
//!
//! This example demonstrates how to use ConnectionPool
//! for managing multiple reusable connections.

use dlms_client::{
    ConnectionBuilder, ConnectionPool, ConnectionPoolConfig, ConnectionType, ConnectionKey
};
use dlms_core::ObisCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("DLMS/COSEM Connection Pool Example");
    println!("==================================\n");

    // Create pool configuration
    let config = ConnectionPoolConfig::new()
        .with_max_connections(10)
        .with_min_idle(2)
        .with_connection_timeout(Duration::from_secs(30))
        .with_idle_timeout(Duration::from_secs(300))
        .with_health_check_interval(Duration::from_secs(60));

    // Create connection pool
    let pool = ConnectionPool::new(config).await?;

    println!("Connection pool created");

    // === Add connections to the pool ===

    // Meter 1
    let key1 = ConnectionKey::new("meter1", ConnectionType::TcpWrapper);
    let conn1 = ConnectionBuilder::new()
        .tcp()
        .wrapper()
        .logical_name("127.0.0.1:4059")
        .with_authentication("low", "MD5_PASSWORD_HERE")
        .build()?;

    pool.add_connection(key1.clone(), conn1).await?;
    println!("Added meter1 to pool");

    // Meter 2
    let key2 = ConnectionKey::new("meter2", ConnectionType::TcpWrapper);
    let conn2 = ConnectionBuilder::new()
        .tcp()
        .wrapper()
        .logical_name("127.0.0.1:4060")
        .with_authentication("low", "MD5_PASSWORD_HERE")
        .build()?;

    pool.add_connection(key2.clone(), conn2).await?;
    println!("Added meter2 to pool");

    // === Get connection and read data ===

    println!("\n--- Reading from Meter 1 ---");
    if let Some(mut conn) = pool.get_connection(&key1).await? {
        let obis_code = ObisCode::new(1, 1, 1, 8, 0, 255);
        match conn.get_attribute(obis_code, 3, 2).await {
            Ok(value) => {
                println!("  Active energy: {}", value);
            }
            Err(e) => {
                eprintln!("  Error: {}", e);
            }
        }
        // Connection is automatically returned to pool when dropped
    }

    println!("\n--- Reading from Meter 2 ---");
    if let Some(mut conn) = pool.get_connection(&key2).await? {
        let obis_code = ObisCode::new(1, 1, 1, 8, 0, 255);
        match conn.get_attribute(obis_code, 3, 2).await {
            Ok(value) => {
                println!("  Active energy: {}", value);
            }
            Err(e) => {
                eprintln!("  Error: {}", e);
            }
        }
    }

    // === Parallel reads from multiple meters ===

    println!("\n--- Parallel Reads ---");

    let handles: Vec<_> = vec![key1.clone(), key2.clone()]
        .into_iter()
        .map(|key| {
            let pool = pool.clone();
            tokio::spawn(async move {
                if let Some(mut conn) = pool.get_connection(&key).await.ok().flatten() {
                    let obis_code = ObisCode::new(1, 1, 1, 8, 0, 255);
                    match conn.get_attribute(obis_code, 3, 2).await {
                        Ok(value) => Some((key.name().to_string(), value)),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            })
        })
        .collect();

    for handle in handles {
        if let Ok(Some((name, value))) = handle.await {
            println!("  {}: {}", name, value);
        }
    }

    // === Pool statistics ===

    let stats = pool.statistics();
    println!("\n--- Pool Statistics ---");
    println!("  Total connections: {}", stats.total_connections);
    println!("  Active connections: {}", stats.active_connections);
    println!("  Idle connections: {}", stats.idle_connections);
    println!("  Total acquires: {}", stats.total_acquires);
    println!("  Total releases: {}", stats.total_releases);

    // === Health check ===

    println!("\n--- Health Check ---");
    let health = pool.health_check().await?;
    println!("  Healthy connections: {}", health.healthy_count);
    println!("  Unhealthy connections: {}", health.unhealthy_count);

    println!("\nExample complete!");

    Ok(())
}
