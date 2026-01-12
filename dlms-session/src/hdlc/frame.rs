//! HDLC frame structure and encoding/decoding

use crate::error::{DlmsError, DlmsResult};
use crate::hdlc::address::{HdlcAddress, HdlcAddressPair};
use crate::hdlc::fcs::FcsCalc;
use std::fmt;

/// HDLC frame flag
pub const FLAG: u8 = 0x7E;

/// Logical Link Control (LLC) Request
/// Used for client-to-server communication (requests)
pub const LLC_REQUEST: [u8; 3] = [0xE6, 0xE6, 0x00];

/// Logical Link Control (LLC) Response
/// Used for server-to-client communication (responses)
/// According to DLMS standard, responses use 0xE7 in the second byte instead of 0xE6
pub const LLC_RESPONSE: [u8; 3] = [0xE6, 0xE7, 0x00];

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
    ///
    /// # Frame Length Calculation
    /// Total frame length (excluding 0x7E flags):
    /// - Frame format: 2 bytes
    /// - Destination address: 1-4 bytes
    /// - Source address: 1-4 bytes
    /// - Control field: 1 byte
    /// - HCS: 2 bytes (Header Check Sequence)
    /// - Information field: variable (if present)
    /// - FCS: 2 bytes (Frame Check Sequence)
    ///
    /// # Why This Design?
    /// The length field is used for frame boundary detection and validation.
    /// Including HCS in the length calculation ensures the frame format length
    /// field accurately represents the total frame size.
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
            + 2 // HCS (Header Check Sequence)
            + info_field.len()
            + 2; // FCS (Frame Check Sequence)

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

    /// Create a new RR (Receive Ready) frame with receive sequence number
    ///
    /// # Arguments
    /// * `address_pair` - Source and destination addresses
    /// * `receive_sequence` - N(R) value indicating the next expected sequence number (0-7)
    ///
    /// # RR Frame Format
    /// According to HDLC standard, RR frame control byte format:
    /// - Bit 0: 1 (indicates RR frame)
    /// - Bits 1-3: 000
    /// - Bits 5-7: N(R) (next expected receive sequence number)
    ///
    /// # Why This Method?
    /// RR frames are used for:
    /// - Flow control: Acknowledging received frames
    /// - Segmented frame handling: Requesting next segment (per dlms-docs/dlms/长数据帧处理.txt)
    ///
    /// # Usage
    /// When receiving a segmented frame (S bit = 1), send an RR frame with N(R) = expected next sequence
    /// to request the next segment.
    pub fn new_receive_ready(
        address_pair: HdlcAddressPair,
        receive_sequence: u8,
    ) -> Self {
        // Validate receive sequence (0-7)
        let recv_seq = receive_sequence % 8;
        let control_field = FrameType::ReceiveReady.to_control_byte(None, Some(recv_seq));
        
        let length = 2 // Frame format
            + address_pair.destination().byte_length()
            + address_pair.source().byte_length()
            + 1 // Control field
            + 2 // HCS (Header Check Sequence)
            + 2; // FCS (Frame Check Sequence)
        // Note: RR frames have no information field

        Self {
            frame_type: FrameType::ReceiveReady,
            information_field: Vec::new(),
            send_sequence: None,
            receive_sequence: Some(recv_seq),
            segmented: false,
            control_field,
            address_pair,
            length,
        }
    }

    /// Create a new information frame with sequence numbers
    ///
    /// # Frame Length Calculation
    /// Total frame length (excluding 0x7E flags):
    /// - Frame format: 2 bytes
    /// - Destination address: 1-4 bytes
    /// - Source address: 1-4 bytes
    /// - Control field: 1 byte
    /// - HCS: 2 bytes (Header Check Sequence)
    /// - Information field: variable
    /// - FCS: 2 bytes (Frame Check Sequence)
    ///
    /// # Segmentation
    /// The `segmented` parameter indicates if this frame is part of a larger
    /// message that spans multiple frames. When true, the S bit in the frame
    /// format byte is set to 1.
    pub fn new_information(
        address_pair: HdlcAddressPair,
        information_field: Vec<u8>,
        send_sequence: u8,
        receive_sequence: u8,
        segmented: bool,
    ) -> Self {
        let control_field = (send_sequence << 1) | (receive_sequence << 5);
        // Note: Segmentation bit is set in the frame format byte during encoding

        let length = 2 // Frame format
            + address_pair.destination().byte_length()
            + address_pair.source().byte_length()
            + 1 // Control field
            + 2 // HCS (Header Check Sequence)
            + information_field.len()
            + 2; // FCS (Frame Check Sequence)

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
    ///
    /// # HDLC Frame Structure (per dlms-docs/dlms/hdlc帧格式.txt)
    ///
    /// Expected frame structure:
    /// ```
    /// 标志(0x7E) | 帧格式(2) | 目的地址 | 源地址 | 控制(1) | HCS(2) | 信息 | FCS(2) | 标志(0x7E)
    /// ```
    ///
    /// # Decoding Process
    /// 1. Read and validate frame format
    /// 2. Read destination address
    /// 3. Read source address
    /// 4. Read control field
    /// 5. Read and validate HCS (Header Check Sequence)
    /// 6. Read information field (if present)
    /// 7. Read and validate FCS (Frame Check Sequence)
    ///
    /// # Error Handling
    /// - Frame too short: Returns `DlmsError::FrameInvalid`
    /// - Invalid frame format: Returns `DlmsError::FrameInvalid`
    /// - HCS validation failure: Returns `DlmsError::FrameInvalid`
    /// - FCS validation failure: Returns `DlmsError::FrameInvalid`
    ///
    /// # Why Two Checksums?
    /// - **HCS**: Validates header integrity early, allowing fast rejection of corrupted headers
    ///   without processing the information field. This is especially important for large frames.
    /// - **FCS**: Validates the complete frame, ensuring end-to-end integrity.
    ///
    /// # Optimization Considerations
    /// - Early HCS validation allows skipping information field processing for corrupted headers
    /// - FCS validation is performed after all fields are read to ensure complete integrity
    /// - Address reading uses a helper function to handle variable-length addresses efficiently
    pub fn decode(frame: &[u8]) -> DlmsResult<Self> {
        // Minimum frame size: frame format(2) + addresses(2) + control(1) + HCS(2) + FCS(2) = 9 bytes
        if frame.len() < 9 {
            return Err(DlmsError::FrameInvalid(format!(
                "Frame too short: expected at least 9 bytes, got {}",
                frame.len()
            )));
        }

        let mut pos = 0;

        // Step 1: Read and validate frame format
        let frame_format_h = frame[pos];
        if (frame_format_h & 0xF0) != 0xA0 {
            return Err(DlmsError::FrameInvalid(format!(
                "Illegal frame format: expected 0xA0-0xAF, got 0x{:02X}",
                frame_format_h
            )));
        }
        let segmented = (frame_format_h & 0x08) == 0x08;
        pos += 1;

        let frame_format_l = frame[pos];
        pos += 1;

        // Step 2-4: Read addresses and control field, calculate HCS
        let mut hcs_calc = FcsCalc::new();
        hcs_calc.update(frame_format_h);
        hcs_calc.update(frame_format_l);

        // Read destination address
        let (destination, dest_len) = Self::read_address(&mut hcs_calc, &frame[pos..])?;
        pos += dest_len;

        // Read source address
        let (source, src_len) = Self::read_address(&mut hcs_calc, &frame[pos..])?;
        pos += src_len;

        // Read control field
        if pos >= frame.len() {
            return Err(DlmsError::FrameInvalid("Frame too short for control field".to_string()));
        }
        let control_field = frame[pos];
        hcs_calc.update(control_field);
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

        // Step 5: Read and validate HCS (Header Check Sequence)
        if pos + 2 > frame.len() {
            return Err(DlmsError::FrameInvalid("Frame too short for HCS".to_string()));
        }
        hcs_calc.update(frame[pos]);
        hcs_calc.update(frame[pos + 1]);
        hcs_calc.validate()?;
        pos += 2;

        // Step 6: Read information field if present
        // Information field is between HCS and FCS
        // Remaining bytes = total - pos - FCS(2)
        let information_field = if pos < frame.len() - 2 {
            let info_len = frame.len() - pos - 2; // Remaining bytes minus FCS
            if info_len > 0 {
                frame[pos..pos + info_len].to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        pos += information_field.len();

        // Step 7: Read and validate FCS (Frame Check Sequence)
        // FCS validates: frame format(2) + addresses + control(1) + HCS(2) + information field
        if pos + 2 > frame.len() {
            return Err(DlmsError::FrameInvalid("Frame too short for FCS".to_string()));
        }
        let mut fcs_calc = FcsCalc::new();
        // Update with all fields except FCS itself (last 2 bytes)
        for i in 0..pos {
            fcs_calc.update(frame[i]);
        }
        fcs_calc.update(frame[pos]);
        fcs_calc.update(frame[pos + 1]);
        fcs_calc.validate()?;

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
    ///
    /// # HDLC Frame Structure (per dlms-docs/dlms/hdlc帧格式.txt)
    ///
    /// Correct frame structure:
    /// ```
    /// 标志(0x7E) | 帧格式(2) | 目的地址 | 源地址 | 控制(1) | HCS(2) | 信息 | FCS(2) | 标志(0x7E)
    /// ```
    ///
    /// # Why HCS and FCS?
    /// - **HCS (Header Check Sequence)**: Validates the frame header (format + addresses + control)
    ///   This allows early detection of header corruption before processing the information field.
    /// - **FCS (Frame Check Sequence)**: Validates the entire frame (header + HCS + information)
    ///   This provides end-to-end integrity checking for the complete frame.
    ///
    /// # Encoding Process
    /// 1. Encode frame format (initial values, length will be calculated later)
    /// 2. Encode destination address
    /// 3. Encode source address
    /// 4. Encode control field
    /// 5. Calculate and encode HCS (header checksum)
    /// 6. Encode information field (if present)
    /// 7. Calculate and encode FCS (full frame checksum, includes HCS)
    /// 8. Calculate and update frame format length field
    ///
    /// # Optimization Considerations
    /// - HCS and FCS use the same CRC algorithm, so we can reuse FcsCalc
    /// - Frame length is calculated after all fields are encoded to ensure accuracy
    /// - HCS recalculation is not needed after length update, as HCS only covers header fields
    ///   (frame format bytes are updated, but HCS calculation doesn't include the length field itself)
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut result = Vec::new();
        
        // Step 1: Encode frame format (initial values, length will be calculated later)
        let frame_format_h = if self.segmented {
            0xA8 // Segmented bit set (bit 3 = 1)
        } else {
            0xA0 // Frame type = 0x0A (1010), S bit = 0
        };
        result.push(frame_format_h);
        let frame_format_l = 0x00; // Length will be calculated and updated later
        result.push(frame_format_l);

        // Step 2-4: Encode addresses and control field
        // Calculate HCS for header fields: frame format + addresses + control
        let mut hcs_calc = FcsCalc::new();
        hcs_calc.update(frame_format_h);
        hcs_calc.update(frame_format_l);

        // Destination address
        let dest_addr = self.address_pair.destination().encode()?;
        for &byte in &dest_addr {
            hcs_calc.update(byte);
        }
        result.extend_from_slice(&dest_addr);

        // Source address
        let src_addr = self.address_pair.source().encode()?;
        for &byte in &src_addr {
            hcs_calc.update(byte);
        }
        result.extend_from_slice(&src_addr);

        // Control field
        let control = if let (Some(send_seq), Some(recv_seq)) = (self.send_sequence, self.receive_sequence) {
            FrameType::Information.to_control_byte(Some(send_seq), Some(recv_seq))
        } else {
            self.frame_type.to_control_byte(None, None)
        };
        result.push(control);
        hcs_calc.update(control);

        // Step 5: Calculate and encode HCS (Header Check Sequence)
        // HCS validates: frame format(2) + destination address + source address + control(1)
        let hcs = hcs_calc.fcs_value_bytes();
        result.extend_from_slice(&hcs);

        // Step 6: Encode information field if present
        if !self.information_field.is_empty() {
            result.extend_from_slice(&self.information_field);
        }

        // Step 7: Calculate and encode FCS (Frame Check Sequence)
        // FCS validates: frame format(2) + addresses + control(1) + HCS(2) + information field
        let mut fcs_calc = FcsCalc::new();
        // Update with all fields except FCS itself
        for &byte in &result {
            fcs_calc.update(byte);
        }
        let fcs = fcs_calc.fcs_value_bytes();
        result.extend_from_slice(&fcs);

        // Step 8: Calculate and update frame format length field
        // Length field: total frame size excluding the two 0x7E flags
        // According to document: "长度子域的值是除两个 7E 标志位之外的8位位组数"
        let dest_len = self.address_pair.destination().byte_length();
        let src_len = self.address_pair.source().byte_length();
        let total_length = 2 // Frame format
            + dest_len
            + src_len
            + 1 // Control field
            + 2 // HCS
            + self.information_field.len()
            + 2; // FCS
        
        // Update frame format with actual length (11-bit length field spans both bytes)
        // HDLC frame format: byte 0 bits 2-0 contain length bits 10-8, byte 1 contains length bits 7-0
        let length = total_length as u16;
        
        // Extract high 3 bits of length (bits 10-8) and put them in byte 0 bits 2-0
        let length_high = ((length >> 8) & 0x07) as u8;
        result[0] = (result[0] & 0xF8) | length_high; // Preserve frame type and S bit, update length high bits
        
        // Put low 8 bits of length (bits 7-0) in byte 1
        result[1] = (length & 0xFF) as u8;
        
        // Recalculate HCS since we updated the frame format bytes
        // HCS includes: frame format(2) + addresses + control field
        let mut hcs_recalc = FcsCalc::new();
        hcs_recalc.update(result[0]); // Updated frame format byte 0
        hcs_recalc.update(result[1]); // Updated frame format byte 1
        
        // Calculate positions
        let addr_start = 2; // After frame format
        let control_pos = addr_start + dest_len + src_len;
        let hcs_pos = control_pos + 1;
        
        // Update HCS with addresses and control field
        for i in addr_start..control_pos {
            hcs_recalc.update(result[i]);
        }
        hcs_recalc.update(result[control_pos]);
        
        // Replace old HCS with new HCS
        let new_hcs = hcs_recalc.fcs_value_bytes();
        result[hcs_pos] = new_hcs[0];
        result[hcs_pos + 1] = new_hcs[1];
        
        // Recalculate FCS since we updated frame format and HCS
        // FCS includes: frame format(2) + addresses + control(1) + HCS(2) + information + FCS itself
        let mut fcs_recalc = FcsCalc::new();
        // Update with all fields except FCS itself (last 2 bytes)
        for i in 0..(result.len() - 2) {
            fcs_recalc.update(result[i]);
        }
        let new_fcs = fcs_recalc.fcs_value_bytes();
        let fcs_pos = result.len() - 2;
        result[fcs_pos] = new_fcs[0];
        result[fcs_pos + 1] = new_fcs[1];

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