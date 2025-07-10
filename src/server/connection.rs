/*!
 * Connection management
 */

use crate::http::HttpRequestParser;
use crate::utils::buffer::Buffer;
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::time::{Duration, Instant};

/// Connection state
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Reading,
    Writing,
    KeepAlive,
    Closed,
}

/// Individual client connection
pub struct Connection {
    pub fd: RawFd,
    pub state: ConnectionState,
    pub read_buffer: Buffer,
    pub write_buffer: Buffer,
    pub last_activity: Instant,
    pub keep_alive: bool,
    pub request_count: usize,
    pub http_parser: HttpRequestParser,
}

impl Connection {
    pub fn new(fd: RawFd) -> Self {
        Self {
            fd,
            state: ConnectionState::Reading,
            read_buffer: Buffer::new(8192),
            write_buffer: Buffer::new(8192),
            last_activity: Instant::now(),
            keep_alive: false,
            request_count: 0,
            http_parser: HttpRequestParser::new(),
        }
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Check if connection has timed out
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    /// Reset connection for keep-alive
    pub fn reset_for_keep_alive(&mut self) {
        self.read_buffer.clear();
        self.write_buffer.clear();
        self.state = ConnectionState::Reading;
        self.request_count += 1;
        self.http_parser.reset();
        self.touch();
    }

    /// Check if connection should be closed
    pub fn should_close(&self) -> bool {
        matches!(self.state, ConnectionState::Closed) ||
        (!self.keep_alive && self.write_buffer.is_empty())
    }
}

/// Connection manager
pub struct ConnectionManager {
    connections: HashMap<RawFd, Connection>,
    timeout: Duration,
}

impl ConnectionManager {
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            connections: HashMap::new(),
            timeout: Duration::from_secs(timeout_seconds),
        }
    }

    /// Add a new connection
    pub fn add_connection(&mut self, fd: RawFd) {
        let connection = Connection::new(fd);
        self.connections.insert(fd, connection);
    }

    /// Get a connection by file descriptor
    pub fn get_connection(&self, fd: RawFd) -> Option<&Connection> {
        self.connections.get(&fd)
    }

    /// Get a mutable connection by file descriptor
    pub fn get_connection_mut(&mut self, fd: RawFd) -> Option<&mut Connection> {
        self.connections.get_mut(&fd)
    }

    /// Remove a connection
    pub fn remove_connection(&mut self, fd: RawFd) -> Option<Connection> {
        self.connections.remove(&fd)
    }

    /// Get all connection file descriptors
    pub fn get_all_fds(&self) -> Vec<RawFd> {
        self.connections.keys().copied().collect()
    }

    /// Remove timed out connections
    pub fn cleanup_timed_out(&mut self) -> Vec<RawFd> {
        let mut timed_out = Vec::new();

        self.connections.retain(|&fd, conn| {
            if conn.is_timed_out(self.timeout) {
                timed_out.push(fd);
                false
            } else {
                true
            }
        });

        timed_out
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Check if a connection exists
    pub fn has_connection(&self, fd: RawFd) -> bool {
        self.connections.contains_key(&fd)
    }
}
