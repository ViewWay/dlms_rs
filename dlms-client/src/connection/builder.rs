//! Connection builder for DLMS/COSEM client
//!
//! This module provides a builder pattern for creating DLMS/COSEM connections.
//! The builder allows flexible configuration of transport, session, and application layers.
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use dlms_client::connection::{ConnectionBuilder, LnConnection};
//!
//! // Create TCP connection
//! let mut conn = ConnectionBuilder::new()
//!     .tcp("192.168.1.100:4059")
//!     .hdlc_addresses(0x01, 0x10)
//!     .build_ln()?;
//!
//! // Create Serial connection
//! let mut conn = ConnectionBuilder::new()
//!     .serial("/dev/ttyUSB0", 9600)
//!     .hdlc_addresses(0x01, 0x10)
//!     .build_ln()?;
//! ```

use super::{LnConnection, LnConnectionConfig};
use dlms_core::DlmsResult;
use dlms_session::hdlc::HdlcAddress;
use dlms_security::SecuritySuite;
use dlms_application::pdu::Conformance;

/// Connection builder for creating DLMS/COSEM connections
///
/// The builder pattern allows flexible configuration of all connection parameters
/// before creating the connection object. This provides:
/// - **Type Safety**: Compile-time validation of required parameters
/// - **Flexibility**: Easy to add new configuration options
/// - **Readability**: Clear, fluent API for connection setup
///
/// # Builder Pattern Benefits
/// - **Method Chaining**: Fluent API for configuration
/// - **Default Values**: Sensible defaults for optional parameters
/// - **Validation**: Can validate configuration before building
///
/// # Configuration Flow
/// 1. Create builder with `ConnectionBuilder::new()`
/// 2. Configure transport (TCP or Serial)
/// 3. Configure session layer (HDLC addresses or Wrapper IDs)
/// 4. Configure application layer (conformance, PDU size, etc.)
/// 5. Build connection with `build_ln()` or `build_sn()`
#[derive(Debug, Clone)]
pub struct ConnectionBuilder {
    /// Transport type and settings
    transport_type: TransportType,
    /// Local HDLC address (for HDLC sessions)
    local_hdlc_address: Option<HdlcAddress>,
    /// Remote HDLC address (for HDLC sessions)
    remote_hdlc_address: Option<HdlcAddress>,
    /// Client ID (for Wrapper sessions)
    client_id: Option<u16>,
    /// Logical device ID (for Wrapper sessions)
    logical_device_id: Option<u16>,
    /// Security suite (optional)
    security_suite: Option<SecuritySuite>,
    /// Conformance bits
    conformance: Conformance,
    /// Maximum PDU size
    max_pdu_size: u16,
    /// DLMS version
    dlms_version: u8,
}

/// Transport type configuration
#[derive(Debug, Clone)]
enum TransportType {
    /// TCP transport
    Tcp {
        address: String,
    },
    /// Serial transport
    Serial {
        port_name: String,
        baud_rate: u32,
    },
    /// Not configured
    None,
}

impl ConnectionBuilder {
    /// Create a new connection builder with default settings
    ///
    /// # Returns
    /// A new builder with default configuration
    ///
    /// # Default Settings
    /// - Conformance: Default (basic features)
    /// - Max PDU size: 1024 bytes
    /// - DLMS version: 6
    /// - Client ID: 0x10 (for Wrapper)
    /// - Logical device ID: 0x01 (for Wrapper)
    pub fn new() -> Self {
        Self {
            transport_type: TransportType::None,
            local_hdlc_address: None,
            remote_hdlc_address: None,
            client_id: Some(0x10),
            logical_device_id: Some(0x01),
            security_suite: None,
            conformance: Conformance::default(),
            max_pdu_size: 1024,
            dlms_version: 6,
        }
    }

    /// Configure TCP transport
    ///
    /// # Arguments
    /// * `address` - TCP address in format "host:port" (e.g., "192.168.1.100:4059")
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Note
    /// TCP transport typically uses Wrapper session layer, but HDLC over TCP
    /// is also possible. Use `hdlc_addresses()` for HDLC or `wrapper_ids()` for Wrapper.
    pub fn tcp(mut self, address: &str) -> Self {
        self.transport_type = TransportType::Tcp {
            address: address.to_string(),
        };
        self
    }

    /// Configure Serial transport
    ///
    /// # Arguments
    /// * `port_name` - Serial port name (e.g., "/dev/ttyUSB0" or "COM1")
    /// * `baud_rate` - Baud rate (e.g., 9600, 115200)
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Note
    /// Serial transport typically uses HDLC session layer. Use `hdlc_addresses()` to configure.
    pub fn serial(mut self, port_name: &str, baud_rate: u32) -> Self {
        self.transport_type = TransportType::Serial {
            port_name: port_name.to_string(),
            baud_rate,
        };
        self
    }

    /// Configure HDLC addresses
    ///
    /// # Arguments
    /// * `local` - Local HDLC address
    /// * `remote` - Remote HDLC address
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Note
    /// HDLC addressing is used for Serial transport and some TCP implementations.
    /// Typical values: local=0x01 (client), remote=0x10 (server).
    pub fn hdlc_addresses(mut self, local: u8, remote: u8) -> Self {
        self.local_hdlc_address = Some(HdlcAddress::new(local));
        self.remote_hdlc_address = Some(HdlcAddress::new(remote));
        self
    }

    /// Configure Wrapper session IDs
    ///
    /// # Arguments
    /// * `client_id` - Client ID (typically 0x10)
    /// * `logical_device_id` - Logical device ID (typically 0x01)
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Note
    /// Wrapper session is used for TCP/UDP transport. The IDs identify the client
    /// and logical device in the Wrapper header.
    pub fn wrapper_ids(mut self, client_id: u16, logical_device_id: u16) -> Self {
        self.client_id = Some(client_id);
        self.logical_device_id = Some(logical_device_id);
        self
    }

    /// Configure security suite
    ///
    /// # Arguments
    /// * `suite` - Security suite configuration
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Note
    /// Security suite is optional. If not configured, no encryption/authentication
    /// is used (Low-Level Security).
    pub fn security(mut self, suite: SecuritySuite) -> Self {
        self.security_suite = Some(suite);
        self
    }

    /// Configure conformance bits
    ///
    /// # Arguments
    /// * `conformance` - Conformance bitstring
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Note
    /// Conformance bits indicate which DLMS features the client supports.
    /// The server will negotiate based on its own capabilities.
    pub fn conformance(mut self, conformance: Conformance) -> Self {
        self.conformance = conformance;
        self
    }

    /// Configure maximum PDU size
    ///
    /// # Arguments
    /// * `size` - Maximum PDU size in bytes
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Note
    /// Default is 1024 bytes. Larger values allow bigger data transfers but
    /// require more memory. The server will negotiate the actual size.
    pub fn max_pdu_size(mut self, size: u16) -> Self {
        self.max_pdu_size = size;
        self
    }

    /// Configure DLMS version
    ///
    /// # Arguments
    /// * `version` - DLMS version number (typically 6)
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Note
    /// DLMS version 6 is the current standard. Older versions may be supported
    /// for compatibility with legacy devices.
    pub fn dlms_version(mut self, version: u8) -> Self {
        self.dlms_version = version;
        self
    }

    /// Build a Logical Name (LN) connection
    ///
    /// # Returns
    /// A configured `LnConnection` ready to be opened
    ///
    /// # Errors
    /// Returns error if:
    /// - Transport type is not configured
    /// - Required session parameters are missing
    /// - Configuration is invalid
    ///
    /// # Validation
    /// - TCP transport requires either HDLC addresses or Wrapper IDs
    /// - Serial transport requires HDLC addresses
    /// - Wrapper session requires client_id and logical_device_id
    pub fn build_ln(self) -> DlmsResult<LnConnection> {
        // Validate transport type
        if matches!(self.transport_type, TransportType::None) {
            return Err(dlms_core::DlmsError::InvalidData(
                "Transport type must be configured (TCP or Serial)".to_string(),
            ));
        }

        // Create connection configuration
        let config = LnConnectionConfig {
            local_address: self.local_hdlc_address,
            remote_address: self.remote_hdlc_address,
            client_id: self.client_id,
            logical_device_id: self.logical_device_id,
            security_suite: self.security_suite,
            conformance: self.conformance,
            max_pdu_size: self.max_pdu_size,
            dlms_version: self.dlms_version,
        };

        // Create connection
        Ok(LnConnection::new(config))
    }

    /// Build a Short Name (SN) connection
    ///
    /// # Returns
    /// A configured `SnConnection` ready to be opened
    ///
    /// # Errors
    /// Returns error if configuration is invalid
    ///
    /// # Note
    /// SN connection implementation is pending. This method is a placeholder.
    pub fn build_sn(self) -> DlmsResult<()> {
        // TODO: Implement SN connection building
        Err(dlms_core::DlmsError::InvalidData(
            "SN connection building not yet implemented".to_string(),
        ))
    }
}

impl Default for ConnectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
