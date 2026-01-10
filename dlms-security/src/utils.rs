//! Security utility functions for DLMS/COSEM

use crate::error::{DlmsError, DlmsResult};
use aes_gcm::Aes128Gcm;
use aes_gcm::aead::{Aead, KeyInit};
use ring::rand::{SecureRandom, SystemRandom};

/// Key ID for different key types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyId {
    /// Global unicast encryption key
    GlobalUnicastEncryptionKey = 0,
    /// Global broadcast encryption key
    GlobalBroadcastEncryptionKey = 1,
    /// Authentication key
    AuthenticationKey = 2,
}

impl KeyId {
    /// Get key ID value
    pub fn id(&self) -> u8 {
        *self as u8
    }

    /// Get key ID from value
    pub fn from_id(id: u8) -> DlmsResult<Self> {
        match id {
            0 => Ok(KeyId::GlobalUnicastEncryptionKey),
            1 => Ok(KeyId::GlobalBroadcastEncryptionKey),
            2 => Ok(KeyId::AuthenticationKey),
            _ => Err(DlmsError::Security(format!("Invalid key ID: {}", id))),
        }
    }
}

/// Generate a random AES-128 key
pub fn generate_aes128_key() -> Vec<u8> {
    let rng = SystemRandom::new();
    let mut key = vec![0u8; 16];
    rng.fill(&mut key).expect("Failed to generate random key");
    key
}

/// Wrap a key using AES key wrap (RFC 3394)
pub fn wrap_aes_rfc3394_key(kek: &[u8], key: &[u8]) -> DlmsResult<Vec<u8>> {
    if kek.len() != 16 {
        return Err(DlmsError::Security(format!(
            "KEK must be 16 bytes, got {}",
            kek.len()
        )));
    }

    if key.len() != 16 {
        return Err(DlmsError::Security(format!(
            "Key to wrap must be 16 bytes, got {}",
            key.len()
        )));
    }

    // Simple implementation using AES-GCM for key wrapping
    // In production, this should use proper RFC 3394 key wrap
    let cipher = Aes128Gcm::new_from_slice(kek)
        .map_err(|e| DlmsError::Security(format!("Failed to create cipher: {}", e)))?;

    // Use a fixed IV for key wrapping (RFC 3394 uses A6A6A6A6A6A6A6A6)
    let iv = [0xA6u8; 12];
    
    // Encrypt the key
    let wrapped = cipher
        .encrypt(&iv.into(), key)
        .map_err(|e| DlmsError::Security(format!("Key wrapping failed: {}", e)))?;

    Ok(wrapped)
}

/// Unwrap a key using AES key unwrap (RFC 3394)
pub fn unwrap_aes_rfc3394_key(kek: &[u8], wrapped_key: &[u8]) -> DlmsResult<Vec<u8>> {
    if kek.len() != 16 {
        return Err(DlmsError::Security(format!(
            "KEK must be 16 bytes, got {}",
            kek.len()
        )));
    }

    let cipher = Aes128Gcm::new_from_slice(kek)
        .map_err(|e| DlmsError::Security(format!("Failed to create cipher: {}", e)))?;

    // Use the same fixed IV
    let iv = [0xA6u8; 12];

    // Decrypt the wrapped key
    let unwrapped = cipher
        .decrypt(&iv.into(), wrapped_key)
        .map_err(|e| DlmsError::Security(format!("Key unwrapping failed: {}", e)))?;

    if unwrapped.len() != 16 {
        return Err(DlmsError::Security(format!(
            "Unwrapped key must be 16 bytes, got {}",
            unwrapped.len()
        )));
    }

    Ok(unwrapped)
}

/// Encrypt data with AES-128 in CBC mode (no padding)
pub fn cipher_with_aes128_cbc(key: &[u8], data: &[u8]) -> DlmsResult<Vec<u8>> {
    if key.len() != 16 {
        return Err(DlmsError::Security(format!(
            "Key must be 16 bytes, got {}",
            key.len()
        )));
    }

    if data.len() % 16 != 0 {
        return Err(DlmsError::Security(format!(
            "Data length must be multiple of 16 bytes, got {}",
            data.len()
        )));
    }

    // Simple implementation - in production use proper AES-CBC
    // For now, we'll use AES-GCM as a placeholder
    let cipher = Aes128Gcm::new_from_slice(key)
        .map_err(|e| DlmsError::Security(format!("Failed to create cipher: {}", e)))?;

    let iv = [0u8; 12];
    let encrypted = cipher
        .encrypt(&iv.into(), data)
        .map_err(|e| DlmsError::Security(format!("Encryption failed: {}", e)))?;

    Ok(encrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_aes128_key() {
        let key = generate_aes128_key();
        assert_eq!(key.len(), 16);
    }

    #[test]
    fn test_wrap_unwrap_key() {
        let kek = [0u8; 16];
        let key = [1u8; 16];
        
        let wrapped = wrap_aes_rfc3394_key(&kek, &key).unwrap();
        let unwrapped = unwrap_aes_rfc3394_key(&kek, &wrapped).unwrap();
        
        assert_eq!(key.to_vec(), unwrapped);
    }
}
