//! Connection management module
//!
//! This module provides connection management for DLMS/COSEM client operations.
//!
//! # Connection Types
//!
//! - **Logical Name (LN) Connection**: Uses OBIS codes for object addressing
//! - **Short Name (SN) Connection**: Uses 2-byte addresses for object addressing
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_client::connection::{Connection, LnConnection, LnConnectionConfig};
//!
//! // Create connection
//! let config = LnConnectionConfig::default();
//! let mut conn = LnConnection::new(config);
//!
//! // Open connection
//! conn.open().await?;
//!
//! // Perform operations
//! let value = conn.get_attribute(obis_code, class_id, attribute_id).await?;
//!
//! // Close connection
//! conn.close().await?;
//! ```

pub mod builder;
pub mod connection;
pub mod tcp_builder;
pub mod serial_builder;
pub mod ln_connection;
pub mod sn_connection;

pub use connection::{Connection, ConnectionState};
pub use ln_connection::{LnConnection, LnConnectionConfig};
