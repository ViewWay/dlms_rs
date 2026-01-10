//! HDLC address types

use crate::error::{DlmsError, DlmsResult};
use std::fmt;

/// Reserved HDLC addresses
pub mod reserved {
    /// Guaranteed to be received by no one
    pub const NO_STATION: u16 = 0x00;
    
    /// Client management process
    pub const CLIENT_MANAGEMENT_PROCESS: u16 = 0x01;
    
    /// Client public client
    pub const CLIENT_PUBLIC_CLIENT: u16 = 0x10;
    
    /// Client all station (broadcast)
    pub const CLIENT_ALL_STATION: u16 = 0x7F;
    
    /// Server upper management logical device
    pub const SERVER_UPPER_MANAGEMENT_LOGICAL_DEVICE: u16 = 0x01;
    
    /// Server upper all stations (1 byte)
    pub const SERVER_UPPER_ALL_STATIONS_1BYTE: u16 = 0x7F;
    
    /// Server upper all stations (2 byte)
    pub const SERVER_UPPER_ALL_STATIONS_2BYTE: u16 = 0x3FFF;
    
    /// Server lower calling (1 byte)
    pub const SERVER_LOWER_CALLING_1BYTE: u16 = 0x7E;
    
    /// Server lower calling (2 byte)
    pub const SERVER_LOWER_CALLING_2BYTE: u16 = 0x3FFE;
}

const ONE_BYTE_UPPER_BOUND: u16 = 0x7F;
const TWO_BYTE_UPPER_BOUND: u16 = 0x3FFF;

/// HDLC address
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HdlcAddress {
    byte_length: usize,
    logical_id: u16,
    physical_id: u16,
}

impl HdlcAddress {
    /// Create a new HDLC address with logical ID only
    pub fn new(logical_id: u16) -> DlmsResult<Self> {
        if logical_id > ONE_BYTE_UPPER_BOUND {
            return Err(DlmsError::InvalidData(format!(
                "One byte address exceeded upper bound of 0x{:02X}",
                ONE_BYTE_UPPER_BOUND
            )));
        }
        Ok(Self {
            byte_length: 1,
            logical_id,
            physical_id: 0,
        })
    }

    /// Create a new HDLC address with logical and physical ID
    pub fn new_with_physical(logical_id: u16, physical_id: u16) -> DlmsResult<Self> {
        let logical_size = Self::address_size_of(logical_id)?;
        let physical_size = Self::address_size_of(physical_id)?;
        let byte_length = if physical_id == 0 {
            logical_size
        } else {
            logical_size.max(physical_size) * 2
        };

        Ok(Self {
            byte_length,
            logical_id,
            physical_id,
        })
    }

    fn address_size_of(address: u16) -> DlmsResult<usize> {
        if address <= ONE_BYTE_UPPER_BOUND {
            Ok(1)
        } else if address <= TWO_BYTE_UPPER_BOUND {
            Ok(2)
        } else {
            Err(DlmsError::InvalidData(format!(
                "Address 0x{:X} is out of upper bound 0x{:X}",
                address, TWO_BYTE_UPPER_BOUND
            )))
        }
    }

    /// Get logical ID
    pub fn logical_id(&self) -> u16 {
        self.logical_id
    }

    /// Get physical ID
    pub fn physical_id(&self) -> u16 {
        self.physical_id
    }

    /// Get byte length
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }

    /// Encode address to bytes
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        self.validate()?;

        let upper_length = (self.byte_length + 1) / 2;
        let lower_length = self.byte_length / 2;

        let mut result = vec![0u8; self.byte_length];

        // Encode logical ID
        for i in 0..upper_length {
            let shift = 7 * (upper_length - i - 1);
            result[i] = (((self.logical_id & (0x7F << shift)) >> shift) << 1) as u8;
        }

        // Encode physical ID
        for i in 0..lower_length {
            let shift = 7 * (upper_length - i - 1);
            result[upper_length + i] = (((self.physical_id & (0x7F << shift)) >> shift) << 1) as u8;
        }

        // Set stop bit
        result[self.byte_length - 1] |= 1;

        Ok(result)
    }

    /// Decode address from bytes
    pub fn decode(data: &[u8], length: usize) -> DlmsResult<Self> {
        if length > data.len() {
            return Err(DlmsError::InvalidData(format!(
                "Length {} exceeds data length {}",
                length,
                data.len()
            )));
        }

        let (logical_device_addr, physical_dev_addr) = match length {
            1 => {
                let logical = ((data[0] & 0xFF) >> 1) as u16;
                (logical, 0)
            }
            2 => {
                let logical = ((data[0] & 0xFF) >> 1) as u16;
                let physical = ((data[1] & 0xFF) >> 1) as u16;
                (logical, physical)
            }
            4 => {
                let logical = (((data[0] & 0xFF) >> 1) as u16) << 7
                    | ((data[1] & 0xFF) >> 1) as u16;
                let physical = (((data[2] & 0xFF) >> 1) as u16) << 7
                    | ((data[3] & 0xFF) >> 1) as u16;
                (logical, physical)
            }
            _ => {
                return Err(DlmsError::InvalidData(format!(
                    "Received HdlcAddress has an invalid byte length of {}",
                    length
                )));
            }
        };

        Self::new_with_physical(logical_device_addr, physical_dev_addr)
    }

    fn validate(&self) -> DlmsResult<()> {
        if self.byte_length != 1 && self.byte_length != 2 && self.byte_length != 4 {
            return Err(DlmsError::InvalidData(format!(
                "HdlcAddress has an invalid byte length: {}",
                self.byte_length
            )));
        }

        let upper_length = (self.byte_length + 1) / 2;
        let lower_length = self.byte_length / 2;

        let max_logical = (1u32 << (7 * upper_length)) - 1;
        let max_physical = (1u32 << (7 * lower_length)) - 1;

        if self.logical_id as u32 >= max_logical || self.physical_id as u32 >= max_physical {
            return Err(DlmsError::InvalidData(format!(
                "HdlcAddress values out of range: logical={}, physical={}",
                self.logical_id, self.physical_id
            )));
        }

        Ok(())
    }

    /// Check if this is an all-station (broadcast) address
    pub fn is_all_station(&self) -> bool {
        if self.byte_length == 1 || self.byte_length == 2 {
            self.logical_id == reserved::SERVER_UPPER_ALL_STATIONS_1BYTE
        } else if self.byte_length == 4 {
            self.logical_id == reserved::SERVER_UPPER_ALL_STATIONS_2BYTE
        } else {
            false
        }
    }

    /// Check if this is a no-station address
    pub fn is_no_station(&self) -> bool {
        self.logical_id == reserved::NO_STATION && self.physical_id == reserved::NO_STATION
    }

    /// Check if this is a calling station address
    pub fn is_calling(&self) -> bool {
        if self.byte_length == 2 {
            self.physical_id == reserved::SERVER_LOWER_CALLING_1BYTE
        } else if self.byte_length == 4 {
            self.physical_id == reserved::SERVER_LOWER_CALLING_2BYTE
        } else {
            false
        }
    }
}

impl fmt::Display for HdlcAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ld_length = ((self.byte_length + 1) / 2) * 2;
        let ph_length = (self.byte_length / 2) * 2;

        write!(f, "{:0width$X}", self.logical_id, width = ld_length)?;

        if ph_length > 0 && self.physical_id != 0 {
            write!(f, "-{:0width$X}", self.physical_id, width = ph_length)?;
        }

        Ok(())
    }
}

/// HDLC address pair (source and destination)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HdlcAddressPair {
    source: HdlcAddress,
    destination: HdlcAddress,
}

impl HdlcAddressPair {
    /// Create a new address pair
    pub fn new(source: HdlcAddress, destination: HdlcAddress) -> Self {
        Self { source, destination }
    }

    /// Get source address
    pub fn source(&self) -> HdlcAddress {
        self.source
    }

    /// Get destination address
    pub fn destination(&self) -> HdlcAddress {
        self.destination
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdlc_address_new() {
        let addr = HdlcAddress::new(0x10).unwrap();
        assert_eq!(addr.logical_id(), 0x10);
        assert_eq!(addr.byte_length(), 1);
    }

    #[test]
    fn test_hdlc_address_encode_decode() {
        let addr = HdlcAddress::new_with_physical(0x10, 0x20).unwrap();
        let encoded = addr.encode().unwrap();
        let decoded = HdlcAddress::decode(&encoded, encoded.len()).unwrap();
        assert_eq!(addr, decoded);
    }

    #[test]
    fn test_hdlc_address_pair() {
        let src = HdlcAddress::new(0x10).unwrap();
        let dst = HdlcAddress::new(0x20).unwrap();
        let pair = HdlcAddressPair::new(src, dst);
        assert_eq!(pair.source(), src);
        assert_eq!(pair.destination(), dst);
    }
}
