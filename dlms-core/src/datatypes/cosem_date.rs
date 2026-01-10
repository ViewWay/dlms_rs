//! COSEM date/time types for DLMS/COSEM protocol

use crate::error::{DlmsError, DlmsResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Field types for COSEM date/time formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Year,
    Month,
    DayOfMonth,
    DayOfWeek,
    Hour,
    Minute,
    Second,
    Hundredths,
    Deviation,
    ClockStatus,
}

/// Trait for COSEM date/time format types
pub trait CosemDateFormat {
    /// Encode the date/time to a byte array
    fn encode(&self) -> Vec<u8>;
    
    /// Get the length of the encoded byte array
    fn length(&self) -> usize;
    
    /// Get a field value
    fn get(&self, field: Field) -> Result<u32, DlmsError>;
}

/// Constants for COSEM Date
const NOT_SPECIFIED: u8 = 0xff;
const DAYLIGHT_SAVINGS_END: u8 = 0xfd;
const DAYLIGHT_SAVINGS_BEGIN: u8 = 0xfe;
const LAST_DAY_OF_MONTH: u8 = 0xfe;
const SECOND_LAST_DAY_OF_MONTH: u8 = 0xfd;

/// Month enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl Month {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Month::January),
            2 => Some(Month::February),
            3 => Some(Month::March),
            4 => Some(Month::April),
            5 => Some(Month::May),
            6 => Some(Month::June),
            7 => Some(Month::July),
            8 => Some(Month::August),
            9 => Some(Month::September),
            10 => Some(Month::October),
            11 => Some(Month::November),
            12 => Some(Month::December),
            _ => None,
        }
    }

    pub fn value(&self) -> u8 {
        *self as u8
    }
}

/// Class representing a COSEM Date
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CosemDate {
    octet_string: [u8; 5],
}

impl CosemDate {
    pub const LENGTH: usize = 5;

    /// Constructs a COSEM Date
    ///
    /// # Arguments
    ///
    /// * `year` - The year from 0 to 0xffff
    /// * `month` - The month from 1 to 12, or 0xff if not specified
    /// * `day_of_month` - The day of the month from 1 to 31, or special values:
    ///   - 0xfe for the last day of a month
    ///   - 0xfd for the second last day of a month
    ///   - 0xff if not specified
    ///
    /// # Errors
    ///
    /// Returns an error if parameters are out of range
    pub fn new(year: u16, month: u8, day_of_month: u8) -> DlmsResult<Self> {
        Self::new_with_day_of_week(year, month, day_of_month, NOT_SPECIFIED)
    }

    /// Constructs a COSEM Date with month enum
    pub fn new_with_month(year: u16, month: Month, day_of_month: u8) -> DlmsResult<Self> {
        Self::new_with_day_of_week(year, month.value(), day_of_month, NOT_SPECIFIED)
    }

    /// Constructs a COSEM Date with day of week
    ///
    /// # Arguments
    ///
    /// * `year` - The year from 0 to 0xffff
    /// * `month` - The month from 1 to 12, or 0xff if not specified
    /// * `day_of_month` - The day of the month from 1 to 31, or special values
    /// * `day_of_week` - The day of week from 1 to 7 (1 is Monday), or 0xff if not specified
    pub fn new_with_day_of_week(
        year: u16,
        month: u8,
        day_of_month: u8,
        day_of_week: u8,
    ) -> DlmsResult<Self> {
        Self::verify_year(year)?;
        Self::verify_month(month)?;
        Self::verify_day_of_week(day_of_week)?;
        Self::verify_day_of_month(day_of_month)?;

        let mut octet_string = [0u8; 5];
        octet_string[0] = ((year & 0xff00) >> 8) as u8;
        octet_string[1] = (year & 0xff) as u8;
        octet_string[2] = month;
        octet_string[3] = day_of_month;
        octet_string[4] = day_of_week;

        Ok(Self { octet_string })
    }

    /// Decode a COSEM Date from a byte array
    pub fn decode(octet_string: &[u8]) -> DlmsResult<Self> {
        if octet_string.len() != Self::LENGTH {
            return Err(DlmsError::InvalidData(format!(
                "Wrong size. Expected {}, got {}",
                Self::LENGTH,
                octet_string.len()
            )));
        }

        let mut bytes = [0u8; 5];
        bytes.copy_from_slice(octet_string);
        Ok(Self { octet_string: bytes })
    }

    fn verify_year(_year: u16) -> DlmsResult<()> {
        // Year is u16, so it's always in valid range [0, 0xffff]
        // No validation needed as the type system enforces the constraint
        Ok(())
    }

    fn verify_month(month: u8) -> DlmsResult<()> {
        let month_too_small = month < 1;
        let is_not_status_flag = month != DAYLIGHT_SAVINGS_END
            && month != DAYLIGHT_SAVINGS_BEGIN
            && month != NOT_SPECIFIED;
        let month_too_large = month > 12 && is_not_status_flag;

        if month_too_small || month_too_large {
            Err(DlmsError::InvalidData(format!(
                "Parameter month is out of range, got {}",
                month
            )))
        } else {
            Ok(())
        }
    }

    fn verify_day_of_month(day_of_month: u8) -> DlmsResult<()> {
        let month_too_small = day_of_month < 1;
        let month_too_large = day_of_month > 31;
        let is_not_status_flag = day_of_month != SECOND_LAST_DAY_OF_MONTH
            && day_of_month != LAST_DAY_OF_MONTH
            && day_of_month != NOT_SPECIFIED;

        if month_too_small || (month_too_large && is_not_status_flag) {
            Err(DlmsError::InvalidData(format!(
                "Parameter day of month is out of range, got {}",
                day_of_month
            )))
        } else {
            Ok(())
        }
    }

    fn verify_day_of_week(day_of_week: u8) -> DlmsResult<()> {
        if (day_of_week < 1 || day_of_week > 7) && day_of_week != NOT_SPECIFIED {
            Err(DlmsError::InvalidData(format!(
                "Parameter day of week is out of range [1, 7], got {}",
                day_of_week
            )))
        } else {
            Ok(())
        }
    }
}

impl CosemDateFormat for CosemDate {
    fn encode(&self) -> Vec<u8> {
        self.octet_string.to_vec()
    }

    fn length(&self) -> usize {
        Self::LENGTH
    }

    fn get(&self, field: Field) -> Result<u32, DlmsError> {
        match field {
            Field::Year => {
                let year = ((self.octet_string[0] as u16) << 8) | (self.octet_string[1] as u16);
                Ok(year as u32)
            }
            Field::Month => Ok(self.octet_string[2] as u32),
            Field::DayOfMonth => Ok(self.octet_string[3] as u32),
            Field::DayOfWeek => Ok(self.octet_string[4] as u32),
            _ => Err(DlmsError::InvalidData(format!(
                "Field {:?} not found in CosemDate",
                field
            ))),
        }
    }
}

impl fmt::Display for CosemDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let year = self.get(Field::Year).unwrap_or(0);
        let month = self.get(Field::Month).unwrap_or(0);
        let day = self.get(Field::DayOfMonth).unwrap_or(0);
        write!(f, "{:04}-{:02}-{:02}", year, month, day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosem_date_new() {
        let date = CosemDate::new(2024, 1, 15).unwrap();
        assert_eq!(date.get(Field::Year).unwrap(), 2024);
        assert_eq!(date.get(Field::Month).unwrap(), 1);
        assert_eq!(date.get(Field::DayOfMonth).unwrap(), 15);
    }

    #[test]
    fn test_cosem_date_decode() {
        let bytes = [0x07, 0xE8, 0x01, 0x0F, 0xFF]; // 2024-01-15
        let date = CosemDate::decode(&bytes).unwrap();
        assert_eq!(date.get(Field::Year).unwrap(), 2024);
    }

    #[test]
    fn test_cosem_date_invalid() {
        assert!(CosemDate::new(0x10000, 1, 1).is_err());
        assert!(CosemDate::new(2024, 13, 1).is_err());
        assert!(CosemDate::new(2024, 1, 32).is_err());
    }
}
