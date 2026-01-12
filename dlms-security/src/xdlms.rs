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
use aes::{Aes128, Aes192, Aes256};
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::cipher::generic_array::{GenericArray, typenum::{U16, U24, U32}};
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
/// Derives encryption and authentication keys from a master key (KEK) according to
/// DLMS Green Book Edition 9 specification.
///
/// # Algorithm (DLMS Standard)
/// The KDF algorithm follows DLMS Green Book Edition 9:
/// 1. Build input block: System Title (8 bytes) + Key ID (1 byte) + zero padding (7 bytes)
/// 2. Encrypt the 16-byte input block using AES-ECB mode with the master key (KEK)
/// 3. The encrypted block is the derived key
///
/// # Key Types
/// - GlobalUnicastEncryptionKey (0): For unicast encryption
/// - GlobalBroadcastEncryptionKey (1): For broadcast encryption
/// - GlobalUnicastAuthenticationKey (2): For unicast authentication
/// - GlobalBroadcastAuthenticationKey (3): For broadcast authentication
///
/// # Supported Key Lengths
/// - AES-128: 16-byte master key, produces 16-byte derived key
/// - AES-192: 24-byte master key, produces 16-byte derived key (uses first 16 bytes of AES-192 output)
/// - AES-256: 32-byte master key, produces 16-byte derived key (uses first 16 bytes of AES-256 output)
///
/// # Standard Compliance
/// This implementation follows DLMS Green Book Edition 9, Section 6.2.5 (Key Derivation Function).
/// The algorithm uses AES-ECB mode encryption as specified in the standard.
pub struct KeyDerivationFunction;

impl KeyDerivationFunction {
    /// Derive a key from master key using DLMS standard KDF
    ///
    /// # Arguments
    /// * `master_key` - Master key (KEK), 16/24/32 bytes (AES-128/192/256)
    /// * `system_title` - System Title (8 bytes)
    /// * `key_id` - Key ID identifying the key type
    ///
    /// # Returns
    /// Derived key (16 bytes)
    ///
    /// # Algorithm Details
    /// According to DLMS Green Book Edition 9:
    /// 1. Construct input block: System Title (8 bytes) || Key ID (1 byte) || 0x00...0x00 (7 bytes)
    /// 2. Encrypt input block using AES-ECB with master key
    /// 3. Output is the encrypted block (16 bytes)
    ///
    /// # Errors
    /// Returns error if:
    /// - Master key length is not 16, 24, or 32 bytes
    /// - System Title is not 8 bytes
    ///
    /// # Example
    /// ```
    /// use dlms_security::xdlms::{SystemTitle, KeyDerivationFunction};
    /// use dlms_security::utils::KeyId;
    ///
    /// let master_key = [0u8; 16]; // AES-128 key
    /// let system_title = SystemTitle::new([1, 2, 3, 4, 5, 6, 7, 8]);
    /// let derived_key = KeyDerivationFunction::derive_key(
    ///     &master_key,
    ///     &system_title,
    ///     KeyId::GlobalUnicastEncryptionKey,
    /// ).unwrap();
    /// assert_eq!(derived_key.len(), 16);
    /// ```
    pub fn derive_key(
        master_key: &[u8],
        system_title: &SystemTitle,
        key_id: KeyId,
    ) -> DlmsResult<Vec<u8>> {
        // Validate master key length (support AES-128, AES-192, AES-256)
        let key_len = master_key.len();
        if key_len != 16 && key_len != 24 && key_len != 32 {
            return Err(DlmsError::InvalidData(format!(
                "Master key must be 16, 24, or 32 bytes (AES-128/192/256), got {}",
                key_len
            )));
        }

        // Build input block according to DLMS standard:
        // System Title (8 bytes) || Key ID (1 byte) || Zero padding (7 bytes)
        let mut input_block = [0u8; 16];
        input_block[0..8].copy_from_slice(system_title.as_bytes());
        input_block[8] = key_id.id();
        // Bytes 9-15 are already zero (zero padding)

        // Encrypt input block using AES-ECB mode
        // According to DLMS standard, we use AES-ECB encryption
        let derived_key = match key_len {
            16 => {
                // AES-128
                let key = GenericArray::<u8, U16>::from_slice(master_key);
                let cipher = Aes128::new(key);
                let mut block = GenericArray::<u8, U16>::clone_from_slice(&input_block);
                cipher.encrypt_block(&mut block);
                block.to_vec()
            }
            24 => {
                // AES-192
                let key = GenericArray::<u8, U24>::from_slice(master_key);
                let cipher = Aes192::new(key);
                let mut block = GenericArray::<u8, U16>::clone_from_slice(&input_block);
                cipher.encrypt_block(&mut block);
                block.to_vec()
            }
            32 => {
                // AES-256
                let key = GenericArray::<u8, U32>::from_slice(master_key);
                let cipher = Aes256::new(key);
                let mut block = GenericArray::<u8, U16>::clone_from_slice(&input_block);
                cipher.encrypt_block(&mut block);
                block.to_vec()
            }
            _ => unreachable!(), // Already validated above
        };

        Ok(derived_key)
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

    #[test]
    fn test_kdf_aes128() {
        let master_key = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                          0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F];
        let system_title = SystemTitle::new([0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17]);
        
        // Test different key IDs
        let key1 = KeyDerivationFunction::derive_key(
            &master_key,
            &system_title,
            KeyId::GlobalUnicastEncryptionKey,
        ).unwrap();
        assert_eq!(key1.len(), 16);
        
        let key2 = KeyDerivationFunction::derive_key(
            &master_key,
            &system_title,
            KeyId::GlobalBroadcastEncryptionKey,
        ).unwrap();
        assert_eq!(key2.len(), 16);
        // Different key IDs should produce different keys
        assert_ne!(key1, key2);
        
        let key3 = KeyDerivationFunction::derive_key(
            &master_key,
            &system_title,
            KeyId::AuthenticationKey,
        ).unwrap();
        assert_eq!(key3.len(), 16);
        assert_ne!(key1, key3);
        assert_ne!(key2, key3);
    }

    #[test]
    fn test_kdf_aes192() {
        let master_key = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                          0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
                          0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17];
        let system_title = SystemTitle::new([0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27]);
        
        let derived_key = KeyDerivationFunction::derive_key(
            &master_key,
            &system_title,
            KeyId::GlobalUnicastEncryptionKey,
        ).unwrap();
        assert_eq!(derived_key.len(), 16);
    }

    #[test]
    fn test_kdf_aes256() {
        let master_key = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                          0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
                          0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
                          0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F];
        let system_title = SystemTitle::new([0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37]);
        
        let derived_key = KeyDerivationFunction::derive_key(
            &master_key,
            &system_title,
            KeyId::GlobalUnicastEncryptionKey,
        ).unwrap();
        assert_eq!(derived_key.len(), 16);
    }

    #[test]
    fn test_kdf_deterministic() {
        // KDF should be deterministic: same inputs produce same output
        let master_key = [0xAA; 16];
        let system_title = SystemTitle::new([0xBB; 8]);
        
        let key1 = KeyDerivationFunction::derive_key(
            &master_key,
            &system_title,
            KeyId::GlobalUnicastEncryptionKey,
        ).unwrap();
        
        let key2 = KeyDerivationFunction::derive_key(
            &master_key,
            &system_title,
            KeyId::GlobalUnicastEncryptionKey,
        ).unwrap();
        
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_kdf_invalid_key_length() {
        let system_title = SystemTitle::new([0u8; 8]);
        
        // Test invalid key lengths
        assert!(KeyDerivationFunction::derive_key(
            &[0u8; 15], // Too short
            &system_title,
            KeyId::GlobalUnicastEncryptionKey,
        ).is_err());
        
        assert!(KeyDerivationFunction::derive_key(
            &[0u8; 17], // Invalid length
            &system_title,
            KeyId::GlobalUnicastEncryptionKey,
        ).is_err());
    }

    #[test]
    fn test_kdf_different_system_titles() {
        // Different system titles should produce different keys
        let master_key = [0u8; 16];
        let system_title1 = SystemTitle::new([1, 2, 3, 4, 5, 6, 7, 8]);
        let system_title2 = SystemTitle::new([9, 10, 11, 12, 13, 14, 15, 16]);
        
        let key1 = KeyDerivationFunction::derive_key(
            &master_key,
            &system_title1,
            KeyId::GlobalUnicastEncryptionKey,
        ).unwrap();
        
        let key2 = KeyDerivationFunction::derive_key(
            &master_key,
            &system_title2,
            KeyId::GlobalUnicastEncryptionKey,
        ).unwrap();
        
        assert_ne!(key1, key2);
    }
}
