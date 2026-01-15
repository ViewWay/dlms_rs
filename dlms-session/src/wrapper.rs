//! Wrapper session layer for DLMS/COSEM

use crate::error::{DlmsError, DlmsResult};
use dlms_transport::{StreamAccessor, TransportLayer};
use std::time::Duration;

/// Wrapper header length
pub const WRAPPER_HEADER_LENGTH: usize = 8;

/// Wrapper header
#[derive(Debug, Clone)]
pub struct WrapperHeader {
    client_id: u16,
    logical_device_id: u16,
    length: u16,
}

impl WrapperHeader {
    /// Create a new wrapper header
    pub fn new(client_id: u16, logical_device_id: u16, length: u16) -> Self {
        Self {
            client_id,
            logical_device_id,
            length,
        }
    }

    /// Encode header to bytes (big-endian)
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(WRAPPER_HEADER_LENGTH);
        
        // Version (big-endian, 2 bytes: 0x00 0x01)
        result.push(0x00);
        result.push(0x01);
        
        // Source W-Port (big-endian, 2 bytes)
        result.extend_from_slice(&self.client_id.to_be_bytes());
        
        // Destination W-Port (big-endian, 2 bytes)
        result.extend_from_slice(&self.logical_device_id.to_be_bytes());
        
        // Length (big-endian, 2 bytes)
        result.extend_from_slice(&self.length.to_be_bytes());
        
        result
    }

    /// Decode header from bytes
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.len() < WRAPPER_HEADER_LENGTH {
            return Err(DlmsError::InvalidData(format!(
                "Wrapper header too short: expected {}, got {}",
                WRAPPER_HEADER_LENGTH,
                data.len()
            )));
        }

        // Check first byte to determine byte order
        // 0x00 = big-endian format, 0x01 = little-endian format
        let first_byte = data[0];
        let (client_id, logical_device_id, length) = if first_byte == 0x00 {
            // Big-endian format: first byte is 0x00, second byte is version
            let version = data[1];
            if version != 1 {
                return Err(DlmsError::InvalidData(format!(
                    "Header version was {}, this stack is only compatible to version 1",
                    version
                )));
            }
            let client_id = u16::from_be_bytes([data[2], data[3]]);
            let logical_device_id = u16::from_be_bytes([data[4], data[5]]);
            let length = u16::from_be_bytes([data[6], data[7]]);
            (client_id, logical_device_id, length)
        } else if first_byte == 0x01 {
            // Little-endian format: first byte is 0x01, second byte should be 0x00 (version 1 in little-endian)
            let version_byte2 = data[1];
            if version_byte2 != 0x00 {
                return Err(DlmsError::InvalidData(format!(
                    "Header version was {}, this stack is only compatible to version 1",
                    (version_byte2 as u16) << 8 | first_byte as u16
                )));
            }
            // Read values as little-endian
            let client_id = u16::from_le_bytes([data[2], data[3]]);
            let logical_device_id = u16::from_le_bytes([data[4], data[5]]);
            let length = u16::from_le_bytes([data[6], data[7]]);
            (client_id, logical_device_id, length)
        } else {
            return Err(DlmsError::InvalidData(format!(
                "Invalid wrapper header first byte: expected 0x00 (big-endian) or 0x01 (little-endian), got 0x{:02X}",
                first_byte
            )));
        };

        Ok(Self {
            client_id,
            logical_device_id,
            length,
        })
    }

    /// Get client ID
    pub fn client_id(&self) -> u16 {
        self.client_id
    }

    /// Get logical device ID
    pub fn logical_device_id(&self) -> u16 {
        self.logical_device_id
    }

    /// Get payload length
    pub fn payload_length(&self) -> u16 {
        self.length
    }
}

/// Wrapper PDU (Protocol Data Unit)
#[derive(Debug, Clone)]
pub struct WrapperPdu {
    header: WrapperHeader,
    data: Vec<u8>,
}

impl WrapperPdu {
    /// Create a new wrapper PDU
    pub fn new(header: WrapperHeader, data: Vec<u8>) -> Self {
        Self { header, data }
    }

    /// Encode PDU to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut result = self.header.encode();
        result.extend_from_slice(&self.data);
        result
    }

    /// Decode PDU from stream
    pub async fn decode<S: StreamAccessor>(stream: &mut S) -> DlmsResult<Self> {
        // Read header
        let mut header_bytes = vec![0u8; WRAPPER_HEADER_LENGTH];
        let mut pos = 0;
        while pos < WRAPPER_HEADER_LENGTH {
            let n = stream.read(&mut header_bytes[pos..]).await?;
            if n == 0 {
                return Err(DlmsError::Connection(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Unexpected end of stream while reading wrapper header",
                )));
            }
            pos += n;
        }

        let header = WrapperHeader::decode(&header_bytes)?;
        let payload_length = header.payload_length() as usize;

        // Read payload
        let mut data = vec![0u8; payload_length];
        let mut pos = 0;
        while pos < payload_length {
            let n = stream.read(&mut data[pos..]).await?;
            if n == 0 {
                return Err(DlmsError::Connection(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Unexpected end of stream while reading wrapper payload",
                )));
            }
            pos += n;
        }

        Ok(Self { header, data })
    }

    /// Get header
    pub fn header(&self) -> &WrapperHeader {
        &self.header
    }

    /// Get data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Wrapper session layer
#[derive(Debug)]
pub struct WrapperSession<T: TransportLayer> {
    transport: T,
    client_id: u16,
    logical_device_id: u16,
    closed: bool,
}

impl<T: TransportLayer> WrapperSession<T> {
    /// Create a new wrapper session
    pub fn new(transport: T, client_id: u16, logical_device_id: u16) -> Self {
        Self {
            transport,
            client_id,
            logical_device_id,
            closed: true,
        }
    }

    /// Open the wrapper session
    pub async fn open(&mut self) -> DlmsResult<()> {
        self.transport.open().await?;
        self.closed = false;
        Ok(())
    }

    /// Send data through wrapper session
    pub async fn send(&mut self, data: &[u8]) -> DlmsResult<()> {
        if self.closed {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Wrapper session is closed",
            )));
        }

        let header = WrapperHeader::new(self.client_id, self.logical_device_id, data.len() as u16);
        let pdu = WrapperPdu::new(header, data.to_vec());
        let encoded = pdu.encode();

        self.transport.write_all(&encoded).await?;
        self.transport.flush().await?;
        Ok(())
    }

    /// Receive data from wrapper session
    pub async fn receive(&mut self, timeout: Option<Duration>) -> DlmsResult<Vec<u8>> {
        if self.closed {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Wrapper session is closed",
            )));
        }

        if let Some(timeout) = timeout {
            self.transport.set_timeout(Some(timeout)).await?;
        }

        let pdu = WrapperPdu::decode(&mut self.transport).await?;
        Ok(pdu.data().to_vec())
    }

    /// Check if session is closed
    pub fn is_closed(&self) -> bool {
        self.closed || self.transport.is_closed()
    }

    /// Close the session
    pub async fn close(&mut self) -> DlmsResult<()> {
        if !self.closed {
            self.transport.close().await?;
            self.closed = true;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapper_header_encode_decode() {
        let header = WrapperHeader::new(0x0001, 0x0001, 100);
        let encoded = header.encode();
        assert_eq!(encoded.len(), WRAPPER_HEADER_LENGTH);
        
        let decoded = WrapperHeader::decode(&encoded).unwrap();
        assert_eq!(decoded.client_id(), 0x0001);
        assert_eq!(decoded.logical_device_id(), 0x0001);
        assert_eq!(decoded.payload_length(), 100);
    }
}
