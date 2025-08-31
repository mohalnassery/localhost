/*!
 * Core server implementation
 */

use crate::config::Config;
use crate::error::{ServerError, ServerResult, HttpStatus};
use crate::error::pages::ErrorPageManager;
use crate::http::HttpResponse;
use crate::http::methods::MethodHandler;
use crate::session::SessionManager;
use crate::server::connection::{ConnectionManager, ConnectionState};
use crate::server::epoll::{Epoll, EPOLLIN, EPOLLOUT, EPOLLERR, EPOLLHUP, create_epoll_event, get_fd_from_event};
use crate::server::socket::{
    accept_connection, bind_socket, close_socket, create_tcp_socket, listen_socket,
};
use std::collections::HashMap;
use std::os::unix::io::RawFd;

/// Main HTTP server structure
pub struct Server {
    config: Config,
    epoll: Epoll,
    server_sockets: HashMap<RawFd, (String, u16)>, // fd -> (host, port)
    connection_manager: ConnectionManager,
    method_handler: MethodHandler,
    error_manager: ErrorPageManager,
    #[allow(dead_code)] // TODO: Implement session management
    session_manager: SessionManager,
    running: bool,
}

impl Server {
    /// Create a new server with the given configuration
    pub fn new(config: Config) -> ServerResult<Self> {
        let epoll = Epoll::new()?;
        let connection_manager = ConnectionManager::new(30); // 30 second timeout
        let method_handler = MethodHandler::new(config.clone());

        // Create error manager from first server's configuration
        let error_manager = if let Some(server) = config.servers.first() {
            ErrorPageManager::from_config(server)
        } else {
            ErrorPageManager::new()
        };

        Ok(Server {
            config,
            epoll,
            server_sockets: HashMap::new(),
            connection_manager,
            method_handler,
            error_manager,
            session_manager: SessionManager::with_defaults(),
            running: false,
        })
    }

    /// Run the server (main event loop)
    pub fn run(&mut self) -> ServerResult<()> {
        println!("Starting localhost HTTP server...");

        // Create and bind server sockets
        self.setup_server_sockets()?;

        println!("Server listening on {} socket(s)", self.server_sockets.len());
        for (_, (host, port)) in &self.server_sockets {
            println!("  http://{}:{}", host, port);
        }

        self.running = true;

        // Main event loop
        self.event_loop()
    }

    /// Setup server sockets for all configured servers
    fn setup_server_sockets(&mut self) -> ServerResult<()> {
        for server_config in &self.config.servers {
            for &port in &server_config.ports {
                let socket_fd = create_tcp_socket()?;

                // Bind to address
                bind_socket(socket_fd, &server_config.host, port)?;

                // Start listening
                listen_socket(socket_fd, 128)?;

                // Add to epoll for accepting connections
                self.epoll.add(socket_fd, EPOLLIN)?;

                // Store socket info
                self.server_sockets.insert(socket_fd, (server_config.host.clone(), port));

                println!("Bound to {}:{}", server_config.host, port);
            }
        }

        if self.server_sockets.is_empty() {
            return Err(ServerError::Config("No server sockets configured".to_string()));
        }

        Ok(())
    }

    /// Main event loop
    fn event_loop(&mut self) -> ServerResult<()> {
        let mut events = vec![create_epoll_event(0, 0); crate::defaults::MAX_EVENTS];

        while self.running {
            // Wait for events with 1 second timeout
            let event_count = self.epoll.wait(&mut events, 1000)?;

            // Check for timed out connections
            let _ = self.cleanup_timed_out_connections();

            // Process events
            for i in 0..event_count {
                let event = &events[i];
                let fd = get_fd_from_event(event);
                let event_flags = event.events; // Copy to avoid packed field access

                if let Err(e) = self.handle_event(fd, event_flags) {
                    eprintln!("Error handling event for fd {}: {}", fd, e);
                    self.cleanup_connection(fd);
                }
            }

            // Cleanup timed out connections
            let _ = self.cleanup_timed_out_connections();
        }

        self.shutdown()
    }

    /// Handle a single epoll event
    fn handle_event(&mut self, fd: RawFd, events: u32) -> ServerResult<()> {
        // Check for errors first
        if events & (EPOLLERR | EPOLLHUP) != 0 {
            self.cleanup_connection(fd);
            return Ok(());
        }

        // Check if this is a server socket (accepting new connections)
        if self.server_sockets.contains_key(&fd) {
            if events & EPOLLIN != 0 {
                self.accept_new_connections(fd)?;
            }
            return Ok(());
        }

        // Handle client connection events
        if events & EPOLLIN != 0 {
            self.handle_read(fd)?;
        }

        if events & EPOLLOUT != 0 {
            self.handle_write(fd)?;
        }

        Ok(())
    }

    /// Accept new connections on a server socket
    fn accept_new_connections(&mut self, server_fd: RawFd) -> ServerResult<()> {
        loop {
            match accept_connection(server_fd)? {
                Some(client_fd) => {
                    // Add client to epoll for reading
                    self.epoll.add(client_fd, EPOLLIN)?;

                    // Add to connection manager
                    match self.connection_manager.add_connection(client_fd) {
                        Ok(()) => {
                            println!("New connection accepted: fd {}", client_fd);
                        }
                        Err(e) => {
                            eprintln!("Failed to add connection {}: {}", client_fd, e);
                            self.connection_manager.record_error();
                            close_socket(client_fd);
                            self.epoll.remove(client_fd)?;
                        }
                    }
                }
                None => break, // No more connections to accept
            }
        }
        Ok(())
    }

    /// Handle read event on client connection
    fn handle_read(&mut self, fd: RawFd) -> ServerResult<()> {
        if let Some(connection) = self.connection_manager.get_connection_mut(fd) {
            connection.touch();

            match connection.read_buffer.read_from_fd(fd) {
                Ok(0) => {
                    // Client closed connection
                    self.cleanup_connection(fd);
                }
                Ok(_bytes_read) => {
                    // Try to parse HTTP request
                    let data = connection.read_buffer.readable_data();
                    match connection.http_parser.parse(data) {
                        Ok((Some(request), consumed)) => {
                            // Consume only the parsed data from buffer
                            connection.read_buffer.consume(consumed);

                            // Release the connection reference before calling update_activity

                            // Update connection activity
                            self.connection_manager.update_activity(fd, consumed, true);

                            // Process the request and generate response
                            self.process_http_request(fd, request)?;
                        }
                        Ok((None, consumed)) => {
                            // Need more data to complete parsing
                            // Consume any processed data
                            if consumed > 0 {
                                connection.read_buffer.consume(consumed);
                            }
                        }
                        Err(e) => {
                            eprintln!("HTTP parsing error on fd {}: {}", fd, e);
                            self.connection_manager.record_error();
                            self.send_error_response(fd, HttpStatus::BadRequest, Some("Invalid HTTP request"))?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Read error on fd {}: {}", fd, e);
                    self.cleanup_connection(fd);
                }
            }
        }
        Ok(())
    }

    /// Handle write event on client connection
    fn handle_write(&mut self, fd: RawFd) -> ServerResult<()> {
        if let Some(connection) = self.connection_manager.get_connection_mut(fd) {
            connection.touch();

            match connection.write_buffer.write_to_fd(fd) {
                Ok(_bytes_written) => {
                    // Check if we've finished writing the response
                    if connection.write_buffer.is_empty() {
                        if connection.keep_alive {
                            // Reset for next request
                            connection.reset_for_keep_alive();
                            // Switch back to reading mode
                            self.epoll.modify(fd, EPOLLIN)?;
                        } else {
                            // Close connection
                            self.cleanup_connection(fd);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Write error on fd {}: {}", fd, e);
                    self.cleanup_connection(fd);
                }
            }
        }
        Ok(())
    }

    /// Process HTTP request and generate response
    fn process_http_request(&mut self, fd: RawFd, request: crate::http::HttpRequest) -> ServerResult<()> {
        // Use the method handler to process the request
        let response = self.method_handler.handle_request(&request)
            .unwrap_or_else(|e| {
                eprintln!("Error processing request: {}", e);
                self.error_manager.generate_error_response(HttpStatus::InternalServerError, Some("Internal server error"))
            });

        // Record the completed request
        let response_size = response.to_bytes().len();
        self.connection_manager.record_request(fd, response_size);

        self.send_response(fd, response, request.keep_alive())
    }

    /// Send an error response
    fn send_error_response(&mut self, fd: RawFd, status: HttpStatus, message: Option<&str>) -> ServerResult<()> {
        let response = self.error_manager.generate_error_response(status, message);
        self.send_response(fd, response, false)
    }

    /// Send HTTP response to client
    fn send_response(&mut self, fd: RawFd, mut response: HttpResponse, keep_alive: bool) -> ServerResult<()> {
        response.set_keep_alive(keep_alive);
        let response_bytes = response.to_bytes();

        if let Some(connection) = self.connection_manager.get_connection_mut(fd) {
            connection.write_buffer.append(&response_bytes);
            connection.keep_alive = keep_alive;

            // Switch to writing mode and modify epoll to watch for write events
            connection.state = ConnectionState::Writing;
            self.epoll.modify(fd, EPOLLOUT)?;
        }

        Ok(())
    }

    /// Cleanup a connection
    fn cleanup_connection(&mut self, fd: RawFd) {
        if let Some(_connection) = self.connection_manager.remove_connection(fd) {
            let _ = self.epoll.remove(fd);
            close_socket(fd);
        }
    }

    /// Cleanup timed out connections
    fn cleanup_timed_out_connections(&mut self) -> ServerResult<()> {
        let timed_out = self.connection_manager.cleanup_expired();
        for fd in timed_out {
            println!("Connection {} timed out, cleaning up", fd);
            let _ = self.epoll.remove(fd);
            close_socket(fd);
        }
        Ok(())
    }

    /// Shutdown the server
    fn shutdown(&mut self) -> ServerResult<()> {
        println!("Shutting down server...");

        // Close all client connections
        for fd in self.connection_manager.get_all_fds() {
            self.cleanup_connection(fd);
        }

        // Close server sockets
        for (&fd, _) in &self.server_sockets {
            let _ = self.epoll.remove(fd);
            close_socket(fd);
        }

        self.server_sockets.clear();
        println!("Server shutdown complete");

        Ok(())
    }

    /// Get server statistics
    pub fn get_stats(&self) -> (crate::utils::TimeoutStats, crate::utils::ResourceStats) {
        (
            self.connection_manager.get_timeout_stats(),
            self.connection_manager.get_resource_stats(),
        )
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if self.running {
            let _ = self.shutdown();
        }
    }
}
