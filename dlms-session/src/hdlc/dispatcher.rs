//! HDLC dispatcher for handling frame routing

use crate::error::{DlmsError, DlmsResult};
use crate::hdlc::frame::HdlcFrame;
use crate::hdlc::address::HdlcAddress;
use std::collections::VecDeque;
use std::fmt;
use tokio::sync::mpsc;

/// Wrapper for mpsc::Receiver that implements Debug
struct DebugReceiver<T>(mpsc::Receiver<T>);

impl<T> fmt::Debug for DebugReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Receiver").finish()
    }
}

impl<T> std::ops::Deref for DebugReceiver<T> {
    type Target = mpsc::Receiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for DebugReceiver<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// HDLC message queue
#[derive(Debug)]
pub struct HdlcMessageQueue {
    queue: VecDeque<HdlcFrame>,
}

impl HdlcMessageQueue {
    /// Create a new message queue
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Add a frame to the queue
    pub fn enqueue(&mut self, frame: HdlcFrame) {
        self.queue.push_back(frame);
    }

    /// Remove and return the next frame
    pub fn dequeue(&mut self) -> Option<HdlcFrame> {
        self.queue.pop_front()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

impl Default for HdlcMessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// HDLC dispatcher for routing frames
#[derive(Debug)]
pub struct HdlcDispatcher {
    local_address: HdlcAddress,
    message_queue: HdlcMessageQueue,
    frame_receiver: Option<DebugReceiver<HdlcFrame>>,
}

impl HdlcDispatcher {
    /// Create a new HDLC dispatcher
    pub fn new(local_address: HdlcAddress) -> Self {
        Self {
            local_address,
            message_queue: HdlcMessageQueue::new(),
            frame_receiver: None,
        }
    }

    /// Set the frame receiver channel
    pub fn set_receiver(&mut self, receiver: mpsc::Receiver<HdlcFrame>) {
        self.frame_receiver = Some(DebugReceiver(receiver));
    }

    /// Process incoming frames
    pub async fn process_frames(&mut self) -> DlmsResult<()> {
        // Collect all pending frames first to avoid borrow checker issues
        let mut frames = Vec::new();
        if let Some(ref mut receiver) = self.frame_receiver {
            while let Ok(frame) = receiver.try_recv() {
                frames.push(frame);
            }
        }

        // Now process frames without holding the receiver borrow
        for frame in frames {
            if self.should_accept_frame(&frame) {
                self.message_queue.enqueue(frame);
            }
        }
        Ok(())
    }

    /// Check if frame should be accepted
    fn should_accept_frame(&self, frame: &HdlcFrame) -> bool {
        let destination = frame.address_pair().destination();
        
        // Accept if:
        // 1. Frame is addressed to our local address
        // 2. Frame is a broadcast (all-station)
        // 3. Frame is addressed to no-station (for initial connection)
        destination == self.local_address
            || destination.is_all_station()
            || destination.is_no_station()
    }

    /// Get next frame from queue
    pub fn next_frame(&mut self) -> Option<HdlcFrame> {
        self.message_queue.dequeue()
    }

    /// Get local address
    pub fn local_address(&self) -> HdlcAddress {
        self.local_address
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hdlc::address::HdlcAddressPair;

    #[test]
    fn test_message_queue() {
        let mut queue = HdlcMessageQueue::new();
        assert!(queue.is_empty());
        
        // Note: Would need a frame to test, but frame creation requires more setup
    }
}
