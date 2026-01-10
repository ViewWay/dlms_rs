//! Encryption functionality for DLMS/COSEM

use crate::error::{DlmsError, DlmsResult};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
    Aes128Gcm, Key, Nonce,
};

/// AES-GCM encryption context
pub struct AesGcmEncryption {
    cipher: Aes128Gcm,
}

impl AesGcmEncryption {
    /// Create a new AES-GCM encryption context
    pub fn new(key: &[u8]) -> DlmsResult<Self> {
        if key.len() != 16 {
            return Err(DlmsError::Security(format!(
                "Invalid AES-128 key length: expected 16 bytes, got {}",
                key.len()
            )));
        }

        let key = Key::<Aes128Gcm>::from_slice(key);
        let cipher = Aes128Gcm::new(key);

        Ok(Self { cipher })
    }

    /// Encrypt data with AES-GCM
    pub fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> DlmsResult<(Vec<u8>, Vec<u8>)> {
        // Generate a random nonce (12 bytes for AES-GCM)
        let nonce = Aes128Gcm::generate_nonce(&mut OsRng);

        // Create payload with plaintext and AAD
        let payload = Payload {
            msg: plaintext,
            aad,
        };

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(&nonce, payload)
            .map_err(|e| DlmsError::Security(format!("Encryption failed: {}", e)))?;

        Ok((ciphertext, nonce.to_vec()))
    }

    /// Decrypt data with AES-GCM
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8], aad: &[u8]) -> DlmsResult<Vec<u8>> {
        if nonce.len() != 12 {
            return Err(DlmsError::Security(format!(
                "Invalid nonce length: expected 12 bytes, got {}",
                nonce.len()
            )));
        }

        let nonce = Nonce::from_slice(nonce);

        // Create payload with ciphertext and AAD
        let payload = Payload {
            msg: ciphertext,
            aad,
        };

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, payload)
            .map_err(|e| DlmsError::Security(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }
}

/// Security control byte for DLMS APDU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityControl {
    byte: u8,
}

impl SecurityControl {
    /// Create a new security control byte
    pub fn new(
        security_suite_id: u8,
        authenticated: bool,
        encrypted: bool,
        key_set: bool,
    ) -> Self {
        let mut byte = security_suite_id & 0x0F;
        if authenticated {
            byte |= 0x10;
        }
        if encrypted {
            byte |= 0x20;
        }
        if key_set {
            byte |= 0x40;
        }
        Self { byte }
    }

    /// Decode from byte
    pub fn from_byte(byte: u8) -> Self {
        Self { byte }
    }

    /// Get the byte value
    pub fn to_byte(&self) -> u8 {
        self.byte
    }

    /// Get security suite ID
    pub fn security_suite_id(&self) -> u8 {
        self.byte & 0x0F
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        (self.byte & 0x10) != 0
    }

    /// Check if encrypted
    pub fn is_encrypted(&self) -> bool {
        (self.byte & 0x20) != 0
    }

    /// Check if key set
    pub fn is_key_set(&self) -> bool {
        (self.byte & 0x40) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_encrypt_decrypt() {
        let key = [0u8; 16];
        let enc = AesGcmEncryption::new(&key).unwrap();
        let plaintext = b"Hello, World!";
        let aad = b"";

        let (ciphertext, nonce) = enc.encrypt(plaintext, aad).unwrap();
        let decrypted = enc.decrypt(&ciphertext, &nonce, aad).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_security_control() {
        let ctrl = SecurityControl::new(0, true, true, false);
        assert!(ctrl.is_authenticated());
        assert!(ctrl.is_encrypted());
        assert!(!ctrl.is_key_set());
    }
}
