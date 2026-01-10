//! COSEM DateTime type for DLMS/COSEM protocol

use crate::error::{DlmsError, DlmsResult};
use crate::datatypes::cosem_date::{CosemDate, CosemDateFormat, Field};
use crate::datatypes::cosem_time::CosemTime;
use serde::{Deserialize, Serialize};
use std::fmt;

const DEVIATION_NOT_SPECIFIED: i16 = 0x8000;

/// Clock status flags for COSEM DateTime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockStatus {
    InvalidValue = 0x01,
    DoubtfulValue = 0x02,
    DifferentClockBase = 0x04,
    InvalidClockStatus = 0x08,
    DaylightSavingActive = 0x80,
}

impl ClockStatus {
    /// Convert clock status flags to a byte
    pub fn to_byte(statuses: &[ClockStatus]) -> u8 {
        let mut byte = 0u8;
        for status in statuses {
            byte |= *status as u8;
        }
        byte
    }

    /// Parse clock status from a byte
    pub fn from_byte(byte: u8) -> Vec<ClockStatus> {
        let mut statuses = Vec::new();
        if byte & ClockStatus::InvalidValue as u8 != 0 {
            statuses.push(ClockStatus::InvalidValue);
        }
        if byte & ClockStatus::DoubtfulValue as u8 != 0 {
            statuses.push(ClockStatus::DoubtfulValue);
        }
        if byte & ClockStatus::DifferentClockBase as u8 != 0 {
            statuses.push(ClockStatus::DifferentClockBase);
        }
        if byte & ClockStatus::InvalidClockStatus as u8 != 0 {
            statuses.push(ClockStatus::InvalidClockStatus);
        }
        if byte & ClockStatus::DaylightSavingActive as u8 != 0 {
            statuses.push(ClockStatus::DaylightSavingActive);
        }
        statuses
    }
}

/// Class representing a COSEM DateTime
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CosemDateTime {
    date: CosemDate,
    time: CosemTime,
    deviation: i16,
    clock_status: u8,
}

impl CosemDateTime {
    pub const LENGTH: usize = 12;

    /// Constructs a COSEM DateTime
    ///
    /// # Arguments
    ///
    /// * `year` - The year from 0 to 0xffff
    /// * `month` - The month from 1 to 12, or 0xff if not specified
    /// * `day_of_month` - The day of the month from 1 to 31, or special values
    /// * `hour` - The hour from 0 to 23, or 0xff if not specified
    /// * `minute` - The minute from 0 to 59, or 0xff if not specified
    /// * `second` - The second from 0 to 59, or 0xff if not specified
    /// * `deviation` - The deviation in minutes from local time to GMT (-720 to 720), or 0x8000 if not specified
    /// * `clock_status` - Clock status flags
    pub fn new(
        year: u16,
        month: u8,
        day_of_month: u8,
        hour: u8,
        minute: u8,
        second: u8,
        deviation: i16,
        clock_status: &[ClockStatus],
    ) -> DlmsResult<Self> {
        Self::new_with_details(
            year,
            month,
            day_of_month,
            0xff,
            hour,
            minute,
            second,
            0xff,
            deviation,
            clock_status,
        )
    }

    /// Constructs a COSEM DateTime with all details
    pub fn new_with_details(
        year: u16,
        month: u8,
        day_of_month: u8,
        day_of_week: u8,
        hour: u8,
        minute: u8,
        second: u8,
        hundredths: u8,
        deviation: i16,
        clock_status: &[ClockStatus],
    ) -> DlmsResult<Self> {
        let date = CosemDate::new_with_day_of_week(year, month, day_of_month, day_of_week)?;
        let time = CosemTime::new_with_hundredths(hour, minute, second, hundredths)?;

        Self::validate_deviation(deviation)?;

        Ok(Self {
            date,
            time,
            deviation,
            clock_status: ClockStatus::to_byte(clock_status),
        })
    }

    /// Constructs a COSEM DateTime from date and time
    pub fn from_date_time(
        date: CosemDate,
        time: CosemTime,
        deviation: i16,
        clock_status: &[ClockStatus],
    ) -> DlmsResult<Self> {
        Self::validate_deviation(deviation)?;

        Ok(Self {
            date,
            time,
            deviation,
            clock_status: ClockStatus::to_byte(clock_status),
        })
    }

    /// Decode a COSEM DateTime from a byte array
    pub fn decode(octet_string: &[u8]) -> DlmsResult<Self> {
        if octet_string.len() != Self::LENGTH {
            return Err(DlmsError::InvalidData(format!(
                "Array has an invalid length. Expected {}, got {}",
                Self::LENGTH,
                octet_string.len()
            )));
        }

        let date = CosemDate::decode(&octet_string[0..5])?;
        let time = CosemTime::decode(&octet_string[5..9])?;

        let deviation = ((octet_string[9] as i16) << 8) | (octet_string[10] as i16);
        let clock_status = octet_string[11];

        Ok(Self {
            date,
            time,
            deviation,
            clock_status,
        })
    }

    fn validate_deviation(deviation: i16) -> DlmsResult<()> {
        if (deviation < -720 || deviation > 720) && deviation != DEVIATION_NOT_SPECIFIED {
            Err(DlmsError::InvalidData(format!(
                "Deviation is out of range [-720, 720], got {}",
                deviation
            )))
        } else {
            Ok(())
        }
    }

    /// Get the date component
    pub fn date(&self) -> &CosemDate {
        &self.date
    }

    /// Get the time component
    pub fn time(&self) -> &CosemTime {
        &self.time
    }

    /// Get the deviation
    pub fn deviation(&self) -> i16 {
        self.deviation
    }

    /// Get the clock status flags
    pub fn clock_status(&self) -> Vec<ClockStatus> {
        ClockStatus::from_byte(self.clock_status)
    }
}

impl CosemDateFormat for CosemDateTime {
    fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(Self::LENGTH);
        result.extend_from_slice(&self.date.encode());
        result.extend_from_slice(&self.time.encode());
        result.push(((self.deviation & 0xff00) >> 8) as u8);
        result.push((self.deviation & 0xff) as u8);
        result.push(self.clock_status);
        result
    }

    fn length(&self) -> usize {
        Self::LENGTH
    }

    fn get(&self, field: Field) -> Result<u32, DlmsError> {
        match field {
            Field::Year | Field::Month | Field::DayOfMonth | Field::DayOfWeek => {
                self.date.get(field)
            }
            Field::Hour | Field::Minute | Field::Second | Field::Hundredths => self.time.get(field),
            Field::Deviation => {
                let deviation = if self.deviation == DEVIATION_NOT_SPECIFIED {
                    0
                } else {
                    self.deviation as u32
                };
                Ok(deviation)
            }
            Field::ClockStatus => Ok(self.clock_status as u32),
        }
    }
}

impl fmt::Display for CosemDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.date, self.time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosem_date_time_new() {
        let dt = CosemDateTime::new(2024, 1, 15, 14, 30, 45, 0, &[]).unwrap();
        assert_eq!(dt.get(Field::Year).unwrap(), 2024);
        assert_eq!(dt.get(Field::Hour).unwrap(), 14);
    }

    #[test]
    fn test_cosem_date_time_decode() {
        let bytes = [
            0x07, 0xE8, 0x01, 0x0F, 0xFF, // Date: 2024-01-15
            0x0E, 0x1E, 0x2D, 0xFF,       // Time: 14:30:45
            0x00, 0x00,                    // Deviation: 0
            0x00,                          // Clock status: 0
        ];
        let dt = CosemDateTime::decode(&bytes).unwrap();
        assert_eq!(dt.get(Field::Year).unwrap(), 2024);
    }
}
