//! xDLMS encrypted frame construction and parsing
//!
//! This module provides functionality for building and parsing encrypted DLMS frames
//! according to the xDLMS specification.
//!
//! # Encrypted Frame Format
//!
//! According to DLMS Green Book, an encrypted frame has the following structure:
//! ```
//! Security Control (1 byte)
//! System Title (8 bytes, optional - present if key_set bit is set)
//! Frame Counter (4 bytes, optional - present if encrypted bit is set)
//! Encrypted Data (variable length)
//! Authentication Tag (12 bytes, optional - present if authenticated bit is set)
//! ```
//!
//! # Security Control Byte
//! Bits 0-3: Security Suite ID (0-15)
//! Bit 4: Authenticated (1 = authenticated, 0 = not authenticated)
//! Bit 5: Encrypted (1 = encrypted, 0 = not encrypted)
//! Bit 6: Key Set (1 = System Title present, 0 = System Title not present)
//! Bit 7: Reserved (must be 0)
//!
//! # Why This Design?
//! - **Security Control**: Indicates which security features are active
//! - **System Title**: Identifies the device for key derivation
//! - **Frame Counter**: Prevents replay attacks
//! - **Encrypted Data**: The actual PDU data encrypted with AES-GCM
//! - **Authentication Tag**: GMAC tag for integrity verification

use crate::error::{DlmsError, DlmsResult};
use crate::encryption::{AesGcmEncryption, SecurityControl};
use crate::xdlms::{SystemTitle, XdlmsContext};
use std::sync::Arc;

/// Encrypted frame builder
///
/// Builds encrypted DLMS frames according to xDLMS specification.
/// Handles encryption, System Title embedding, Frame Counter embedding,
/// and authentication tag generation.
pub struct EncryptedFrameBuilder {
    /// xDLMS context containing keys and counters
    context: Arc<XdlmsContext>,
    /// Security suite ID (0-15)
    security_suite_id: u8,
}

impl EncryptedFrameBuilder {
    /// Create a new encrypted frame builder
    ///
    /// # Arguments
    /// * `context` - xDLMS context with keys and counters
    /// * `security_suite_id` - Security suite ID (0-15)
    pub fn new(context: Arc<XdlmsContext>, security_suite_id: u8) -> Self {
        Self {
            context,
            security_suite_id: security_suite_id & 0x0F, // Ensure only 4 bits
        }
    }

    /// Build an encrypted frame from plaintext PDU
    ///
    /// # Arguments
    /// * `plaintext` - Plaintext PDU data to encrypt
    /// * `authenticated` - Whether to include authentication tag
    /// * `encrypted` - Whether to encrypt the data
    /// * `include_system_title` - Whether to include System Title in frame
    /// * `is_broadcast` - Whether this is a broadcast frame (affects key selection)
    ///
    /// # Returns
    /// Encrypted frame bytes ready for transmission
    ///
    /// # Process
    /// 1. Increment send frame counter
    /// 2. Build AAD (Additional Authenticated Data) from System Title and Frame Counter
    /// 3. Encrypt plaintext with AES-GCM
    /// 4. Build frame: Security Control | System Title | Frame Counter | Encrypted Data | Auth Tag
    ///
    /// # Error Handling
    /// - Returns error if encryption fails
    /// - Returns error if keys are not available
    pub fn build_encrypted_frame(
        &self,
        plaintext: &[u8],
        authenticated: bool,
        encrypted: bool,
        include_system_title: bool,
        is_broadcast: bool,
    ) -> DlmsResult<Vec<u8>> {
        // Get encryption key
        let encryption_key = if is_broadcast {
            self.context.broadcast_encryption_key()
        } else {
            self.context.unicast_encryption_key()
        };

        let encryption_key = encryption_key.ok_or_else(|| {
            DlmsError::Security("Encryption key not available. Call set_master_key() first.".to_string())
        })?;

        // Create encryption context
        let cipher = AesGcmEncryption::new(encryption_key)?;

        // Increment frame counter and get current value
        let frame_counter = self.context.send_frame_counter.increment();

        // Build nonce for AES-GCM
        // According to DLMS standard, nonce = System Title (8 bytes) || Frame Counter (4 bytes, big-endian)
        // Nonce must be 12 bytes for AES-128-GCM
        let mut nonce = Vec::with_capacity(12);
        nonce.extend_from_slice(self.context.server_system_title.as_bytes());
        nonce.extend_from_slice(&frame_counter.to_be_bytes());

        // Build AAD (Additional Authenticated Data)
        // AAD is typically empty for DLMS, but can include additional authenticated data
        // For now, we use empty AAD as per DLMS standard
        let aad = &[];

        // Encrypt plaintext
        let ciphertext = if encrypted {
            // Use the nonce we built from System Title and Frame Counter
            cipher.encrypt_with_nonce(plaintext, &nonce, aad)?
        } else {
            // If not encrypted, just return plaintext
            plaintext.to_vec()
        };

        // Build Security Control byte
        let security_control = SecurityControl::new(
            self.security_suite_id,
            authenticated,
            encrypted,
            include_system_title,
        );

        // Build frame
        let mut frame = Vec::new();

        // 1. Security Control (1 byte)
        frame.push(security_control.to_byte());

        // 2. System Title (8 bytes, optional)
        if include_system_title {
            frame.extend_from_slice(self.context.server_system_title.as_bytes());
        }

        // 3. Frame Counter (4 bytes, optional - always present if encrypted)
        if encrypted {
            frame.extend_from_slice(&frame_counter.to_be_bytes());
        }

        // 4. Encrypted Data
        // Note: For DLMS, the nonce is NOT prepended to ciphertext
        // The nonce is constructed from System Title and Frame Counter (already in frame)
        // The ciphertext includes the authentication tag (12 bytes) at the end
        if encrypted {
            frame.extend_from_slice(&ciphertext);
        } else {
            // If not encrypted, plaintext is sent as-is
            frame.extend_from_slice(plaintext);
        }

        // 5. Authentication Tag (12 bytes, optional)
        // Note: For AES-GCM, the authentication tag is part of the ciphertext
        // The actual tag is appended by AES-GCM during encryption
        // If authenticated but not encrypted, we need to compute GMAC separately
        // For now, we assume authenticated encryption (AES-GCM) which includes the tag

        Ok(frame)
    }
}

/// Encrypted frame parser
///
/// Parses encrypted DLMS frames according to xDLMS specification.
/// Handles decryption, System Title extraction, Frame Counter extraction,
/// and authentication tag verification.
pub struct EncryptedFrameParser {
    /// xDLMS context containing keys and counters
    context: Arc<XdlmsContext>,
}

impl EncryptedFrameParser {
    /// Create a new encrypted frame parser
    ///
    /// # Arguments
    /// * `context` - xDLMS context with keys and counters
    pub fn new(context: Arc<XdlmsContext>) -> Self {
        Self { context }
    }

    /// Parse and decrypt an encrypted frame
    ///
    /// # Arguments
    /// * `frame` - Encrypted frame bytes
    /// * `is_broadcast` - Whether this is a broadcast frame (affects key selection)
    ///
    /// # Returns
    /// Plaintext PDU data
    ///
    /// # Process
    /// 1. Parse Security Control byte
    /// 2. Extract System Title (if present)
    /// 3. Extract Frame Counter (if present)
    /// 4. Verify frame counter (prevent replay attacks)
    /// 5. Decrypt data with AES-GCM
    /// 6. Verify authentication tag
    ///
    /// # Error Handling
    /// - Returns error if frame format is invalid
    /// - Returns error if decryption fails
    /// - Returns error if frame counter is invalid (replay attack)
    /// - Returns error if authentication tag verification fails
    pub fn parse_encrypted_frame(
        &self,
        frame: &[u8],
        is_broadcast: bool,
    ) -> DlmsResult<Vec<u8>> {
        if frame.is_empty() {
            return Err(DlmsError::InvalidData("Empty encrypted frame".to_string()));
        }

        let mut pos = 0;

        // 1. Parse Security Control byte
        let security_control_byte = frame[pos];
        let security_control = SecurityControl::from_byte(security_control_byte);
        pos += 1;

        let authenticated = security_control.is_authenticated();
        let encrypted = security_control.is_encrypted();
        let include_system_title = security_control.is_key_set();

        // 2. Extract System Title (8 bytes, optional)
        let system_title = if include_system_title {
            if pos + 8 > frame.len() {
                return Err(DlmsError::InvalidData(
                    "Frame too short for System Title".to_string(),
                ));
            }
            let st_bytes = &frame[pos..pos + 8];
            pos += 8;
            Some(SystemTitle::from_slice(st_bytes)?)
        } else {
            None
        };

        // 3. Extract Frame Counter (4 bytes, optional - always present if encrypted)
        let frame_counter = if encrypted {
            if pos + 4 > frame.len() {
                return Err(DlmsError::InvalidData(
                    "Frame too short for Frame Counter".to_string(),
                ));
            }
            let counter_bytes = &frame[pos..pos + 4];
            pos += 4;
            Some(u32::from_be_bytes([
                counter_bytes[0],
                counter_bytes[1],
                counter_bytes[2],
                counter_bytes[3],
            ]))
        } else {
            None
        };

        // 4. Verify frame counter (prevent replay attacks)
        if let Some(received_counter) = frame_counter {
            let expected_counter = self.context.receive_frame_counter.get();
            
            // Frame counter must be greater than the last received counter
            // This prevents replay attacks
            if received_counter <= expected_counter {
                return Err(DlmsError::Security(format!(
                    "Frame counter validation failed: received {} <= expected {} (possible replay attack)",
                    received_counter,
                    expected_counter
                )));
            }

            // Update receive frame counter
            self.context.receive_frame_counter.set(received_counter);
        }

        // 5. Extract encrypted data
        let encrypted_data = &frame[pos..];

        // 6. Decrypt data
        if encrypted {
            // Get decryption key
            let decryption_key = if is_broadcast {
                self.context.broadcast_encryption_key()
            } else {
                self.context.unicast_encryption_key()
            };

            let decryption_key = decryption_key.ok_or_else(|| {
                DlmsError::Security(
                    "Decryption key not available. Call set_master_key() first.".to_string(),
                )
            })?;

            // Create decryption context
            let cipher = AesGcmEncryption::new(decryption_key)?;

            // For DLMS, nonce is NOT prepended to ciphertext
            // Nonce is constructed from System Title and Frame Counter (already extracted)
            // The ciphertext includes the authentication tag (12 bytes) at the end
            let ciphertext = encrypted_data;
            
            // Build nonce from System Title and Frame Counter
            let mut nonce = Vec::with_capacity(12);
            if let Some(ref st) = system_title {
                nonce.extend_from_slice(st.as_bytes());
            } else {
                // If System Title not in frame, use context System Title
                nonce.extend_from_slice(self.context.client_system_title.as_bytes());
            }
            if let Some(fc) = frame_counter {
                nonce.extend_from_slice(&fc.to_be_bytes());
            } else {
                // This should not happen if encrypted is true, but handle it
                return Err(DlmsError::InvalidData(
                    "Frame Counter missing in encrypted frame".to_string(),
                ));
            }

            // Build AAD (Additional Authenticated Data)
            // AAD = System Title (8 bytes) || Frame Counter (4 bytes, big-endian)
            let mut aad = Vec::with_capacity(12);
            if let Some(ref st) = system_title {
                aad.extend_from_slice(st.as_bytes());
            } else {
                // If System Title not in frame, use context System Title
                aad.extend_from_slice(self.context.client_system_title.as_bytes());
            }
            if let Some(fc) = frame_counter {
                aad.extend_from_slice(&fc.to_be_bytes());
            } else {
                // If Frame Counter not in frame, use current receive counter
                aad.extend_from_slice(&self.context.receive_frame_counter.get().to_be_bytes());
            }

            // Decrypt
            let plaintext = cipher.decrypt(ciphertext, nonce, &aad)?;

            Ok(plaintext)
        } else {
            // If not encrypted, return data as-is
            Ok(encrypted_data.to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xdlms::{XdlmsContext, SystemTitle};

    #[test]
    fn test_encrypted_frame_build_and_parse() {
        // Create test context
        let client_st = SystemTitle::new([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
        let server_st = SystemTitle::new([0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18]);
        let mut context = XdlmsContext::new(client_st, server_st);
        
        // Set master key
        let master_key = vec![0u8; 16]; // Test key (all zeros)
        context.set_master_key(master_key).unwrap();
        
        let context = Arc::new(context);
        
        // Build encrypted frame
        let builder = EncryptedFrameBuilder::new(context.clone(), 0);
        let plaintext = b"Hello, DLMS!";
        let encrypted_frame = builder
            .build_encrypted_frame(plaintext, true, true, true, false)
            .unwrap();
        
        // Parse encrypted frame
        let parser = EncryptedFrameParser::new(context);
        let decrypted = parser.parse_encrypted_frame(&encrypted_frame, false).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
}
