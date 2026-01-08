# jDLMS Rust

Rust implementation of DLMS/COSEM protocol for smart meter communication.

## Status

🚧 **Work in Progress** - This is an active migration from Java to Rust.

## Architecture

This project is organized as a Cargo workspace with multiple crates:

- **dlms-core**: Core types, error handling, OBIS codes, and data types
- **dlms-asn1**: ASN.1 encoding/decoding (A-XDR, COSEM ASN.1, ISO-ACSE)
- **dlms-transport**: Transport layer (TCP, UDP, Serial)
- **dlms-session**: Session layer (HDLC, Wrapper)
- **dlms-security**: Security layer (encryption, authentication)
- **dlms-application**: Application layer (PDU, services)
- **dlms-interface**: COSEM interface classes
- **dlms-client**: Client implementation
- **dlms-server**: Server implementation
- **dlms**: Main entry point, re-exports all public APIs

## Features

- DLMS/COSEM client and server implementation
- Support for HDLC/serial, HDLC/TCP, and Wrapper/TCP
- Full protocol stack implementation
- Secure communication with encryption and authentication
- Modular architecture with clear separation of concerns

## Building

Build all crates in the workspace:

```bash
cargo build --workspace
```

Build a specific crate:

```bash
cargo build -p dlms-core
cargo build -p dlms-client
```

## Running Tests

Run all tests:

```bash
cargo test --workspace
```

Run tests for a specific crate:

```bash
cargo test -p dlms-core
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
dlms = { path = "../dlms" }
```

Then use in your code:

```rust
use dlms::{DlmsError, DlmsResult, ObisCode};
use dlms::client::ConnectionBuilder;
```

## License

GPL-3.0
