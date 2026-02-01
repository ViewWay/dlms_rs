//! Server request statistics module
//!
//! This module provides comprehensive statistics tracking for DLMS/COSEM server
//! operations including request counts, error tracking, and performance metrics.

use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Request type enumeration for statistics tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestType {
    /// Initiate request (association establishment)
    Initiate,
    /// Get request (read attributes)
    Get,
    /// Set request (write attributes)
    Set,
    /// Action request (invoke methods)
    Action,
    /// Access request (access control)
    Access,
    /// GetRequest-WithList (batch read)
    GetWithList,
    /// GetRequest-Next (block transfer read)
    GetNext,
    /// SetRequest-WithList (batch write)
    SetWithList,
    /// SetRequest-WithFirstDataBlock (block transfer start)
    SetFirstDataBlock,
    /// SetRequest-WithDataBlock (block transfer continue)
    SetDataBlock,
    /// Release request (association termination)
    Release,
    /// Unknown/unrecognized request type
    Unknown,
}

impl RequestType {
    /// Get a human-readable name for the request type
    pub fn name(&self) -> &'static str {
        match self {
            Self::Initiate => "Initiate",
            Self::Get => "Get",
            Self::Set => "Set",
            Self::Action => "Action",
            Self::Access => "Access",
            Self::GetWithList => "GetWithList",
            Self::GetNext => "GetNext",
            Self::SetWithList => "SetWithList",
            Self::SetFirstDataBlock => "SetFirstDataBlock",
            Self::SetDataBlock => "SetDataBlock",
            Self::Release => "Release",
            Self::Unknown => "Unknown",
        }
    }
}

/// Error statistics for a specific error type
#[derive(Debug, Clone)]
pub struct ErrorStats {
    /// Number of times this error occurred
    pub count: u64,
    /// Error description
    pub description: String,
    /// Last time this error occurred
    pub last_occurred: Option<Instant>,
}

impl ErrorStats {
    /// Create new error statistics
    pub fn new(description: String) -> Self {
        Self {
            count: 0,
            description,
            last_occurred: None,
        }
    }

    /// Increment error count and update last occurred time
    pub fn increment(&mut self) {
        self.count += 1;
        self.last_occurred = Some(Instant::now());
    }
}

/// Performance metrics for request processing
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Total processing time for all requests
    pub total_processing_time: Duration,
    /// Minimum processing time observed
    pub min_processing_time: Option<Duration>,
    /// Maximum processing time observed
    pub max_processing_time: Option<Duration>,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            total_processing_time: Duration::ZERO,
            min_processing_time: None,
            max_processing_time: None,
            successful_requests: 0,
            failed_requests: 0,
        }
    }
}

impl PerformanceMetrics {
    /// Record a request completion
    pub fn record_request(&mut self, processing_time: Duration, success: bool) {
        self.total_requests += 1;
        self.total_processing_time += processing_time;

        // Update min/max processing times
        if let Some(min) = self.min_processing_time {
            self.min_processing_time = Some(min.min(processing_time));
        } else {
            self.min_processing_time = Some(processing_time);
        }

        if let Some(max) = self.max_processing_time {
            self.max_processing_time = Some(max.max(processing_time));
        } else {
            self.max_processing_time = Some(processing_time);
        }

        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
    }

    /// Get average processing time
    pub fn average_processing_time(&self) -> Option<Duration> {
        if self.total_requests > 0 {
            Some(self.total_processing_time / self.total_requests as u32)
        } else {
            None
        }
    }

    /// Get success rate as a percentage (0-100)
    pub fn success_rate(&self) -> Option<f64> {
        if self.total_requests > 0 {
            Some((self.successful_requests as f64 / self.total_requests as f64) * 100.0)
        } else {
            None
        }
    }

    /// Get requests per second (based on average processing time)
    pub fn requests_per_second(&self) -> Option<f64> {
        if let Some(avg) = self.average_processing_time() {
            if avg.as_secs_f64() > 0.0 {
                Some(1.0 / avg.as_secs_f64())
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Statistics for a specific request type
#[derive(Debug, Clone)]
pub struct RequestTypeStats {
    /// Number of requests of this type
    pub count: u64,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// First time this request type was seen
    pub first_seen: Instant,
    /// Last time this request type was seen
    pub last_seen: Instant,
}

impl RequestTypeStats {
    /// Create new request type statistics
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            count: 0,
            performance: PerformanceMetrics::default(),
            first_seen: now,
            last_seen: now,
        }
    }

    /// Record a request of this type
    pub fn record_request(&mut self, processing_time: Duration, success: bool) {
        self.count += 1;
        self.last_seen = Instant::now();
        self.performance.record_request(processing_time, success);
    }
}

impl Default for RequestTypeStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Overall server request statistics
#[derive(Debug, Clone)]
pub struct ServerRequestStats {
    /// Statistics per request type
    pub request_types: HashMap<RequestType, RequestTypeStats>,
    /// Error statistics grouped by error message
    pub errors: HashMap<String, ErrorStats>,
    /// Total bytes received
    pub bytes_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Server start time
    pub start_time: Instant,
    /// Last reset time
    pub last_reset_time: Instant,
}

impl ServerRequestStats {
    /// Create new server statistics
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            request_types: HashMap::new(),
            errors: HashMap::new(),
            bytes_received: 0,
            bytes_sent: 0,
            start_time: now,
            last_reset_time: now,
        }
    }

    /// Record a request being processed
    pub fn record_request_start(&mut self, request_type: RequestType) -> RequestTracker {
        // Ensure the entry exists (but don't increment count yet)
        self.request_types
            .entry(request_type)
            .or_insert_with(RequestTypeStats::new);

        RequestTracker {
            start_time: Instant::now(),
            request_type,
        }
    }

    /// Record a request completion
    pub fn record_request_completion(
        &mut self,
        tracker: RequestTracker,
        success: bool,
        bytes_received: u64,
        bytes_sent: u64,
    ) {
        let processing_time = tracker.start_time.elapsed();
        self.bytes_received += bytes_received;
        self.bytes_sent += bytes_sent;

        let stats = self.request_types
            .entry(tracker.request_type)
            .or_insert_with(RequestTypeStats::new);

        stats.record_request(processing_time, success);

        if !success {
            // Record failure - error details should be logged separately
        }
    }

    /// Record an error
    pub fn record_error(&mut self, error_message: String) {
        let stats = self.errors.entry(error_message.clone()).or_insert_with(|| {
            ErrorStats::new(error_message)
        });
        stats.increment();
    }

    /// Get total request count across all types
    pub fn total_requests(&self) -> u64 {
        self.request_types.values().map(|s| s.count).sum()
    }

    /// Get total successful requests
    pub fn total_successful(&self) -> u64 {
        self.request_types.values()
            .map(|s| s.performance.successful_requests)
            .sum()
    }

    /// Get total failed requests
    pub fn total_failed(&self) -> u64 {
        self.request_types.values()
            .map(|s| s.performance.failed_requests)
            .sum()
    }

    /// Get overall success rate
    pub fn success_rate(&self) -> Option<f64> {
        let total = self.total_requests();
        if total > 0 {
            Some((self.total_successful() as f64 / total as f64) * 100.0)
        } else {
            None
        }
    }

    /// Get server uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get time since last reset
    pub fn time_since_reset(&self) -> Duration {
        self.last_reset_time.elapsed()
    }

    /// Calculate requests per second since server start
    pub fn requests_per_second(&self) -> f64 {
        let uptime_secs = self.uptime().as_secs_f64();
        if uptime_secs > 0.0 {
            self.total_requests() as f64 / uptime_secs
        } else {
            0.0
        }
    }

    /// Reset statistics (keeping start time)
    pub fn reset(&mut self) {
        let start = self.start_time;
        *self = Self::new();
        self.start_time = start;
        self.last_reset_time = Instant::now();
    }

    /// Get a summary of all statistics
    pub fn summary(&self) -> ServerStatsSummary {
        ServerStatsSummary {
            uptime: self.uptime(),
            total_requests: self.total_requests(),
            successful_requests: self.total_successful(),
            failed_requests: self.total_failed(),
            success_rate: self.success_rate(),
            requests_per_second: self.requests_per_second(),
            bytes_received: self.bytes_received,
            bytes_sent: self.bytes_sent,
            request_type_counts: self.request_types.iter()
                .map(|(rt, stats)| (*rt, stats.count))
                .collect(),
            error_count: self.errors.values().map(|e| e.count).sum(),
        }
    }
}

impl Default for ServerRequestStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Request tracker for measuring processing time
///
/// Created when a request starts processing and used to record
/// completion time.
#[derive(Debug)]
pub struct RequestTracker {
    start_time: Instant,
    request_type: RequestType,
}

impl RequestTracker {
    /// Complete the request tracking and get processing time
    pub fn complete(self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get the request type
    pub fn request_type(&self) -> RequestType {
        self.request_type
    }

    /// Get elapsed time so far
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// A concise summary of server statistics
#[derive(Debug, Clone)]
pub struct ServerStatsSummary {
    /// Server uptime
    pub uptime: Duration,
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Overall success rate percentage
    pub success_rate: Option<f64>,
    /// Average requests per second
    pub requests_per_second: f64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Request counts by type
    pub request_type_counts: HashMap<RequestType, u64>,
    /// Total error count
    pub error_count: u64,
}

impl ServerStatsSummary {
    /// Format the summary for display
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str("=== DLMS Server Statistics ===\n");
        output.push_str(&format!("Uptime: {:.2}s\n", self.uptime.as_secs_f64()));
        output.push_str(&format!("Total Requests: {}\n", self.total_requests));
        output.push_str(&format!("Successful: {} ({:.1}%)\n",
            self.successful_requests,
            self.success_rate.unwrap_or(0.0)
        ));
        output.push_str(&format!("Failed: {}\n", self.failed_requests));
        output.push_str(&format!("Requests/sec: {:.2}\n", self.requests_per_second));
        output.push_str(&format!("Bytes Received: {}\n", self.bytes_received));
        output.push_str(&format!("Bytes Sent: {}\n", self.bytes_sent));
        output.push_str("\nRequest Types:\n");

        let mut sorted_types: Vec<_> = self.request_type_counts.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));

        for (req_type, count) in sorted_types {
            output.push_str(&format!("  {}: {}\n", req_type.name(), count));
        }

        if self.error_count > 0 {
            output.push_str(&format!("\nTotal Errors: {}\n", self.error_count));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_type_names() {
        assert_eq!(RequestType::Get.name(), "Get");
        assert_eq!(RequestType::Set.name(), "Set");
        assert_eq!(RequestType::Action.name(), "Action");
        assert_eq!(RequestType::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::default();

        metrics.record_request(Duration::from_millis(100), true);
        metrics.record_request(Duration::from_millis(200), true);
        metrics.record_request(Duration::from_millis(50), false);

        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 1);

        let avg = metrics.average_processing_time().unwrap();
        // Integer division: (100+200+50)/3 = 350/3 = 116
        assert_eq!(avg.as_millis(), 116);

        assert_eq!(metrics.min_processing_time, Some(Duration::from_millis(50)));
        assert_eq!(metrics.max_processing_time, Some(Duration::from_millis(200)));

        let success_rate = metrics.success_rate().unwrap();
        assert!((success_rate - 66.66).abs() < 0.1);
    }

    #[test]
    fn test_server_stats() {
        let mut stats = ServerRequestStats::new();

        // Simulate some requests
        let tracker1 = stats.record_request_start(RequestType::Get);
        std::thread::sleep(Duration::from_millis(10));
        stats.record_request_completion(tracker1, true, 100, 50);

        let tracker2 = stats.record_request_start(RequestType::Set);
        stats.record_request_completion(tracker2, false, 50, 20);

        assert_eq!(stats.total_requests(), 2);
        assert_eq!(stats.total_successful(), 1);
        assert_eq!(stats.total_failed(), 1);
        assert_eq!(stats.bytes_received, 150);
        assert_eq!(stats.bytes_sent, 70);

        let summary = stats.summary();
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.successful_requests, 1);
    }

    #[test]
    fn test_error_tracking() {
        let mut stats = ServerRequestStats::new();

        stats.record_error("Test error 1".to_string());
        stats.record_error("Test error 2".to_string());
        stats.record_error("Test error 1".to_string()); // Duplicate

        assert_eq!(stats.errors.len(), 2);
        assert_eq!(stats.errors.get("Test error 1").unwrap().count, 2);
        assert_eq!(stats.errors.get("Test error 2").unwrap().count, 1);
    }

    #[test]
    fn test_stats_format() {
        let mut stats = ServerRequestStats::new();

        let tracker = stats.record_request_start(RequestType::Get);
        stats.record_request_completion(tracker, true, 100, 50);

        let summary = stats.summary();
        let formatted = summary.format();

        assert!(formatted.contains("Total Requests: 1"));
        assert!(formatted.contains("Get: 1"));
    }

    #[test]
    fn test_stats_reset() {
        let mut stats = ServerRequestStats::new();

        let tracker = stats.record_request_start(RequestType::Get);
        stats.record_request_completion(tracker, true, 100, 50);

        let start = stats.start_time;
        stats.reset();

        assert_eq!(stats.total_requests(), 0);
        assert_eq!(stats.start_time, start); // Start time should be preserved
    }
}
