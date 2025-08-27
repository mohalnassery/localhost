/*!
 * Socket management
 */

use crate::error::{ServerError, ServerResult};
use libc::{self, c_int, sockaddr, sockaddr_in, socklen_t};
use std::mem;
use std::os::unix::io::RawFd;

/// Create a non-blocking TCP socket
pub fn create_tcp_socket() -> ServerResult<RawFd> {
    let socket_fd = unsafe {
        libc::socket(libc::AF_INET, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0)
    };

    if socket_fd == -1 {
        return Err(ServerError::Io(std::io::Error::last_os_error()));
    }

    // Set socket to non-blocking
    set_nonblocking(socket_fd)?;

    // Set SO_REUSEADDR to allow quick restart
    set_reuseaddr(socket_fd)?;

    Ok(socket_fd)
}

/// Set socket to non-blocking mode
pub fn set_nonblocking(fd: RawFd) -> ServerResult<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags == -1 {
        return Err(ServerError::Io(std::io::Error::last_os_error()));
    }

    let result = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    if result == -1 {
        return Err(ServerError::Io(std::io::Error::last_os_error()));
    }

    Ok(())
}

/// Set SO_REUSEADDR option
pub fn set_reuseaddr(fd: RawFd) -> ServerResult<()> {
    let optval: c_int = 1;
    let result = unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_REUSEADDR,
            &optval as *const c_int as *const libc::c_void,
            mem::size_of::<c_int>() as socklen_t,
        )
    };

    if result == -1 {
        return Err(ServerError::Io(std::io::Error::last_os_error()));
    }

    Ok(())
}

/// Bind socket to address and port
pub fn bind_socket(fd: RawFd, host: &str, port: u16) -> ServerResult<()> {
    let addr = create_sockaddr_in(host, port)?;

    let result = unsafe {
        libc::bind(
            fd,
            &addr as *const sockaddr_in as *const sockaddr,
            mem::size_of::<sockaddr_in>() as socklen_t,
        )
    };

    if result == -1 {
        return Err(ServerError::Io(std::io::Error::last_os_error()));
    }

    Ok(())
}

/// Listen on socket
pub fn listen_socket(fd: RawFd, backlog: c_int) -> ServerResult<()> {
    let result = unsafe { libc::listen(fd, backlog) };

    if result == -1 {
        return Err(ServerError::Io(std::io::Error::last_os_error()));
    }

    Ok(())
}

/// Accept a connection
pub fn accept_connection(fd: RawFd) -> ServerResult<Option<RawFd>> {
    let client_fd = unsafe { libc::accept(fd, std::ptr::null_mut(), std::ptr::null_mut()) };

    if client_fd == -1 {
        let error = std::io::Error::last_os_error();
        match error.raw_os_error() {
            Some(libc::EAGAIN) => Ok(None),
            _ => Err(ServerError::Io(error)),
        }
    } else {
        // Set client socket to non-blocking
        set_nonblocking(client_fd)?;
        Ok(Some(client_fd))
    }
}

/// Create sockaddr_in structure
fn create_sockaddr_in(host: &str, port: u16) -> ServerResult<sockaddr_in> {
    let mut addr: sockaddr_in = unsafe { mem::zeroed() };
    addr.sin_family = libc::AF_INET as u16;
    addr.sin_port = port.to_be();

    // Parse IP address
    let ip_addr = if host == "0.0.0.0" || host == "*" {
        libc::INADDR_ANY
    } else {
        parse_ip_address(host)?
    };

    addr.sin_addr.s_addr = ip_addr.to_be();
    Ok(addr)
}

/// Parse IP address string to u32
fn parse_ip_address(ip: &str) -> ServerResult<u32> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return Err(ServerError::Config(format!("Invalid IP address: {}", ip)));
    }

    let mut addr: u32 = 0;
    for (i, part) in parts.iter().enumerate() {
        let octet: u8 = part.parse()
            .map_err(|_| ServerError::Config(format!("Invalid IP address: {}", ip)))?;
        addr |= (octet as u32) << (8 * (3 - i));
    }

    Ok(addr)
}

/// Close socket
pub fn close_socket(fd: RawFd) {
    unsafe {
        libc::close(fd);
    }
}
