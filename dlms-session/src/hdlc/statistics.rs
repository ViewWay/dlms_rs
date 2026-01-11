//! HDLC statistics collection

/// HDLC connection statistics
///
/// Tracks various metrics for HDLC connection monitoring and debugging.
/// Similar to C++ implementation's `HDLCStatistics` class.
///
/// # Why Statistics?
/// - **Performance Monitoring**: Track throughput, frame rates, and latency
/// - **Debugging**: Identify issues like excessive errors or timeouts
/// - **Quality Assurance**: Monitor connection health and reliability
///
/// # Usage
/// Statistics are automatically updated by the HDLC connection during operation.
/// Users can query statistics at any time to monitor connection status.
#[derive(Debug, Clone, Default)]
pub struct HdlcStatistics {
    /// Total number of frames sent
    pub frames_sent: u64,
    /// Total number of frames received
    pub frames_received: u64,
    /// Number of frames rejected due to errors
    pub frames_rejected: u64,
    /// Number of timeout events
    pub timeouts: u64,
    /// Number of FCS (Frame Check Sequence) errors
    pub fcs_errors: u64,
    /// Number of HCS (Header Check Sequence) errors
    pub hcs_errors: u64,
    /// Number of sequence number mismatches
    pub sequence_errors: u64,
    /// Number of retransmitted frames
    pub retransmissions: u64,
}

impl HdlcStatistics {
    /// Create new statistics with all counters at zero
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all statistics counters
    ///
    /// Resets all counters to zero, similar to C++ implementation's `Clear()` method.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Increment frames sent counter
    pub fn increment_frames_sent(&mut self) {
        self.frames_sent += 1;
    }

    /// Increment frames received counter
    pub fn increment_frames_received(&mut self) {
        self.frames_received += 1;
    }

    /// Increment frames rejected counter
    pub fn increment_frames_rejected(&mut self) {
        self.frames_rejected += 1;
    }

    /// Increment timeout counter
    pub fn increment_timeouts(&mut self) {
        self.timeouts += 1;
    }

    /// Increment FCS error counter
    pub fn increment_fcs_errors(&mut self) {
        self.fcs_errors += 1;
    }

    /// Increment HCS error counter
    pub fn increment_hcs_errors(&mut self) {
        self.hcs_errors += 1;
    }

    /// Increment sequence error counter
    pub fn increment_sequence_errors(&mut self) {
        self.sequence_errors += 1;
    }

    /// Increment retransmission counter
    pub fn increment_retransmissions(&mut self) {
        self.retransmissions += 1;
    }

    /// Get error rate as a percentage
    ///
    /// Calculates the percentage of frames that resulted in errors.
    /// Returns 0.0 if no frames have been received.
    pub fn error_rate(&self) -> f64 {
        let total_errors = self.frames_rejected
            + self.fcs_errors
            + self.hcs_errors
            + self.sequence_errors;
        let total_frames = self.frames_received + self.frames_sent;
        if total_frames == 0 {
            0.0
        } else {
            (total_errors as f64 / total_frames as f64) * 100.0
        }
    }
}
