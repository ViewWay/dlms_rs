//! Short Name (SN) Addressing PDU types for DLMS/COSEM
//!
//! This module provides PDU types for Short Name addressing mode in DLMS/COSEM.
//!
//! # Short Name vs Logical Name Addressing
//!
//! DLMS/COSEM supports two addressing modes:
//! - **Logical Name (LN)**: Uses 6-byte OBIS codes for object identification
//! - **Short Name (SN)**: Uses 2-byte base names for efficient addressing
//!
//! SN addressing is more compact and efficient for devices with limited resources.
//!
//! # PDU Mapping
//!
//! | SN PDU | LN PDU Equivalent | Tag |
//! |--------|------------------|-----|
//! | ReadRequest | GetRequest | 1 |
//! | ReadResponse | GetResponse | 8 |
//! | WriteRequest | SetRequest | 2 |
//! | WriteResponse | SetResponse | 3 |
//! | UnconfirmedWriteRequest | - | 4 |
//! | InformationReportRequest | EventNotification | 5 |
//!
//! # SN Address Format
//!
//! SN addresses use a 2-byte base name (uint16) instead of 6-byte OBIS codes.
//! This reduces message size and improves parsing efficiency.
//!
//! # Encoding Format
//!
//! All SN PDUs are encoded using A-XDR format with a tag byte followed by
//! the PDU-specific fields.

use dlms_core::{DlmsError, DlmsResult};
use dlms_asn1::{AxdrEncoder, AxdrDecoder};
use crate::pdu::{InvokeIdAndPriority, DataObject, GetDataResult, SetDataResult, ActionResult};

/// SN PDU tag values
///
/// Each SN PDU type has a unique tag byte identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SnPduTag {
    /// ReadRequest (SN GetRequest)
    ReadRequest = 1,
    /// ReadResponse (SN GetResponse)
    ReadResponse = 8,
    /// WriteRequest (SN SetRequest)
    WriteRequest = 2,
    /// WriteResponse (SN SetResponse)
    WriteResponse = 3,
    /// UnconfirmedWriteRequest (SN only)
    UnconfirmedWriteRequest = 4,
    /// InformationReportRequest (SN EventNotification)
    InformationReportRequest = 5,
}

impl SnPduTag {
    /// Create from u8 value
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::ReadRequest),
            8 => Some(Self::ReadResponse),
            2 => Some(Self::WriteRequest),
            3 => Some(Self::WriteResponse),
            4 => Some(Self::UnconfirmedWriteRequest),
            5 => Some(Self::InformationReportRequest),
            _ => None,
        }
    }

    /// Get the u8 value
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Short Name address (2 bytes)
///
/// SN addresses use a 16-bit unsigned integer for compact addressing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShortName(pub u16);

impl ShortName {
    /// Create a new short name
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Get the short name value
    #[must_use]
    pub const fn value(&self) -> u16 {
        self.0
    }
}

impl From<u16> for ShortName {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

/// SN ReadRequest (equivalent to LN GetRequest)
///
/// Used to read attribute values from COSEM objects using Short Name addressing.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadRequest {
    /// Invoke ID and priority
    pub invoke_id: InvokeIdAndPriority,
    /// Short name of the object to read
    pub short_name: ShortName,
}

impl ReadRequest {
    /// Create a new ReadRequest
    ///
    /// # Arguments
    /// * `invoke_id` - Invoke ID and priority
    /// * `short_name` - Short name of the object to read
    #[must_use]
    pub fn new(invoke_id: InvokeIdAndPriority, short_name: ShortName) -> Self {
        Self {
            invoke_id,
            short_name,
        }
    }

    /// Encode to A-XDR format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Tag
        encoder.encode_u8(SnPduTag::ReadRequest.as_u8())?;

        // Invoke ID (1 byte)
        encoder.encode_u8(self.invoke_id.value())?;

        // Short name (2 bytes, big-endian)
        encoder.encode_u16(self.short_name.value())?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty ReadRequest".to_string()));
        }

        let tag = data[0];
        if tag != SnPduTag::ReadRequest.as_u8() {
            return Err(DlmsError::InvalidData(format!(
                "Expected ReadRequest tag (1), got {}",
                tag
            )));
        }

        if data.len() < 4 {
            return Err(DlmsError::InvalidData(
                "ReadRequest too short".to_string(),
            ));
        }

        let invoke_id = InvokeIdAndPriority::new(data[1]);
        let short_name = ShortName(u16::from_be_bytes([data[2], data[3]]));

        Ok(Self {
            invoke_id,
            short_name,
        })
    }
}

/// SN ReadResponse (equivalent to LN GetResponse)
///
/// Response to a ReadRequest containing the requested data.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadResponse {
    /// Invoke ID and priority
    pub invoke_id: InvokeIdAndPriority,
    /// Result data
    pub result: GetDataResult,
}

impl ReadResponse {
    /// Create a new ReadResponse
    ///
    /// # Arguments
    /// * `invoke_id` - Invoke ID and priority
    /// * `result` - Result data
    #[must_use]
    pub fn new(invoke_id: InvokeIdAndPriority, result: GetDataResult) -> Self {
        Self {
            invoke_id,
            result,
        }
    }

    /// Encode to A-XDR format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Tag
        encoder.encode_u8(SnPduTag::ReadResponse.as_u8())?;

        // Invoke ID (1 byte)
        encoder.encode_u8(self.invoke_id.value())?;

        // Result (variable)
        // For simplicity, we encode as DataObject
        match &self.result {
            GetDataResult::Data(data) => {
                encoder.encode_data(data)?;
            }
            GetDataResult::DataBlock(_) => {
                return Err(DlmsError::InvalidData(
                    "Data blocks not yet implemented for SN ReadResponse".to_string(),
                ));
            }
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty ReadResponse".to_string()));
        }

        let tag = data[0];
        if tag != SnPduTag::ReadResponse.as_u8() {
            return Err(DlmsError::InvalidData(format!(
                "Expected ReadResponse tag (8), got {}",
                tag
            )));
        }

        if data.len() < 2 {
            return Err(DlmsError::InvalidData(
                "ReadResponse too short".to_string(),
            ));
        }

        let invoke_id = InvokeIdAndPriority::new(data[1]);

        // Decode result data
        let mut decoder = AxdrDecoder::new(&data[2..]);
        let data_obj = decoder.decode_data()?;
        let result = GetDataResult::Data(data_obj);

        Ok(Self {
            invoke_id,
            result,
        })
    }
}

/// SN WriteRequest (equivalent to LN SetRequest)
///
/// Used to write attribute values to COSEM objects using Short Name addressing.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteRequest {
    /// Invoke ID and priority
    pub invoke_id: InvokeIdAndPriority,
    /// Short name of the object to write
    pub short_name: ShortName,
    /// Data to write
    pub data: DataObject,
    /// Request priority (optional)
    pub priority: Option<u8>,
}

impl WriteRequest {
    /// Create a new WriteRequest
    ///
    /// # Arguments
    /// * `invoke_id` - Invoke ID and priority
    /// * `short_name` - Short name of the object to write
    /// * `data` - Data to write
    #[must_use]
    pub fn new(invoke_id: InvokeIdAndPriority, short_name: ShortName, data: DataObject) -> Self {
        Self {
            invoke_id,
            short_name,
            data,
            priority: None,
        }
    }

    /// Encode to A-XDR format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Tag
        encoder.encode_u8(SnPduTag::WriteRequest.as_u8())?;

        // Invoke ID (1 byte)
        encoder.encode_u8(self.invoke_id.value())?;

        // Short name (2 bytes, big-endian)
        encoder.encode_u16(self.short_name.value())?;

        // Data
        encoder.encode_data(&self.data)?;

        // Priority (optional)
        if let Some(prio) = self.priority {
            encoder.encode_u8(prio)?;
        }

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty WriteRequest".to_string()));
        }

        let tag = data[0];
        if tag != SnPduTag::WriteRequest.as_u8() {
            return Err(DlmsError::InvalidData(format!(
                "Expected WriteRequest tag (2), got {}",
                tag
            )));
        }

        if data.len() < 4 {
            return Err(DlmsError::InvalidData(
                "WriteRequest too short".to_string(),
            ));
        }

        let invoke_id = InvokeIdAndPriority::new(data[1]);
        let short_name = ShortName(u16::from_be_bytes([data[2], data[3]]));

        // Decode data (remaining bytes)
        let mut decoder = AxdrDecoder::new(&data[4..]);
        let data_obj = decoder.decode_data()?;

        Ok(Self {
            invoke_id,
            short_name,
            data: data_obj,
            priority: None,
        })
    }
}

/// SN WriteResponse (equivalent to LN SetResponse)
///
/// Response to a WriteRequest.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteResponse {
    /// Invoke ID and priority
    pub invoke_id: InvokeIdAndPriority,
    /// Result of the write operation
    pub result: SetDataResult,
}

impl WriteResponse {
    /// Create a new WriteResponse
    ///
    /// # Arguments
    /// * `invoke_id` - Invoke ID and priority
    /// * `result` - Result of the write operation
    #[must_use]
    pub fn new(invoke_id: InvokeIdAndPriority, result: SetDataResult) -> Self {
        Self {
            invoke_id,
            result,
        }
    }

    /// Encode to A-XDR format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Tag
        encoder.encode_u8(SnPduTag::WriteResponse.as_u8())?;

        // Invoke ID (1 byte)
        encoder.encode_u8(self.invoke_id.value())?;

        // Result (1 byte for status)
        encoder.encode_u8(self.result.as_u8())?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty WriteResponse".to_string()));
        }

        let tag = data[0];
        if tag != SnPduTag::WriteResponse.as_u8() {
            return Err(DlmsError::InvalidData(format!(
                "Expected WriteResponse tag (3), got {}",
                tag
            )));
        }

        if data.len() < 3 {
            return Err(DlmsError::InvalidData(
                "WriteResponse too short".to_string(),
            ));
        }

        let invoke_id = InvokeIdAndPriority::new(data[1]);
        let result = SetDataResult::from_u8(data[2])?;

        Ok(Self {
            invoke_id,
            result,
        })
    }
}

/// SN UnconfirmedWriteRequest (SN-specific, no LN equivalent)
///
/// Used for unconfirmed write operations using Short Name addressing.
/// This is typically used for one-way communication where confirmation is not required.
#[derive(Debug, Clone, PartialEq)]
pub struct UnconfirmedWriteRequest {
    /// Short name of the object to write
    pub short_name: ShortName,
    /// Data to write
    pub data: DataObject,
    /// Priority (optional)
    pub priority: Option<u8>,
}

impl UnconfirmedWriteRequest {
    /// Create a new UnconfirmedWriteRequest
    ///
    /// # Arguments
    /// * `short_name` - Short name of the object to write
    /// * `data` - Data to write
    #[must_use]
    pub fn new(short_name: ShortName, data: DataObject) -> Self {
        Self {
            short_name,
            data,
            priority: None,
        }
    }

    /// Encode to A-XDR format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Tag
        encoder.encode_u8(SnPduTag::UnconfirmedWriteRequest.as_u8())?;

        // Short name (2 bytes, big-endian)
        encoder.encode_u16(self.short_name.value())?;

        // Data
        encoder.encode_data(&self.data)?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData(
                "Empty UnconfirmedWriteRequest".to_string(),
            ));
        }

        let tag = data[0];
        if tag != SnPduTag::UnconfirmedWriteRequest.as_u8() {
            return Err(DlmsError::InvalidData(format!(
                "Expected UnconfirmedWriteRequest tag (4), got {}",
                tag
            )));
        }

        if data.len() < 3 {
            return Err(DlmsError::InvalidData(
                "UnconfirmedWriteRequest too short".to_string(),
            ));
        }

        let short_name = ShortName(u16::from_be_bytes([data[1], data[2]]));

        // Decode data (remaining bytes)
        let mut decoder = AxdrDecoder::new(&data[3..]);
        let data_obj = decoder.decode_data()?;

        Ok(Self {
            short_name,
            data: data_obj,
            priority: None,
        })
    }
}

/// SN InformationReportRequest (equivalent to LN EventNotification)
///
/// Used for asynchronous event reporting using Short Name addressing.
#[derive(Debug, Clone, PartialEq)]
pub struct InformationReportRequest {
    /// Short name of the reporting object
    pub short_name: ShortName,
    /// Reported data
    pub data: DataObject,
    /// Priority (optional)
    pub priority: Option<u8>,
}

impl InformationReportRequest {
    /// Create a new InformationReportRequest
    ///
    /// # Arguments
    /// * `short_name` - Short name of the reporting object
    /// * `data` - Reported data
    #[must_use]
    pub fn new(short_name: ShortName, data: DataObject) -> Self {
        Self {
            short_name,
            data,
            priority: None,
        }
    }

    /// Encode to A-XDR format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = AxdrEncoder::new();

        // Tag
        encoder.encode_u8(SnPduTag::InformationReportRequest.as_u8())?;

        // Short name (2 bytes, big-endian)
        encoder.encode_u16(self.short_name.value())?;

        // Data
        encoder.encode_data(&self.data)?;

        Ok(encoder.into_bytes())
    }

    /// Decode from A-XDR format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData(
                "Empty InformationReportRequest".to_string(),
            ));
        }

        let tag = data[0];
        if tag != SnPduTag::InformationReportRequest.as_u8() {
            return Err(DlmsError::InvalidData(format!(
                "Expected InformationReportRequest tag (5), got {}",
                tag
            )));
        }

        if data.len() < 3 {
            return Err(DlmsError::InvalidData(
                "InformationReportRequest too short".to_string(),
            ));
        }

        let short_name = ShortName(u16::from_be_bytes([data[1], data[2]]));

        // Decode data (remaining bytes)
        let mut decoder = AxdrDecoder::new(&data[3..]);
        let data_obj = decoder.decode_data()?;

        Ok(Self {
            short_name,
            data: data_obj,
            priority: None,
        })
    }
}

/// SN PDU enumeration
///
/// This enum represents all Short Name addressing PDUs.
#[derive(Debug, Clone, PartialEq)]
pub enum SnPdu {
    /// ReadRequest
    ReadRequest(ReadRequest),
    /// ReadResponse
    ReadResponse(ReadResponse),
    /// WriteRequest
    WriteRequest(WriteRequest),
    /// WriteResponse
    WriteResponse(WriteResponse),
    /// UnconfirmedWriteRequest
    UnconfirmedWriteRequest(UnconfirmedWriteRequest),
    /// InformationReportRequest
    InformationReportRequest(InformationReportRequest),
}

impl SnPdu {
    /// Decode an SN PDU from bytes
    ///
    /// This method automatically detects the PDU type based on the tag byte.
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        if data.is_empty() {
            return Err(DlmsError::InvalidData("Empty SN PDU".to_string()));
        }

        let tag = data[0];
        match SnPduTag::from_u8(tag) {
            Some(SnPduTag::ReadRequest) => {
                Ok(Self::ReadRequest(ReadRequest::decode(data)?))
            }
            Some(SnPduTag::ReadResponse) => {
                Ok(Self::ReadResponse(ReadResponse::decode(data)?))
            }
            Some(SnPduTag::WriteRequest) => {
                Ok(Self::WriteRequest(WriteRequest::decode(data)?))
            }
            Some(SnPduTag::WriteResponse) => {
                Ok(Self::WriteResponse(WriteResponse::decode(data)?))
            }
            Some(SnPduTag::UnconfirmedWriteRequest) => {
                Ok(Self::UnconfirmedWriteRequest(UnconfirmedWriteRequest::decode(data)?))
            }
            Some(SnPduTag::InformationReportRequest) => {
                Ok(Self::InformationReportRequest(InformationReportRequest::decode(data)?))
            }
            None => Err(DlmsError::InvalidData(format!(
                "Unknown SN PDU tag: {}",
                tag
            ))),
        }
    }

    /// Encode the SN PDU to bytes
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        match self {
            Self::ReadRequest(pdu) => pdu.encode(),
            Self::ReadResponse(pdu) => pdu.encode(),
            Self::WriteRequest(pdu) => pdu.encode(),
            Self::WriteResponse(pdu) => pdu.encode(),
            Self::UnconfirmedWriteRequest(pdu) => pdu.encode(),
            Self::InformationReportRequest(pdu) => pdu.encode(),
        }
    }

    /// Get the invoke ID (if applicable)
    #[must_use]
    pub fn invoke_id(&self) -> Option<InvokeIdAndPriority> {
        match self {
            Self::ReadRequest(pdu) => Some(pdu.invoke_id),
            Self::ReadResponse(pdu) => Some(pdu.invoke_id),
            Self::WriteRequest(pdu) => Some(pdu.invoke_id),
            Self::WriteResponse(pdu) => Some(pdu.invoke_id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dlms_core::datatypes::Data;

    #[test]
    fn test_short_name() {
        let sn = ShortName::new(0x1234);
        assert_eq!(sn.value(), 0x1234);
    }

    #[test]
    fn test_sn_pdu_tag_from_u8() {
        assert_eq!(SnPduTag::from_u8(1), Some(SnPduTag::ReadRequest));
        assert_eq!(SnPduTag::from_u8(8), Some(SnPduTag::ReadResponse));
        assert_eq!(SnPduTag::from_u8(2), Some(SnPduTag::WriteRequest));
        assert_eq!(SnPduTag::from_u8(3), Some(SnPduTag::WriteResponse));
        assert_eq!(SnPduTag::from_u8(4), Some(SnPduTag::UnconfirmedWriteRequest));
        assert_eq!(SnPduTag::from_u8(5), Some(SnPduTag::InformationReportRequest));
        assert_eq!(SnPduTag::from_u8(99), None);
    }

    #[test]
    fn test_read_request_encode_decode() {
        let invoke_id = InvokeIdAndPriority::new(0x12);
        let short_name = ShortName::new(0x1234);
        let request = ReadRequest::new(invoke_id, short_name);

        let encoded = request.encode().unwrap();
        assert_eq!(encoded.len(), 4);
        assert_eq!(encoded[0], 0x01); // ReadRequest tag

        let decoded = ReadRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.invoke_id.value(), 0x12);
        assert_eq!(decoded.short_name.value(), 0x1234);
    }

    #[test]
    fn test_read_response_encode_decode() {
        let invoke_id = InvokeIdAndPriority::new(0x12);
        let data = DataObject::new(Data::Unsigned8(42));
        let result = GetDataResult::Data(data);
        let response = ReadResponse::new(invoke_id, result);

        let encoded = response.encode().unwrap();
        assert_eq!(encoded[0], 0x08); // ReadResponse tag

        let decoded = ReadResponse::decode(&encoded).unwrap();
        assert_eq!(decoded.invoke_id.value(), 0x12);
    }

    #[test]
    fn test_write_request_encode_decode() {
        let invoke_id = InvokeIdAndPriority::new(0x12);
        let short_name = ShortName::new(0x1234);
        let data = DataObject::new(Data::Unsigned8(42));
        let request = WriteRequest::new(invoke_id, short_name, data);

        let encoded = request.encode().unwrap();
        assert_eq!(encoded[0], 0x02); // WriteRequest tag

        let decoded = WriteRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.invoke_id.value(), 0x12);
        assert_eq!(decoded.short_name.value(), 0x1234);
    }

    #[test]
    fn test_write_response_encode_decode() {
        let invoke_id = InvokeIdAndPriority::new(0x12);
        let result = SetDataResult::Success;
        let response = WriteResponse::new(invoke_id, result);

        let encoded = response.encode().unwrap();
        assert_eq!(encoded[0], 0x03); // WriteResponse tag

        let decoded = WriteResponse::decode(&encoded).unwrap();
        assert_eq!(decoded.invoke_id.value(), 0x12);
        assert_eq!(decoded.result, SetDataResult::Success);
    }

    #[test]
    fn test_unconfirmed_write_request_encode_decode() {
        let short_name = ShortName::new(0x1234);
        let data = DataObject::new(Data::Unsigned8(42));
        let request = UnconfirmedWriteRequest::new(short_name, data);

        let encoded = request.encode().unwrap();
        assert_eq!(encoded[0], 0x04); // UnconfirmedWriteRequest tag

        let decoded = UnconfirmedWriteRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.short_name.value(), 0x1234);
    }

    #[test]
    fn test_information_report_request_encode_decode() {
        let short_name = ShortName::new(0x1234);
        let data = DataObject::new(Data::Unsigned8(42));
        let request = InformationReportRequest::new(short_name, data);

        let encoded = request.encode().unwrap();
        assert_eq!(encoded[0], 0x05); // InformationReportRequest tag

        let decoded = InformationReportRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.short_name.value(), 0x1234);
    }

    #[test]
    fn test_sn_pdu_auto_decode() {
        let invoke_id = InvokeIdAndPriority::new(0x12);
        let short_name = ShortName::new(0x1234);
        let request = ReadRequest::new(invoke_id, short_name);

        let encoded = request.encode().unwrap();
        let decoded = SnPdu::decode(&encoded).unwrap();

        assert!(matches!(decoded, SnPdu::ReadRequest(_)));
    }
}
