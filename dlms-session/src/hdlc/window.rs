//! HDLC window management and retransmission

use crate::error::{DlmsError, DlmsResult};
use crate::hdlc::frame::HdlcFrame;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Pending frame waiting for acknowledgment
///
/// Tracks a frame that has been sent but not yet acknowledged.
/// Used for implementing sliding window protocol and retransmission.
#[derive(Debug, Clone)]
struct PendingFrame {
    /// The frame that was sent
    frame: HdlcFrame,
    /// Sequence number of this frame (N(S))
    sequence: u8,
    /// Time when frame was sent
    sent_time: Instant,
    /// Number of retransmission attempts
    retry_count: u8,
    /// Encoded frame bytes (for retransmission)
    encoded_bytes: Vec<u8>,
}

impl PendingFrame {
    /// Create a new pending frame
    fn new(frame: HdlcFrame, sequence: u8, encoded_bytes: Vec<u8>) -> Self {
        Self {
            frame,
            sequence,
            sent_time: Instant::now(),
            retry_count: 0,
            encoded_bytes,
        }
    }

    /// Check if this frame has timed out
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait for acknowledgment
    ///
    /// # Returns
    /// `true` if timeout has been exceeded, `false` otherwise
    fn is_timeout(&self, timeout: Duration) -> bool {
        self.sent_time.elapsed() > timeout
    }

    /// Increment retry count
    fn increment_retry(&mut self) {
        self.retry_count += 1;
        self.sent_time = Instant::now(); // Reset timeout
    }

    /// Get retry count
    fn retry_count(&self) -> u8 {
        self.retry_count
    }
}

/// Send window for sliding window protocol
///
/// Manages the sending window for HDLC frames, implementing:
/// - Sliding window protocol
/// - Frame retransmission on timeout
/// - Window size enforcement
///
/// # Window Protocol
/// The send window tracks frames that have been sent but not yet acknowledged.
/// The window size (from `HdlcParameters::window_size_tx`) limits how many
/// frames can be in flight at once.
///
/// # Sequence Numbers
/// HDLC uses 3-bit sequence numbers (0-7), so sequence numbers wrap around.
/// The window size must be <= 7 to prevent ambiguity.
///
/// # Retransmission
/// If a frame is not acknowledged within the timeout period, it is automatically
/// retransmitted. The maximum retry count prevents infinite retransmission.
#[derive(Debug)]
pub struct SendWindow {
    /// Pending frames waiting for acknowledgment
    unacked_frames: VecDeque<PendingFrame>,
    /// Maximum window size (from HdlcParameters)
    window_size: u8,
    /// Next sequence number to use (N(S))
    next_sequence: u8,
    /// Retransmission timeout
    retransmit_timeout: Duration,
    /// Maximum number of retransmission attempts
    max_retries: u8,
}

impl SendWindow {
    /// Create a new send window
    ///
    /// # Arguments
    /// * `window_size` - Maximum number of unacknowledged frames (1-7)
    /// * `retransmit_timeout` - Time to wait before retransmitting
    /// * `max_retries` - Maximum number of retransmission attempts
    ///
    /// # Panics
    /// Panics if `window_size` is 0 or > 7 (HDLC sequence numbers are 3-bit)
    pub fn new(window_size: u8, retransmit_timeout: Duration, max_retries: u8) -> Self {
        assert!(window_size > 0 && window_size <= 7, "Window size must be 1-7");
        Self {
            unacked_frames: VecDeque::new(),
            window_size,
            next_sequence: 0,
            retransmit_timeout,
            max_retries,
        }
    }

    /// Check if window has space for a new frame
    ///
    /// # Returns
    /// `true` if a new frame can be sent, `false` if window is full
    pub fn can_send(&self) -> bool {
        self.unacked_frames.len() < self.window_size as usize
    }

    /// Add a frame to the send window
    ///
    /// # Arguments
    /// * `frame` - Frame to send
    /// * `encoded_bytes` - Encoded frame bytes (for retransmission)
    ///
    /// # Returns
    /// Sequence number assigned to this frame
    ///
    /// # Errors
    /// Returns `DlmsError::InvalidData` if window is full
    pub fn add_frame(&mut self, frame: HdlcFrame, encoded_bytes: Vec<u8>) -> DlmsResult<u8> {
        if !self.can_send() {
            return Err(DlmsError::InvalidData(format!(
                "Send window is full: {} frames pending (window size: {})",
                self.unacked_frames.len(),
                self.window_size
            )));
        }

        let sequence = self.next_sequence;
        let pending = PendingFrame::new(frame, sequence, encoded_bytes);
        self.unacked_frames.push_back(pending);
        self.next_sequence = (self.next_sequence + 1) % 8;
        Ok(sequence)
    }

    /// Acknowledge frames up to a sequence number
    ///
    /// Removes all frames with sequence numbers < `ack_sequence` from the window.
    /// This implements the sliding window protocol: when we receive an acknowledgment
    /// with N(R) = n, we know all frames with N(S) < n have been received.
    ///
    /// # Arguments
    /// * `ack_sequence` - N(R) value from received frame (next expected sequence)
    ///
    /// # Returns
    /// Number of frames acknowledged
    ///
    /// # HDLC Sequence Number Semantics
    /// In HDLC, N(R) in a received frame means "I have received all frames up to
    /// (but not including) sequence number N(R)". So if we receive N(R) = 3, it means
    /// frames with sequence 0, 1, 2 have been received, and we expect sequence 3 next.
    ///
    /// # Wrap-Around Handling
    /// HDLC uses 3-bit sequence numbers (0-7), so we need to handle wrap-around.
    /// The algorithm compares sequences considering the circular nature:
    /// - If ack_sequence > oldest_sequence: acknowledge frames with seq < ack_sequence
    /// - If ack_sequence < oldest_sequence: wrap-around case, acknowledge frames with
    ///   seq < ack_sequence OR seq >= oldest_sequence
    pub fn acknowledge(&mut self, ack_sequence: u8) -> usize {
        let mut acked_count = 0;

        // Remove all frames with sequence < ack_sequence
        // Handle wrap-around for 3-bit sequence numbers (0-7)
        while let Some(front) = self.unacked_frames.front() {
            let seq = front.sequence;
            
            // Determine if this frame should be acknowledged
            // N(R) = n means frames 0..n-1 are acknowledged
            let should_ack = if ack_sequence > seq {
                // Normal case: ack_sequence > seq, so seq < ack_sequence
                true
            } else if ack_sequence < seq {
                // Wrap-around case: ack_sequence < seq
                // This means we've wrapped around. Frames with seq < ack_sequence
                // were sent in the previous cycle and should be acknowledged.
                // Frames with seq >= oldest_sequence are from current cycle and not yet acked.
                // So we only ack if seq < ack_sequence (from previous cycle)
                seq < ack_sequence
            } else {
                // ack_sequence == seq: This frame is the next expected, don't ack it yet
                false
            };

            if should_ack {
                self.unacked_frames.pop_front();
                acked_count += 1;
            } else {
                break;
            }
        }

        acked_count
    }

    /// Get frames that need retransmission
    ///
    /// Checks all pending frames and returns those that have timed out.
    ///
    /// # Returns
    /// Vector of (sequence, encoded_bytes) tuples for frames that need retransmission
    pub fn get_retransmissions(&mut self) -> Vec<(u8, Vec<u8>)> {
        let mut retransmissions = Vec::new();
        let now = Instant::now();

        for pending in &mut self.unacked_frames {
            if pending.is_timeout(self.retransmit_timeout) {
                if pending.retry_count() < self.max_retries {
                    pending.increment_retry();
                    retransmissions.push((pending.sequence, pending.encoded_bytes.clone()));
                } else {
                    // Max retries exceeded - this frame will be dropped
                    // The connection should probably be closed or reset
                }
            }
        }

        retransmissions
    }

    /// Get the oldest unacknowledged frame sequence
    ///
    /// # Returns
    /// Sequence number of the oldest pending frame, or `None` if window is empty
    pub fn oldest_sequence(&self) -> Option<u8> {
        self.unacked_frames.front().map(|p| p.sequence)
    }

    /// Get number of pending frames
    pub fn pending_count(&self) -> usize {
        self.unacked_frames.len()
    }

    /// Peek at the next sequence number that will be assigned
    ///
    /// This allows creating a frame with the correct sequence number
    /// before adding it to the window.
    pub fn peek_next_sequence(&self) -> u8 {
        self.next_sequence
    }

    /// Check if window is empty
    pub fn is_empty(&self) -> bool {
        self.unacked_frames.is_empty()
    }

    /// Reset the send window
    ///
    /// Clears all pending frames and resets the sequence number.
    /// Used when connection is reset or closed.
    pub fn reset(&mut self) {
        self.unacked_frames.clear();
        self.next_sequence = 0;
    }

    /// Update window size
    ///
    /// # Arguments
    /// * `new_size` - New window size (1-7)
    ///
    /// # Panics
    /// Panics if `new_size` is 0 or > 7
    pub fn set_window_size(&mut self, new_size: u8) {
        assert!(new_size > 0 && new_size <= 7, "Window size must be 1-7");
        self.window_size = new_size;
        // If new window size is smaller, we might need to drop some frames
        // For now, we'll just prevent adding new frames until window has space
    }
}

/// Receive window for sliding window protocol
///
/// Manages the receiving window for HDLC frames, tracking:
/// - Expected next sequence number (N(R))
/// - Received frames (for out-of-order handling, if needed)
///
/// # Sequence Number Tracking
/// The receive window tracks the next expected sequence number (N(R)).
/// When we receive a frame with N(S) = N(R), we accept it and increment N(R).
///
/// # Out-of-Order Frames
/// Currently, we only accept frames in order. Out-of-order frames are rejected.
/// Future enhancement: could buffer out-of-order frames and reassemble.
#[derive(Debug)]
pub struct ReceiveWindow {
    /// Next expected sequence number (N(R))
    expected_sequence: u8,
}

impl ReceiveWindow {
    /// Create a new receive window
    pub fn new() -> Self {
        Self {
            expected_sequence: 0,
        }
    }

    /// Check if a received frame has the expected sequence number
    ///
    /// # Arguments
    /// * `sequence` - N(S) from received frame
    ///
    /// # Returns
    /// `true` if sequence matches expected, `false` otherwise
    pub fn is_expected(&self, sequence: u8) -> bool {
        sequence == self.expected_sequence
    }

    /// Accept a frame with the expected sequence number
    ///
    /// # Arguments
    /// * `sequence` - N(S) from received frame
    ///
    /// # Returns
    /// `Ok(())` if sequence matches, `Err` if sequence mismatch
    pub fn accept(&mut self, sequence: u8) -> DlmsResult<()> {
        if !self.is_expected(sequence) {
            return Err(DlmsError::FrameInvalid(format!(
                "Sequence number mismatch: expected {}, got {}",
                self.expected_sequence, sequence
            )));
        }

        // Increment expected sequence (wrap around at 8)
        self.expected_sequence = (self.expected_sequence + 1) % 8;
        Ok(())
    }

    /// Get the next expected sequence number (N(R))
    ///
    /// This value should be sent in the N(R) field of frames we send,
    /// indicating the next sequence number we expect to receive.
    pub fn expected_sequence(&self) -> u8 {
        self.expected_sequence
    }

    /// Reset the receive window
    ///
    /// Resets the expected sequence number to 0.
    /// Used when connection is reset or closed.
    pub fn reset(&mut self) {
        self.expected_sequence = 0;
    }
}

impl Default for ReceiveWindow {
    fn default() -> Self {
        Self::new()
    }
}
