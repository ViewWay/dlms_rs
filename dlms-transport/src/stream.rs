//! Stream accessor trait for transport layer

use crate::error::{DlmsError, DlmsResult};
use async_trait::async_trait;
use bytes::BytesMut;
use std::time::Duration;

/// Stream accessor interface to access a physical stream to a remote meter
#[async_trait]
pub trait StreamAccessor: Send + Sync {
    /// Set the read timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration. None means infinite timeout.
    async fn set_timeout(&mut self, timeout: Option<Duration>) -> DlmsResult<()>;

    /// Read data from the stream
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to read into
    ///
    /// # Returns
    ///
    /// Number of bytes read, or 0 if EOF
    async fn read(&mut self, buf: &mut [u8]) -> DlmsResult<usize>;

    /// Read exact number of bytes from the stream
    ///
    /// # Arguments
    ///
    /// * `buf` - Buffer to read into, will be filled completely
    ///
    /// # Returns
    ///
    /// Returns error if unable to read the exact number of bytes
    async fn read_exact(&mut self, mut buf: &mut [u8]) -> DlmsResult<()> {
        while !buf.is_empty() {
            let n = self.read(buf).await?;
            if n == 0 {
                return Err(DlmsError::Connection(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Failed to read exact number of bytes",
                )));
            }
            buf = &mut buf[n..];
        }
        Ok(())
    }

    /// Write data to the stream
    ///
    /// # Arguments
    ///
    /// * `buf` - Data to write
    ///
    /// # Returns
    ///
    /// Number of bytes written
    async fn write(&mut self, buf: &[u8]) -> DlmsResult<usize>;

    /// Write all data to the stream
    async fn write_all(&mut self, buf: &[u8]) -> DlmsResult<()> {
        let mut written = 0;
        while written < buf.len() {
            let n = self.write(&buf[written..]).await?;
            if n == 0 {
                return Err(DlmsError::Connection(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "Failed to write all data",
                )));
            }
            written += n;
        }
        Ok(())
    }

    /// Flush any buffered data
    async fn flush(&mut self) -> DlmsResult<()>;

    /// Check if the stream is closed
    fn is_closed(&self) -> bool;

    /// Close the stream
    async fn close(&mut self) -> DlmsResult<()>;
}

/// Transport layer trait that extends StreamAccessor
#[async_trait]
pub trait TransportLayer: StreamAccessor {
    /// Open the physical layer connection
    async fn open(&mut self) -> DlmsResult<()>;
}
