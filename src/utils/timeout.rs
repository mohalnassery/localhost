/*!
 * Timeout and resource management utilities
 */

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::os::unix::io::RawFd;

/// Connection timeout manager
#[derive(Debug)]
pub struct TimeoutManager {
    connections: HashMap<RawFd, ConnectionInfo>,
    request_timeout: Duration,
    keep_alive_timeout: Duration,
    max_connections: usize,
}

/// Information about a connection
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub fd: RawFd,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub request_count: usize,
    pub bytes_read: usize,
    pub bytes_written: usize,
    pub state: ConnectionState,
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Reading,
    Processing,
    Writing,
    KeepAlive,
    Closing,
}

impl TimeoutManager {
    /// Create a new timeout manager
    pub fn new(request_timeout: Duration, keep_alive_timeout: Duration, max_connections: usize) -> Self {
        Self {
            connections: HashMap::new(),
            request_timeout,
            keep_alive_timeout,
            max_connections,
        }
    }

    /// Create with default timeouts
    pub fn with_defaults() -> Self {
        Self::new(
            Duration::from_secs(30),    // 30 second request timeout
            Duration::from_secs(60),    // 60 second keep-alive timeout
            1000,                       // Max 1000 connections
        )
    }

    /// Add a new connection
    pub fn add_connection(&mut self, fd: RawFd) -> Result<(), String> {
        if self.connections.len() >= self.max_connections {
            return Err("Maximum connections reached".to_string());
        }

        let now = Instant::now();
        let info = ConnectionInfo {
            fd,
            created_at: now,
            last_activity: now,
            request_count: 0,
            bytes_read: 0,
            bytes_written: 0,
            state: ConnectionState::Reading,
        };

        self.connections.insert(fd, info);
        Ok(())
    }

    /// Remove a connection
    pub fn remove_connection(&mut self, fd: RawFd) -> Option<ConnectionInfo> {
        self.connections.remove(&fd)
    }

    /// Update connection activity
    pub fn update_activity(&mut self, fd: RawFd, bytes_transferred: usize, is_read: bool) {
        if let Some(info) = self.connections.get_mut(&fd) {
            info.last_activity = Instant::now();
            if is_read {
                info.bytes_read += bytes_transferred;
            } else {
                info.bytes_written += bytes_transferred;
            }
        }
    }

    /// Update connection state
    pub fn update_state(&mut self, fd: RawFd, state: ConnectionState) {
        if let Some(info) = self.connections.get_mut(&fd) {
            info.state = state;
            info.last_activity = Instant::now();
        }
    }

    /// Increment request count
    pub fn increment_requests(&mut self, fd: RawFd) {
        if let Some(info) = self.connections.get_mut(&fd) {
            info.request_count += 1;
            info.last_activity = Instant::now();
        }
    }

    /// Get connections that have timed out
    pub fn get_timed_out_connections(&self) -> Vec<RawFd> {
        let now = Instant::now();
        let mut timed_out = Vec::new();

        for (fd, info) in &self.connections {
            let timeout = match info.state {
                ConnectionState::KeepAlive => self.keep_alive_timeout,
                _ => self.request_timeout,
            };

            if now.duration_since(info.last_activity) > timeout {
                timed_out.push(*fd);
            }
        }

        timed_out
    }

    /// Get connection info
    pub fn get_connection(&self, fd: RawFd) -> Option<&ConnectionInfo> {
        self.connections.get(&fd)
    }

    /// Get all connections
    pub fn get_all_connections(&self) -> &HashMap<RawFd, ConnectionInfo> {
        &self.connections
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Check if at connection limit
    pub fn is_at_limit(&self) -> bool {
        self.connections.len() >= self.max_connections
    }

    /// Get timeout statistics
    pub fn get_stats(&self) -> TimeoutStats {
        let now = Instant::now();
        let mut stats = TimeoutStats::default();

        for info in self.connections.values() {
            stats.total_connections += 1;
            stats.total_requests += info.request_count;
            stats.total_bytes_read += info.bytes_read;
            stats.total_bytes_written += info.bytes_written;

            let age = now.duration_since(info.created_at);
            if age > stats.max_connection_age {
                stats.max_connection_age = age;
            }

            let idle = now.duration_since(info.last_activity);
            if idle > stats.max_idle_time {
                stats.max_idle_time = idle;
            }

            match info.state {
                ConnectionState::Reading => stats.reading_connections += 1,
                ConnectionState::Processing => stats.processing_connections += 1,
                ConnectionState::Writing => stats.writing_connections += 1,
                ConnectionState::KeepAlive => stats.keepalive_connections += 1,
                ConnectionState::Closing => stats.closing_connections += 1,
            }
        }

        stats.max_connections = self.max_connections;
        stats
    }

    /// Clean up old connections (for testing/debugging)
    pub fn cleanup_old_connections(&mut self, max_age: Duration) -> usize {
        let now = Instant::now();
        let initial_count = self.connections.len();

        self.connections.retain(|_, info| {
            now.duration_since(info.created_at) <= max_age
        });

        initial_count - self.connections.len()
    }
}

/// Timeout statistics
#[derive(Debug, Default)]
pub struct TimeoutStats {
    pub total_connections: usize,
    pub total_requests: usize,
    pub total_bytes_read: usize,
    pub total_bytes_written: usize,
    pub max_connection_age: Duration,
    pub max_idle_time: Duration,
    pub max_connections: usize,
    pub reading_connections: usize,
    pub processing_connections: usize,
    pub writing_connections: usize,
    pub keepalive_connections: usize,
    pub closing_connections: usize,
}

impl TimeoutStats {
    /// Get connection utilization percentage
    pub fn utilization_percent(&self) -> f64 {
        if self.max_connections == 0 {
            0.0
        } else {
            (self.total_connections as f64 / self.max_connections as f64) * 100.0
        }
    }

    /// Get average requests per connection
    pub fn avg_requests_per_connection(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            self.total_requests as f64 / self.total_connections as f64
        }
    }

    /// Get total bytes transferred
    pub fn total_bytes_transferred(&self) -> usize {
        self.total_bytes_read + self.total_bytes_written
    }
}

/// Resource monitor for tracking system resources
#[derive(Debug)]
pub struct ResourceMonitor {
    start_time: SystemTime,
    peak_connections: usize,
    total_requests_served: usize,
    total_bytes_transferred: usize,
    error_count: usize,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            peak_connections: 0,
            total_requests_served: 0,
            total_bytes_transferred: 0,
            error_count: 0,
        }
    }

    /// Update peak connections
    pub fn update_peak_connections(&mut self, current: usize) {
        if current > self.peak_connections {
            self.peak_connections = current;
        }
    }

    /// Record a served request
    pub fn record_request(&mut self, bytes_transferred: usize) {
        self.total_requests_served += 1;
        self.total_bytes_transferred += bytes_transferred;
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    /// Get uptime
    pub fn uptime(&self) -> Duration {
        SystemTime::now().duration_since(self.start_time).unwrap_or(Duration::ZERO)
    }

    /// Get resource statistics
    pub fn get_stats(&self) -> ResourceStats {
        ResourceStats {
            uptime: self.uptime(),
            peak_connections: self.peak_connections,
            total_requests_served: self.total_requests_served,
            total_bytes_transferred: self.total_bytes_transferred,
            error_count: self.error_count,
            start_time: self.start_time,
        }
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub uptime: Duration,
    pub peak_connections: usize,
    pub total_requests_served: usize,
    pub total_bytes_transferred: usize,
    pub error_count: usize,
    pub start_time: SystemTime,
}

impl ResourceStats {
    /// Get requests per second
    pub fn requests_per_second(&self) -> f64 {
        let uptime_secs = self.uptime.as_secs_f64();
        if uptime_secs > 0.0 {
            self.total_requests_served as f64 / uptime_secs
        } else {
            0.0
        }
    }

    /// Get bytes per second
    pub fn bytes_per_second(&self) -> f64 {
        let uptime_secs = self.uptime.as_secs_f64();
        if uptime_secs > 0.0 {
            self.total_bytes_transferred as f64 / uptime_secs
        } else {
            0.0
        }
    }

    /// Get error rate percentage
    pub fn error_rate_percent(&self) -> f64 {
        if self.total_requests_served == 0 {
            0.0
        } else {
            (self.error_count as f64 / self.total_requests_served as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_manager() {
        let mut manager = TimeoutManager::with_defaults();

        // Add connection
        assert!(manager.add_connection(1).is_ok());
        assert_eq!(manager.connection_count(), 1);

        // Update activity
        manager.update_activity(1, 100, true);
        let info = manager.get_connection(1).unwrap();
        assert_eq!(info.bytes_read, 100);

        // Remove connection
        assert!(manager.remove_connection(1).is_some());
        assert_eq!(manager.connection_count(), 0);
    }

    #[test]
    fn test_resource_monitor() {
        let mut monitor = ResourceMonitor::new();

        monitor.record_request(1024);
        monitor.record_error();
        monitor.update_peak_connections(5);

        let stats = monitor.get_stats();
        assert_eq!(stats.total_requests_served, 1);
        assert_eq!(stats.total_bytes_transferred, 1024);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.peak_connections, 5);
    }
}
