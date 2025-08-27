# Localhost HTTP Server - Audit Testing Guide

## Overview

This repository contains a comprehensive audit testing suite for the Localhost HTTP Server project. All audit requirements are thoroughly tested with automated scripts and detailed documentation.

## Quick Start

### 1. Run Demo
```bash
./demo_audit_tests.sh
```

### 2. Run All Audit Tests
```bash
./test_all_audit_requirements.sh
```

### 3. View Audit Answers
```bash
cat auditanswers.md
```

## Test Structure

### Main Test Script: `test_all_audit_requirements.sh`

Comprehensive test suite covering all audit requirements:

- **HTTP Server Basics** - Protocol compliance, request/response handling
- **I/O Multiplexing** - Epoll usage verification with system call tracing
- **Single Thread** - Thread count validation under concurrent load
- **HTTP Methods** - GET, POST, DELETE method support and status codes
- **Error Handling** - Custom error pages, malformed request handling
- **Configuration** - Multi-port, virtual hosts, route configuration
- **CGI Support** - Python CGI execution with environment variables
- **Sessions/Cookies** - Session management and cookie persistence
- **File Uploads** - Upload handling with size limits and integrity checks
- **Stress Testing** - Performance, availability, and memory stability
- **Browser Compatibility** - Real browser request header handling

### Individual Test Categories

Run specific test categories:

```bash
./test_all_audit_requirements.sh basic      # HTTP server basics
./test_all_audit_requirements.sh epoll      # I/O multiplexing
./test_all_audit_requirements.sh methods    # HTTP methods
./test_all_audit_requirements.sh cgi        # CGI support
./test_all_audit_requirements.sh stress     # Performance testing
```

### Demo Script: `demo_audit_tests.sh`

Interactive demo showcasing key features:
- Quick basic tests
- Full comprehensive testing
- Individual category selection
- Audit answers viewing

## Audit Documentation

### `auditanswers.md`

Complete audit answers with:
- ✅ YES/NO responses to all audit questions
- 🔧 Actionable proof commands
- 📊 Expected outputs for verification
- 🏗️ Architecture diagrams (Mermaid)
- 📈 Performance benchmarks
- 🔒 Security testing results

### Key Features Demonstrated

1. **HTTP/1.1 Compliance**
   - Proper protocol headers
   - Status code handling
   - Keep-alive connections

2. **Epoll-based I/O Multiplexing**
   - Single epoll instance
   - Non-blocking operations
   - Event-driven architecture

3. **Single-threaded Design**
   - Thread count verification
   - Stability under load
   - No race conditions

4. **Multi-method Support**
   - GET for static files
   - POST for form data and uploads
   - DELETE for resource management

5. **CGI Integration**
   - Python script execution
   - Environment variable setup
   - Input/output handling

6. **Session Management**
   - Cookie setting and retrieval
   - Session persistence
   - Secure attributes

7. **Configuration Flexibility**
   - Multiple ports
   - Virtual hosts
   - Route-based settings

8. **Error Handling**
   - Custom error pages
   - Proper status codes
   - Graceful failure handling

9. **Performance & Reliability**
   - >99.5% availability under stress
   - Memory leak prevention
   - Concurrent connection handling

## Prerequisites

### Required Tools
- Rust (cargo)
- curl
- netcat (nc)
- Python3 (for CGI tests)

### Optional Tools (for enhanced testing)
- siege (stress testing)
- Apache Bench (ab)
- strace (system call tracing)
- valgrind (memory leak detection)

## File Structure

```
├── test_all_audit_requirements.sh  # Main test suite
├── demo_audit_tests.sh             # Interactive demo
├── auditanswers.md                  # Complete audit answers
├── config/
│   ├── test.conf                    # Test configuration
│   └── test-listing.conf            # Directory listing config
├── cgi-bin/
│   ├── hello.py                     # Python CGI script
│   └── session_test.py              # Session testing script
├── www/
│   ├── index.html                   # Static content
│   └── static/                      # Static files
└── tests/
    ├── run_tests.sh                 # Additional test runner
    ├── stress_test.sh               # Stress testing
    └── integration_tests.rs         # Rust integration tests
```

## Expected Results

### Successful Test Run
```
╔══════════════════════════════════════════════════════════════════════════════╗
║ COMPREHENSIVE AUDIT TEST REPORT                                             ║
╚══════════════════════════════════════════════════════════════════════════════╝

Test Execution Summary:
=======================
Total Tests Run:    50+
Tests Passed:       50+
Tests Failed:       0
Success Rate:       100%

✓ ALL AUDIT REQUIREMENTS PASSED

🎉 The localhost HTTP server is FULLY COMPLIANT with all audit requirements!

Ready for production deployment! 🚀
```

## Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```bash
   # Kill any existing server processes
   pkill -f localhost-server
   ```

2. **Permission Denied (CGI)**
   ```bash
   # Make CGI scripts executable
   chmod +x cgi-bin/*.py
   ```

3. **Missing Dependencies**
   ```bash
   # Install required tools
   sudo apt-get install curl netcat-openbsd python3
   ```

### Debug Mode

Run tests with verbose output:
```bash
RUST_LOG=debug ./test_all_audit_requirements.sh
```

## Contributing

When adding new tests:
1. Follow the existing test pattern
2. Include clear success/failure criteria
3. Add corresponding audit answer documentation
4. Update this README if needed

## License

This testing suite is part of the Localhost HTTP Server project and follows the same license terms.
