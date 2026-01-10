//! ISO-ACSE PDU structures
//!
//! This module provides the main ISO-ACSE PDU structures:
//! - AARQ (Association Request)
//! - AARE (Association Response)
//! - RLRQ (Release Request)
//! - RLRE (Release Response)
//!
//! # Encoding Format
//!
//! All ACSE PDUs are encoded using BER (Basic Encoding Rules) with:
//! - Application class tags (tags 0-3)
//! - Constructed encoding (contains nested TLVs)
//! - Context-specific tags for optional fields
//!
//! # Field Encoding Order
//!
//! Fields are encoded in reverse order (last field first) as per BER encoding
//! conventions. Optional fields are encoded only if present, using context-specific
//! tags to identify them.

use crate::error::{DlmsError, DlmsResult};
use crate::ber::{BerEncoder, BerDecoder, BerTag, BerTagClass, BerLength};
use super::types::*;

/// AARQ (Association Request) PDU
///
/// This PDU is sent by the client to request an association with the server.
/// It contains information about the client's capabilities and authentication.
///
/// # Structure
///
/// AARQ contains many optional fields, all encoded using context-specific tags:
/// - Tag 0: protocolVersion (optional BIT STRING)
/// - Tag 1: applicationContextName (required OBJECT IDENTIFIER)
/// - Tag 2: calledAPTitle (optional APTitle)
/// - Tag 3: calledAEQualifier (optional AEQualifier)
/// - Tag 4: calledAPInvocationIdentifier (optional INTEGER)
/// - Tag 5: calledAEInvocationIdentifier (optional INTEGER)
/// - Tag 6: callingAPTitle (optional APTitle)
/// - Tag 7: callingAEQualifier (optional AEQualifier)
/// - Tag 8: callingAPInvocationIdentifier (optional INTEGER)
/// - Tag 9: callingAEInvocationIdentifier (optional INTEGER)
/// - Tag 10: senderAcseRequirements (optional BIT STRING)
/// - Tag 11: mechanismName (optional OBJECT IDENTIFIER)
/// - Tag 12: callingAuthenticationValue (optional AuthenticationValue)
/// - Tag 13: applicationContextNameList (optional)
/// - Tag 29: implementationInformation (optional OCTET STRING)
/// - Tag 30: userInformation (optional AssociationInformation)
///
/// # Why So Many Optional Fields?
/// ISO-ACSE is designed to be flexible and support various authentication and
/// addressing schemes. DLMS/COSEM typically uses only a subset of these fields.
///
/// # Optimization Note
/// For DLMS/COSEM use cases, we can optimize by only encoding commonly used
/// fields, reducing message size and processing time.
#[derive(Debug, Clone, PartialEq)]
pub struct AARQApdu {
    /// Protocol version (optional)
    pub protocol_version: Option<Vec<u8>>, // BIT STRING
    /// Application context name (required)
    pub application_context_name: Vec<u32>, // OBJECT IDENTIFIER
    /// Called AP Title (optional)
    pub called_ap_title: Option<APTitle>,
    /// Called AE Qualifier (optional)
    pub called_ae_qualifier: Option<AEQualifier>,
    /// Called AP Invocation Identifier (optional)
    pub called_ap_invocation_identifier: Option<APInvocationIdentifier>,
    /// Called AE Invocation Identifier (optional)
    pub called_ae_invocation_identifier: Option<AEInvocationIdentifier>,
    /// Calling AP Title (optional)
    pub calling_ap_title: Option<APTitle>,
    /// Calling AE Qualifier (optional)
    pub calling_ae_qualifier: Option<AEQualifier>,
    /// Calling AP Invocation Identifier (optional)
    pub calling_ap_invocation_identifier: Option<APInvocationIdentifier>,
    /// Calling AE Invocation Identifier (optional)
    pub calling_ae_invocation_identifier: Option<AEInvocationIdentifier>,
    /// Sender ACSE requirements (optional)
    pub sender_acse_requirements: Option<ACSERequirements>,
    /// Mechanism name (optional)
    pub mechanism_name: Option<MechanismName>,
    /// Calling authentication value (optional)
    pub calling_authentication_value: Option<AuthenticationValue>,
    /// Application context name list (optional)
    pub application_context_name_list: Option<ApplicationContextNameList>,
    /// Implementation information (optional)
    pub implementation_information: Option<ImplementationData>,
    /// User information (optional) - typically contains InitiateRequest PDU
    pub user_information: Option<AssociationInformation>,
}

impl AARQApdu {
    /// Create a new AARQ PDU with minimal required fields
    ///
    /// # Arguments
    /// * `application_context_name` - Application context OID (required)
    ///
    /// # Returns
    /// Returns `Ok(AARQApdu)` if valid, `Err` otherwise
    ///
    /// # Why This Constructor?
    /// Most DLMS/COSEM implementations only require the application context name.
    /// This constructor provides a convenient way to create a minimal AARQ.
    pub fn new(application_context_name: Vec<u32>) -> Self {
        Self {
            protocol_version: None,
            application_context_name,
            called_ap_title: None,
            called_ae_qualifier: None,
            called_ap_invocation_identifier: None,
            called_ae_invocation_identifier: None,
            calling_ap_title: None,
            calling_ae_qualifier: None,
            calling_ap_invocation_identifier: None,
            calling_ae_invocation_identifier: None,
            sender_acse_requirements: None,
            mechanism_name: None,
            calling_authentication_value: None,
            application_context_name_list: None,
            implementation_information: None,
            user_information: None,
        }
    }

    /// Encode AARQ to BER format
    ///
    /// # Encoding Process
    /// 1. Encode application tag (Application, Constructed, 0)
    /// 2. Encode length of all fields
    /// 3. Encode fields in reverse order (last field first)
    /// 4. Each optional field is encoded with its context-specific tag
    ///
    /// # Why Reverse Order?
    /// BER encoding uses reverse order for efficiency in stream processing.
    /// This allows the decoder to process fields as they arrive.
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        let mut field_encoder = BerEncoder::new();

        // Encode fields in reverse order (as per BER convention)
        // Tag 30: userInformation (optional, constructed)
        if let Some(ref user_info) = self.user_information {
            let user_info_bytes = user_info.encode()?;
            field_encoder.encode_context_specific(30, &user_info_bytes, true)?;
        }

        // Tag 29: implementationInformation (optional, primitive)
        if let Some(ref impl_info) = self.implementation_information {
            let impl_info_bytes = impl_info.encode()?;
            field_encoder.encode_context_specific(29, &impl_info_bytes, false)?;
        }

        // Tag 13: applicationContextNameList (optional, constructed)
        // TODO: Implement when ApplicationContextNameList encoding is complete
        // if let Some(ref name_list) = self.application_context_name_list {
        //     let name_list_bytes = name_list.encode()?;
        //     field_encoder.encode_context_specific(13, &name_list_bytes, true)?;
        // }

        // Tag 12: callingAuthenticationValue (optional, constructed)
        if let Some(ref auth_value) = self.calling_authentication_value {
            let auth_bytes = auth_value.encode()?;
            field_encoder.encode_context_specific(12, &auth_bytes, true)?;
        }

        // Tag 11: mechanismName (optional, primitive)
        if let Some(ref mechanism) = self.mechanism_name {
            let mechanism_bytes = mechanism.encode()?;
            field_encoder.encode_context_specific(11, &mechanism_bytes, false)?;
        }

        // Tag 10: senderAcseRequirements (optional, primitive)
        if let Some(ref requirements) = self.sender_acse_requirements {
            let req_bytes = requirements.encode()?;
            field_encoder.encode_context_specific(10, &req_bytes, false)?;
        }

        // Tag 9: callingAEInvocationIdentifier (optional, constructed)
        if let Some(ref id) = self.calling_ae_invocation_identifier {
            let id_bytes = id.encode()?;
            field_encoder.encode_context_specific(9, &id_bytes, true)?;
        }

        // Tag 8: callingAPInvocationIdentifier (optional, constructed)
        if let Some(ref id) = self.calling_ap_invocation_identifier {
            let id_bytes = id.encode()?;
            field_encoder.encode_context_specific(8, &id_bytes, true)?;
        }

        // Tag 7: callingAEQualifier (optional, constructed)
        if let Some(ref qualifier) = self.calling_ae_qualifier {
            let qual_bytes = qualifier.encode()?;
            field_encoder.encode_context_specific(7, &qual_bytes, true)?;
        }

        // Tag 6: callingAPTitle (optional, constructed)
        if let Some(ref title) = self.calling_ap_title {
            let title_bytes = title.encode()?;
            field_encoder.encode_context_specific(6, &title_bytes, true)?;
        }

        // Tag 5: calledAEInvocationIdentifier (optional, constructed)
        if let Some(ref id) = self.called_ae_invocation_identifier {
            let id_bytes = id.encode()?;
            field_encoder.encode_context_specific(5, &id_bytes, true)?;
        }

        // Tag 4: calledAPInvocationIdentifier (optional, constructed)
        if let Some(ref id) = self.called_ap_invocation_identifier {
            let id_bytes = id.encode()?;
            field_encoder.encode_context_specific(4, &id_bytes, true)?;
        }

        // Tag 3: calledAEQualifier (optional, constructed)
        if let Some(ref qualifier) = self.called_ae_qualifier {
            let qual_bytes = qualifier.encode()?;
            field_encoder.encode_context_specific(3, &qual_bytes, true)?;
        }

        // Tag 2: calledAPTitle (optional, constructed)
        if let Some(ref title) = self.called_ap_title {
            let title_bytes = title.encode()?;
            field_encoder.encode_context_specific(2, &title_bytes, true)?;
        }

        // Tag 1: applicationContextName (required, constructed)
        let mut app_ctx_encoder = BerEncoder::new();
        app_ctx_encoder.encode_object_identifier(&self.application_context_name)?;
        let app_ctx_bytes = app_ctx_encoder.into_bytes();
        field_encoder.encode_context_specific(1, &app_ctx_bytes, true)?;

        // Tag 0: protocolVersion (optional, primitive)
        if let Some(ref protocol_ver) = self.protocol_version {
            // Protocol version is a BIT STRING
            let mut bit_str_encoder = BerEncoder::new();
            let num_bits = protocol_ver.len() * 8;
            bit_str_encoder.encode_bit_string(protocol_ver, num_bits, 0)?;
            let bit_str_bytes = bit_str_encoder.into_bytes();
            field_encoder.encode_context_specific(0, &bit_str_bytes, false)?;
        }

        // Encode as SEQUENCE (constructed)
        let field_bytes = field_encoder.into_bytes();
        encoder.encode_sequence(&field_bytes)?;

        // Encode application tag (Application, Constructed, 0)
        let sequence_bytes = encoder.into_bytes();
        let mut final_encoder = BerEncoder::new();
        let tag = BerTag::application(true, 0); // AARQ tag
        final_encoder.encode_tlv(&tag, &sequence_bytes)?;

        Ok(final_encoder.into_bytes())
    }

    /// Decode AARQ from BER format
    ///
    /// # Decoding Process
    /// 1. Decode application tag (must be Application, Constructed, 0)
    /// 2. Decode length
    /// 3. Decode fields in forward order (as they appear in the stream)
    /// 4. Identify optional fields by their context-specific tags
    ///
    /// # Error Handling
    /// Returns error if:
    /// - Tag is not AARQ tag
    /// - Required field (applicationContextName) is missing
    /// - Invalid encoding format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);

        // Decode application tag
        let (tag, value, _) = decoder.decode_tlv()?;
        if tag.class() != BerTagClass::Application
            || !tag.is_constructed()
            || tag.number() != 0
        {
            return Err(DlmsError::InvalidData(format!(
                "Expected AARQ tag (Application, Constructed, 0), got {:?}",
                tag
            )));
        }

        // Decode SEQUENCE
        let mut seq_decoder = BerDecoder::new(value);
        let mut aarq = AARQApdu {
            protocol_version: None,
            application_context_name: Vec::new(), // Will be set below
            called_ap_title: None,
            called_ae_qualifier: None,
            called_ap_invocation_identifier: None,
            called_ae_invocation_identifier: None,
            calling_ap_title: None,
            calling_ae_qualifier: None,
            calling_ap_invocation_identifier: None,
            calling_ae_invocation_identifier: None,
            sender_acse_requirements: None,
            mechanism_name: None,
            calling_authentication_value: None,
            application_context_name_list: None,
            implementation_information: None,
            user_information: None,
        };

        // Decode fields (in forward order)
        while seq_decoder.has_remaining() {
            let (field_tag, field_value, _) = seq_decoder.decode_tlv()?;

            if field_tag.class() != BerTagClass::ContextSpecific {
                continue; // Skip non-context-specific tags
            }

            let tag_num = field_tag.number();

            match tag_num {
                0 => {
                    // protocolVersion (BIT STRING)
                    let mut bit_str_decoder = BerDecoder::new(field_value);
                    let (bits, _num_bits, _unused) = bit_str_decoder.decode_bit_string()?;
                    aarq.protocol_version = Some(bits);
                }
                1 => {
                    // applicationContextName (OBJECT IDENTIFIER) - required
                    let mut oid_decoder = BerDecoder::new(field_value);
                    aarq.application_context_name = oid_decoder.decode_object_identifier()?;
                }
                2 => {
                    // calledAPTitle
                    aarq.called_ap_title = Some(APTitle::decode(field_value)?);
                }
                3 => {
                    // calledAEQualifier
                    aarq.called_ae_qualifier = Some(AEQualifier::decode(field_value)?);
                }
                4 => {
                    // calledAPInvocationIdentifier
                    aarq.called_ap_invocation_identifier =
                        Some(APInvocationIdentifier::decode(field_value)?);
                }
                5 => {
                    // calledAEInvocationIdentifier
                    aarq.called_ae_invocation_identifier =
                        Some(AEInvocationIdentifier::decode(field_value)?);
                }
                6 => {
                    // callingAPTitle
                    aarq.calling_ap_title = Some(APTitle::decode(field_value)?);
                }
                7 => {
                    // callingAEQualifier
                    aarq.calling_ae_qualifier = Some(AEQualifier::decode(field_value)?);
                }
                8 => {
                    // callingAPInvocationIdentifier
                    aarq.calling_ap_invocation_identifier =
                        Some(APInvocationIdentifier::decode(field_value)?);
                }
                9 => {
                    // callingAEInvocationIdentifier
                    aarq.calling_ae_invocation_identifier =
                        Some(AEInvocationIdentifier::decode(field_value)?);
                }
                10 => {
                    // senderAcseRequirements
                    aarq.sender_acse_requirements = Some(ACSERequirements::decode(field_value)?);
                }
                11 => {
                    // mechanismName
                    aarq.mechanism_name = Some(MechanismName::decode(field_value)?);
                }
                12 => {
                    // callingAuthenticationValue
                    aarq.calling_authentication_value = Some(AuthenticationValue::decode(field_value)?);
                }
                13 => {
                    // applicationContextNameList
                    // TODO: Implement when ApplicationContextNameList decoding is complete
                    // aarq.application_context_name_list = Some(ApplicationContextNameList::decode(field_value)?);
                }
                29 => {
                    // implementationInformation
                    aarq.implementation_information = Some(ImplementationData::decode(field_value)?);
                }
                30 => {
                    // userInformation
                    aarq.user_information = Some(AssociationInformation::decode(field_value)?);
                }
                _ => {
                    // Unknown tag - skip (for forward compatibility)
                    continue;
                }
            }
        }

        // Validate required field
        if aarq.application_context_name.is_empty() {
            return Err(DlmsError::InvalidData(
                "AARQ must contain applicationContextName".to_string(),
            ));
        }

        Ok(aarq)
    }
}

/// AARE (Association Response) PDU
///
/// This PDU is sent by the server in response to an AARQ.
/// It indicates whether the association was accepted and provides server information.
///
/// # Structure
///
/// AARE contains similar fields to AARQ, plus:
/// - Tag 2: result (required AssociateResult)
/// - Tag 3: resultSourceDiagnostic (required AssociateSourceDiagnostic)
/// - Tag 4-30: Similar optional fields as AARQ (with "responding" prefix)
///
/// # Required Fields
/// - applicationContextName (tag 1)
/// - result (tag 2)
/// - resultSourceDiagnostic (tag 3)
#[derive(Debug, Clone, PartialEq)]
pub struct AAREApdu {
    /// Protocol version (optional)
    pub protocol_version: Option<Vec<u8>>, // BIT STRING
    /// Application context name (required)
    pub application_context_name: Vec<u32>, // OBJECT IDENTIFIER
    /// Association result (required)
    pub result: AssociateResult,
    /// Result source diagnostic (required)
    pub result_source_diagnostic: AssociateSourceDiagnostic,
    /// Responding AP Title (optional)
    pub responding_ap_title: Option<APTitle>,
    /// Responding AE Qualifier (optional)
    pub responding_ae_qualifier: Option<AEQualifier>,
    /// Responding AP Invocation Identifier (optional)
    pub responding_ap_invocation_identifier: Option<APInvocationIdentifier>,
    /// Responding AE Invocation Identifier (optional)
    pub responding_ae_invocation_identifier: Option<AEInvocationIdentifier>,
    /// Responder ACSE requirements (optional)
    pub responder_acse_requirements: Option<ACSERequirements>,
    /// Mechanism name (optional)
    pub mechanism_name: Option<MechanismName>,
    /// Responding authentication value (optional)
    pub responding_authentication_value: Option<AuthenticationValue>,
    /// Application context name list (optional)
    pub application_context_name_list: Option<ApplicationContextNameList>,
    /// Implementation information (optional)
    pub implementation_information: Option<ImplementationData>,
    /// User information (optional) - typically contains InitiateResponse PDU
    pub user_information: Option<AssociationInformation>,
}

impl AAREApdu {
    /// Create a new AARE PDU with required fields
    ///
    /// # Arguments
    /// * `application_context_name` - Application context OID
    /// * `result` - Association result
    /// * `result_source_diagnostic` - Result source diagnostic
    pub fn new(
        application_context_name: Vec<u32>,
        result: AssociateResult,
        result_source_diagnostic: AssociateSourceDiagnostic,
    ) -> Self {
        Self {
            protocol_version: None,
            application_context_name,
            result,
            result_source_diagnostic,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_identifier: None,
            responding_ae_invocation_identifier: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            application_context_name_list: None,
            implementation_information: None,
            user_information: None,
        }
    }

    /// Encode AARE to BER format
    ///
    /// Encoding follows the same pattern as AARQ, with additional required fields.
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        let mut field_encoder = BerEncoder::new();

        // Encode fields in reverse order
        // Tag 30: userInformation
        if let Some(ref user_info) = self.user_information {
            let user_info_bytes = user_info.encode()?;
            field_encoder.encode_context_specific(30, &user_info_bytes, true)?;
        }

        // Tag 29: implementationInformation
        if let Some(ref impl_info) = self.implementation_information {
            let impl_info_bytes = impl_info.encode()?;
            field_encoder.encode_context_specific(29, &impl_info_bytes, false)?;
        }

        // Tag 11: applicationContextNameList
        // TODO: Implement when ApplicationContextNameList encoding is complete

        // Tag 10: respondingAuthenticationValue
        if let Some(ref auth_value) = self.responding_authentication_value {
            let auth_bytes = auth_value.encode()?;
            field_encoder.encode_context_specific(10, &auth_bytes, true)?;
        }

        // Tag 9: mechanismName
        if let Some(ref mechanism) = self.mechanism_name {
            let mechanism_bytes = mechanism.encode()?;
            field_encoder.encode_context_specific(9, &mechanism_bytes, false)?;
        }

        // Tag 8: responderAcseRequirements
        if let Some(ref requirements) = self.responder_acse_requirements {
            let req_bytes = requirements.encode()?;
            field_encoder.encode_context_specific(8, &req_bytes, false)?;
        }

        // Tag 7: respondingAEInvocationIdentifier
        if let Some(ref id) = self.responding_ae_invocation_identifier {
            let id_bytes = id.encode()?;
            field_encoder.encode_context_specific(7, &id_bytes, true)?;
        }

        // Tag 6: respondingAPInvocationIdentifier
        if let Some(ref id) = self.responding_ap_invocation_identifier {
            let id_bytes = id.encode()?;
            field_encoder.encode_context_specific(6, &id_bytes, true)?;
        }

        // Tag 5: respondingAEQualifier
        if let Some(ref qualifier) = self.responding_ae_qualifier {
            let qual_bytes = qualifier.encode()?;
            field_encoder.encode_context_specific(5, &qual_bytes, true)?;
        }

        // Tag 4: respondingAPTitle
        if let Some(ref title) = self.responding_ap_title {
            let title_bytes = title.encode()?;
            field_encoder.encode_context_specific(4, &title_bytes, true)?;
        }

        // Tag 3: resultSourceDiagnostic (required)
        let diag_bytes = self.result_source_diagnostic.encode()?;
        field_encoder.encode_context_specific(3, &diag_bytes, true)?;

        // Tag 2: result (required)
        let result_bytes = self.result.encode()?;
        field_encoder.encode_context_specific(2, &result_bytes, true)?;

        // Tag 1: applicationContextName (required)
        let mut app_ctx_encoder = BerEncoder::new();
        app_ctx_encoder.encode_object_identifier(&self.application_context_name)?;
        let app_ctx_bytes = app_ctx_encoder.into_bytes();
        field_encoder.encode_context_specific(1, &app_ctx_bytes, true)?;

        // Tag 0: protocolVersion (optional)
        if let Some(ref protocol_ver) = self.protocol_version {
            let mut bit_str_encoder = BerEncoder::new();
            let num_bits = protocol_ver.len() * 8;
            bit_str_encoder.encode_bit_string(protocol_ver, num_bits, 0)?;
            let bit_str_bytes = bit_str_encoder.into_bytes();
            field_encoder.encode_context_specific(0, &bit_str_bytes, false)?;
        }

        // Encode as SEQUENCE
        let field_bytes = field_encoder.into_bytes();
        encoder.encode_sequence(&field_bytes)?;

        // Encode application tag (Application, Constructed, 1)
        let sequence_bytes = encoder.into_bytes();
        let mut final_encoder = BerEncoder::new();
        let tag = BerTag::application(true, 1); // AARE tag
        final_encoder.encode_tlv(&tag, &sequence_bytes)?;

        Ok(final_encoder.into_bytes())
    }

    /// Decode AARE from BER format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);

        // Decode application tag
        let (tag, value, _) = decoder.decode_tlv()?;
        if tag.class() != BerTagClass::Application
            || !tag.is_constructed()
            || tag.number() != 1
        {
            return Err(DlmsError::InvalidData(format!(
                "Expected AARE tag (Application, Constructed, 1), got {:?}",
                tag
            )));
        }

        // Decode SEQUENCE
        let mut seq_decoder = BerDecoder::new(value);
        let mut aare = AAREApdu {
            protocol_version: None,
            application_context_name: Vec::new(),
            result: AssociateResult::Accepted, // Will be set below
            result_source_diagnostic: AssociateSourceDiagnostic::new(0), // Will be set below
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_identifier: None,
            responding_ae_invocation_identifier: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            application_context_name_list: None,
            implementation_information: None,
            user_information: None,
        };

        // Decode fields
        while seq_decoder.has_remaining() {
            let (field_tag, field_value, _) = seq_decoder.decode_tlv()?;

            if field_tag.class() != BerTagClass::ContextSpecific {
                continue;
            }

            let tag_num = field_tag.number();

            match tag_num {
                0 => {
                    let mut bit_str_decoder = BerDecoder::new(field_value);
                    let (bits, _num_bits, _unused) = bit_str_decoder.decode_bit_string()?;
                    aare.protocol_version = Some(bits);
                }
                1 => {
                    let mut oid_decoder = BerDecoder::new(field_value);
                    aare.application_context_name = oid_decoder.decode_object_identifier()?;
                }
                2 => {
                    aare.result = AssociateResult::decode(field_value)?;
                }
                3 => {
                    aare.result_source_diagnostic = AssociateSourceDiagnostic::decode(field_value)?;
                }
                4 => {
                    aare.responding_ap_title = Some(APTitle::decode(field_value)?);
                }
                5 => {
                    aare.responding_ae_qualifier = Some(AEQualifier::decode(field_value)?);
                }
                6 => {
                    aare.responding_ap_invocation_identifier =
                        Some(APInvocationIdentifier::decode(field_value)?);
                }
                7 => {
                    aare.responding_ae_invocation_identifier =
                        Some(AEInvocationIdentifier::decode(field_value)?);
                }
                8 => {
                    aare.responder_acse_requirements = Some(ACSERequirements::decode(field_value)?);
                }
                9 => {
                    aare.mechanism_name = Some(MechanismName::decode(field_value)?);
                }
                10 => {
                    aare.responding_authentication_value = Some(AuthenticationValue::decode(field_value)?);
                }
                11 => {
                    // applicationContextNameList
                    // TODO: Implement when ApplicationContextNameList decoding is complete
                }
                29 => {
                    aare.implementation_information = Some(ImplementationData::decode(field_value)?);
                }
                30 => {
                    aare.user_information = Some(AssociationInformation::decode(field_value)?);
                }
                _ => {
                    continue;
                }
            }
        }

        // Validate required fields
        if aare.application_context_name.is_empty() {
            return Err(DlmsError::InvalidData(
                "AARE must contain applicationContextName".to_string(),
            ));
        }

        Ok(aare)
    }
}

/// RLRQ (Release Request) PDU
///
/// This PDU is sent to request release of an association.
///
/// # Structure
/// - Tag 0: reason (optional ReleaseRequestReason)
/// - Tag 30: userInformation (optional AssociationInformation)
#[derive(Debug, Clone, PartialEq)]
pub struct RLRQApdu {
    /// Release reason (optional)
    pub reason: Option<ReleaseRequestReason>,
    /// User information (optional)
    pub user_information: Option<AssociationInformation>,
}

impl RLRQApdu {
    /// Create a new RLRQ PDU
    pub fn new() -> Self {
        Self {
            reason: None,
            user_information: None,
        }
    }

    /// Encode RLRQ to BER format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        let mut field_encoder = BerEncoder::new();

        // Encode fields in reverse order
        // Tag 30: userInformation
        if let Some(ref user_info) = self.user_information {
            let user_info_bytes = user_info.encode()?;
            field_encoder.encode_context_specific(30, &user_info_bytes, true)?;
        }

        // Tag 0: reason
        if let Some(ref reason) = self.reason {
            let reason_bytes = reason.encode()?;
            field_encoder.encode_context_specific(0, &reason_bytes, false)?;
        }

        // Encode as SEQUENCE
        let field_bytes = field_encoder.into_bytes();
        encoder.encode_sequence(&field_bytes)?;

        // Encode application tag (Application, Constructed, 2)
        let sequence_bytes = encoder.into_bytes();
        let mut final_encoder = BerEncoder::new();
        let tag = BerTag::application(true, 2); // RLRQ tag
        final_encoder.encode_tlv(&tag, &sequence_bytes)?;

        Ok(final_encoder.into_bytes())
    }

    /// Decode RLRQ from BER format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);

        // Decode application tag
        let (tag, value, _) = decoder.decode_tlv()?;
        if tag.class() != BerTagClass::Application
            || !tag.is_constructed()
            || tag.number() != 2
        {
            return Err(DlmsError::InvalidData(format!(
                "Expected RLRQ tag (Application, Constructed, 2), got {:?}",
                tag
            )));
        }

        // Decode SEQUENCE
        let mut seq_decoder = BerDecoder::new(value);
        let mut rlrq = RLRQApdu::new();

        while seq_decoder.has_remaining() {
            let (field_tag, field_value, _) = seq_decoder.decode_tlv()?;

            if field_tag.class() != BerTagClass::ContextSpecific {
                continue;
            }

            match field_tag.number() {
                0 => {
                    rlrq.reason = Some(ReleaseRequestReason::decode(field_value)?);
                }
                30 => {
                    rlrq.user_information = Some(AssociationInformation::decode(field_value)?);
                }
                _ => {
                    continue;
                }
            }
        }

        Ok(rlrq)
    }
}

impl Default for RLRQApdu {
    fn default() -> Self {
        Self::new()
    }
}

/// RLRE (Release Response) PDU
///
/// This PDU is sent in response to an RLRQ.
///
/// # Structure
/// - Tag 0: reason (optional ReleaseResponseReason)
/// - Tag 30: userInformation (optional AssociationInformation)
#[derive(Debug, Clone, PartialEq)]
pub struct RLREApdu {
    /// Release reason (optional)
    pub reason: Option<ReleaseResponseReason>,
    /// User information (optional)
    pub user_information: Option<AssociationInformation>,
}

impl RLREApdu {
    /// Create a new RLRE PDU
    pub fn new() -> Self {
        Self {
            reason: None,
            user_information: None,
        }
    }

    /// Encode RLRE to BER format
    pub fn encode(&self) -> DlmsResult<Vec<u8>> {
        let mut encoder = BerEncoder::new();
        let mut field_encoder = BerEncoder::new();

        // Encode fields in reverse order
        // Tag 30: userInformation
        if let Some(ref user_info) = self.user_information {
            let user_info_bytes = user_info.encode()?;
            field_encoder.encode_context_specific(30, &user_info_bytes, true)?;
        }

        // Tag 0: reason
        if let Some(ref reason) = self.reason {
            let reason_bytes = reason.encode()?;
            field_encoder.encode_context_specific(0, &reason_bytes, false)?;
        }

        // Encode as SEQUENCE
        let field_bytes = field_encoder.into_bytes();
        encoder.encode_sequence(&field_bytes)?;

        // Encode application tag (Application, Constructed, 3)
        let sequence_bytes = encoder.into_bytes();
        let mut final_encoder = BerEncoder::new();
        let tag = BerTag::application(true, 3); // RLRE tag
        final_encoder.encode_tlv(&tag, &sequence_bytes)?;

        Ok(final_encoder.into_bytes())
    }

    /// Decode RLRE from BER format
    pub fn decode(data: &[u8]) -> DlmsResult<Self> {
        let mut decoder = BerDecoder::new(data);

        // Decode application tag
        let (tag, value, _) = decoder.decode_tlv()?;
        if tag.class() != BerTagClass::Application
            || !tag.is_constructed()
            || tag.number() != 3
        {
            return Err(DlmsError::InvalidData(format!(
                "Expected RLRE tag (Application, Constructed, 3), got {:?}",
                tag
            )));
        }

        // Decode SEQUENCE
        let mut seq_decoder = BerDecoder::new(value);
        let mut rlre = RLREApdu::new();

        while seq_decoder.has_remaining() {
            let (field_tag, field_value, _) = seq_decoder.decode_tlv()?;

            if field_tag.class() != BerTagClass::ContextSpecific {
                continue;
            }

            match field_tag.number() {
                0 => {
                    rlre.reason = Some(ReleaseResponseReason::decode(field_value)?);
                }
                30 => {
                    rlre.user_information = Some(AssociationInformation::decode(field_value)?);
                }
                _ => {
                    continue;
                }
            }
        }

        Ok(rlre)
    }
}

impl Default for RLREApdu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aarq_encode_decode() {
        // Create minimal AARQ
        let aarq = AARQApdu::new(vec![1, 0, 17, 0, 0, 128, 0, 1]); // DLMS application context

        let encoded = aarq.encode().unwrap();
        let decoded = AARQApdu::decode(&encoded).unwrap();

        assert_eq!(aarq.application_context_name, decoded.application_context_name);
    }

    #[test]
    fn test_aare_encode_decode() {
        // Create minimal AARE
        let aare = AAREApdu::new(
            vec![1, 0, 17, 0, 0, 128, 0, 1], // DLMS application context
            AssociateResult::Accepted,
            AssociateSourceDiagnostic::new(0),
        );

        let encoded = aare.encode().unwrap();
        let decoded = AAREApdu::decode(&encoded).unwrap();

        assert_eq!(aare.application_context_name, decoded.application_context_name);
        assert_eq!(aare.result, decoded.result);
    }

    #[test]
    fn test_rlrq_encode_decode() {
        let rlrq = RLRQApdu::new();
        let encoded = rlrq.encode().unwrap();
        let decoded = RLRQApdu::decode(&encoded).unwrap();
        assert_eq!(rlrq, decoded);
    }

    #[test]
    fn test_rlre_encode_decode() {
        let rlre = RLREApdu::new();
        let encoded = rlre.encode().unwrap();
        let decoded = RLREApdu::decode(&encoded).unwrap();
        assert_eq!(rlre, decoded);
    }
}
