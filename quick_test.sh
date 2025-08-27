#!/bin/bash

# Quick test script to verify basic functionality
# This is a simplified version to test core features quickly

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

SERVER_BINARY="./target/release/localhost-server"
CONFIG_FILE="config/test.conf"
TEST_PORT="8888"
TEST_HOST="127.0.0.1"
SERVER_PID=""

print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_failure() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Cleanup function
cleanup() {
    if [ -n "$SERVER_PID" ] && kill -0 $SERVER_PID 2>/dev/null; then
        print_info "Stopping server (PID: $SERVER_PID)"
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
}

trap cleanup EXIT

print_header "Quick Localhost HTTP Server Test"

# Build project
print_info "Building project..."
if cargo build --release; then
    print_success "Project built successfully"
else
    print_failure "Failed to build project"
    exit 1
fi

# Start server
print_info "Starting server..."
$SERVER_BINARY $CONFIG_FILE > /dev/null 2>&1 &
SERVER_PID=$!
sleep 2

if ! kill -0 $SERVER_PID 2>/dev/null; then
    print_failure "Server failed to start"
    exit 1
fi

print_success "Server started (PID: $SERVER_PID)"

# Test 1: Basic HTTP request
print_header "Test 1: Basic HTTP Request"
if timeout 5 curl -s -f http://$TEST_HOST:$TEST_PORT/ > /dev/null; then
    print_success "Basic HTTP request works"
else
    print_failure "Basic HTTP request failed"
fi

# Test 2: HTTP headers
print_header "Test 2: HTTP Headers"
HEADERS=$(timeout 5 curl -s -I http://$TEST_HOST:$TEST_PORT/ 2>/dev/null || echo "")
if echo "$HEADERS" | grep -q "HTTP/1.1"; then
    print_success "HTTP/1.1 protocol detected"
    echo "$HEADERS" | head -3
else
    print_failure "HTTP/1.1 protocol not detected"
fi

# Test 3: Thread count
print_header "Test 3: Single Thread Verification"
THREAD_COUNT=$(ps -o nlwp= -p $SERVER_PID 2>/dev/null | tr -d ' ')
if [ "$THREAD_COUNT" = "1" ]; then
    print_success "Server runs in single thread (NLWP: $THREAD_COUNT)"
else
    print_failure "Server using multiple threads (NLWP: $THREAD_COUNT)"
fi

# Test 4: Multiple requests
print_header "Test 4: Multiple Sequential Requests"
SUCCESS_COUNT=0
for i in {1..5}; do
    if timeout 5 curl -s -f http://$TEST_HOST:$TEST_PORT/ > /dev/null 2>&1; then
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    fi
done

if [ $SUCCESS_COUNT -eq 5 ]; then
    print_success "All 5 sequential requests succeeded"
else
    print_failure "Only $SUCCESS_COUNT/5 requests succeeded"
fi

# Test 5: CGI (if available)
print_header "Test 5: CGI Support"
if timeout 5 curl -s http://$TEST_HOST:$TEST_PORT/cgi-bin/hello.py > /dev/null 2>&1; then
    print_success "CGI request works"
else
    print_info "CGI request failed (may not be configured)"
fi

# Test 6: Error handling
print_header "Test 6: Error Handling"
STATUS_CODE=$(timeout 5 curl -s -w '%{http_code}' -o /dev/null http://$TEST_HOST:$TEST_PORT/nonexistent 2>/dev/null || echo "000")
if [ "$STATUS_CODE" = "404" ]; then
    print_success "404 error handling works"
else
    print_info "Error handling status: $STATUS_CODE"
fi

# Test 7: Server stability after bad request
print_header "Test 7: Server Stability"
echo -e "INVALID REQUEST\r\n\r\n" | timeout 2 nc $TEST_HOST $TEST_PORT >/dev/null 2>&1 || true

# Check if server is still responding
if timeout 5 curl -s -f http://$TEST_HOST:$TEST_PORT/ > /dev/null 2>&1; then
    print_success "Server remains stable after malformed request"
else
    print_failure "Server became unresponsive after malformed request"
fi

print_header "Test Summary"
print_info "Quick test completed!"
print_info "For comprehensive testing, run: ./test_all_audit_requirements.sh"
print_info "For interactive demo, run: ./demo_audit_tests.sh"

cleanup
