//! Serial port transport implementation

use crate::error::{DlmsError, DlmsResult};
use crate::stream::{StreamAccessor, TransportLayer};
use async_trait::async_trait;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

/// Wrapper for SerialStream that implements Debug
struct DebugSerialStream(SerialStream);

impl fmt::Debug for DebugSerialStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SerialStream").finish()
    }
}

impl Deref for DebugSerialStream {
    type Target = SerialStream;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DebugSerialStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Serial port transport layer settings
#[derive(Debug, Clone)]
pub struct SerialSettings {
    pub port_name: String,
    pub baud_rate: u32,
    pub data_bits: tokio_serial::DataBits,
    pub stop_bits: tokio_serial::StopBits,
    pub parity: tokio_serial::Parity,
    pub flow_control: tokio_serial::FlowControl,
    pub timeout: Option<Duration>,
}

impl SerialSettings {
    /// Create new serial settings with default parameters
    pub fn new(port_name: String, baud_rate: u32) -> Self {
        Self {
            port_name,
            baud_rate,
            data_bits: tokio_serial::DataBits::Eight,
            stop_bits: tokio_serial::StopBits::One,
            parity: tokio_serial::Parity::None,
            flow_control: tokio_serial::FlowControl::None,
            timeout: Some(Duration::from_secs(30)),
        }
    }

    /// Create serial settings with timeout
    pub fn with_timeout(port_name: String, baud_rate: u32, timeout: Duration) -> Self {
        Self {
            port_name,
            baud_rate,
            data_bits: tokio_serial::DataBits::Eight,
            stop_bits: tokio_serial::StopBits::One,
            parity: tokio_serial::Parity::None,
            flow_control: tokio_serial::FlowControl::None,
            timeout: Some(timeout),
        }
    }
}

/// Serial port transport layer implementation
#[derive(Debug)]
pub struct SerialTransport {
    stream: Option<DebugSerialStream>,
    settings: SerialSettings,
    closed: bool,
}

impl SerialTransport {
    /// Create a new serial transport layer
    pub fn new(settings: SerialSettings) -> Self {
        Self {
            stream: None,
            settings,
            closed: true,
        }
    }

    /// Create serial transport with port name and baud rate
    pub fn new_simple(port_name: String, baud_rate: u32) -> Self {
        Self::new(SerialSettings::new(port_name, baud_rate))
    }
}

#[async_trait]
impl TransportLayer for SerialTransport {
    async fn open(&mut self) -> DlmsResult<()> {
        if !self.closed {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Connection has already been opened",
            )));
        }

        let mut builder = tokio_serial::new(&self.settings.port_name, self.settings.baud_rate)
            .data_bits(self.settings.data_bits)
            .stop_bits(self.settings.stop_bits)
            .parity(self.settings.parity)
            .flow_control(self.settings.flow_control);

        let stream = SerialStream::open(&builder)
            .map_err(|e| DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open serial port: {}", e),
            )))?;

        self.stream = Some(DebugSerialStream(stream));
        self.closed = false;
        Ok(())
    }
}

#[async_trait]
impl StreamAccessor for SerialTransport {
    async fn set_timeout(&mut self, timeout: Option<Duration>) -> DlmsResult<()> {
        self.settings.timeout = timeout;
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> DlmsResult<usize> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Serial stream not connected",
            ))
        })?;

        let result = if let Some(timeout) = self.settings.timeout {
            tokio::time::timeout(timeout, stream.read(buf)).await
                .map_err(|_| DlmsError::Timeout)?
                .map_err(|e| DlmsError::Connection(e))
        } else {
            stream.read(buf).await.map_err(|e| DlmsError::Connection(e))
        };

        match result {
            Ok(0) => {
                self.closed = true;
                Ok(0)
            }
            Ok(n) => Ok(n),
            Err(e) => {
                self.closed = true;
                Err(e)
            }
        }
    }

    async fn write(&mut self, buf: &[u8]) -> DlmsResult<usize> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Serial stream not connected",
            ))
        })?;

        if let Some(timeout) = self.settings.timeout {
            tokio::time::timeout(timeout, stream.write(buf)).await
                .map_err(|_| DlmsError::Timeout)?
                .map_err(|e| DlmsError::Connection(e))
        } else {
            stream.write(buf).await.map_err(|e| DlmsError::Connection(e))
        }
    }

    async fn flush(&mut self) -> DlmsResult<()> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Serial stream not connected",
            ))
        })?;

        stream.flush().await.map_err(|e| DlmsError::Connection(e))
    }

    fn is_closed(&self) -> bool {
        self.closed
    }

    async fn close(&mut self) -> DlmsResult<()> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.flush().await;
        }
        self.closed = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_settings() {
        let settings = SerialSettings::new("/dev/ttyUSB0".to_string(), 9600);
        assert_eq!(settings.port_name, "/dev/ttyUSB0");
        assert_eq!(settings.baud_rate, 9600);
    }
}
