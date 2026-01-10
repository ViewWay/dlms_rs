//! Frame Check Sequence (FCS) calculation for HDLC

use crate::error::{DlmsError, DlmsResult};

/// FCS calculation constants
const INITIAL_FCS: u16 = 0xFFFF;
const GOOD_FCS: u16 = 0xF0B8;
const KEY: u16 = 0x8408; // Bit-reversed 1021

/// Precomputed FCS table
static FCS_TABLE: once_cell::sync::Lazy<[u16; 256]> = once_cell::sync::Lazy::new(|| {
    let mut table = [0u16; 256];
    for b in 0..=0xFF {
        let mut v = b as u16;
        for _ in 0..8 {
            if (v & 1) == 1 {
                v = (v >> 1) ^ KEY;
            } else {
                v = v >> 1;
            }
        }
        table[b as usize] = v & 0xFFFF;
    }
    table
});

/// Frame Check Sequence calculator
pub struct FcsCalc {
    fcs_value: u16,
}

impl FcsCalc {
    /// Create a new FCS calculator
    pub fn new() -> Self {
        Self {
            fcs_value: INITIAL_FCS,
        }
    }

    /// Reset the FCS value to initial state
    pub fn reset(&mut self) {
        self.fcs_value = INITIAL_FCS;
    }

    /// Update the FCS value with a single byte
    pub fn update(&mut self, data: u8) {
        self.fcs_value = ((self.fcs_value & 0xFFFF) >> 8)
            ^ FCS_TABLE[((self.fcs_value ^ data as u16) & 0xFF) as usize];
    }

    /// Update the FCS value with multiple bytes
    pub fn update_bytes(&mut self, data: &[u8]) {
        for &byte in data {
            self.update(byte);
        }
    }

    /// Update the FCS value with a slice of bytes
    pub fn update_slice(&mut self, data: &[u8], length: usize) {
        for &byte in data.iter().take(length) {
            self.update(byte);
        }
    }

    /// Get the FCS value as bytes (little-endian)
    pub fn fcs_value_bytes(&self) -> [u8; 2] {
        let inv_fcs = self.fcs_value ^ 0xFFFF;
        [(inv_fcs & 0xFF) as u8, ((inv_fcs & 0xFF00) >> 8) as u8]
    }

    /// Validate the current FCS value
    pub fn validate(&self) -> DlmsResult<()> {
        if self.fcs_value != GOOD_FCS {
            Err(DlmsError::FrameInvalid(format!(
                "FCS has wrong value: 0x{:04X}, expected 0x{:04X}",
                self.fcs_value, GOOD_FCS
            )))
        } else {
            Ok(())
        }
    }

    /// Get the current FCS value
    pub fn value(&self) -> u16 {
        self.fcs_value
    }
}

impl Default for FcsCalc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fcs_calc() {
        let mut calc = FcsCalc::new();
        calc.update(0x01);
        calc.update(0x02);
        calc.update(0x03);
        let bytes = calc.fcs_value_bytes();
        assert_eq!(bytes.len(), 2);
    }

    #[test]
    fn test_fcs_reset() {
        let mut calc = FcsCalc::new();
        calc.update(0x01);
        calc.reset();
        assert_eq!(calc.value(), INITIAL_FCS);
    }
}
