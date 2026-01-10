//! HDLC connection implementation

use crate::error::{DlmsError, DlmsResult};
use crate::hdlc::address::{HdlcAddress, HdlcAddressPair};
use crate::hdlc::decoder::HdlcMessageDecoder;
use crate::hdlc::dispatcher::HdlcDispatcher;
use crate::hdlc::frame::{FrameType, HdlcFrame, FLAG};
use dlms_transport::{StreamAccessor, TransportLayer};
use std::time::Duration;
use tokio::sync::mpsc;

/// HDLC connection parameters
#[derive(Debug, Clone)]
pub struct HdlcParameters {
    pub max_information_field_length_tx: u16,
    pub max_information_field_length_rx: u16,
    pub window_size_tx: u8,
    pub window_size_rx: u8,
}

impl Default for HdlcParameters {
    fn default() -> Self {
        Self {
            max_information_field_length_tx: 128,
            max_information_field_length_rx: 128,
            window_size_tx: 1,
            window_size_rx: 1,
        }
    }
}

/// HDLC connection
pub struct HdlcConnection<T: TransportLayer> {
    transport: T,
    local_address: HdlcAddress,
    remote_address: HdlcAddress,
    dispatcher: HdlcDispatcher,
    parameters: HdlcParameters,
    send_sequence: u8,
    receive_sequence: u8,
    closed: bool,
}

impl<T: TransportLayer> HdlcConnection<T> {
    /// Create a new HDLC connection
    pub fn new(
        transport: T,
        local_address: HdlcAddress,
        remote_address: HdlcAddress,
    ) -> Self {
        let dispatcher = HdlcDispatcher::new(local_address);
        Self {
            transport,
            local_address,
            remote_address,
            dispatcher,
            parameters: HdlcParameters::default(),
            send_sequence: 0,
            receive_sequence: 0,
            closed: true,
        }
    }

    /// Open the HDLC connection
    pub async fn open(&mut self) -> DlmsResult<()> {
        self.transport.open().await?;
        self.closed = false;
        Ok(())
    }

    /// Send an HDLC frame
    pub async fn send_frame(&mut self, frame: HdlcFrame) -> DlmsResult<()> {
        if self.closed {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "HDLC connection is closed",
            )));
        }

        let encoded = frame.encode()?;
        let mut data = vec![FLAG];
        data.extend_from_slice(&encoded);
        data.push(FLAG);

        self.transport.write_all(&data).await?;
        self.transport.flush().await?;
        Ok(())
    }

    /// Send an information frame
    pub async fn send_information(
        &mut self,
        information_field: Vec<u8>,
        segmented: bool,
    ) -> DlmsResult<()> {
        let address_pair = HdlcAddressPair::new(self.local_address, self.remote_address);
        let frame = HdlcFrame::new_information(
            address_pair,
            information_field,
            self.send_sequence,
            self.receive_sequence,
            segmented,
        );
        
        self.send_frame(frame).await?;
        self.send_sequence = (self.send_sequence + 1) % 8;
        Ok(())
    }

    /// Receive HDLC frames
    pub async fn receive_frames(&mut self, timeout: Option<Duration>) -> DlmsResult<Vec<HdlcFrame>> {
        HdlcMessageDecoder::decode(&mut self.transport, timeout).await
    }

    /// Set HDLC parameters
    pub fn set_parameters(&mut self, parameters: HdlcParameters) {
        self.parameters = parameters;
    }

    /// Get HDLC parameters
    pub fn parameters(&self) -> &HdlcParameters {
        &self.parameters
    }

    /// Check if connection is closed
    pub fn is_closed(&self) -> bool {
        self.closed || self.transport.is_closed()
    }

    /// Close the connection
    pub async fn close(&mut self) -> DlmsResult<()> {
        if !self.closed {
            // Send disconnect frame
            let address_pair = HdlcAddressPair::new(self.local_address, self.remote_address);
            let disconnect_frame = HdlcFrame::new(address_pair, FrameType::Disconnect, None);
            let _ = self.send_frame(disconnect_frame).await;
            
            self.transport.close().await?;
            self.closed = true;
        }
        Ok(())
    }
}

// Note: Drop implementation with async close is not straightforward in Rust
// The connection should be explicitly closed before dropping

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdlc_parameters_default() {
        let params = HdlcParameters::default();
        assert_eq!(params.max_information_field_length_tx, 128);
        assert_eq!(params.window_size_tx, 1);
    }
}
