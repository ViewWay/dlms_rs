//! HDLC message decoder

use crate::error::{DlmsError, DlmsResult};
use crate::hdlc::frame::{HdlcFrame, FLAG};
use dlms_transport::StreamAccessor;
use std::time::Duration;

const HDLC_LENGTH_MASK: u16 = 0x07FF;

/// HDLC message decoder
pub struct HdlcMessageDecoder;

impl HdlcMessageDecoder {
    /// Decode HDLC frames from stream
    pub async fn decode<S: StreamAccessor>(
        stream: &mut S,
        timeout: Option<Duration>,
    ) -> DlmsResult<Vec<HdlcFrame>> {
        let mut frames = Vec::new();

        // Set timeout to 0 for initial read
        stream.set_timeout(Some(Duration::from_secs(0))).await?;

        // Read and validate starting flag
        let mut flag_buf = [0u8; 1];
        stream.read(&mut flag_buf).await?;
        Self::validate_flag(flag_buf[0])?;

        loop {
            // Read frame
            let frame_bytes = Self::read_frame(stream, timeout).await?;

            // Read and validate ending flag
            let mut flag_buf = [0u8; 1];
            let n = stream.read(&mut flag_buf).await?;
            if n == 0 {
                break; // EOF
            }
            Self::validate_flag(flag_buf[0])?;

            // Decode frame
            match HdlcFrame::decode(&frame_bytes) {
                Ok(frame) => frames.push(frame),
                Err(e) => {
                    // Ignore invalid frames, but log the error
                    eprintln!("Failed to decode HDLC frame: {}", e);
                }
            }

            // Check if more data is available
            // Note: This is a simplified check - in practice you might want
            // to peek at the next byte to see if there's another frame
        }

        Ok(frames)
    }

    async fn read_frame<S: StreamAccessor>(
        stream: &mut S,
        timeout: Option<Duration>,
    ) -> DlmsResult<Vec<u8>> {
        // Set timeout for frame reading
        if let Some(timeout) = timeout {
            stream.set_timeout(Some(timeout)).await?;
        }

        // Read frame format (2 bytes)
        let mut frame_format = [0u8; 2];
        stream.read_exact(&mut frame_format).await?;

        let frame_format_short = u16::from_be_bytes(frame_format);
        let length = (frame_format_short & HDLC_LENGTH_MASK) as usize;

        if length < 2 {
            return Err(DlmsError::FrameInvalid(format!(
                "Frame length too short: {}",
                length
            )));
        }

        // Read remaining frame data
        let mut data = vec![0u8; length];
        data[0] = frame_format[0];
        data[1] = frame_format[1];
        
        let remaining = length - 2;
        if remaining > 0 {
            stream.read_exact(&mut data[2..]).await?;
        }

        Ok(data)
    }

    fn validate_flag(flag: u8) -> DlmsResult<()> {
        if flag != FLAG {
            Err(DlmsError::FrameInvalid(format!(
                "Expected HDLC flag 0x7E, but received: 0x{:02X}",
                flag
            )))
        } else {
            Ok(())
        }
    }

    /// Helper function for reading exact number of bytes
    async fn read_exact<S: StreamAccessor>(stream: &mut S, buf: &mut [u8]) -> DlmsResult<()> {
        let mut pos = 0;
        while pos < buf.len() {
            let n = stream.read(&mut buf[pos..]).await?;
            if n == 0 {
                return Err(DlmsError::Connection(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Unexpected end of stream",
                )));
            }
            pos += n;
        }
        Ok(())
    }
}
