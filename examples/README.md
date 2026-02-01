# DLMS/COSEM Rust Examples

This directory contains comprehensive examples demonstrating how to use the DLMS/COSEM Rust library.

## Examples Overview

### Client Examples

| Example | Description |
|---------|-------------|
| `basic_client` | Simple client that connects to a meter and reads attributes |
| `batch_operations` | Batch read/write operations for efficiency |
| `block_transfer` | Large value transfer using block transfer |
| `object_browser` | Discovering and browsing COSEM objects on a meter |
| `connection_pool` | Managing multiple reusable connections |
| `event_handling` | Event notification handling and subscription |

### Server Examples

| Example | Description |
|---------|-------------|
| `basic_server` | Basic server with COSEM object registration |
| `server_statistics` | Server statistics monitoring and reporting |

## Running the Examples

Make sure you have the Rust toolchain installed:

```bash
# Run a specific example
cargo run --example basic_client

# Run with arguments
cargo run --example basic_client -- --help

# Run server examples
cargo run --example basic_server
cargo run --example server_statistics
```

## Common Patterns

### 1. Creating a Connection

```rust
use dlms_client::{ConnectionBuilder, DlmsClient};
use dlms_client::ClientConfig;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build connection using fluent API
    let mut connection = ConnectionBuilder::new()
        .tcp()                              // Use TCP transport
        .wrapper()                          // Use Wrapper protocol
        .logical_name("127.0.0.1:4059")     // Server address
        .with_authentication("low", "PASSWORD")
        .connect()
        .await?;

    // Wrap with high-level client
    let config = ClientConfig::new()
        .with_auto_retry(true)
        .with_timeout(Duration::from_secs(30));

    let mut client = DlmsClient::with_config(connection, config);

    Ok(())
}
```

### 2. Reading Attributes

```rust
use dlms_core::ObisCode;

// Read with type inference
let value: u32 = client.get_attribute_typed(
    ObisCode::new(1, 1, 1, 8, 0, 255),  // Active energy OBIS
    3,                                    // Class ID (Register)
    2,                                    // Attribute ID (value)
).await?;

// Read raw DataObject
let value = client.get_attribute(
    ObisCode::new(1, 1, 1, 8, 0, 255),
    3,
    2
).await?;
```

### 3. Writing Attributes

```rust
use dlms_core::DataObject;

// Single attribute write
client.set_attribute(
    ObisCode::new(1, 1, 1, 8, 0, 255),
    3,
    2,
    DataObject::Unsigned32(12345)
).await?;

// Batch write
use dlms_client::{BatchWriter, AttributeValue};

let mut batch_writer = BatchWriter::new(&mut connection);
let attributes = vec![
    AttributeValue::new(obis1, 3, 2, DataObject::Unsigned32(100)),
    AttributeValue::new(obis2, 3, 2, DataObject::Unsigned32(200)),
];
batch_writer.write_attributes(attributes).await?;
```

### 4. Handling Large Values

```rust
use dlms_client::{BlockTransferWriter, BlockTransferConfig};

let config = BlockTransferConfig::new()
    .with_max_block_size(1024)
    .with_timeout(Duration::from_secs(30));

let mut writer = BlockTransferWriter::with_config(&mut connection, config);

// Write large data automatically
let large_data = vec![0u8; 5000];
writer.write_attribute_raw(
    ObisCode::new(0, 0, 1, 0, 0, 255),
    1,
    2,
    &large_data,
).await?;
```

### 5. Creating a Server

```rust
use dlms_server::{DlmsServer, ServerConfig};
use dlms_interface::{Register, CosemObject};
use dlms_core::ObisCode;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = DlmsServer::new();

    // Register COSEM objects
    let register = Register::new(ObisCode::new(1, 1, 1, 8, 0, 255));
    register.set_attribute(2, DataObject::Unsigned32(0)).await?;
    server.register_object(Arc::new(register)).await?;

    // Start server
    server.serve().await?;

    Ok(())
}
```

## Advanced COSEM Object Usage

### Register with Change Notifications

The Register interface class supports change notification callbacks:

```rust
use dlms_interface::{Register, ScalerUnit, RegisterChangeCallback};
use dlms_core::{ObisCode, DataObject};
use std::sync::Arc;

let register = Register::new(
    ObisCode::new(1, 1, 1, 8, 0, 255),
    DataObject::Unsigned32(1000),
    ScalerUnit::new(-1, 30), // 0.1 kWh
    None
);

// Register a callback for value changes
register.register_change_callback(
    "monitor".to_string(),
    Arc::new(|new_value| {
        println!("Value changed to: {:?}", new_value);
    })
).await?;

// Arithmetic operations
register.add(500).await?;        // Add 500
register.multiply(1.1).await?;   // Multiply by 1.1
register.reset().await?;         // Reset to zero

// Validation
let is_valid = register.is_within_range(0.0, 2000.0).await?;
println!("Value type: {}", register.value_type().await);
```

### Profile Generic Buffer Management

The ProfileGeneric interface class provides efficient buffer access:

```rust
use dlms_interface::ProfileGeneric;
use dlms_core::datatypes::CosemDateTime;

let profile = ProfileGeneric::with_default_obis(100);

// Check buffer status
if profile.is_buffer_full().await {
    println!("Buffer full: {}% used", profile.buffer_usage_percent().await);
}

// Read ranges efficiently
let recent = profile.get_newest_entries(10).await;
let oldest = profile.get_oldest_entries(10).await;
let range = profile.get_range(5, Some(15)).await;

// Time-based filtering
let from = CosemDateTime::new(2024, 1, 1, 0, 0, 0, 0, &[])?;
let to = CosemDateTime::new(2024, 12, 31, 23, 59, 59, 0, &[])?;
let entries = profile.get_entries_by_time_range(&from, &to).await;

// Flexible searching with predicates
let high_values = profile.find_entries(|entry| {
    entry.values.get(0).map_or(false, |v| {
        matches!(v, DataObject::Unsigned64(n) if *n > 1000)
    })
}).await;
```

## Transport Options

The library supports multiple transport combinations:

| Transport | Description | Builder Chain |
|-----------|-------------|---------------|
| HDLC over TCP | HDLC framing over TCP | `.tcp().hdlc()` |
| Wrapper over TCP | TCP/IP wrapper | `.tcp().wrapper()` |
| HDLC over Serial | HDLC over RS485/RS232 | `.serial("/dev/ttyUSB0").hdlc()` |
| Wrapper over Serial | Serial wrapper | `.serial("/dev/ttyUSB0").wrapper()` |

## Addressing Modes

### Logical Name (LN) Addressing

Uses 6-byte OBIS codes for object identification:

```rust
.logical_name("127.0.0.1:4059")
```

### Short Name (SN) Addressing

Uses 16-bit base names for compact addressing:

```rust
.short_name("127.0.0.1:4059")
```

## Authentication

The library supports multiple authentication levels:

| Level | Description | Usage |
|-------|-------------|-------|
| `low` | Low-level authentication | `.with_authentication("low", "password")` |
| `high` | High-level authentication | `.with_authentication("high", "password")` |
| `hls_gmac` | HLS with GMAC | `.with_authentication("hls_gmac", "password")` |

## Error Handling

Most operations return `DlmsResult<T>` which can be handled using Rust's standard error handling:

```rust
match client.get_attribute(obis, class_id, attr_id).await {
    Ok(value) => println!("Value: {}", value),
    Err(dlms_core::DlmsError::AccessDenied(msg)) => {
        eprintln!("Access denied: {}", msg);
    }
    Err(dlms_core::DlmsError::Timeout) => {
        eprintln!("Request timed out");
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Additional Resources

- [DLMS/COSEM Blue Book](https://dlms.com/) - Protocol specification
- [DLMS/COSEM Green Book](https://dlms.com/) - Object modeling
- [OBIS Code System](https://dlms.com/) - Identification system
- Main README.md - Project overview and architecture

## Troubleshooting

### Connection Issues

1. **Timeout**: Increase timeout in `ClientConfig`
2. **Authentication failed**: Verify password and authentication level
3. **Handshake failed**: Check if the server uses HDLC or Wrapper

### Attribute Access Issues

1. **Access denied**: Check ACL settings on the server
2. **Invalid attribute**: Verify OBIS code, class ID, and attribute ID
3. **Type mismatch**: Use appropriate type conversion methods

### Performance Tips

1. Use `BatchReader` for multiple attribute reads
2. Use `BatchWriter` for multiple attribute writes
3. Use `ConnectionPool` for managing multiple connections
4. Adjust block size in `BlockTransferConfig` for large values
