//! Protocol Identification Service for DLMS/COSEM
//!
//! This module provides protocol identification functionality to automatically
//! detect and identify DLMS/COSEM protocol versions, capabilities, and features
//! supported by devices.
//!
//! # Overview
//!
//! Protocol identification is a key feature of DLMS/COSEM protocol that allows
//! clients to automatically discover device capabilities without manual configuration.
//! This service analyzes InitiateResponse PDUs and other protocol information to
//! determine:
//! - DLMS protocol version
//! - Supported protocol features (Conformance bits)
//! - Device capabilities (PDU size, services, etc.)
//! - Security features
//!
//! # Usage
//!
//! ```rust,no_run
//! use dlms_application::protocol_identification::{ProtocolIdentification, ProtocolInfo};
//! use dlms_application::pdu::InitiateResponse;
//!
//! // After receiving InitiateResponse
//! let initiate_response = InitiateResponse::decode(&data)?;
//! let protocol_info = ProtocolIdentification::identify(&initiate_response)?;
//!
//! // Check protocol version
//! assert_eq!(protocol_info.dlms_version(), 6);
//!
//! // Check capabilities
//! if protocol_info.supports_get() {
//!     // Device supports GET service
//! }
//! ```

use crate::pdu::{InitiateResponse, Conformance, DLMS_VERSION_6};
use dlms_core::{DlmsError, DlmsResult};

/// Protocol information extracted from device
///
/// Contains all information about the device's protocol capabilities
/// identified through protocol identification service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolInfo {
    /// DLMS protocol version (typically 6)
    dlms_version: u8,
    /// Negotiated conformance bits
    conformance: Conformance,
    /// Maximum PDU size supported by server
    max_pdu_size: u16,
    /// Server max receive PDU size
    server_max_receive_pdu_size: u16,
}

impl ProtocolInfo {
    /// Create a new ProtocolInfo
    pub fn new(
        dlms_version: u8,
        conformance: Conformance,
        max_pdu_size: u16,
        server_max_receive_pdu_size: u16,
    ) -> Self {
        Self {
            dlms_version,
            conformance,
            max_pdu_size,
            server_max_receive_pdu_size,
        }
    }

    /// Get DLMS protocol version
    pub fn dlms_version(&self) -> u8 {
        self.dlms_version
    }

    /// Get negotiated conformance bits
    pub fn conformance(&self) -> &Conformance {
        &self.conformance
    }

    /// Get maximum PDU size
    pub fn max_pdu_size(&self) -> u16 {
        self.max_pdu_size
    }

    /// Get server max receive PDU size
    pub fn server_max_receive_pdu_size(&self) -> u16 {
        self.server_max_receive_pdu_size
    }

    /// Check if device supports GET service (bit 19)
    pub fn supports_get(&self) -> bool {
        self.conformance.get_bit(19).unwrap_or(false)
    }

    /// Check if device supports SET service (bit 20)
    pub fn supports_set(&self) -> bool {
        self.conformance.get_bit(20).unwrap_or(false)
    }

    /// Check if device supports ACTION service (bit 23)
    pub fn supports_action(&self) -> bool {
        self.conformance.get_bit(23).unwrap_or(false)
    }

    /// Check if device supports selective access (bit 21)
    pub fn supports_selective_access(&self) -> bool {
        self.conformance.get_bit(21).unwrap_or(false)
    }

    /// Check if device supports event notification (bit 22)
    pub fn supports_event_notification(&self) -> bool {
        self.conformance.get_bit(22).unwrap_or(false)
    }

    /// Check if device supports block read (bit 3)
    pub fn supports_block_read(&self) -> bool {
        self.conformance.get_bit(3).unwrap_or(false)
    }

    /// Check if device supports block write (bit 4)
    pub fn supports_block_write(&self) -> bool {
        self.conformance.get_bit(4).unwrap_or(false)
    }

    /// Check if device supports multiple references (bit 14)
    pub fn supports_multiple_references(&self) -> bool {
        self.conformance.get_bit(14).unwrap_or(false)
    }

    /// Check if device supports parameterized access (bit 18)
    pub fn supports_parameterized_access(&self) -> bool {
        self.conformance.get_bit(18).unwrap_or(false)
    }

    /// Check if device supports information report (bit 15)
    pub fn supports_information_report(&self) -> bool {
        self.conformance.get_bit(15).unwrap_or(false)
    }

    /// Check if device supports data notification (bit 16)
    pub fn supports_data_notification(&self) -> bool {
        self.conformance.get_bit(16).unwrap_or(false)
    }

    /// Check if device supports priority management (bit 9)
    pub fn supports_priority_management(&self) -> bool {
        self.conformance.get_bit(9).unwrap_or(false)
    }

    /// Check if device supports attribute 0 with GET (bit 10)
    pub fn supports_attribute_0_get(&self) -> bool {
        self.conformance.get_bit(10).unwrap_or(false)
    }

    /// Check if device supports attribute 0 with SET (bit 8)
    pub fn supports_attribute_0_set(&self) -> bool {
        self.conformance.get_bit(8).unwrap_or(false)
    }

    /// Get a summary of supported services as a string
    pub fn services_summary(&self) -> String {
        let mut services = Vec::new();
        
        if self.supports_get() {
            services.push("GET");
        }
        if self.supports_set() {
            services.push("SET");
        }
        if self.supports_action() {
            services.push("ACTION");
        }
        if self.supports_event_notification() {
            services.push("EventNotification");
        }
        if self.supports_information_report() {
            services.push("InformationReport");
        }
        if self.supports_data_notification() {
            services.push("DataNotification");
        }
        
        if services.is_empty() {
            "None".to_string()
        } else {
            services.join(", ")
        }
    }

    /// Get a summary of supported features as a string
    pub fn features_summary(&self) -> String {
        let mut features = Vec::new();
        
        if self.supports_selective_access() {
            features.push("SelectiveAccess");
        }
        if self.supports_block_read() {
            features.push("BlockRead");
        }
        if self.supports_block_write() {
            features.push("BlockWrite");
        }
        if self.supports_multiple_references() {
            features.push("MultipleReferences");
        }
        if self.supports_parameterized_access() {
            features.push("ParameterizedAccess");
        }
        if self.supports_priority_management() {
            features.push("PriorityManagement");
        }
        
        if features.is_empty() {
            "None".to_string()
        } else {
            features.join(", ")
        }
    }
}

/// Protocol Identification Service
///
/// Provides functionality to identify and analyze DLMS/COSEM protocol
/// capabilities from device responses.
pub struct ProtocolIdentification;

impl ProtocolIdentification {
    /// Identify protocol information from InitiateResponse
    ///
    /// This is the primary method for protocol identification. It extracts
    /// all relevant protocol information from an InitiateResponse PDU.
    ///
    /// # Arguments
    /// * `initiate_response` - The InitiateResponse PDU received from the device
    ///
    /// # Returns
    /// ProtocolInfo containing all identified protocol information
    ///
    /// # Errors
    /// Returns error if the InitiateResponse contains invalid data
    ///
    /// # Example
    /// ```rust,no_run
    /// use dlms_application::protocol_identification::ProtocolIdentification;
    /// use dlms_application::pdu::InitiateResponse;
    ///
    /// let response = InitiateResponse::decode(&data)?;
    /// let info = ProtocolIdentification::identify(&response)?;
    /// println!("DLMS Version: {}", info.dlms_version());
    /// println!("Supports GET: {}", info.supports_get());
    /// ```
    pub fn identify(initiate_response: &InitiateResponse) -> DlmsResult<ProtocolInfo> {
        // Extract DLMS version
        let dlms_version = initiate_response.negotiated_dlms_version_number;
        
        // Validate DLMS version
        if dlms_version > DLMS_VERSION_6 {
            return Err(DlmsError::InvalidData(format!(
                "Unsupported DLMS version: {} (max supported: {})",
                dlms_version, DLMS_VERSION_6
            )));
        }
        
        // Extract conformance bits
        let conformance = initiate_response.negotiated_conformance.clone();
        
        // Extract PDU size information
        let server_max_receive_pdu_size = initiate_response.server_max_receive_pdu_size;
        
        // Use server max receive PDU size as the effective max PDU size
        // (this is what the server can actually receive)
        let max_pdu_size = server_max_receive_pdu_size;
        
        Ok(ProtocolInfo::new(
            dlms_version,
            conformance,
            max_pdu_size,
            server_max_receive_pdu_size,
        ))
    }

    /// Check if device supports a specific DLMS version
    ///
    /// # Arguments
    /// * `initiate_response` - The InitiateResponse PDU
    /// * `version` - DLMS version to check (typically 6)
    ///
    /// # Returns
    /// `true` if device supports the specified version, `false` otherwise
    pub fn supports_version(initiate_response: &InitiateResponse, version: u8) -> bool {
        initiate_response.negotiated_dlms_version_number == version
    }

    /// Check if device supports a specific service based on conformance bits
    ///
    /// # Arguments
    /// * `initiate_response` - The InitiateResponse PDU
    /// * `bit_index` - Conformance bit index (0-23)
    ///
    /// # Returns
    /// `Some(true)` if bit is set, `Some(false)` if bit is clear, `None` if invalid bit index
    pub fn supports_feature(initiate_response: &InitiateResponse, bit_index: usize) -> Option<bool> {
        initiate_response.negotiated_conformance.get_bit(bit_index)
    }

    /// Get a human-readable description of device capabilities
    ///
    /// # Arguments
    /// * `initiate_response` - The InitiateResponse PDU
    ///
    /// # Returns
    /// A formatted string describing device capabilities
    pub fn describe_capabilities(initiate_response: &InitiateResponse) -> DlmsResult<String> {
        let info = Self::identify(initiate_response)?;
        
        let mut description = format!(
            "DLMS/COSEM Protocol Information:\n\
             - DLMS Version: {}\n\
             - Max PDU Size: {} bytes\n\
             - Server Max Receive PDU Size: {} bytes\n\
             - Supported Services: {}\n\
             - Supported Features: {}",
            info.dlms_version(),
            info.max_pdu_size(),
            info.server_max_receive_pdu_size(),
            info.services_summary(),
            info.features_summary()
        );
        
        Ok(description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdu::{InitiateResponse, Conformance};
    use dlms_core::datatypes::BitString;

    fn create_test_initiate_response() -> InitiateResponse {
        // Create a conformance with GET, SET, and ACTION bits set
        let mut conformance = Conformance::new();
        conformance.set_bit(19, true).unwrap(); // GET
        conformance.set_bit(20, true).unwrap(); // SET
        conformance.set_bit(23, true).unwrap(); // ACTION
        conformance.set_bit(21, true).unwrap(); // Selective access
        
        InitiateResponse {
            negotiated_dlms_version_number: 6,
            negotiated_conformance: conformance,
            server_max_receive_pdu_size: 1024,
            vaa_name: 0x0007, // Standard DLMS VAA
            negotiated_quality_of_service: None,
        }
    }

    #[test]
    fn test_identify_protocol() {
        let response = create_test_initiate_response();
        let info = ProtocolIdentification::identify(&response).unwrap();
        
        assert_eq!(info.dlms_version(), 6);
        assert_eq!(info.max_pdu_size(), 1024);
        assert!(info.supports_get());
        assert!(info.supports_set());
        assert!(info.supports_action());
        assert!(info.supports_selective_access());
    }

    #[test]
    fn test_supports_version() {
        let response = create_test_initiate_response();
        assert!(ProtocolIdentification::supports_version(&response, 6));
        assert!(!ProtocolIdentification::supports_version(&response, 5));
    }

    #[test]
    fn test_supports_feature() {
        let response = create_test_initiate_response();
        assert_eq!(ProtocolIdentification::supports_feature(&response, 19), Some(true)); // GET
        assert_eq!(ProtocolIdentification::supports_feature(&response, 20), Some(true)); // SET
        assert_eq!(ProtocolIdentification::supports_feature(&response, 0), Some(false)); // Not set
    }

    #[test]
    fn test_describe_capabilities() {
        let response = create_test_initiate_response();
        let description = ProtocolIdentification::describe_capabilities(&response).unwrap();
        
        assert!(description.contains("DLMS Version: 6"));
        assert!(description.contains("Max PDU Size: 1024"));
        assert!(description.contains("GET"));
        assert!(description.contains("SET"));
        assert!(description.contains("ACTION"));
    }

    #[test]
    fn test_services_summary() {
        let response = create_test_initiate_response();
        let info = ProtocolIdentification::identify(&response).unwrap();
        let summary = info.services_summary();
        
        assert!(summary.contains("GET"));
        assert!(summary.contains("SET"));
        assert!(summary.contains("ACTION"));
    }

    #[test]
    fn test_features_summary() {
        let response = create_test_initiate_response();
        let info = ProtocolIdentification::identify(&response).unwrap();
        let summary = info.features_summary();
        
        assert!(summary.contains("SelectiveAccess"));
    }
}
