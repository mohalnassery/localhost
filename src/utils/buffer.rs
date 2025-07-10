/*!
 * Buffer management for I/O operations
 */

use std::io::{self, Read, Write};
use std::os::unix::io::RawFd;

/// A growable buffer for I/O operations
pub struct Buffer {
    data: Vec<u8>,
    read_pos: usize,
    write_pos: usize,
}

impl Buffer {
    /// Create a new buffer with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0; capacity],
            read_pos: 0,
            write_pos: 0,
        }
    }

    /// Get the available data for reading
    pub fn readable_data(&self) -> &[u8] {
        &self.data[self.read_pos..self.write_pos]
    }

    /// Get the available space for writing
    pub fn writable_space(&mut self) -> &mut [u8] {
        &mut self.data[self.write_pos..]
    }

    /// Mark bytes as consumed (advance read position)
    pub fn consume(&mut self, bytes: usize) {
        self.read_pos = std::cmp::min(self.read_pos + bytes, self.write_pos);

        // If we've read everything, reset positions
        if self.read_pos == self.write_pos {
            self.read_pos = 0;
            self.write_pos = 0;
        }
    }

    /// Mark bytes as written (advance write position)
    pub fn advance_write(&mut self, bytes: usize) {
        self.write_pos = std::cmp::min(self.write_pos + bytes, self.data.len());
    }

    /// Get the number of bytes available for reading
    pub fn readable_bytes(&self) -> usize {
        self.write_pos - self.read_pos
    }

    /// Get the number of bytes available for writing
    pub fn writable_bytes(&self) -> usize {
        self.data.len() - self.write_pos
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.readable_bytes() == 0
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
    }

    /// Ensure buffer has enough space for writing
    pub fn ensure_writable_space(&mut self, needed: usize) {
        if self.writable_bytes() < needed {
            // Try to compact first
            if self.read_pos > 0 {
                self.data.copy_within(self.read_pos..self.write_pos, 0);
                self.write_pos -= self.read_pos;
                self.read_pos = 0;
            }

            // If still not enough space, grow the buffer
            if self.writable_bytes() < needed {
                let new_size = std::cmp::max(self.data.len() * 2, self.data.len() + needed);
                self.data.resize(new_size, 0);
            }
        }
    }

    /// Read from a file descriptor into the buffer
    pub fn read_from_fd(&mut self, fd: RawFd) -> io::Result<usize> {
        self.ensure_writable_space(1024);

        let bytes_read = unsafe {
            libc::read(
                fd,
                self.data[self.write_pos..].as_mut_ptr() as *mut libc::c_void,
                self.writable_bytes(),
            )
        };

        if bytes_read == -1 {
            let error = io::Error::last_os_error();
            match error.raw_os_error() {
                Some(libc::EAGAIN) | Some(libc::EWOULDBLOCK) => Ok(0),
                _ => Err(error),
            }
        } else {
            self.advance_write(bytes_read as usize);
            Ok(bytes_read as usize)
        }
    }

    /// Write from the buffer to a file descriptor
    pub fn write_to_fd(&mut self, fd: RawFd) -> io::Result<usize> {
        if self.is_empty() {
            return Ok(0);
        }

        let bytes_written = unsafe {
            libc::write(
                fd,
                self.readable_data().as_ptr() as *const libc::c_void,
                self.readable_bytes(),
            )
        };

        if bytes_written == -1 {
            let error = io::Error::last_os_error();
            match error.raw_os_error() {
                Some(libc::EAGAIN) | Some(libc::EWOULDBLOCK) => Ok(0),
                _ => Err(error),
            }
        } else {
            self.consume(bytes_written as usize);
            Ok(bytes_written as usize)
        }
    }

    /// Append data to the buffer
    pub fn append(&mut self, data: &[u8]) {
        self.ensure_writable_space(data.len());
        self.data[self.write_pos..self.write_pos + data.len()].copy_from_slice(data);
        self.advance_write(data.len());
    }

    /// Get all data as a string (for debugging)
    pub fn as_string(&self) -> String {
        String::from_utf8_lossy(self.readable_data()).to_string()
    }
}
