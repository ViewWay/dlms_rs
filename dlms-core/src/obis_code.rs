use crate::error::{DlmsError, DlmsResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// OBIS (Object Identification System) code for identifying COSEM objects
///
/// OBIS codes are 6-byte identifiers used in DLMS/COSEM to uniquely identify
/// objects in a logical device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObisCode {
    bytes: [u8; 6],
}

impl ObisCode {
    /// Create a new OBIS code from individual bytes
    ///
    /// # Arguments
    ///
    /// * `a` - First byte (A value)
    /// * `b` - Second byte (B value)
    /// * `c` - Third byte (C value)
    /// * `d` - Fourth byte (D value)
    /// * `e` - Fifth byte (E value)
    /// * `f` - Sixth byte (F value)
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Self {
        Self {
            bytes: [a, b, c, d, e, f],
        }
    }
    
    /// Parse an OBIS code from string format
    ///
    /// Supports formats like:
    /// - "1.1.1.8.0.255"
    /// - "1-b:8.29.0*2"
    ///
    /// # Arguments
    ///
    /// * `s` - String representation of OBIS code
    ///
    /// # Returns
    ///
    /// Returns `Ok(ObisCode)` if parsing succeeds, `Err(DlmsError)` otherwise
    pub fn from_string(s: &str) -> DlmsResult<Self> {
        // Simple format: "1.1.1.8.0.255"
        if let Ok(bytes) = Self::parse_dot_format(s) {
            return Ok(bytes);
        }
        
        // Extended format: "1-b:8.29.0*2"
        if let Ok(bytes) = Self::parse_extended_format(s) {
            return Ok(bytes);
        }
        
        Err(DlmsError::InvalidData(format!("Invalid OBIS code format: {}", s)))
    }
    
    fn parse_dot_format(s: &str) -> DlmsResult<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 6 {
            return Err(DlmsError::InvalidData("Expected 6 dot-separated values".to_string()));
        }
        
        let mut bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            bytes[i] = part.parse::<u8>()
                .map_err(|_| DlmsError::InvalidData(format!("Invalid byte value: {}", part)))?;
        }
        
        Ok(Self { bytes })
    }
    
    fn parse_extended_format(s: &str) -> DlmsResult<Self> {
        // TODO: Implement extended format parsing
        // Format: "A-B:C.D.E*F" or "A-B:C.D.E"
        Err(DlmsError::InvalidData("Extended format not yet implemented".to_string()))
    }
    
    /// Get the OBIS code as a byte array
    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.bytes
    }

    /// Get the OBIS code as a copied byte array
    pub fn to_bytes(&self) -> [u8; 6] {
        self.bytes
    }
    
    /// Get the A value (first byte)
    pub fn a(&self) -> u8 {
        self.bytes[0]
    }
    
    /// Get the B value (second byte)
    pub fn b(&self) -> u8 {
        self.bytes[1]
    }
    
    /// Get the C value (third byte)
    pub fn c(&self) -> u8 {
        self.bytes[2]
    }
    
    /// Get the D value (fourth byte)
    pub fn d(&self) -> u8 {
        self.bytes[3]
    }
    
    /// Get the E value (fifth byte)
    pub fn e(&self) -> u8 {
        self.bytes[4]
    }
    
    /// Get the F value (sixth byte)
    pub fn f(&self) -> u8 {
        self.bytes[5]
    }
}

impl fmt::Display for ObisCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}.{}.{}",
            self.bytes[0], self.bytes[1], self.bytes[2],
            self.bytes[3], self.bytes[4], self.bytes[5]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_obis_code_new() {
        let code = ObisCode::new(1, 1, 1, 8, 0, 255);
        assert_eq!(code.a(), 1);
        assert_eq!(code.f(), 255);
    }
    
    #[test]
    fn test_obis_code_from_string() {
        let code = ObisCode::from_string("1.1.1.8.0.255").unwrap();
        assert_eq!(code, ObisCode::new(1, 1, 1, 8, 0, 255));
    }
    
    #[test]
    fn test_obis_code_display() {
        let code = ObisCode::new(1, 1, 1, 8, 0, 255);
        assert_eq!(format!("{}", code), "1.1.1.8.0.255");
    }
}
