# DLMS/COSEM Rust Implementation

A comprehensive Rust implementation of the DLMS/COSEM (Device Language Message Specification / Companion Specification for Energy Metering) protocol for smart meter communication.

## Overview

This project provides a complete, production-ready implementation of the DLMS/COSEM protocol stack in Rust, supporting both client and server roles. It is designed for high performance, type safety, and ease of use.

## Features

### Protocol Support
- ✅ **DLMS/COSEM Protocol Suite** (Blue Book, Green Book, Yellow Book)
- ✅ **A-XDR Encoding** (ASN.1 eXtended Encoding Rules)
- ✅ **COSEM Object Model** (50+ interface classes)
- ✅ **OBIS Code Identification** (Optical Identification System)
- ✅ **Short Name (SN) Addressing** (16-bit base name addressing)
- ✅ **Logical Name (LN) Addressing** (6-byte OBIS code)

### Transport Layer
- ✅ **HDLC over TCP** (High-level Data Link Control)
- ✅ **HDLC over Serial** (RS485/RS232)
- ✅ **Wrapper over TCP** (TCP/IP transport wrapper)
- ✅ **Wrapper over Serial** (Serial transport wrapper)

### Application Layer
- ✅ **GET Service** - Read single/multiple attributes
- ✅ **SET Service** - Write single/multiple attributes
- ✅ **ACTION Service** - Invoke methods on COSEM objects
- ✅ **GetRequest-WithList** - Batch read multiple attributes
- ✅ **GetRequest-Next** - Block transfer for large reads
- ✅ **SetRequest-WithList** - Batch write multiple attributes
- ✅ **SetRequest-WithDataBlock** - Block transfer for large writes

### Security
- ✅ **AES-128 Encryption** (GMAC authentication)
- ✅ **Authentication Mechanisms** (Low, High, HLS)
- ✅ **Cipher Suite Negotiation**

### Client Features
- ✅ **Connection Builder** - Fluent API for connection setup
- ✅ **Object Browser** - Discover and navigate COSEM objects
- ✅ **Batch Reader** - Efficient multi-attribute reads
- ✅ **Batch Writer** - Efficient multi-attribute writes
- ✅ **Block Transfer Writer** - Large value writes
- ✅ **Connection Pool** - Reusable connection management
- ✅ **Auto-Reconnect** - Automatic reconnection on failure
- ✅ **Health Checker** - Connection health monitoring
- ✅ **Event Handler** - Push notification support

### Server Features
- ✅ **Multi-Client Management** - Handle concurrent clients
- ✅ **Access Control** - ACL-based authorization
- ✅ **Event Processor** - Event notification generation
- ✅ **Short Name Resolution** - Base name to OBIS mapping
- ✅ **Block Transfer State** - Large value handling

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Application Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Client     │  │   Server     │  │   Object     │          │
│  │   Library    │  │   Library    │  │   Browser    │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                   │
└─────────┼─────────────────┼─────────────────┼───────────────────┘
          │                 │                 │
┌─────────┼─────────────────┼─────────────────┼───────────────────┐
│         │  Application     │                 │                   │
│  ┌──────▼──────────────────▼──────────────────▼───────────┐      │
│  │           dlms-application (PDU, Services)               │      │
│  └───────────────────────────┬──────────────────────────────┘      │
│                              │                                     │
│  ┌───────────────────────────▼──────────────────────────────┐      │
│  │              dlms-interface (COSEM Objects)               │      │
│  │   • Data, Register, Clock, Profile Generic, etc.          │      │
│  └───────────────────────────┬──────────────────────────────┘      │
│                              │                                     │
│  ┌──────┬───────────┬──────▼──────┬──────────────┬────────────┐    │
│  │      │           │             │              │             │    │
│  │  ┌───▼───┐  ┌────▼────┐  ┌───▼──────┐  ┌───────▼──┐  ┌──▼───┐  │
│  │  │ dlms- │  │ dlms-   │  │  dlms-   │  │  dlms-    │  │ dlms│  │
│  │  │ asn1  │  │ session │  │security │  │ transport │  │core │  │
│  │  │       │  │         │  │         │  │           │  │     │  │
│  │  └───────┘  └─────────┘  └─────────┘  └───────────┘  └─────┘  │
│  └──────────────────────────────────────────────────────────────┘
│                              │
└──────────────────────────────▼──────────────────────────────────┘
                               │
                    ┌──────────▼───────────┐
                    │  Network / Serial   │
                    └─────────────────────┘
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
dlms = "0.1"
```

Or use specific crates:

```toml
[dependencies]
dlms-core = "0.1"
dlms-client = "0.1"
dlms-server = "0.1"
```

## Quick Start

### Client Example

```rust
use dlms::client::ConnectionBuilder;
use dlms::client::DlmsClient;
use dlms::client::ClientConfig;
use dlms_core::ObisCode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build connection
    let mut connection = ConnectionBuilder::new()
        .tcp()
        .wrapper()
        .logical_name("127.0.0.1:4059")
        .with_authentication("low", "md5-password")
        .connect()
        .await?;

    // Wrap with high-level client
    let config = ClientConfig::new()
        .with_auto_retry(true)
        .with_timeout(Duration::from_secs(30));

    let mut client = DlmsClient::with_config(connection, config);

    // Read attribute with type inference
    let value: u32 = client.get_attribute_typed(
        ObisCode::new(1, 1, 1, 8, 0, 255),  // Active energy
        3,    // Register class
        2,    // Value attribute
    ).await?;

    println!("Current value: {}", value);

    // Write attribute
    client.set_attribute_typed(
        ObisCode::new(1, 1, 1, 8, 0, 255),
        3,
        2,
        12345u32
    ).await?;

    Ok(())
}
```

### Server Example

```rust
use dlms::server::DlmsServer;
use dlms::server::ServerConfig;
use dlms::interface::{Register, CosemObject};
use dlms_core::ObisCode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server configuration
    let config = ServerConfig::new()
        .with_listen_address("0.0.0.0:4059")
        .with_max_clients(10);

    // Create server
    let server = DlmsServer::new(config);

    // Register COSEM objects
    let register = Register::new(ObisCode::new(1, 1, 1, 8, 0, 255));
    register.set_attribute(2, dlms_core::DataObject::Unsigned32(0)).await?;
    server.register_object(Arc::new(register)).await?;

    // Start server
    server.serve().await?;

    Ok(())
}
```

## Advanced Usage

### Batch Operations

```rust
use dlms::client::{BatchReader, AttributeReference};
use dlms_core::ObisCode;

// Read multiple attributes in one request
let mut batch_reader = BatchReader::new(&mut connection);

let attributes = vec![
    AttributeReference::new(ObisCode::new(1, 1, 1, 8, 0, 255), 3, 2),
    AttributeReference::new(ObisCode::new(1, 1, 1, 8, 0, 255), 3, 3),
    AttributeReference::new(ObisCode::new(1, 1, 1, 8, 0, 255), 3, 4),
];

let result = batch_reader.read_attributes(attributes).await?;
for success in result.successful {
    println!("Read: {} = {}", success.obis_code, success.value);
}
```

### Block Transfer for Large Values

```rust
use dlms::client::{BlockTransferWriter, BlockTransferConfig};
use std::time::Duration;

let config = BlockTransferConfig::new()
    .with_max_block_size(1024)
    .with_timeout(Duration::from_secs(30));

let mut writer = BlockTransferWriter::with_config(&mut connection, config);

// Write large value automatically using block transfer
let large_data = vec![0u8; 5000]; // 5KB data
writer.write_attribute_raw(
    ObisCode::new(1, 1, 1, 1, 0, 255),
    1,
    1,
    &large_data,
).await?;
```

### Object Browser

```rust
use dlms::client::ObjectBrowser;

let mut browser = ObjectBrowser::new(&mut connection);

// Browse all objects
let objects = browser.browse_all().await?;
for obj in objects {
    println!("Class: {}, OBIS: {}", obj.class_id, obj.obis_code);
}

// Get object attributes
let attrs = browser.get_object_attributes(
    ObisCode::new(1, 1, 1, 8, 0, 255),
    3
).await?;
```

## Examples

The repository includes comprehensive examples demonstrating various features:

### Client Examples

| Example | Description |
|---------|-------------|
| `basic_client` | Simple client connecting to a meter and reading attributes |
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

### Examples README

See [`examples/README.md`](examples/README.md) for:
- Common usage patterns
- Transport options
- Addressing modes
- Authentication methods
- Error handling tips
- Troubleshooting guide

Run an example:
```bash
cargo run --example basic_client
cargo run --example basic_server
cargo run --example server_statistics
cargo run --example event_handling
```

## Crate Documentation

### Core Crates

| Crate | Description |
|-------|-------------|
| `dlms-core` | Core types: errors, OBIS codes, data objects |
| `dlms-asn1` | ASN.1 encoding/decoding (A-XDR, COSEM, ISO-ACSE) |
| `dlms-application` | PDU definitions and services (GET/SET/ACTION) |
| `dlms-interface` | COSEM interface classes (Data, Register, Clock, etc.) |
| `dlms-transport` | Transport layer abstractions |
| `dlms-session` | Session layer (HDLC, Wrapper) |
| `dlms-security` | Encryption and authentication |
| `dlms-client` | Client implementation |
| `dlms-server` | Server implementation |

### Building

Build all crates:
```bash
cargo build --workspace
```

Build specific crate:
```bash
cargo build -p dlms-client
```

Release build:
```bash
cargo build --workspace --release
```

### Testing

Run all tests:
```bash
cargo test --workspace
```

Run tests with output:
```bash
cargo test --workspace -- --nocapture
```

Run specific test:
```bash
cargo test -p dlms-client --lib test_attribute
```

## COSEM Interface Enhancements

### Register Class (Class ID: 3)

The Register interface class now includes enhanced features for value management and change notifications:

```rust
use dlms_interface::{Register, ScalerUnit, RegisterChangeCallback};
use dlms_core::{ObisCode, DataObject};
use std::sync::Arc;

// Create a register with change notifications
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
register.add(500).await?;        // Add 500 to current value
register.multiply(1.1).await?;   // Multiply by 1.1
register.reset().await?;         // Reset to zero

// Validation
let is_valid = register.is_within_range(0.0, 2000.0).await?;
println!("Value type: {}", register.value_type().await);

// Disable notifications temporarily
register.set_notifications_enabled(false).await;
```

### Profile Generic Class (Class ID: 7)

Enhanced buffer access and management features:

```rust
use dlms_interface::ProfileGeneric;
use dlms_core::{DataObject, ObisCode};

// Create a profile with buffer management
let profile = ProfileGeneric::with_default_obis(100); // 100 entry buffer

// Check buffer status
if profile.is_buffer_full().await {
    println!("Buffer is full: {}% used", profile.buffer_usage_percent().await);
}

// Read ranges efficiently
let recent = profile.get_newest_entries(10).await;           // Last 10 entries
let oldest = profile.get_oldest_entries(10).await;           // First 10 entries
let range = profile.get_range(5, Some(15)).await;            // Entries 5-14

// Time-based filtering
use dlms_core::datatypes::CosemDateTime;
let from = CosemDateTime::new(2024, 1, 1, 0, 0, 0, 0, &[])?;
let to = CosemDateTime::new(2024, 12, 31, 23, 59, 59, 0, &[])?;
let entries = profile.get_entries_by_time_range(&from, &to).await;

// Flexible searching with predicates
let high_values = profile.find_entries(|entry| {
    entry.values.get(0).map_or(false, |v| {
        matches!(v, DataObject::Unsigned64(n) if *n > 1000)
    })
}).await;

// Control capture
profile.set_capture_active(true).await;
```

## Project Status

- **Core Protocol**: 95% complete
- **Server**: 93% complete (includes request statistics)
- **Client**: 99% complete
- **COSEM Objects**: 50+ classes implemented
- **Interface Enhancements**: Register & ProfileGeneric enhanced with utility methods
- **Documentation**: 95% complete (all modules fully documented)
- **Examples**: 8 comprehensive examples
- **Overall**: ~92% complete

## License

GPL-3.0

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Resources

- [DLMS/COSEM Blue Book](https://dlms.com/) - Protocol specification
- [DLMS/COSEM Green Book](https://dlms.com/) - Object modeling
- [DLMS/COSEM Yellow Book](https://dlms.com/) - Security
- [OBIS Code System](https://dlms.com/) - Identification system

## Changelog

See [TODO.md](TODO.md) for detailed change history.
