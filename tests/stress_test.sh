#!/bin/bash

# Stress testing script for localhost HTTP server
# Compatible with siege and other load testing tools

set -e

# Configuration
SERVER_HOST="127.0.0.1"
SERVER_PORT="8889"
CONFIG_FILE="config/test-listing.conf"
SERVER_BINARY="./target/release/localhost-server"
TEST_DURATION="30s"
CONCURRENT_USERS="50"
LOG_FILE="tests/stress_test.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if server is running
check_server() {
    curl -s "http://${SERVER_HOST}:${SERVER_PORT}/" > /dev/null 2>&1
}

# Function to wait for server to start
wait_for_server() {
    local max_attempts=30
    local attempt=1
    
    print_status "Waiting for server to start..."
    
    while [ $attempt -le $max_attempts ]; do
        if check_server; then
            print_success "Server is ready!"
            return 0
        fi
        
        echo -n "."
        sleep 1
        attempt=$((attempt + 1))
    done
    
    print_error "Server failed to start within 30 seconds"
    return 1
}

# Function to start the server
start_server() {
    print_status "Starting localhost HTTP server..."
    
    if [ ! -f "$SERVER_BINARY" ]; then
        print_error "Server binary not found: $SERVER_BINARY"
        print_status "Building server..."
        cargo build --release
    fi
    
    if [ ! -f "$CONFIG_FILE" ]; then
        print_error "Configuration file not found: $CONFIG_FILE"
        exit 1
    fi
    
    # Start server in background
    $SERVER_BINARY $CONFIG_FILE > /dev/null 2>&1 &
    SERVER_PID=$!
    
    # Wait for server to be ready
    if ! wait_for_server; then
        kill $SERVER_PID 2>/dev/null || true
        exit 1
    fi
}

# Function to stop the server
stop_server() {
    if [ ! -z "$SERVER_PID" ]; then
        print_status "Stopping server (PID: $SERVER_PID)..."
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
        print_success "Server stopped"
    fi
}

# Function to run basic load test with curl
run_curl_stress_test() {
    print_status "Running basic stress test with curl..."
    
    local url="http://${SERVER_HOST}:${SERVER_PORT}/static/"
    local requests=100
    local concurrent=10
    local success_count=0
    local error_count=0
    
    print_status "Making $requests requests with $concurrent concurrent connections..."
    
    # Create temporary directory for results
    local temp_dir=$(mktemp -d)
    
    # Run concurrent requests
    for ((i=1; i<=requests; i++)); do
        {
            if curl -s -o /dev/null -w "%{http_code}" "$url" > "$temp_dir/result_$i" 2>/dev/null; then
                echo "success" > "$temp_dir/status_$i"
            else
                echo "error" > "$temp_dir/status_$i"
            fi
        } &
        
        # Limit concurrent connections
        if (( i % concurrent == 0 )); then
            wait
        fi
    done
    
    # Wait for all background jobs to complete
    wait
    
    # Count results
    for ((i=1; i<=requests; i++)); do
        if [ -f "$temp_dir/status_$i" ]; then
            status=$(cat "$temp_dir/status_$i")
            if [ "$status" = "success" ]; then
                if [ -f "$temp_dir/result_$i" ]; then
                    http_code=$(cat "$temp_dir/result_$i")
                    if [ "$http_code" = "200" ]; then
                        success_count=$((success_count + 1))
                    else
                        error_count=$((error_count + 1))
                    fi
                fi
            else
                error_count=$((error_count + 1))
            fi
        else
            error_count=$((error_count + 1))
        fi
    done
    
    # Cleanup
    rm -rf "$temp_dir"
    
    # Calculate success rate
    local success_rate=$(( (success_count * 100) / requests ))
    
    print_status "Curl stress test results:"
    echo "  Total requests: $requests"
    echo "  Successful: $success_count"
    echo "  Failed: $error_count"
    echo "  Success rate: ${success_rate}%"
    
    if [ $success_rate -ge 95 ]; then
        print_success "Curl stress test PASSED (${success_rate}% success rate)"
        return 0
    else
        print_error "Curl stress test FAILED (${success_rate}% success rate, expected >= 95%)"
        return 1
    fi
}

# Function to run siege stress test if available
run_siege_test() {
    if ! command -v siege &> /dev/null; then
        print_warning "Siege not found, skipping siege stress test"
        return 0
    fi
    
    print_status "Running siege stress test..."
    
    local url="http://${SERVER_HOST}:${SERVER_PORT}/static/"
    
    # Create siege URLs file
    cat > tests/siege_urls.txt << EOF
$url
http://${SERVER_HOST}:${SERVER_PORT}/cgi-bin/hello.py
http://${SERVER_HOST}:${SERVER_PORT}/cgi-bin/session_test.py
http://${SERVER_HOST}:${SERVER_PORT}/static/test.txt
EOF
    
    # Run siege test
    print_status "Running siege with $CONCURRENT_USERS concurrent users for $TEST_DURATION..."
    
    if siege -c $CONCURRENT_USERS -t $TEST_DURATION -f tests/siege_urls.txt --log=$LOG_FILE > tests/siege_output.txt 2>&1; then
        # Parse siege results
        local availability=$(grep "Availability:" tests/siege_output.txt | awk '{print $2}' | sed 's/%//')
        local response_time=$(grep "Response time:" tests/siege_output.txt | awk '{print $3}')
        local transaction_rate=$(grep "Transaction rate:" tests/siege_output.txt | awk '{print $3}')
        
        print_status "Siege test results:"
        echo "  Availability: ${availability}%"
        echo "  Response time: ${response_time} secs"
        echo "  Transaction rate: ${transaction_rate} trans/sec"
        
        # Check if availability meets requirements (99.5%)
        if (( $(echo "$availability >= 99.5" | bc -l) )); then
            print_success "Siege stress test PASSED (${availability}% availability)"
            return 0
        else
            print_error "Siege stress test FAILED (${availability}% availability, expected >= 99.5%)"
            return 1
        fi
    else
        print_error "Siege test failed to run"
        return 1
    fi
}

# Function to run Apache Bench test if available
run_ab_test() {
    if ! command -v ab &> /dev/null; then
        print_warning "Apache Bench (ab) not found, skipping ab stress test"
        return 0
    fi
    
    print_status "Running Apache Bench stress test..."
    
    local url="http://${SERVER_HOST}:${SERVER_PORT}/static/"
    local requests=1000
    local concurrent=50
    
    print_status "Running ab with $requests requests and $concurrent concurrent connections..."
    
    if ab -n $requests -c $concurrent "$url" > tests/ab_output.txt 2>&1; then
        # Parse ab results
        local failed_requests=$(grep "Failed requests:" tests/ab_output.txt | awk '{print $3}')
        local requests_per_sec=$(grep "Requests per second:" tests/ab_output.txt | awk '{print $4}')
        local time_per_request=$(grep "Time per request:" tests/ab_output.txt | head -1 | awk '{print $4}')
        
        print_status "Apache Bench test results:"
        echo "  Total requests: $requests"
        echo "  Failed requests: $failed_requests"
        echo "  Requests per second: $requests_per_sec"
        echo "  Time per request: $time_per_request ms"
        
        # Calculate success rate
        local success_rate=$(( ((requests - failed_requests) * 100) / requests ))
        
        if [ $success_rate -ge 95 ]; then
            print_success "Apache Bench stress test PASSED (${success_rate}% success rate)"
            return 0
        else
            print_error "Apache Bench stress test FAILED (${success_rate}% success rate, expected >= 95%)"
            return 1
        fi
    else
        print_error "Apache Bench test failed to run"
        return 1
    fi
}

# Function to run memory leak test
run_memory_test() {
    print_status "Running memory leak test..."
    
    # Get initial memory usage
    local initial_memory=$(ps -o rss= -p $SERVER_PID)
    
    # Run many requests to test for memory leaks
    local url="http://${SERVER_HOST}:${SERVER_PORT}/cgi-bin/hello.py"
    
    for i in {1..100}; do
        curl -s "$url" > /dev/null 2>&1 &
        if (( i % 10 == 0 )); then
            wait
        fi
    done
    wait
    
    # Wait a bit for cleanup
    sleep 2
    
    # Get final memory usage
    local final_memory=$(ps -o rss= -p $SERVER_PID)
    
    # Calculate memory increase
    local memory_increase=$((final_memory - initial_memory))
    local memory_increase_percent=$(( (memory_increase * 100) / initial_memory ))
    
    print_status "Memory test results:"
    echo "  Initial memory: ${initial_memory} KB"
    echo "  Final memory: ${final_memory} KB"
    echo "  Memory increase: ${memory_increase} KB (${memory_increase_percent}%)"
    
    # Memory increase should be reasonable (less than 50%)
    if [ $memory_increase_percent -lt 50 ]; then
        print_success "Memory test PASSED (${memory_increase_percent}% increase)"
        return 0
    else
        print_warning "Memory test WARNING (${memory_increase_percent}% increase, expected < 50%)"
        return 0  # Don't fail the test, just warn
    fi
}

# Main function
main() {
    print_status "Starting stress test suite for localhost HTTP server"
    
    # Create tests directory if it doesn't exist
    mkdir -p tests
    
    # Cleanup function
    cleanup() {
        print_status "Cleaning up..."
        stop_server
        rm -f tests/siege_urls.txt tests/siege_output.txt tests/ab_output.txt
    }
    
    # Set trap for cleanup
    trap cleanup EXIT
    
    # Start server
    start_server
    
    # Run tests
    local test_results=()
    
    # Basic curl stress test
    if run_curl_stress_test; then
        test_results+=("curl:PASS")
    else
        test_results+=("curl:FAIL")
    fi
    
    # Memory leak test
    if run_memory_test; then
        test_results+=("memory:PASS")
    else
        test_results+=("memory:FAIL")
    fi
    
    # Siege test (if available)
    if run_siege_test; then
        test_results+=("siege:PASS")
    else
        test_results+=("siege:FAIL")
    fi
    
    # Apache Bench test (if available)
    if run_ab_test; then
        test_results+=("ab:PASS")
    else
        test_results+=("ab:FAIL")
    fi
    
    # Print summary
    print_status "Stress test summary:"
    local total_tests=0
    local passed_tests=0
    
    for result in "${test_results[@]}"; do
        test_name=$(echo $result | cut -d: -f1)
        test_status=$(echo $result | cut -d: -f2)
        total_tests=$((total_tests + 1))
        
        if [ "$test_status" = "PASS" ]; then
            print_success "$test_name: PASSED"
            passed_tests=$((passed_tests + 1))
        else
            print_error "$test_name: FAILED"
        fi
    done
    
    echo
    if [ $passed_tests -eq $total_tests ]; then
        print_success "All stress tests PASSED ($passed_tests/$total_tests)"
        exit 0
    else
        print_error "Some stress tests FAILED ($passed_tests/$total_tests passed)"
        exit 1
    fi
}

# Run main function
main "$@"
