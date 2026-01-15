//! UDP transport implementation

use crate::error::{DlmsError, DlmsResult};
use crate::stream::{StreamAccessor, TransportLayer};
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

/// Maximum UDP payload size
pub const MAX_UDP_PAYLOAD_SIZE: usize = 65507;

/// UDP transport layer settings
#[derive(Debug, Clone)]
pub struct UdpSettings {
    pub remote_address: SocketAddr,
    pub timeout: Option<Duration>,
}

impl UdpSettings {
    /// Create new UDP settings
    pub fn new(remote_address: SocketAddr) -> Self {
        Self {
            remote_address,
            timeout: Some(Duration::from_secs(30)),
        }
    }

    /// Create UDP settings with timeout
    pub fn with_timeout(remote_address: SocketAddr, timeout: Duration) -> Self {
        Self {
            remote_address,
            timeout: Some(timeout),
        }
    }
}

/// UDP transport layer implementation
pub struct UdpTransport {
    socket: Option<Arc<UdpSocket>>,
    settings: UdpSettings,
    closed: bool,
    read_buffer: Arc<Mutex<Vec<u8>>>,
    read_position: Arc<Mutex<usize>>,
}

impl UdpTransport {
    /// Create a new UDP transport layer
    pub fn new(settings: UdpSettings) -> Self {
        Self {
            socket: None,
            settings,
            closed: true,
            read_buffer: Arc::new(Mutex::new(Vec::new())),
            read_position: Arc::new(Mutex::new(0)),
        }
    }

    /// Create UDP transport from address string
    pub fn from_address(address: &str) -> DlmsResult<Self> {
        let addr: SocketAddr = address.parse().map_err(|e| {
            DlmsError::InvalidData(format!("Invalid UDP address: {}", e))
        })?;
        Ok(Self::new(UdpSettings::new(addr)))
    }

    async fn read_next_packet(&self) -> DlmsResult<()> {
        loop {
            let socket = self.socket.as_ref().ok_or_else(|| {
                DlmsError::Connection(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "UDP socket not connected",
                ))
            })?;

            let mut buf = vec![0u8; MAX_UDP_PAYLOAD_SIZE];

            let (len, addr) = if let Some(timeout) = self.settings.timeout {
                tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await
                    .map_err(|_| DlmsError::Timeout)?
                    .map_err(|e| DlmsError::Connection(e))?
            } else {
                socket.recv_from(&mut buf).await
                    .map_err(|e| DlmsError::Connection(e))?
            };

            // Verify the packet is from the expected address
            if addr == self.settings.remote_address {
                let mut buffer = self.read_buffer.lock().await;
                *buffer = buf[..len].to_vec();
                let mut position = self.read_position.lock().await;
                *position = 0;
                return Ok(());
            }
            // Otherwise, continue loop to wait for next packet
        }
    }
}

#[async_trait]
impl TransportLayer for UdpTransport {
    async fn open(&mut self) -> DlmsResult<()> {
        if !self.closed {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Connection has already been opened",
            )));
        }

        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| DlmsError::Connection(e))?;

        self.socket = Some(Arc::new(socket));
        self.closed = false;
        Ok(())
    }
}

#[async_trait]
impl StreamAccessor for UdpTransport {
    async fn set_timeout(&mut self, timeout: Option<Duration>) -> DlmsResult<()> {
        self.settings.timeout = timeout;
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> DlmsResult<usize> {
        let mut buffer = self.read_buffer.lock().await;
        let mut position = self.read_position.lock().await;

        // Read next packet if buffer is empty
        if *position >= buffer.len() {
            drop(buffer);
            drop(position);
            self.read_next_packet().await?;
            buffer = self.read_buffer.lock().await;
            position = self.read_position.lock().await;
        }

        let available = buffer.len() - *position;
        let to_read = buf.len().min(available);
        
        if to_read > 0 {
            buf[..to_read].copy_from_slice(&buffer[*position..*position + to_read]);
            *position += to_read;
        }

        Ok(to_read)
    }

    async fn write(&mut self, buf: &[u8]) -> DlmsResult<usize> {
        let socket = self.socket.as_ref().ok_or_else(|| {
            DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "UDP socket not connected",
            ))
        })?;

        // UDP sends entire packet, so we need to handle large buffers
        let mut written = 0;
        let mut remaining = buf;

        while !remaining.is_empty() {
            let to_send = remaining.len().min(MAX_UDP_PAYLOAD_SIZE);
            let packet = &remaining[..to_send];

            let sent = if let Some(timeout) = self.settings.timeout {
                tokio::time::timeout(timeout, socket.send_to(packet, self.settings.remote_address))
                    .await
                    .map_err(|_| DlmsError::Timeout)?
                    .map_err(|e| DlmsError::Connection(e))?
            } else {
                socket.send_to(packet, self.settings.remote_address)
                    .await
                    .map_err(|e| DlmsError::Connection(e))?
            };

            written += sent;
            remaining = &remaining[sent..];
        }

        Ok(written)
    }

    async fn flush(&mut self) -> DlmsResult<()> {
        // UDP is connectionless, no flush needed
        Ok(())
    }

    fn is_closed(&self) -> bool {
        self.closed
    }

    async fn close(&mut self) -> DlmsResult<()> {
        self.socket = None;
        self.closed = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_udp_settings() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let settings = UdpSettings::new(addr);
        assert_eq!(settings.remote_address, addr);
        assert!(settings.timeout.is_some());
    }
}
