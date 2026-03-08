//! DLMS/COSEM Core Types and Utilities
//!
//! This crate provides the foundational types, error handling, and utilities
//! used throughout the DLMS/COSEM protocol implementation.
//!
//! # Overview
//!
//! DLMS (Device Language Message Specification) / COSEM ( Companion Specification
//! for Energy Metering) is a standard protocol for communication with smart meters
//! and other energy management devices.
//!
//! This crate implements:
//!
//! - **Error Handling**: Unified error types via [`DlmsError`] and [`DlmsResult`]
//! - **OBIS Codes**: The Object Identification System (OBIS) via [`ObisCode`]
//! - **Data Types**: Complete COSEM data type hierarchy ( [`DataObject`], [`CosemDate`], etc.)
//! - **Memory Management**: Buffer pooling and zero-copy operations
//!
//! # Data Types
//!
//! The core data type is [`DataObject`], which represents all possible COSEM data
//! values including:
//!
//! - Numeric types: integers (8/16/32/64-bit), signed/unsigned, floats
//! - String types: OctetString, VisibleString, Utf8String
//! - Composite types: Array, Structure, CompactArray
//! - Special types: BitString, Date, Time, DateTime
//! - Extended types: BCD, Enumerate
//!
//! # OBIS Codes
//!
//! The [`ObisCode`] type represents the OBIS (Object Identification System) code
//! used to identify COSEM objects. An OBIS code consists of 6 values (A-E groups and F):
//!
//! ```rust
//! use dlms_core::ObisCode;
//!
//! // Create an OBIS code for active energy (A=1, B=1, C=1, D=8, E=0, F=255)
//! let obis = ObisCode::new(1, 1, 1, 8, 0, 255);
//!
//! // Format as string (dot-separated)
//! assert_eq!(obis.to_string(), "1.1.1.8.0.255");
//!
//! // Access individual components
//! assert_eq!(obis.a(), 1);
//! assert_eq!(obis.d(), 8);
//! ```
//!
//! # Error Handling
//!
//! All functions return [`DlmsResult<T>`] which is a type alias for
//! `Result<T, DlmsError>`. The [`DlmsError`] enum covers all error conditions:
//!
//! ```rust
//! use dlms_core::{DlmsError, DlmsResult};
//!
//! fn parse_value(input: &str) -> DlmsResult<u32> {
//!     input.parse::<u32>()
//!         .map_err(|e| DlmsError::InvalidData(format!("Parse error: {}", e)))
//! }
//! ```
//!
//! # Performance Features
//!
//! ## Buffer Pool
//!
//! The buffer pool reduces memory allocation overhead by reusing buffers:
//!
//! ```rust
//! use dlms_core::pool::{BufferPool, BufferPoolConfig};
//!
//! let pool = BufferPool::with_config(
//!     BufferPoolConfig::default()
//!         .with_initial_capacity(10)
//!         .with_buffer_size(1024),
//! );
//!
//! {
//!     let mut buffer = pool.acquire();
//!     buffer.extend_from_slice(&[1, 2, 3]);
//! } // Buffer is automatically returned to the pool
//!
//! assert_eq!(pool.available_count(), 10);
//! ```
//!
//! ## Zero-Copy Operations
//!
//! [`ByteSlice`] provides a zero-copy view into buffer data:
//!
//! ```rust
//! use dlms_core::pool::ByteSlice;
//!
//! let data = vec![1, 2, 3, 4, 5];
//! let slice = ByteSlice::from(&data);
//!
//! assert_eq!(slice.as_bytes(), &[1, 2, 3, 4, 5]);
//! ```
//!
//! # Module Structure
//!
//! - [`error`] - Error types and result definitions
//! - [`obis_code`] - OBIS code implementation
//! - [`datatypes`] - COSEM data types (DataObject, Date, Time, etc.)
//! - [`pool`] - Memory management utilities
//!
//! # Features
//!
//! - `serde`: Serialization support for data types
//! - `compression`: Compression support for large data transfers

pub mod error;
pub mod obis_code;
pub mod datatypes;
pub mod pool;

pub use error::{DlmsError, DlmsResult};
pub use obis_code::ObisCode;
pub use datatypes::*;
pub use pool::{BufferPool, BufferPoolConfig, PooledBuffer, ByteSlice, Lazy, lazy};
