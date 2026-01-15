//! TCP transport implementation

use crate::error::{DlmsError, DlmsResult};
use crate::stream::{StreamAccessor, TransportLayer};
use async_trait::async_trait;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// TCP transport layer settings
#[derive(Debug, Clone)]
pub struct TcpSettings {
    pub address: SocketAddr,
    pub timeout: Option<Duration>,
}

impl TcpSettings {
    /// Create new TCP settings
    pub fn new(address: SocketAddr) -> Self {
        Self {
            address,
            timeout: Some(Duration::from_secs(30)),
        }
    }

    /// Create TCP settings with timeout
    pub fn with_timeout(address: SocketAddr, timeout: Duration) -> Self {
        Self {
            address,
            timeout: Some(timeout),
        }
    }
}

/// TCP transport layer implementation
pub struct TcpTransport {
    stream: Option<TcpStream>,
    settings: TcpSettings,
    closed: bool,
}

impl TcpTransport {
    /// Create a new TCP transport layer
    pub fn new(settings: TcpSettings) -> Self {
        Self {
            stream: None,
            settings,
            closed: true,
        }
    }

    /// Create TCP transport from address string
    pub fn from_address(address: &str) -> DlmsResult<Self> {
        let addr: SocketAddr = address.parse().map_err(|e| {
            DlmsError::InvalidData(format!("Invalid TCP address: {}", e))
        })?;
        Ok(Self::new(TcpSettings::new(addr)))
    }
}

#[async_trait]
impl TransportLayer for TcpTransport {
    async fn open(&mut self) -> DlmsResult<()> {
        if !self.closed {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Connection has already been opened",
            )));
        }

        // Apply timeout to connection establishment if specified
        let stream = if let Some(timeout) = self.settings.timeout {
            tokio::time::timeout(timeout, TcpStream::connect(self.settings.address))
                .await
                .map_err(|_| DlmsError::Timeout)?
                .map_err(|e| DlmsError::Connection(e))?
        } else {
            TcpStream::connect(self.settings.address)
                .await
                .map_err(|e| DlmsError::Connection(e))?
        };

        self.stream = Some(stream);
        self.closed = false;
        Ok(())
    }
}

#[async_trait]
impl StreamAccessor for TcpTransport {
    async fn set_timeout(&mut self, timeout: Option<Duration>) -> DlmsResult<()> {
        self.settings.timeout = timeout;
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> DlmsResult<usize> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "TCP stream not connected",
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
                "TCP stream not connected",
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
                "TCP stream not connected",
            ))
        })?;

        stream.flush().await.map_err(|e| DlmsError::Connection(e))
    }

    fn is_closed(&self) -> bool {
        self.closed
    }

    async fn close(&mut self) -> DlmsResult<()> {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
        self.closed = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tcp_settings() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let settings = TcpSettings::new(addr);
        assert_eq!(settings.address, addr);
        assert!(settings.timeout.is_some());
    }
}
