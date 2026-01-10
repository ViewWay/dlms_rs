//! COSEM Time type for DLMS/COSEM protocol

use crate::error::{DlmsError, DlmsResult};
use crate::datatypes::cosem_date::{CosemDateFormat, Field};
use serde::{Deserialize, Serialize};
use std::fmt;

const NOT_SPECIFIED: u8 = 0xff;

/// Class representing a COSEM Time
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CosemTime {
    octet_string: [u8; 4],
}

impl CosemTime {
    pub const LENGTH: usize = 4;

    /// Constructs a COSEM Time
    ///
    /// # Arguments
    ///
    /// * `hour` - The hour from 0 to 23, or 0xff if not specified
    /// * `minute` - The minute from 0 to 59, or 0xff if not specified
    /// * `second` - The second from 0 to 59, or 0xff if not specified
    pub fn new(hour: u8, minute: u8, second: u8) -> DlmsResult<Self> {
        Self::new_with_hundredths(hour, minute, second, NOT_SPECIFIED)
    }

    /// Constructs a COSEM Time with hundredths
    ///
    /// # Arguments
    ///
    /// * `hour` - The hour from 0 to 23, or 0xff if not specified
    /// * `minute` - The minute from 0 to 59, or 0xff if not specified
    /// * `second` - The second from 0 to 59, or 0xff if not specified
    /// * `hundredths` - The hundredths seconds from 0 to 99, or 0xff if not specified
    pub fn new_with_hundredths(
        hour: u8,
        minute: u8,
        second: u8,
        hundredths: u8,
    ) -> DlmsResult<Self> {
        Self::verify(hour, "Hour", 0, 23)?;
        Self::verify(minute, "Minute", 0, 59)?;
        Self::verify(second, "Second", 0, 59)?;
        Self::verify(hundredths, "Hundredths", 0, 99)?;

        let octet_string = [hour, minute, second, hundredths];
        Ok(Self { octet_string })
    }

    /// Decode a COSEM Time from a byte array
    pub fn decode(octet_string: &[u8]) -> DlmsResult<Self> {
        if octet_string.len() != Self::LENGTH {
            return Err(DlmsError::InvalidData(format!(
                "Wrong size. Expected {}, got {}",
                Self::LENGTH,
                octet_string.len()
            )));
        }

        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(octet_string);
        Ok(Self { octet_string: bytes })
    }

    fn verify(value: u8, name: &str, lower_bound: u8, upper_bound: u8) -> DlmsResult<()> {
        if (value < lower_bound || value > upper_bound) && value != NOT_SPECIFIED {
            Err(DlmsError::InvalidData(format!(
                "{} is out of range [{}, {}], got {}",
                name, lower_bound, upper_bound, value
            )))
        } else {
            Ok(())
        }
    }
}

impl CosemDateFormat for CosemTime {
    fn encode(&self) -> Vec<u8> {
        self.octet_string.to_vec()
    }

    fn length(&self) -> usize {
        Self::LENGTH
    }

    fn get(&self, field: Field) -> Result<u32, DlmsError> {
        match field {
            Field::Hour => Ok(self.octet_string[0] as u32),
            Field::Minute => Ok(self.octet_string[1] as u32),
            Field::Second => Ok(self.octet_string[2] as u32),
            Field::Hundredths => Ok(self.octet_string[3] as u32),
            _ => Err(DlmsError::InvalidData(format!(
                "Field {:?} not found in CosemTime",
                field
            ))),
        }
    }
}

impl fmt::Display for CosemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hour = self.get(Field::Hour).unwrap_or(0);
        let minute = self.get(Field::Minute).unwrap_or(0);
        let second = self.get(Field::Second).unwrap_or(0);
        write!(f, "{:02}:{:02}:{:02}", hour, minute, second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosem_time_new() {
        let time = CosemTime::new(14, 30, 45).unwrap();
        assert_eq!(time.get(Field::Hour).unwrap(), 14);
        assert_eq!(time.get(Field::Minute).unwrap(), 30);
        assert_eq!(time.get(Field::Second).unwrap(), 45);
    }

    #[test]
    fn test_cosem_time_decode() {
        let bytes = [0x0E, 0x1E, 0x2D, 0xFF]; // 14:30:45
        let time = CosemTime::decode(&bytes).unwrap();
        assert_eq!(time.get(Field::Hour).unwrap(), 14);
    }

    #[test]
    fn test_cosem_time_invalid() {
        assert!(CosemTime::new(24, 0, 0).is_err());
        assert!(CosemTime::new(0, 60, 0).is_err());
        assert!(CosemTime::new(0, 0, 60).is_err());
    }
}
