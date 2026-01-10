//! HDLC frame structure and encoding/decoding

use crate::error::{DlmsError, DlmsResult};
use crate::hdlc::address::{HdlcAddress, HdlcAddressPair};
use crate::hdlc::fcs::FcsCalc;
use std::fmt;

/// HDLC frame flag
pub const FLAG: u8 = 0x7E;

/// Logical Link Control (LLC) Request
pub const LLC_REQUEST: [u8; 3] = [0xE6, 0xE6, 0x00];

/// HDLC frame type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Information,
    ReceiveReady,
    ReceiveNotReady,
    SetNormalResponseMode,
    Disconnect,
    UnnumberedAcknowledge,
    DisconnectMode,
    FrameReject,
    UnnumberedInformation,
    InvalidType,
}

impl FrameType {
    /// Get frame type from control byte
    pub fn from_control_byte(control_byte: u8) -> Self {
        let value = control_byte & 0xFF;
        
        match value {
            x if (x & 0x01) == 0x00 => FrameType::Information,
            x if (x & 0x0F) == 0x01 => FrameType::ReceiveReady,
            x if (x & 0x0F) == 0x05 => FrameType::ReceiveNotReady,
            x if (x & 0xEF) == 0x83 => FrameType::SetNormalResponseMode,
            x if (x & 0xEF) == 0x43 => FrameType::Disconnect,
            x if (x & 0xEF) == 0x63 => FrameType::UnnumberedAcknowledge,
            x if (x & 0xEF) == 0x0F => FrameType::DisconnectMode,
            x if (x & 0xEF) == 0x87 => FrameType::FrameReject,
            x if (x & 0xEF) == 0xC0 => FrameType::UnnumberedInformation,
            _ => FrameType::InvalidType,
        }
    }

    /// Get control byte value for this frame type
    pub fn to_control_byte(&self, send_seq: Option<u8>, recv_seq: Option<u8>) -> u8 {
        match self {
            FrameType::Information => {
                let seq = send_seq.unwrap_or(0);
                (seq << 1) | (recv_seq.unwrap_or(0) << 5)
            }
            FrameType::ReceiveReady => {
                0x01 | (recv_seq.unwrap_or(0) << 5)
            }
            FrameType::ReceiveNotReady => {
                0x05 | (recv_seq.unwrap_or(0) << 5)
            }
            FrameType::SetNormalResponseMode => 0x83,
            FrameType::Disconnect => 0x43,
            FrameType::UnnumberedAcknowledge => 0x63,
            FrameType::DisconnectMode => 0x0F,
            FrameType::FrameReject => 0x87,
            FrameType::UnnumberedInformation => 0xC0,
            FrameType::InvalidType => 0xFF,
        }
    }
}

/// HDLC frame
#[derive(Debug, Clone, PartialEq)]
pub struct HdlcFrame {
    frame_type: FrameType,
    information_field: Vec<u8>,
    send_sequence: Option<u8>,
    receive_sequence: Option<u8>,
    segmented: bool,
    control_field: u8,
    address_pair: HdlcAddressPair,
    length: usize,
}

impl HdlcFrame {
    /// Create a new HDLC frame
    pub fn new(
        address_pair: HdlcAddressPair,
        frame_type: FrameType,
        information_field: Option<Vec<u8>>,
    ) -> Self {
        let info_field = information_field.unwrap_or_default();
        let control_field = frame_type.to_control_byte(None, None);
        
        let length = 2 // Frame format
            + address_pair.destination().byte_length()
            + address_pair.source().byte_length()
            + 1 // Control field
            + 2; // FCS
        
        let length = if !info_field.is_empty() {
            length + info_field.len() + 2 // Additional FCS for info field
        } else {
            length
        };

        Self {
            frame_type,
            information_field: info_field,
            send_sequence: None,
            receive_sequence: None,
            segmented: false,
            control_field,
            address_pair,
            length,
        }
    }

    /// Create a new information frame with sequence numbers
    pub fn new_information(
        address_pair: HdlcAddressPair,
        information_field: Vec<u8>,
        send_sequence: u8,
        receive_sequence: u8,
        segmented: bool,
    ) -> Self {
        let control_field = (send_sequence << 1) | (receive_sequence << 5);
        if segmented {
            // Set segmentation bit
            // This is typically in the frame format byte
        }

        let length = 2 // Frame format
            + address_pair.destination().byte_length()
            + address_pair.source().byte_length()
            + 1 // Control field
            + information_field.len()
            + 2 // FCS
            + 2; // Additional FCS

        Self {
            frame_type: FrameType::Information,
            information_field,
            send_sequence: Some(send_sequence),
            receive_sequence: Some(receive_sequence),
            segmented,
            control_field,
            address_pair,
            length,
        }
    }

    /// Decode an HDLC frame from bytes
    pub fn decode(frame: &[u8]) -> DlmsResult<Self> {
        if frame.len() < 5 {
            return Err(DlmsError::FrameInvalid("Frame too short".to_string()));
        }

        let mut fcs_calc = FcsCalc::new();
        let mut pos = 0;

        // Read frame format
        let frame_format_h = frame[pos];
        if (frame_format_h & 0xF0) != 0xA0 {
            return Err(DlmsError::FrameInvalid("Illegal frame format".to_string()));
        }
        let segmented = (frame_format_h & 0x08) == 0x08;
        fcs_calc.update(frame_format_h);
        pos += 1;

        let frame_format_l = frame[pos];
        fcs_calc.update(frame_format_l);
        pos += 1;

        // Read destination address
        let (destination, dest_len) = Self::read_address(&mut fcs_calc, &frame[pos..])?;
        pos += dest_len;

        // Read source address
        let (source, src_len) = Self::read_address(&mut fcs_calc, &frame[pos..])?;
        pos += src_len;

        // Read control field
        if pos >= frame.len() {
            return Err(DlmsError::FrameInvalid("Frame too short for control field".to_string()));
        }
        let control_field = frame[pos];
        fcs_calc.update(control_field);
        pos += 1;

        let frame_type = FrameType::from_control_byte(control_field);
        if frame_type == FrameType::InvalidType {
            return Err(DlmsError::FrameInvalid(format!(
                "Control field unknown: 0x{:02X}",
                control_field
            )));
        }

        // Extract sequence numbers
        let (send_sequence, receive_sequence) = match frame_type {
            FrameType::Information => {
                let send_seq = Some((control_field & 0x0E) >> 1);
                let recv_seq = Some((control_field & 0xE0) >> 5);
                (send_seq, recv_seq)
            }
            FrameType::ReceiveReady | FrameType::ReceiveNotReady => {
                let recv_seq = Some((control_field & 0xE0) >> 5);
                (None, recv_seq)
            }
            _ => (None, None),
        };

        // Verify FCS
        if pos + 2 > frame.len() {
            return Err(DlmsError::FrameInvalid("Frame too short for FCS".to_string()));
        }
        fcs_calc.update(frame[pos]);
        fcs_calc.update(frame[pos + 1]);
        fcs_calc.validate()?;
        pos += 2;

        // Read information field if present
        let information_field = if pos < frame.len() - 2 {
            let info_len = frame.len() - pos - 2; // Remaining bytes minus FCS
            let mut info_field = vec![0u8; info_len];
            let mut info_fcs = FcsCalc::new();
            
            // Update FCS with info field
            for i in 0..info_len {
                info_field[i] = frame[pos + i];
                info_fcs.update(frame[pos + i]);
            }
            pos += info_len;

            // Verify info field FCS
            if pos + 2 > frame.len() {
                return Err(DlmsError::FrameInvalid("Frame too short for info field FCS".to_string()));
            }
            info_fcs.update(frame[pos]);
            info_fcs.update(frame[pos + 1]);
            info_fcs.validate()?;

            info_field
        } else {
            Vec::new()
        };

        let address_pair = HdlcAddressPair::new(source, destination);

        Ok(Self {
            frame_type,
            information_field,
            send_sequence,
            receive_sequence,
            segmented,
            control_field,
            address_pair,
            length: frame.len(),
        })
    }

    /// Encode frame to bytes
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut result = Vec::new();
        let mut fcs_calc = FcsCalc::new();

        // Frame format
        let frame_format_h = if self.segmented {
            0xA8 // Segmented bit set
        } else {
            0xA0
        };
        result.push(frame_format_h);
        fcs_calc.update(frame_format_h);

        let frame_format_l = 0x00; // Length will be calculated
        result.push(frame_format_l);
        fcs_calc.update(frame_format_l);

        // Destination address
        let dest_addr = self.address_pair.destination().encode()?;
        for &byte in &dest_addr {
            fcs_calc.update(byte);
        }
        result.extend_from_slice(&dest_addr);

        // Source address
        let src_addr = self.address_pair.source().encode()?;
        for &byte in &src_addr {
            fcs_calc.update(byte);
        }
        result.extend_from_slice(&src_addr);

        // Control field
        let control = if let (Some(send_seq), Some(recv_seq)) = (self.send_sequence, self.receive_sequence) {
            FrameType::Information.to_control_byte(Some(send_seq), Some(recv_seq))
        } else {
            self.frame_type.to_control_byte(None, None)
        };
        result.push(control);
        fcs_calc.update(control);

        // FCS
        let fcs = fcs_calc.fcs_value_bytes();
        result.extend_from_slice(&fcs);

        // Information field if present
        if !self.information_field.is_empty() {
            let mut info_fcs = FcsCalc::new();
            for &byte in &self.information_field {
                result.push(byte);
                info_fcs.update(byte);
            }
            let info_fcs_bytes = info_fcs.fcs_value_bytes();
            result.extend_from_slice(&info_fcs_bytes);
        }

        // Update frame format with actual length (11-bit length field spans both bytes)
        // HDLC frame format: byte 0 bits 2-0 contain length bits 10-8, byte 1 contains length bits 7-0
        let length = result.len() as u16;
        
        // Extract high 3 bits of length (bits 10-8) and put them in byte 0 bits 2-0
        let length_high = ((length >> 8) & 0x07) as u8;
        result[0] = result[0] | length_high;
        
        // Put low 8 bits of length (bits 7-0) in byte 1
        result[1] = (length & 0xFF) as u8;
        
        // Recalculate FCS since we updated the frame format bytes
        // FCS includes: frame format (2 bytes) + addresses + control field
        let mut fcs_recalc = FcsCalc::new();
        fcs_recalc.update(result[0]); // Updated frame format byte 0
        fcs_recalc.update(result[1]); // Updated frame format byte 1
        
        // Calculate positions
        let dest_len = self.address_pair.destination().byte_length();
        let src_len = self.address_pair.source().byte_length();
        let addr_start = 2; // After frame format
        let control_pos = addr_start + dest_len + src_len;
        let fcs_pos = control_pos + 1;
        
        // Update FCS with addresses and control field
        for i in addr_start..control_pos {
            fcs_recalc.update(result[i]);
        }
        fcs_recalc.update(result[control_pos]);
        
        // Replace old FCS with new FCS
        let new_fcs = fcs_recalc.fcs_value_bytes();
        result[fcs_pos] = new_fcs[0];
        result[fcs_pos + 1] = new_fcs[1];
        
        // Note: Information field FCS doesn't need recalculation as it's independent

        Ok(result)
    }

    fn read_address(fcs_calc: &mut FcsCalc, data: &[u8]) -> DlmsResult<(HdlcAddress, usize)> {
        let mut addr_bytes = [0u8; 4];
        let mut length = 0;
        
        while length < 4 {
            if length >= data.len() {
                return Err(DlmsError::FrameInvalid("HDLC address is illegal in frame".to_string()));
            }
            
            let current_byte = data[length];
            fcs_calc.update(current_byte);
            addr_bytes[length] = current_byte;
            length += 1;
            
            if (current_byte & 0x01) != 0 {
                break; // Stop bit set
            }
        }

        let address = HdlcAddress::decode(&addr_bytes, length)?;
        Ok((address, length))
    }

    /// Get frame type
    pub fn frame_type(&self) -> FrameType {
        self.frame_type
    }

    /// Get information field
    pub fn information_field(&self) -> &[u8] {
        &self.information_field
    }

    /// Get send sequence number
    pub fn send_sequence(&self) -> Option<u8> {
        self.send_sequence
    }

    /// Get receive sequence number
    pub fn receive_sequence(&self) -> Option<u8> {
        self.receive_sequence
    }

    /// Check if frame is segmented
    pub fn is_segmented(&self) -> bool {
        self.segmented
    }

    /// Get address pair
    pub fn address_pair(&self) -> HdlcAddressPair {
        self.address_pair
    }

    /// Get frame length
    pub fn length(&self) -> usize {
        self.length
    }
}

impl fmt::Display for HdlcFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HDLC Frame: type={:?}, src={}, dst={}, len={}",
            self.frame_type,
            self.address_pair.source(),
            self.address_pair.destination(),
            self.length
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_type_from_control() {
        assert_eq!(FrameType::from_control_byte(0x00), FrameType::Information);
        assert_eq!(FrameType::from_control_byte(0x01), FrameType::ReceiveReady);
    }
}