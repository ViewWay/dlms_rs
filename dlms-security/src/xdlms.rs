//! xDLMS (Extended DLMS) specific functionality
//!
//! This module provides xDLMS-specific features including:
//! - System Title management
//! - Frame Counter management
//! - Key derivation functions (KDF)
//! - Encrypted frame construction and parsing
//! - xDLMS context management
//!
//! # xDLMS Overview
//!
//! xDLMS (Extended DLMS) extends the base DLMS protocol with:
//! - Enhanced security features (encryption, authentication)
//! - System Title for device identification
//! - Frame counters for replay attack prevention
//! - Key derivation for secure key management
//!
//! # System Title
//!
//! System Title is an 8-byte identifier that uniquely identifies a device.
//! It is used in:
//! - Key derivation
//! - Frame authentication
//! - Device identification
//!
//! # Frame Counter
//!
//! Frame counter is a 32-bit counter that increments with each encrypted frame.
//! It prevents replay attacks by ensuring frames are processed in order.
//!
//! # Key Derivation Function (KDF)
//!
//! KDF is used to derive encryption and authentication keys from a master key.
//! The derivation uses:
//! - Master key (KEK - Key Encryption Key)
//! - System Title
//! - Key ID (GlobalUnicastEncryptionKey, GlobalBroadcastEncryptionKey, etc.)

use crate::error::{DlmsError, DlmsResult};
use crate::utils::KeyId;
use aes::Aes128;
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::cipher::generic_array::GenericArray;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// System Title
///
/// An 8-byte identifier that uniquely identifies a DLMS/COSEM device.
/// System Title is used in key derivation and frame authentication.
///
/// # Format
/// System Title is typically:
/// - 4 bytes: Manufacturer ID (from OBIS code A field)
/// - 4 bytes: Device serial number or timestamp
///
/// # Usage
/// System Title is sent in InitiateRequest/Response and used in:
/// - Key derivation (KDF)
/// - Frame authentication (GMAC)
/// - Device identification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemTitle {
    /// The 8-byte system title value
    value: [u8; 8],
}

impl SystemTitle {
    /// Create a new System Title from bytes
    ///
    /// # Arguments
    /// * `bytes` - 8-byte array containing the system title
    ///
    /// # Errors
    /// Returns error if bytes length is not 8
    pub fn new(bytes: [u8; 8]) -> Self {
        Self { value: bytes }
    }

    /// Create System Title from slice
    ///
    /// # Arguments
    /// * `bytes` - Slice containing exactly 8 bytes
    ///
    /// # Errors
    /// Returns error if bytes length is not 8
    pub fn from_slice(bytes: &[u8]) -> DlmsResult<Self> {
        if bytes.len() != 8 {
            return Err(DlmsError::InvalidData(format!(
                "System Title must be 8 bytes, got {}",
                bytes.len()
            )));
        }
        let mut value = [0u8; 8];
        value.copy_from_slice(bytes);
        Ok(Self { value })
    }

    /// Generate a System Title from current timestamp
    ///
    /// This is useful for testing or when a device doesn't have a fixed system title.
    /// In production, System Title should be derived from device-specific information.
    ///
    /// # Format
    /// - Bytes 0-3: Unix timestamp (seconds since epoch)
    /// - Bytes 4-7: Random or device-specific identifier
    pub fn from_timestamp() -> DlmsResult<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| DlmsError::InvalidData(format!("Invalid system time: {}", e)))?
            .as_secs() as u32;

        let mut value = [0u8; 8];
        value[0..4].copy_from_slice(&timestamp.to_be_bytes());
        // Bytes 4-7: Use a default value (0) or could be random
        // In production, these should be device-specific

        Ok(Self { value })
    }

    /// Get the System Title as bytes
    pub fn as_bytes(&self) -> &[u8; 8] {
        &self.value
    }

    /// Get the System Title as slice
    pub fn as_slice(&self) -> &[u8] {
        &self.value
    }
}

impl Default for SystemTitle {
    fn default() -> Self {
        // Default system title (all zeros)
        // In production, this should never be used
        Self {
            value: [0u8; 8],
        }
    }
}

/// Frame Counter
///
/// A 32-bit counter that increments with each encrypted frame.
/// Used to prevent replay attacks by ensuring frames are processed in order.
///
/// # Thread Safety
/// Frame counter is wrapped in `Arc<Mutex<>>` to allow safe concurrent access.
#[derive(Debug, Clone)]
pub struct FrameCounter {
    /// The current frame counter value
    counter: Arc<Mutex<u32>>,
}

impl FrameCounter {
    /// Create a new Frame Counter starting at 0
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a new Frame Counter with initial value
    ///
    /// # Arguments
    /// * `initial` - Initial frame counter value
    pub fn with_initial(initial: u32) -> Self {
        Self {
            counter: Arc::new(Mutex::new(initial)),
        }
    }

    /// Get the current frame counter value
    ///
    /// # Returns
    /// Current frame counter value
    pub fn get(&self) -> u32 {
        *self.counter.lock().unwrap()
    }

    /// Increment the frame counter and return the new value
    ///
    /// # Returns
    /// The new frame counter value after incrementing
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently.
    pub fn increment(&self) -> u32 {
        let mut counter = self.counter.lock().unwrap();
        *counter = counter.wrapping_add(1);
        *counter
    }

    /// Set the frame counter to a specific value
    ///
    /// # Arguments
    /// * `value` - New frame counter value
    pub fn set(&self, value: u32) {
        let mut counter = self.counter.lock().unwrap();
        *counter = value;
    }

    /// Reset the frame counter to 0
    pub fn reset(&self) {
        self.set(0);
    }
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Key Derivation Function (KDF)
///
/// Derives encryption and authentication keys from a master key (KEK).
///
/// # Algorithm
/// The KDF uses:
/// - Master key (KEK - Key Encryption Key)
/// - System Title (8 bytes)
/// - Key ID (identifies the key type)
///
/// # Key Types
/// - GlobalUnicastEncryptionKey (0): For unicast encryption
/// - GlobalBroadcastEncryptionKey (1): For broadcast encryption
/// - GlobalUnicastAuthenticationKey (2): For unicast authentication
/// - GlobalBroadcastAuthenticationKey (3): For broadcast authentication
pub struct KeyDerivationFunction;

impl KeyDerivationFunction {
    /// Derive a key from master key using KDF
    ///
    /// # Arguments
    /// * `master_key` - Master key (KEK), typically 16 bytes (AES-128)
    /// * `system_title` - System Title (8 bytes)
    /// * `key_id` - Key ID identifying the key type
    ///
    /// # Returns
    /// Derived key (same length as master key)
    ///
    /// # Algorithm
    /// The KDF algorithm (simplified):
    /// 1. Concatenate: System Title (8 bytes) + Key ID (1 byte) + padding
    /// 2. Use AES encryption with master key to derive the key
    ///
    /// # Note
    /// This is a simplified implementation. The full DLMS standard specifies
    /// a more complex KDF algorithm. This implementation should be enhanced
    /// to match the standard exactly.
    pub fn derive_key(
        master_key: &[u8],
        system_title: &SystemTitle,
        key_id: KeyId,
    ) -> DlmsResult<Vec<u8>> {
        if master_key.len() != 16 {
            return Err(DlmsError::InvalidData(format!(
                "Master key must be 16 bytes (AES-128), got {}",
                master_key.len()
            )));
        }

        // Prepare input data for KDF
        // Format: System Title (8 bytes) + Key ID (1 byte) + padding (7 bytes)
        let mut input = Vec::with_capacity(16);
        input.extend_from_slice(system_title.as_bytes());
        input.push(key_id.id());
        // Pad to 16 bytes (AES block size)
        input.extend_from_slice(&[0u8; 7]);

        // Use AES encryption to derive the key
        // In the full standard, this uses a more complex algorithm
        // For now, we use a simplified approach: AES-ECB encrypt the input with master key
        let key = GenericArray::from_slice(master_key);
        let cipher = Aes128::new(key);
        let mut block = GenericArray::clone_from_slice(&input[0..16]);
        cipher.encrypt_block(&mut block);

        Ok(block.to_vec())
    }

    /// Derive encryption key for unicast communication
    ///
    /// # Arguments
    /// * `master_key` - Master key (KEK)
    /// * `system_title` - System Title
    ///
    /// # Returns
    /// Derived unicast encryption key
    pub fn derive_unicast_encryption_key(
        master_key: &[u8],
        system_title: &SystemTitle,
    ) -> DlmsResult<Vec<u8>> {
        Self::derive_key(master_key, system_title, KeyId::GlobalUnicastEncryptionKey)
    }

    /// Derive encryption key for broadcast communication
    ///
    /// # Arguments
    /// * `master_key` - Master key (KEK)
    /// * `system_title` - System Title
    ///
    /// # Returns
    /// Derived broadcast encryption key
    pub fn derive_broadcast_encryption_key(
        master_key: &[u8],
        system_title: &SystemTitle,
    ) -> DlmsResult<Vec<u8>> {
        Self::derive_key(master_key, system_title, KeyId::GlobalBroadcastEncryptionKey)
    }

    /// Derive authentication key for unicast communication
    ///
    /// # Arguments
    /// * `master_key` - Master key (KEK)
    /// * `system_title` - System Title
    ///
    /// # Returns
    /// Derived unicast authentication key
    pub fn derive_unicast_authentication_key(
        master_key: &[u8],
        system_title: &SystemTitle,
    ) -> DlmsResult<Vec<u8>> {
        // Use AuthenticationKey (ID 2) for authentication key derivation
        Self::derive_key(master_key, system_title, KeyId::AuthenticationKey)
    }

    /// Derive authentication key for broadcast communication
    ///
    /// # Arguments
    /// * `master_key` - Master key (KEK)
    /// * `system_title` - System Title
    ///
    /// # Returns
    /// Derived broadcast authentication key
    pub fn derive_broadcast_authentication_key(
        master_key: &[u8],
        system_title: &SystemTitle,
    ) -> DlmsResult<Vec<u8>> {
        // Use AuthenticationKey (ID 2) for authentication key derivation
        // Note: In full standard, there might be separate broadcast authentication key
        Self::derive_key(master_key, system_title, KeyId::AuthenticationKey)
    }
}

/// xDLMS Context
///
/// Manages xDLMS-specific context information for a connection:
/// - System Title (client and server)
/// - Frame Counters (send and receive)
/// - Derived keys
/// - Security parameters
///
/// # Thread Safety
/// All fields are thread-safe and can be accessed concurrently.
#[derive(Debug, Clone)]
pub struct XdlmsContext {
    /// Client System Title
    pub client_system_title: SystemTitle,
    /// Server System Title
    pub server_system_title: SystemTitle,
    /// Send frame counter (for frames we send)
    pub send_frame_counter: FrameCounter,
    /// Receive frame counter (for frames we receive)
    pub receive_frame_counter: FrameCounter,
    /// Master key (KEK) for key derivation
    master_key: Option<Vec<u8>>,
    /// Derived unicast encryption key (cached)
    unicast_encryption_key: Option<Vec<u8>>,
    /// Derived broadcast encryption key (cached)
    broadcast_encryption_key: Option<Vec<u8>>,
}

impl XdlmsContext {
    /// Create a new xDLMS context
    ///
    /// # Arguments
    /// * `client_system_title` - Client System Title
    /// * `server_system_title` - Server System Title
    pub fn new(client_system_title: SystemTitle, server_system_title: SystemTitle) -> Self {
        Self {
            client_system_title,
            server_system_title,
            send_frame_counter: FrameCounter::new(),
            receive_frame_counter: FrameCounter::new(),
            master_key: None,
            unicast_encryption_key: None,
            broadcast_encryption_key: None,
        }
    }

    /// Set the master key (KEK) and derive encryption keys
    ///
    /// # Arguments
    /// * `master_key` - Master key (KEK), typically 16 bytes
    ///
    /// # Returns
    /// `Ok(())` if successful, error otherwise
    pub fn set_master_key(&mut self, master_key: Vec<u8>) -> DlmsResult<()> {
        self.master_key = Some(master_key.clone());

        // Derive encryption keys
        self.unicast_encryption_key = Some(KeyDerivationFunction::derive_unicast_encryption_key(
            &master_key,
            &self.server_system_title,
        )?);

        self.broadcast_encryption_key =
            Some(KeyDerivationFunction::derive_broadcast_encryption_key(
                &master_key,
                &self.server_system_title,
            )?);

        Ok(())
    }

    /// Get the unicast encryption key
    ///
    /// # Returns
    /// Unicast encryption key if master key is set, `None` otherwise
    pub fn unicast_encryption_key(&self) -> Option<&Vec<u8>> {
        self.unicast_encryption_key.as_ref()
    }

    /// Get the broadcast encryption key
    ///
    /// # Returns
    /// Broadcast encryption key if master key is set, `None` otherwise
    pub fn broadcast_encryption_key(&self) -> Option<&Vec<u8>> {
        self.broadcast_encryption_key.as_ref()
    }

    /// Increment send frame counter and return new value
    ///
    /// # Returns
    /// New frame counter value
    pub fn increment_send_counter(&self) -> u32 {
        self.send_frame_counter.increment()
    }

    /// Increment receive frame counter and return new value
    ///
    /// # Returns
    /// New frame counter value
    pub fn increment_receive_counter(&self) -> u32 {
        self.receive_frame_counter.increment()
    }

    /// Get current send frame counter value
    pub fn send_counter(&self) -> u32 {
        self.send_frame_counter.get()
    }

    /// Get current receive frame counter value
    pub fn receive_counter(&self) -> u32 {
        self.receive_frame_counter.get()
    }

    /// Reset frame counters
    pub fn reset_counters(&mut self) {
        self.send_frame_counter.reset();
        self.receive_frame_counter.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_title() {
        let title = SystemTitle::new([1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(title.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);

        let title2 = SystemTitle::from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]).unwrap();
        assert_eq!(title2.as_bytes(), &[9, 10, 11, 12, 13, 14, 15, 16]);
    }

    #[test]
    fn test_frame_counter() {
        let counter = FrameCounter::new();
        assert_eq!(counter.get(), 0);
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.get(), 1);
        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_xdlms_context() {
        let client_title = SystemTitle::new([1, 2, 3, 4, 5, 6, 7, 8]);
        let server_title = SystemTitle::new([9, 10, 11, 12, 13, 14, 15, 16]);
        let mut context = XdlmsContext::new(client_title, server_title);

        let master_key = vec![0u8; 16]; // Test key
        context.set_master_key(master_key).unwrap();

        assert!(context.unicast_encryption_key().is_some());
        assert!(context.broadcast_encryption_key().is_some());
        assert_eq!(context.send_counter(), 0);
        assert_eq!(context.increment_send_counter(), 1);
    }
}
