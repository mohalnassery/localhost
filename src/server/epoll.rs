/*!
 * Epoll wrapper for I/O multiplexing
 */

use crate::error::{ServerError, ServerResult};
use libc::{self, c_int, c_void};
use std::os::unix::io::RawFd;

/// Epoll events
pub const EPOLLIN: u32 = libc::EPOLLIN as u32;
pub const EPOLLOUT: u32 = libc::EPOLLOUT as u32;
pub const EPOLLERR: u32 = libc::EPOLLERR as u32;
pub const EPOLLHUP: u32 = libc::EPOLLHUP as u32;
pub const EPOLLET: u32 = libc::EPOLLET as u32;

/// Epoll control operations
pub const EPOLL_CTL_ADD: c_int = libc::EPOLL_CTL_ADD;
pub const EPOLL_CTL_MOD: c_int = libc::EPOLL_CTL_MOD;
pub const EPOLL_CTL_DEL: c_int = libc::EPOLL_CTL_DEL;

/// Epoll event structure - use libc's epoll_event directly
pub type EpollEvent = libc::epoll_event;

/// Helper functions for EpollEvent
pub fn create_epoll_event(events: u32, fd: RawFd) -> EpollEvent {
    EpollEvent {
        events,
        u64: fd as u64,
    }
}

pub fn get_fd_from_event(event: &EpollEvent) -> RawFd {
    event.u64 as RawFd
}

/// Epoll wrapper
pub struct Epoll {
    epfd: RawFd,
}

impl Epoll {
    /// Create a new epoll instance
    pub fn new() -> ServerResult<Self> {
        let epfd = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
        if epfd == -1 {
            return Err(ServerError::Io(std::io::Error::last_os_error()));
        }

        Ok(Self { epfd })
    }

    /// Add a file descriptor to epoll
    pub fn add(&self, fd: RawFd, events: u32) -> ServerResult<()> {
        let mut event = libc::epoll_event {
            events: events,
            u64: fd as u64,
        };

        let result = unsafe {
            libc::epoll_ctl(self.epfd, EPOLL_CTL_ADD, fd, &mut event as *mut libc::epoll_event)
        };

        if result == -1 {
            return Err(ServerError::Io(std::io::Error::last_os_error()));
        }

        Ok(())
    }

    /// Modify a file descriptor in epoll
    pub fn modify(&self, fd: RawFd, events: u32) -> ServerResult<()> {
        let mut event = libc::epoll_event {
            events: events,
            u64: fd as u64,
        };

        let result = unsafe {
            libc::epoll_ctl(self.epfd, EPOLL_CTL_MOD, fd, &mut event as *mut libc::epoll_event)
        };

        if result == -1 {
            return Err(ServerError::Io(std::io::Error::last_os_error()));
        }

        Ok(())
    }

    /// Remove a file descriptor from epoll
    pub fn remove(&self, fd: RawFd) -> ServerResult<()> {
        let result = unsafe {
            libc::epoll_ctl(self.epfd, EPOLL_CTL_DEL, fd, std::ptr::null_mut())
        };

        if result == -1 {
            return Err(ServerError::Io(std::io::Error::last_os_error()));
        }

        Ok(())
    }

    /// Wait for events
    pub fn wait(&self, events: &mut [EpollEvent], timeout: i32) -> ServerResult<usize> {
        let result = unsafe {
            libc::epoll_wait(
                self.epfd,
                events.as_mut_ptr(),
                events.len() as c_int,
                timeout,
            )
        };

        if result == -1 {
            return Err(ServerError::Io(std::io::Error::last_os_error()));
        }

        Ok(result as usize)
    }
}

impl Drop for Epoll {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.epfd);
        }
    }
}
