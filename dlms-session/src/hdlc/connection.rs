//! HDLC connection implementation

use crate::error::{DlmsError, DlmsResult};
use crate::hdlc::address::{HdlcAddress, HdlcAddressPair};
use crate::hdlc::decoder::HdlcMessageDecoder;
use crate::hdlc::dispatcher::HdlcDispatcher;
use crate::hdlc::frame::{FrameType, HdlcFrame, FLAG, LLC_REQUEST, LLC_RESPONSE};
use crate::hdlc::statistics::HdlcStatistics;
use crate::hdlc::window::{SendWindow, ReceiveWindow};
use crate::hdlc::state::HdlcConnectionState;
use dlms_transport::{StreamAccessor, TransportLayer};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// HDLC connection parameters
///
/// These parameters are negotiated during the SNRM/UA handshake and define
/// the capabilities of the HDLC connection.
///
/// # Why These Parameters?
/// - **Window Size**: Controls how many frames can be sent before waiting for acknowledgment.
///   This affects throughput and latency. Default is 1 for reliability.
/// - **Maximum Information Field Length**: Limits the size of data that can be sent in a single frame.
///   This prevents buffer overflows and ensures compatibility. Default is 128 bytes.
///
/// # Optimization Considerations
/// - Larger window sizes improve throughput but require more buffer space
/// - Larger information field lengths reduce overhead but increase memory requirements
/// - These parameters should be negotiated based on device capabilities and network conditions
#[derive(Debug, Clone)]
pub struct HdlcParameters {
    pub max_information_field_length_tx: u16,
    pub max_information_field_length_rx: u16,
    pub window_size_tx: u8,
    pub window_size_rx: u8,
}

/// UA frame parameters
///
/// Parameters contained in the UA (Unnumbered Acknowledge) frame information field
/// during HDLC connection establishment (SNRM/UA handshake).
///
/// According to dlms-docs/dlms/长数据帧处理.txt, UA帧的信息域包含以下链路参数：
/// - Window_size: 通讯的双方一次发送数据帧的数目（默认值为1）
/// - Maximum_information_field_length: 链路数据帧中用户数据的最大长度（默认值为128）
///
/// # Format
/// The UA frame information field format follows DLMS standard (IEC 62056-47):
/// - Format Identifier: 1 byte (0x81)
/// - Group Identifier: 1 byte (0x80)
/// - Parameter values: variable length
///
/// # Why This Structure?
/// Separating UA frame parameters from HdlcParameters allows us to:
/// - Parse parameters from the UA frame
/// - Validate parameters before applying them
/// - Handle optional or extended parameters in the future
#[derive(Debug, Clone, PartialEq)]
pub struct UaFrameParameters {
    /// Window size for receive (server -> client direction)
    pub window_size_rx: u8,
    /// Maximum information field length for receive (server -> client direction)
    pub max_information_field_length_rx: u16,
    /// Window size for transmit (client -> server direction)
    pub window_size_tx: u8,
    /// Maximum information field length for transmit (client -> server direction)
    pub max_information_field_length_tx: u16,
}

impl UaFrameParameters {

    /// Decode UA frame parameters from information field
    ///
    /// # Format
    /// According to DLMS standard (IEC 62056-47), UA frame information field format:
    /// ```
    /// Format Identifier (1 byte): 0x81
    /// Group Identifier (1 byte): 0x80
    /// Parameter 1 (Window Size RX): 1 byte
    /// Parameter 2 (Max Info Field Length RX): 2 bytes (big-endian)
    /// Parameter 3 (Window Size TX): 1 byte
    /// Parameter 4 (Max Info Field Length TX): 2 bytes (big-endian)
    /// ```
    ///
    /// # Error Handling
    /// - Returns `DlmsError::InvalidData` if the information field is too short
    /// - Returns `DlmsError::InvalidData` if format/group identifiers are invalid
    ///
    /// # Optimization Considerations
    /// - Minimal validation to ensure fast parsing
    /// - Could be extended to support optional parameters in the future
    pub fn decode(info_field: &[u8]) -> DlmsResult<Self> {
        // Minimum size: Format ID(1) + Group ID(1) + Window RX(1) + Max Len RX(2) + Window TX(1) + Max Len TX(2) = 8 bytes
        if info_field.len() < 8 {
            return Err(DlmsError::InvalidData(format!(
                "UA frame information field too short: expected at least 8 bytes, got {}",
                info_field.len()
            )));
        }

        let mut pos = 0;

        // Format Identifier (should be 0x81)
        let format_id = info_field[pos];
        if format_id != 0x81 {
            return Err(DlmsError::InvalidData(format!(
                "Invalid UA frame format identifier: expected 0x81, got 0x{:02X}",
                format_id
            )));
        }
        pos += 1;

        // Group Identifier (should be 0x80)
        let group_id = info_field[pos];
        if group_id != 0x80 {
            return Err(DlmsError::InvalidData(format!(
                "Invalid UA frame group identifier: expected 0x80, got 0x{:02X}",
                group_id
            )));
        }
        pos += 1;

        // Window Size RX
        let window_size_rx = info_field[pos];
        pos += 1;

        // Maximum Information Field Length RX (big-endian, 2 bytes)
        if pos + 2 > info_field.len() {
            return Err(DlmsError::InvalidData("UA frame information field too short for max length RX".to_string()));
        }
        let max_info_field_length_rx = u16::from_be_bytes([info_field[pos], info_field[pos + 1]]);
        pos += 2;

        // Window Size TX
        if pos >= info_field.len() {
            return Err(DlmsError::InvalidData("UA frame information field too short for window size TX".to_string()));
        }
        let window_size_tx = info_field[pos];
        pos += 1;

        // Maximum Information Field Length TX (big-endian, 2 bytes)
        if pos + 2 > info_field.len() {
            return Err(DlmsError::InvalidData("UA frame information field too short for max length TX".to_string()));
        }
        let max_info_field_length_tx = u16::from_be_bytes([info_field[pos], info_field[pos + 1]]);

        // Validate parameters
        if window_size_rx == 0 || window_size_rx > 7 {
            return Err(DlmsError::InvalidData(format!(
                "Invalid window size RX: expected 1-7, got {}",
                window_size_rx
            )));
        }
        if window_size_tx == 0 || window_size_tx > 7 {
            return Err(DlmsError::InvalidData(format!(
                "Invalid window size TX: expected 1-7, got {}",
                window_size_tx
            )));
        }
        if max_info_field_length_rx == 0 {
            return Err(DlmsError::InvalidData("Invalid max information field length RX: must be > 0".to_string()));
        }
        if max_info_field_length_tx == 0 {
            return Err(DlmsError::InvalidData("Invalid max information field length TX: must be > 0".to_string()));
        }

        Ok(Self {
            window_size_rx,
            max_information_field_length_rx,
            window_size_tx,
            max_information_field_length_tx,
        })
    }

    /// Encode UA frame parameters to information field
    ///
    /// # Format
    /// See `decode()` for the format specification.
    ///
    /// # Why This Method?
    /// This method is useful for testing and for implementing server-side UA frame generation.
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(8);
        
        // Format Identifier
        result.push(0x81);
        
        // Group Identifier
        result.push(0x80);
        
        // Window Size RX
        result.push(self.window_size_rx);
        
        // Maximum Information Field Length RX (big-endian)
        result.extend_from_slice(&self.max_information_field_length_rx.to_be_bytes());
        
        // Window Size TX
        result.push(self.window_size_tx);
        
        // Maximum Information Field Length TX (big-endian)
        result.extend_from_slice(&self.max_information_field_length_tx.to_be_bytes());
        
        result
    }
}

impl Default for UaFrameParameters {
    fn default() -> Self {
        Self {
            window_size_rx: 1,
            max_information_field_length_rx: 128,
            window_size_tx: 1,
            max_information_field_length_tx: 128,
        }
    }
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

/// Segmented frame reassembler
///
/// Handles automatic reassembly of segmented HDLC frames according to
/// dlms-docs/dlms/长数据帧处理.txt:
/// - Detects segmented frames (S bit = 1)
/// - Automatically sends RR frames to request next segment
/// - Reassembles segmented data into complete message
///
/// # Why This Design?
/// Separating the reassembly logic from the connection allows:
/// - Clear separation of concerns
/// - Easier testing and debugging
/// - Future enhancements (e.g., multiple concurrent segmented messages)
///
/// # State Management
/// The reassembler maintains state for a single segmented message:
/// - Current reassembled data
/// - Expected next frame sequence number
/// - Timeout tracking
///
/// # Optimization Considerations
/// - Buffer size is limited to prevent memory exhaustion
/// - Timeout prevents indefinite waiting for missing segments
/// - State is cleared after completion or error
#[derive(Debug)]
struct SegmentedFrameReassembler {
    /// Current reassembled message data
    current_message: Vec<u8>,
    /// Expected next frame sequence number (0-7)
    expected_sequence: u8,
    /// Timeout for receiving next segment
    timeout: Duration,
    /// Last receive time for timeout tracking
    last_receive_time: Option<Instant>,
    /// Maximum buffer size to prevent memory exhaustion
    max_buffer_size: usize,
}

impl SegmentedFrameReassembler {
    /// Create a new segmented frame reassembler
    ///
    /// # Default Values
    /// - Timeout: 5 seconds (reasonable for most networks)
    /// - Max buffer size: 64KB (prevents memory exhaustion)
    pub fn new() -> Self {
        Self {
            current_message: Vec::new(),
            expected_sequence: 0,
            timeout: Duration::from_secs(5),
            last_receive_time: None,
            max_buffer_size: 64 * 1024, // 64KB
        }
    }

    /// Start reassembling a new segmented message
    ///
    /// # Arguments
    /// * `first_segment_data` - Data from the first segmented frame
    /// * `next_sequence` - Expected sequence number for next frame
    pub fn start(&mut self, first_segment_data: Vec<u8>, next_sequence: u8) {
        self.current_message = first_segment_data;
        self.expected_sequence = next_sequence;
        self.last_receive_time = Some(Instant::now());
    }

    /// Add a segment to the current message
    ///
    /// # Arguments
    /// * `segment_data` - Data from the segment
    /// * `sequence` - Sequence number of this segment
    /// * `is_last` - Whether this is the last segment (S bit = 0)
    ///
    /// # Returns
    /// * `Ok(Some(Vec<u8>))` - Complete reassembled message if this was the last segment
    /// * `Ok(None)` - Message not yet complete
    /// * `Err` - Error (sequence mismatch, buffer overflow, etc.)
    pub fn add_segment(
        &mut self,
        segment_data: Vec<u8>,
        sequence: u8,
        is_last: bool,
    ) -> DlmsResult<Option<Vec<u8>>> {
        // Check sequence number
        if sequence != self.expected_sequence {
            return Err(DlmsError::FrameInvalid(format!(
                "Sequence number mismatch: expected {}, got {}",
                self.expected_sequence, sequence
            )));
        }

        // Check buffer size
        if self.current_message.len() + segment_data.len() > self.max_buffer_size {
            return Err(DlmsError::InvalidData(format!(
                "Segmented message too large: {} bytes (max: {})",
                self.current_message.len() + segment_data.len(),
                self.max_buffer_size
            )));
        }

        // Add segment data
        self.current_message.extend_from_slice(&segment_data);
        self.last_receive_time = Some(Instant::now());

        // Update expected sequence for next segment
        self.expected_sequence = (self.expected_sequence + 1) % 8;

        // If this is the last segment, return complete message
        if is_last {
            let complete_message = std::mem::take(&mut self.current_message);
            self.reset();
            Ok(Some(complete_message))
        } else {
            Ok(None)
        }
    }

    /// Check if timeout has been exceeded
    ///
    /// # Returns
    /// `true` if timeout has been exceeded, `false` otherwise
    pub fn is_timeout(&self) -> bool {
        if let Some(last_time) = self.last_receive_time {
            last_time.elapsed() > self.timeout
        } else {
            false
        }
    }

    /// Get expected next sequence number
    pub fn expected_sequence(&self) -> u8 {
        self.expected_sequence
    }

    /// Check if reassembler is active (has a message in progress)
    pub fn is_active(&self) -> bool {
        !self.current_message.is_empty() || self.last_receive_time.is_some()
    }

    /// Reset the reassembler (clear state)
    pub fn reset(&mut self) {
        self.current_message.clear();
        self.expected_sequence = 0;
        self.last_receive_time = None;
    }

    /// Get current message length
    pub fn current_length(&self) -> usize {
        self.current_message.len()
    }
}

impl Default for SegmentedFrameReassembler {
    fn default() -> Self {
        Self::new()
    }
}

/// HDLC connection
///
/// Manages HDLC protocol communication including:
/// - Connection establishment (SNRM/UA)
/// - Frame sending and receiving
/// - Segmented frame reassembly
/// - Connection termination (DISC/DM/UA)
pub struct HdlcConnection<T: TransportLayer> {
    transport: T,
    local_address: HdlcAddress,
    remote_address: HdlcAddress,
    dispatcher: HdlcDispatcher,
    parameters: HdlcParameters,
    send_sequence: u8,
    receive_sequence: u8,
    /// Connection state (replaces simple `closed` flag)
    state: HdlcConnectionState,
    /// Legacy closed flag (deprecated, use state instead)
    /// Kept for backward compatibility during migration
    #[deprecated(note = "Use state field instead")]
    closed: bool,
    /// Segmented frame reassembler for automatic RR frame sending
    reassembler: SegmentedFrameReassembler,
    /// Whether to use LLC header for Information frames
    /// 
    /// According to DLMS standard (IEC 62056-47), LLC header [0xE6, 0xE6, 0x00]
    /// should be prepended to Information frame data. This is enabled by default
    /// for protocol compliance, but can be disabled for compatibility with devices
    /// that don't expect LLC header.
    use_llc_header: bool,
    /// Whether this connection is acting as a client (true) or server (false)
    /// 
    /// This determines which LLC header to use when sending Information frames:
    /// - Client: Uses LLC_REQUEST [0xE6, 0xE6, 0x00] for requests
    /// - Server: Uses LLC_RESPONSE [0xE6, 0xE7, 0x00] for responses
    /// 
    /// According to DLMS standard (IEC 62056-47), the second byte of the LLC header
    /// is 0xE6 for requests (client -> server) and 0xE7 for responses (server -> client).
    is_client: bool,
    /// Statistics for monitoring connection performance and debugging
    statistics: HdlcStatistics,
    /// Send window for sliding window protocol and retransmission
    send_window: SendWindow,
    /// Receive window for sequence number tracking
    receive_window: ReceiveWindow,
    /// Retransmission timeout (default: 3 seconds)
    retransmit_timeout: Duration,
    /// Maximum retransmission attempts (default: 3)
    max_retries: u8,
}

impl<T: TransportLayer> HdlcConnection<T> {
    /// Create a new HDLC connection
    ///
    /// # Arguments
    /// * `transport` - Transport layer implementation
    /// * `local_address` - Local HDLC address
    /// * `remote_address` - Remote HDLC address
    ///
    /// # LLC Header
    /// By default, LLC header is enabled for protocol compliance. Set `use_llc_header(false)`
    /// after creation if you need to disable it for compatibility.
    /// 
    /// # Client vs Server
    /// By default, the connection is created as a client. Use `new_server()` to create
    /// a server-side connection, which will use LLC_RESPONSE header for responses.
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
            state: HdlcConnectionState::Closed,
            closed: true, // Keep in sync with state
            reassembler: SegmentedFrameReassembler::new(),
            use_llc_header: true, // Enable LLC header by default for protocol compliance
            is_client: true, // Default to client mode
            statistics: HdlcStatistics::new(),
            send_window: SendWindow::new(
                1, // Default window size (will be updated from UA frame)
                Duration::from_secs(3), // Default retransmit timeout
                3, // Default max retries
            ),
            receive_window: ReceiveWindow::new(),
            retransmit_timeout: Duration::from_secs(3),
            max_retries: 3,
        }
    }

    /// Create a new HDLC connection in server mode
    ///
    /// # Arguments
    /// * `transport` - Transport layer implementation
    /// * `local_address` - Local HDLC address (server address)
    /// * `remote_address` - Remote HDLC address (client address)
    ///
    /// # LLC Header
    /// Server connections use LLC_RESPONSE [0xE6, 0xE7, 0x00] when sending responses,
    /// according to DLMS standard (IEC 62056-47).
    pub fn new_server(
        transport: T,
        local_address: HdlcAddress,
        remote_address: HdlcAddress,
    ) -> Self {
        let mut conn = Self::new(transport, local_address, remote_address);
        conn.is_client = false; // Server mode
        conn
    }

    /// Set whether to use LLC header for Information frames
    ///
    /// # Arguments
    /// * `use_llc` - If true, LLC header [0xE6, 0xE6, 0x00] will be prepended to Information frame data
    ///
    /// # Why This Option?
    /// Some devices may not expect LLC header, so this option allows disabling it
    /// for compatibility. However, according to DLMS standard, LLC header should be used.
    pub fn set_use_llc_header(&mut self, use_llc: bool) {
        self.use_llc_header = use_llc;
    }

    /// Get whether LLC header is enabled
    pub fn use_llc_header(&self) -> bool {
        self.use_llc_header
    }

    /// Get connection statistics
    ///
    /// Returns a reference to the statistics structure for monitoring
    /// connection performance and debugging.
    pub fn statistics(&self) -> &HdlcStatistics {
        &self.statistics
    }

    /// Clear connection statistics
    ///
    /// Resets all statistics counters to zero.
    pub fn clear_statistics(&mut self) {
        self.statistics.clear();
    }

    /// Open the HDLC connection
    ///
    /// # Connection Establishment Process (per dlms-docs/dlms/cosem连接过程.txt)
    ///
    /// The connection establishment follows this sequence:
    /// ```
    /// 客户端 -> SNRM -> 服务器
    /// 客户端 <- UA <- 服务器
    /// ```
    ///
    /// # Process
    /// 1. Open the transport layer
    /// 2. Send SNRM (Set Normal Response Mode) frame
    /// 3. Wait for UA (Unnumbered Acknowledge) response with timeout
    /// 4. Parse UA frame parameters from information field
    /// 5. Update HdlcParameters with negotiated values
    ///
    /// # Why This Design?
    /// - **SNRM Frame**: Initiates the HDLC connection and requests normal response mode
    /// - **UA Frame**: Acknowledges the connection and provides negotiated parameters
    /// - **Parameter Negotiation**: Allows both sides to agree on window size and frame length limits
    ///
    /// # Error Handling
    /// - Transport layer errors: Returns `DlmsError::Connection`
    /// - SNRM send failure: Returns `DlmsError::Connection`
    /// - UA response timeout: Returns `DlmsError::Connection` with timeout error
    /// - UA frame format error: Returns `DlmsError::FrameInvalid`
    /// - Parameter validation error: Returns `DlmsError::InvalidData`
    ///
    /// # Optimization Considerations
    /// - Default timeout is 5 seconds, which should be sufficient for most networks
    /// - Parameters are validated before applying to prevent invalid configurations
    /// - Connection state is only set to open after successful UA reception
    ///
    /// # Future Enhancements
    /// - Configurable timeout duration
    /// - SNRM retry mechanism
    /// - Parameter negotiation (accept/reject based on capabilities)
    pub async fn open(&mut self) -> DlmsResult<()> {
        // Step 1: Open the transport layer
        self.transport.open().await?;

        // Step 2: Send SNRM (Set Normal Response Mode) frame
        // SNRM frame has no information field according to HDLC standard
        let address_pair = HdlcAddressPair::new(self.local_address, self.remote_address);
        let snrm_frame = HdlcFrame::new(address_pair, FrameType::SetNormalResponseMode, None);
        self.send_frame(snrm_frame).await?;

        // Step 3: Wait for UA (Unnumbered Acknowledge) response with timeout
        // Default timeout: 5 seconds (should be sufficient for most networks)
        let timeout = Duration::from_secs(5);
        let frames = self.receive_frames(Some(timeout)).await?;

        // Step 4: Find and parse UA frame
        let ua_frame = frames
            .iter()
            .find(|f| f.frame_type() == FrameType::UnnumberedAcknowledge)
            .ok_or_else(|| {
                DlmsError::Connection(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "UA frame not received within timeout period",
                ))
            })?;

        // Step 5: Parse UA frame parameters and update HdlcParameters
        // UA frame information field contains negotiated parameters
        let info_field = ua_frame.information_field();
        if !info_field.is_empty() {
            // Parse UA frame parameters
            let ua_params = UaFrameParameters::decode(info_field)?;
            
            // Update HdlcParameters with negotiated values
            // Note: RX parameters from server perspective are TX parameters from client perspective
            // and vice versa
            self.parameters.max_information_field_length_tx = ua_params.max_information_field_length_tx;
            self.parameters.max_information_field_length_rx = ua_params.max_information_field_length_rx;
            self.parameters.window_size_tx = ua_params.window_size_tx;
            self.parameters.window_size_rx = ua_params.window_size_rx;
            
            // Update send window size
            self.send_window.set_window_size(self.parameters.window_size_tx);
        } else {
            // If UA frame has no information field, use default parameters
            // This is acceptable according to HDLC standard (parameters are optional)
            // Default parameters are already set in HdlcParameters::default()
        }

        // Connection is now established
        // Use transition_to to ensure closed flag is properly synced
        self.transition_to(HdlcConnectionState::Connected)?;
        
        // Reset windows for new connection
        self.send_window.reset();
        self.receive_window.reset();
        
        Ok(())
    }

    /// Accept an HDLC connection (server-side)
    ///
    /// This method implements the server-side of the SNRM/UA handshake:
    /// 1. Wait for SNRM (Set Normal Response Mode) frame from client
    /// 2. Parse SNRM parameters (if any)
    /// 3. Generate UA (Unnumbered Acknowledge) response with server parameters
    /// 4. Send UA frame to client
    ///
    /// # Connection Establishment Process (per dlms-docs/dlms/cosem连接过程.txt)
    ///
    /// The server-side connection establishment follows this sequence:
    /// ```
    /// 客户端 -> SNRM -> 服务器
    /// 客户端 <- UA <- 服务器
    /// ```
    ///
    /// # Process
    /// 1. Open the transport layer (if not already open)
    /// 2. Wait for SNRM frame from client (with timeout)
    /// 3. Parse SNRM frame (typically has no information field)
    /// 4. Generate UA frame with server parameters
    /// 5. Send UA frame to client
    /// 6. Update connection state to Connected
    ///
    /// # Why This Design?
    /// - **SNRM Frame**: Client requests HDLC connection establishment
    /// - **UA Frame**: Server acknowledges and provides negotiated parameters
    /// - **Parameter Negotiation**: Server provides its capabilities (window size, frame length)
    ///
    /// # Error Handling
    /// - Transport layer errors: Returns `DlmsError::Connection`
    /// - SNRM receive timeout: Returns `DlmsError::Connection` with timeout error
    /// - SNRM frame format error: Returns `DlmsError::FrameInvalid`
    /// - UA send failure: Returns `DlmsError::Connection`
    ///
    /// # Optimization Considerations
    /// - Default timeout is 5 seconds, which should be sufficient for most networks
    /// - Server parameters are taken from current `HdlcParameters`
    /// - Connection state is only set to open after successful UA transmission
    ///
    /// # Future Enhancements
    /// - Configurable timeout duration
    /// - SNRM parameter parsing (if client sends parameters)
    /// - Parameter negotiation (accept/reject based on server capabilities)
    pub async fn accept(&mut self) -> DlmsResult<()> {
        // Step 1: Ensure transport layer is open
        // Note: For server, transport is typically already open (listener accepted connection)
        if self.transport.is_closed() {
            self.transport.open().await?;
        }

        // Step 2: Wait for SNRM (Set Normal Response Mode) frame from client
        // Default timeout: 5 seconds (should be sufficient for most networks)
        let timeout = Duration::from_secs(5);
        let frames = self.receive_frames(Some(timeout)).await?;

        // Step 3: Find and parse SNRM frame
        let snrm_frame = frames
            .iter()
            .find(|f| f.frame_type() == FrameType::SetNormalResponseMode)
            .ok_or_else(|| {
                DlmsError::Connection(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "SNRM frame not received within timeout period",
                ))
            })?;

        // Extract client address from SNRM frame (for updating remote_address)
        // SNRM frame is sent from client to server, so source address is client address
        let client_address = snrm_frame.address_pair().source();
        self.remote_address = client_address;

        // Step 4: Parse SNRM parameters (if any)
        // According to HDLC standard, SNRM frame typically has no information field
        // But we check for it anyway in case of extended formats
        let _snrm_info_field = snrm_frame.information_field();
        // TODO: Parse SNRM parameters if information field is present
        // For now, we use default server parameters

        // Step 5: Generate UA frame with server parameters
        // UA frame information field contains negotiated parameters
        let ua_parameters = UaFrameParameters {
            window_size_rx: self.parameters.window_size_rx,
            max_information_field_length_rx: self.parameters.max_information_field_length_rx,
            window_size_tx: self.parameters.window_size_tx,
            max_information_field_length_tx: self.parameters.max_information_field_length_tx,
        };
        let ua_info_field = ua_parameters.encode();

        // Step 6: Send UA frame to client
        let address_pair = HdlcAddressPair::new(self.local_address, self.remote_address);
        let ua_frame = HdlcFrame::new(
            address_pair,
            FrameType::UnnumberedAcknowledge,
            Some(ua_info_field),
        );
        self.send_frame(ua_frame).await?;

        // Step 7: Update connection state to Connected
        // Use transition_to to ensure closed flag is properly synced
        self.transition_to(HdlcConnectionState::Connected)?;

        // Reset windows for new connection
        self.send_window.reset();
        self.receive_window.reset();

        Ok(())
    }

    /// Send an HDLC frame
    ///
    /// # Why Allow Sending When Closed?
    /// During connection establishment (SNRM/UA handshake), we need to send frames
    /// before the connection is fully established. The `closed` check is relaxed
    /// for control frames (SNRM, DISC) to allow connection setup and teardown.
    ///
    /// # Error Handling
    /// - Returns `DlmsError::Connection` if transport layer is closed
    /// - Returns `DlmsError::FrameInvalid` if frame encoding fails
    pub async fn send_frame(&mut self, frame: HdlcFrame) -> DlmsResult<()> {
        // Allow control frames (SNRM, DISC) even when connection is not fully established
        // This is necessary for connection setup and teardown
        let is_control_frame = matches!(
            frame.frame_type(),
            FrameType::SetNormalResponseMode | FrameType::Disconnect
        );
        
        // Check state: only allow information frames when connected
        if !self.state.can_send_information() && !is_control_frame {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                format!("HDLC connection is not ready: {:?}", self.state),
            )));
        }
        
        // Also check legacy closed flag for backward compatibility
        if self.closed && !is_control_frame {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "HDLC connection is closed",
            )));
        }

        // Check if transport layer is closed
        if self.transport.is_closed() {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Transport layer is closed",
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

    /// Send an information frame with window management
    ///
    /// # Window Management
    /// This method implements sliding window protocol:
    /// 1. Checks if send window has space
    /// 2. Assigns sequence number from send window
    /// 3. Sends frame and adds to send window
    /// 4. Window will slide when acknowledgment is received
    ///
    /// # LLC Header Handling
    /// If `use_llc_header` is enabled (default), the LLC header [0xE6, 0xE6, 0x00]
    /// will be automatically prepended to the information field data before encoding.
    /// This follows the DLMS standard (IEC 62056-47) requirement for LLC layer.
    ///
    /// # Blocking Behavior
    /// If the send window is full, this method will wait for acknowledgments
    /// before sending. This ensures we don't exceed the negotiated window size.
    ///
    /// # Error Handling
    /// - Returns `DlmsError::InvalidData` if window is full and cannot be cleared
    /// - Returns `DlmsError::Connection` if transport layer errors occur
    pub async fn send_information(
        &mut self,
        information_field: Vec<u8>,
        segmented: bool,
    ) -> DlmsResult<()> {
        // Wait for window space if needed
        while !self.send_window.can_send() {
            // Process any pending acknowledgments
            self.process_acknowledgments().await?;
            
            // Check for retransmissions
            self.handle_retransmissions().await?;
            
            // If still full, wait a bit and retry
            if !self.send_window.can_send() {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        // Prepend LLC header if enabled
        // According to DLMS standard (IEC 62056-47):
        // - Clients use LLC_REQUEST [0xE6, 0xE6, 0x00] for requests
        // - Servers use LLC_RESPONSE [0xE6, 0xE7, 0x00] for responses
        let mut data_with_llc = if self.use_llc_header {
            let llc_header = if self.is_client {
                &LLC_REQUEST
            } else {
                &LLC_RESPONSE
            };
            let mut data = Vec::with_capacity(llc_header.len() + information_field.len());
            data.extend_from_slice(llc_header);
            data.extend_from_slice(&information_field);
            data
        } else {
            information_field
        };

        let address_pair = HdlcAddressPair::new(self.local_address, self.remote_address);
        
        // Get expected receive sequence from receive window
        let recv_seq = self.receive_window.expected_sequence();
        
        // Get next sequence number from send window
        let sequence = self.send_window.peek_next_sequence();
        
        // Create frame with the sequence number from window
        let frame = HdlcFrame::new_information(
            address_pair,
            data_with_llc,
            sequence, // Use sequence from window
            recv_seq,
            segmented,
        );
        
        // Encode frame
        let encoded = frame.encode()?;
        
        // Add to send window (window will increment its next_sequence)
        let assigned_sequence = self.send_window.add_frame(frame, encoded.clone())?;
        
        // Verify sequence matches (should always match)
        if sequence != assigned_sequence {
            return Err(DlmsError::InvalidData(format!(
                "Sequence mismatch in send window: expected {}, got {}",
                sequence, assigned_sequence
            )));
        }
        
        // Update send_sequence to keep in sync
        self.send_sequence = (self.send_sequence + 1) % 8;
        
        // Send the frame
        self.send_frame_bytes(&encoded).await?;
        
        // Update statistics
        self.statistics.increment_frames_sent();
        
        Ok(())
    }
    
    /// Send frame bytes directly (internal method)
    ///
    /// This is used for both initial sends and retransmissions.
    async fn send_frame_bytes(&mut self, encoded: &[u8]) -> DlmsResult<()> {
        let mut data = vec![FLAG];
        data.extend_from_slice(encoded);
        data.push(FLAG);
        
        self.transport.write_all(&data).await?;
        self.transport.flush().await?;
        Ok(())
    }
    
    /// Process acknowledgments from received frames
    ///
    /// Checks received frames for N(R) values and acknowledges frames in send window.
    async fn process_acknowledgments(&mut self) -> DlmsResult<()> {
        // This will be called when processing received frames
        // For now, it's a placeholder
        Ok(())
    }
    
    /// Handle frame retransmissions
    ///
    /// Checks for timed-out frames and retransmits them.
    async fn handle_retransmissions(&mut self) -> DlmsResult<()> {
        let retransmissions = self.send_window.get_retransmissions();
        
        for (sequence, encoded_bytes) in retransmissions {
            // Retransmit the frame
            self.send_frame_bytes(&encoded_bytes).await?;
            self.statistics.increment_retransmissions();
        }
        
        Ok(())
    }

    /// Receive HDLC frames
    ///
    /// Receives frames from the transport layer without automatic segmentation handling.
    /// For automatic segmented frame reassembly, use `receive_segmented()` instead.
    ///
    /// # Returns
    /// Vector of decoded HDLC frames
    pub async fn receive_frames(&mut self, timeout: Option<Duration>) -> DlmsResult<Vec<HdlcFrame>> {
        if !self.state.is_ready() && self.state != HdlcConnectionState::Connecting {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                format!("HDLC connection is not ready: {:?}", self.state),
            )));
        }
        
        // Also check legacy closed flag
        if self.closed {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "HDLC connection is closed",
            )));
        }
        HdlcMessageDecoder::decode(&mut self.transport, timeout).await
    }

    /// Receive and automatically reassemble segmented frames
    ///
    /// # Segmented Frame Handling (per dlms-docs/dlms/长数据帧处理.txt)
    ///
    /// When a segmented frame is received (S bit = 1):
    /// 1. Extract the frame data
    /// 2. Automatically send RR frame to request next segment
    /// 3. Continue receiving and reassembling until complete (S bit = 0)
    /// 4. Return the complete reassembled message
    ///
    /// # Process
    /// 1. Receive frames from transport layer
    /// 2. For each Information frame:
    ///    - If segmented (S bit = 1): Start or continue reassembly, send RR frame
    ///    - If not segmented (S bit = 0): Return complete message
    /// 3. Handle timeouts and errors
    ///
    /// # Why This Design?
    /// - **Automatic RR Sending**: Client automatically requests next segment when needed
    ///   This follows the protocol requirement: "Client通过发送RR数据帧来请求被分割的数据帧的其余部分"
    /// - **Transparent Reassembly**: User receives complete message without manual handling
    /// - **Error Recovery**: Timeout and error handling prevent indefinite waiting
    ///
    /// # Error Handling
    /// - Connection closed: Returns `DlmsError::Connection`
    /// - Timeout: Returns `DlmsError::Connection` with timeout error
    /// - Sequence mismatch: Returns `DlmsError::FrameInvalid`
    /// - Buffer overflow: Returns `DlmsError::InvalidData`
    ///
    /// # Optimization Considerations
    /// - Default timeout: 5 seconds per segment
    /// - Maximum buffer size: 64KB (prevents memory exhaustion)
    /// - Automatic cleanup: Reassembler state is cleared after completion or error
    ///
    /// # Future Enhancements
    /// - Configurable timeout and buffer size
    /// - Support for multiple concurrent segmented messages
    /// - Better error recovery (retry mechanism)
    pub async fn receive_segmented(&mut self, timeout: Option<Duration>) -> DlmsResult<Vec<u8>> {
        if !self.state.is_ready() {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                format!("HDLC connection is not ready: {:?}", self.state),
            )));
        }
        
        // Also check legacy closed flag
        if self.closed {
            return Err(DlmsError::Connection(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "HDLC connection is closed",
            )));
        }

        // Use default timeout if not specified
        let receive_timeout = timeout.unwrap_or(Duration::from_secs(5));

        loop {
            // Check for timeout if reassembly is in progress
            if self.reassembler.is_active() && self.reassembler.is_timeout() {
                self.reassembler.reset();
                return Err(DlmsError::Connection(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timeout waiting for segmented frame continuation",
                )));
            }

            // Receive frames
            let frames = self.receive_frames(Some(receive_timeout)).await?;

            // Process each frame
            for frame in frames {
                // Only process Information frames for segmentation
                if frame.frame_type() != FrameType::Information {
                    continue;
                }

                // Extract sequence numbers and segmentation flag
                let send_seq = frame.send_sequence().ok_or_else(|| {
                    DlmsError::FrameInvalid("Information frame missing send sequence".to_string())
                })?;
                let recv_seq = frame.receive_sequence().unwrap_or(0);
                let is_segmented = frame.is_segmented();
                let mut info_data = frame.information_field().to_vec();
                
                // Process acknowledgment (N(R) in received frame acknowledges frames we sent)
                let acked_count = self.send_window.acknowledge(recv_seq);
                if acked_count > 0 {
                    // Window has slid, we may have space for more frames now
                }
                
                // Process received sequence (N(S) in received frame)
                // Validate sequence number using receive window
                if let Err(e) = self.receive_window.accept(send_seq) {
                    // Sequence mismatch - reject this frame
                    self.statistics.increment_sequence_errors();
                    // Continue to next frame
                    continue;
                }
                
                // Remove LLC header if present and enabled
                // According to DLMS standard:
                // - Requests use LLC_REQUEST [0xE6, 0xE6, 0x00] (client -> server)
                // - Responses use LLC_RESPONSE [0xE6, 0xE7, 0x00] (server -> client)
                // We need to check for both when receiving frames
                if self.use_llc_header && info_data.len() >= LLC_REQUEST.len() {
                    if info_data.starts_with(&LLC_REQUEST) {
                        // Request header (from client)
                        info_data.drain(0..LLC_REQUEST.len());
                    } else if info_data.starts_with(&LLC_RESPONSE) {
                        // Response header (from server)
                        info_data.drain(0..LLC_RESPONSE.len());
                    } else {
                        // LLC header expected but not found - this might be an error
                        // However, we'll continue processing to maintain compatibility
                        // In strict mode, we could return an error here
                    }
                }

                // Handle segmented frame
                if is_segmented {
                    // Segmented frame: S bit = 1, more segments to follow
                    if !self.reassembler.is_active() {
                        // First segment: start reassembly
                        // Next expected sequence = (current send_seq + 1) % 8
                        let next_seq = (send_seq + 1) % 8;
                        self.reassembler.start(info_data, next_seq);
                    } else {
                        // Continue reassembly: add this segment
                        let result = self.reassembler.add_segment(info_data, send_seq, false)?;
                        if let Some(complete_message) = result {
                            // Should not happen here (is_last = false)
                            // But handle it gracefully
                            return Ok(complete_message);
                        }
                    }

                    // Send RR frame to request next segment
                    // N(R) in RR frame = expected next sequence number we want to receive
                    let expected_seq = self.reassembler.expected_sequence();
                    self.send_rr_frame(expected_seq).await?;
                } else {
                    // Not segmented: S bit = 0
                    if self.reassembler.is_active() {
                        // This is the last segment of a segmented message
                        let result = self.reassembler.add_segment(info_data, send_seq, true)?;
                        if let Some(complete_message) = result {
                            return Ok(complete_message);
                        } else {
                            // This should not happen, but handle gracefully
                            return Err(DlmsError::FrameInvalid(
                                "Failed to complete segmented message reassembly".to_string(),
                            ));
                        }
                    } else {
                        // Complete message in single frame (not segmented)
                        return Ok(info_data);
                    }
                }
            }
        }
    }

    /// Send an RR (Receive Ready) frame
    ///
    /// # Arguments
    /// * `next_expected_sequence` - N(R) value indicating the next expected sequence number (0-7)
    ///
    /// # RR Frame Purpose (per dlms-docs/dlms/长数据帧处理.txt)
    /// "Client通过发送RR数据帧来请求被分割的数据帧的其余部分"
    /// (Client sends RR data frame to request the remaining parts of the segmented data frame)
    ///
    /// # Why This Method?
    /// Separating RR frame sending allows:
    /// - Reuse in different contexts (segmented frames, flow control)
    /// - Easier testing
    /// - Clearer code organization
    ///
    /// # Control Byte Format
    /// RR frame control byte: 0x01 | (N(R) << 5)
    /// - Bit 0: 1 (indicates RR frame)
    /// - Bits 1-3: 000
    /// - Bits 5-7: N(R) (next expected receive sequence number)
    async fn send_rr_frame(&mut self, next_expected_sequence: u8) -> DlmsResult<()> {
        let address_pair = HdlcAddressPair::new(self.local_address, self.remote_address);
        let rr_frame = HdlcFrame::new_receive_ready(address_pair, next_expected_sequence);
        self.send_frame(rr_frame).await?;
        Ok(())
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
        self.state == HdlcConnectionState::Closed || self.transport.is_closed()
    }
    
    /// Get current connection state
    pub fn state(&self) -> HdlcConnectionState {
        self.state
    }
    
    /// Transition to a new state with validation
    ///
    /// # Arguments
    /// * `new_state` - The target state
    ///
    /// # Returns
    /// `Ok(())` if transition is valid, `Err` otherwise
    pub fn transition_to(&mut self, new_state: HdlcConnectionState) -> DlmsResult<()> {
        self.state.validate_transition(new_state)?;
        self.state = new_state;
        
        // Keep closed flag in sync
        self.closed = matches!(self.state, HdlcConnectionState::Closed);
        
        Ok(())
    }

    /// Close the HDLC connection
    ///
    /// # Connection Termination Process (per dlms-docs/dlms/cosem连接过程.txt)
    ///
    /// The connection termination follows this sequence:
    /// ```
    /// 客户端 -> DISC -> 服务器
    /// 客户端 <- DM/UA <- 服务器
    /// ```
    ///
    /// # Process
    /// 1. Check if connection is already closed (idempotent operation)
    /// 2. Send DISC (Disconnect) frame to server
    /// 3. Wait for DM (Disconnect Mode) or UA (Unnumbered Acknowledge) response with timeout
    /// 4. Close the transport layer
    /// 5. Update connection state to closed
    ///
    /// # Response Types (per dlms-docs/dlms/长数据帧处理.txt)
    /// - **UA**: 表示接收到DISC帧后断开链接 (Acknowledges DISC and disconnects)
    /// - **DM**: 表示在接收到DISC帧之前就已经处于链路断开状态 (Already disconnected)
    ///
    /// # Why This Design?
    /// - **DISC Frame**: Notifies the server that we want to disconnect
    /// - **DM/UA Response**: Confirms disconnection (or indicates already disconnected)
    /// - **Timeout Handling**: If no response is received, we still close the connection
    ///   (server may have already disconnected or network issues)
    ///
    /// # Error Handling
    /// - DISC send failure: Still attempts to close transport layer (best effort)
    /// - Response timeout: Still closes connection (server may have already disconnected)
    /// - Response format error: Logs error but continues with close
    /// - Transport layer close failure: Returns error (this is the critical step)
    ///
    /// # Optimization Considerations
    /// - Default timeout is 3 seconds (shorter than connection establishment, as disconnect is unidirectional)
    /// - Idempotent operation: Can be called multiple times safely
    /// - Best-effort approach: Even if DISC or response fails, we still try to close transport
    ///
    /// # Future Enhancements
    /// - Configurable timeout duration
    /// - Retry mechanism for DISC frame (if needed)
    /// - Better error reporting (distinguish between different failure modes)
    pub async fn close(&mut self) -> DlmsResult<()> {
        // Step 1: Check if connection is already closed (idempotent operation)
        if self.closed {
            return Ok(()); // Already closed, nothing to do
        }

        // Step 2: Send DISC (Disconnect) frame to server
        // DISC frame has no information field according to HDLC standard
        let address_pair = HdlcAddressPair::new(self.local_address, self.remote_address);
        let disc_frame = HdlcFrame::new(address_pair, FrameType::Disconnect, None);
        
        // Send DISC frame (ignore errors - best effort, we'll close transport anyway)
        // This allows close() to work even if the connection is already broken
        let _ = self.send_frame(disc_frame).await;

        // Step 3: Wait for DM (Disconnect Mode) or UA (Unnumbered Acknowledge) response
        // Default timeout: 3 seconds (shorter than connection establishment timeout)
        // This is reasonable as disconnect is a unidirectional operation and server may
        // have already disconnected
        let timeout = Duration::from_secs(3);
        
        // Try to receive response frames
        // We don't fail if this fails - server may have already disconnected or network issues
        if let Ok(frames) = self.receive_frames(Some(timeout)).await {
            // Check for DM or UA frame
            // According to documentation:
            // - DM: Server was already disconnected
            // - UA: Server acknowledges DISC and disconnects
            let _response_received = frames.iter().any(|f| {
                matches!(
                    f.frame_type(),
                    FrameType::DisconnectMode | FrameType::UnnumberedAcknowledge
                )
            });
            // Note: We don't fail if response is not received or is wrong type
            // The important thing is that we close the transport layer
        }
        // If receive fails or times out, we continue with closing transport layer
        // This is acceptable as the server may have already disconnected

        // Step 4: Close the transport layer
        // This is the critical step - even if DISC or response failed, we must close transport
        self.transport.close().await?;

        // Step 5: Update connection state to closed
        self.transition_to(HdlcConnectionState::Closed)?;
        
        // Reset windows when connection is closed
        self.send_window.reset();
        self.receive_window.reset();
        
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
