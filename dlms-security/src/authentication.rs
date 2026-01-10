//! Authentication functionality for DLMS/COSEM

use crate::error::{DlmsError, DlmsResult};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// GMAC authentication
pub struct GmacAuth {
    key: Vec<u8>,
}

impl GmacAuth {
    /// Create a new GMAC authentication context
    pub fn new(key: &[u8]) -> Self {
        Self {
            key: key.to_vec(),
        }
    }

    /// Generate GMAC for data
    pub fn generate_gmac(&self, data: &[u8], aad: &[u8]) -> DlmsResult<Vec<u8>> {
        // GMAC is essentially AES-GCM authentication tag
        // For DLMS, we use HMAC-SHA256 as a simplified implementation
        // In production, this should use actual AES-GCM GMAC
        
        let mut mac = HmacSha256::new_from_slice(&self.key)
            .map_err(|e| DlmsError::Security(format!("Failed to create HMAC: {}", e)))?;
        
        mac.update(aad);
        mac.update(data);
        
        let result = mac.finalize();
        Ok(result.into_bytes().to_vec())
    }

    /// Verify GMAC
    pub fn verify_gmac(&self, data: &[u8], aad: &[u8], gmac: &[u8]) -> DlmsResult<bool> {
        let computed = self.generate_gmac(data, aad)?;
        Ok(computed == gmac)
    }
}

/// Low-level authentication (password-based)
pub struct LowAuth {
    password: Vec<u8>,
}

impl LowAuth {
    /// Create a new low-level authentication context
    pub fn new(password: &[u8]) -> Self {
        Self {
            password: password.to_vec(),
        }
    }

    /// Generate challenge response
    pub fn generate_challenge_response(&self, challenge: &[u8]) -> DlmsResult<Vec<u8>> {
        // Low-level authentication uses password directly
        // The challenge response is typically the password XOR'd with challenge
        let mut response = challenge.to_vec();
        for (i, &p) in self.password.iter().enumerate() {
            if i < response.len() {
                response[i] ^= p;
            }
        }
        Ok(response)
    }

    /// Verify challenge response
    pub fn verify_challenge_response(&self, challenge: &[u8], response: &[u8]) -> DlmsResult<bool> {
        let expected = self.generate_challenge_response(challenge)?;
        Ok(expected == response)
    }
}

/// High-level security authentication (HLS5-GMAC)
pub struct Hls5GmacAuth {
    authentication_key: Vec<u8>,
    encryption_key: Vec<u8>,
}

impl Hls5GmacAuth {
    /// Create a new HLS5-GMAC authentication context
    pub fn new(authentication_key: &[u8], encryption_key: &[u8]) -> DlmsResult<Self> {
        if authentication_key.len() != encryption_key.len() {
            return Err(DlmsError::Security(
                "Authentication key and encryption key must have the same length".to_string(),
            ));
        }

        Ok(Self {
            authentication_key: authentication_key.to_vec(),
            encryption_key: encryption_key.to_vec(),
        })
    }

    /// Generate authentication tag
    pub fn generate_auth_tag(&self, data: &[u8], system_title: &[u8], frame_counter: u32) -> DlmsResult<Vec<u8>> {
        // HLS5-GMAC uses AES-GCM authentication
        // Build AAD: system_title + frame_counter
        let mut aad = Vec::new();
        aad.extend_from_slice(system_title);
        aad.extend_from_slice(&frame_counter.to_be_bytes());

        // Use authentication key for GMAC
        let gmac = GmacAuth::new(&self.authentication_key);
        gmac.generate_gmac(data, &aad)
    }

    /// Verify authentication tag
    pub fn verify_auth_tag(
        &self,
        data: &[u8],
        system_title: &[u8],
        frame_counter: u32,
        auth_tag: &[u8],
    ) -> DlmsResult<bool> {
        let computed = self.generate_auth_tag(data, system_title, frame_counter)?;
        Ok(computed == auth_tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gmac_auth() {
        let key = [0u8; 16];
        let auth = GmacAuth::new(&key);
        let data = b"test data";
        let aad = b"additional data";

        let gmac = auth.generate_gmac(data, aad).unwrap();
        assert!(auth.verify_gmac(data, aad, &gmac).unwrap());
    }

    #[test]
    fn test_low_auth() {
        let password = b"password123";
        let auth = LowAuth::new(password);
        let challenge = b"challenge";

        let response = auth.generate_challenge_response(challenge).unwrap();
        assert!(auth.verify_challenge_response(challenge, &response).unwrap());
    }
}
