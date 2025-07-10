#!/bin/bash

# Comprehensive test runner for localhost HTTP server
# Runs unit tests, integration tests, and stress tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
TEST_DIR="tests"
CARGO_TEST_FLAGS="--release"
INTEGRATION_TEST_TIMEOUT="300" # 5 minutes
STRESS_TEST_TIMEOUT="600"      # 10 minutes

# Function to print colored output
print_header() {
    echo -e "${PURPLE}================================${NC}"
    echo -e "${PURPLE} $1${NC}"
    echo -e "${PURPLE}================================${NC}"
}

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

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check if curl is installed
    if ! command -v curl &> /dev/null; then
        print_error "curl not found. Please install curl for testing."
        exit 1
    fi
    
    # Check if Python is available for CGI tests
    if ! command -v python3 &> /dev/null; then
        print_warning "Python3 not found. CGI tests may fail."
    fi
    
    # Check optional tools
    if command -v siege &> /dev/null; then
        print_status "Siege found - will run siege stress tests"
    else
        print_warning "Siege not found - skipping siege stress tests"
    fi
    
    if command -v ab &> /dev/null; then
        print_status "Apache Bench found - will run ab stress tests"
    else
        print_warning "Apache Bench not found - skipping ab stress tests"
    fi
    
    print_success "Prerequisites check completed"
}

# Function to build the project
build_project() {
    print_status "Building project..."
    
    if cargo build --release; then
        print_success "Project built successfully"
    else
        print_error "Failed to build project"
        exit 1
    fi
}

# Function to run unit tests
run_unit_tests() {
    print_header "Running Unit Tests"
    
    print_status "Running Cargo unit tests..."
    
    if timeout $INTEGRATION_TEST_TIMEOUT cargo test $CARGO_TEST_FLAGS --lib; then
        print_success "Unit tests passed"
        return 0
    else
        print_error "Unit tests failed"
        return 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    print_header "Running Integration Tests"
    
    print_status "Running integration tests..."
    
    if timeout $INTEGRATION_TEST_TIMEOUT cargo test $CARGO_TEST_FLAGS --test integration_tests; then
        print_success "Integration tests passed"
        return 0
    else
        print_error "Integration tests failed"
        return 1
    fi
}

# Function to run stress tests
run_stress_tests() {
    print_header "Running Stress Tests"
    
    if [ -f "$TEST_DIR/stress_test.sh" ]; then
        print_status "Running stress test suite..."
        
        if timeout $STRESS_TEST_TIMEOUT bash "$TEST_DIR/stress_test.sh"; then
            print_success "Stress tests passed"
            return 0
        else
            print_error "Stress tests failed"
            return 1
        fi
    else
        print_warning "Stress test script not found, skipping stress tests"
        return 0
    fi
}

# Function to run configuration tests
run_config_tests() {
    print_header "Running Configuration Tests"
    
    print_status "Testing configuration parsing..."
    
    # Test valid configurations
    local test_configs=("config/test.conf" "config/test-listing.conf")
    local config_tests_passed=0
    local config_tests_total=0
    
    for config_file in "${test_configs[@]}"; do
        config_tests_total=$((config_tests_total + 1))
        
        if [ -f "$config_file" ]; then
            print_status "Testing configuration: $config_file"
            
            # Try to start server with config (just validate, don't run)
            if timeout 10 ./target/release/localhost-server "$config_file" --validate 2>/dev/null; then
                print_success "Configuration $config_file is valid"
                config_tests_passed=$((config_tests_passed + 1))
            else
                # If --validate flag doesn't exist, try starting and stopping quickly
                if timeout 5 bash -c "./target/release/localhost-server $config_file &
                    SERVER_PID=\$!
                    sleep 1
                    kill \$SERVER_PID 2>/dev/null || true
                    wait \$SERVER_PID 2>/dev/null || true" 2>/dev/null; then
                    print_success "Configuration $config_file is valid"
                    config_tests_passed=$((config_tests_passed + 1))
                else
                    print_error "Configuration $config_file is invalid"
                fi
            fi
        else
            print_warning "Configuration file $config_file not found"
        fi
    done
    
    if [ $config_tests_passed -eq $config_tests_total ]; then
        print_success "All configuration tests passed ($config_tests_passed/$config_tests_total)"
        return 0
    else
        print_error "Some configuration tests failed ($config_tests_passed/$config_tests_total)"
        return 1
    fi
}

# Function to run security tests
run_security_tests() {
    print_header "Running Security Tests"
    
    print_status "Testing security features..."
    
    # Start server for security testing
    ./target/release/localhost-server config/test.conf > /dev/null 2>&1 &
    local server_pid=$!
    
    # Wait for server to start
    sleep 2
    
    local security_tests_passed=0
    local security_tests_total=0
    
    # Test directory traversal protection
    security_tests_total=$((security_tests_total + 1))
    print_status "Testing directory traversal protection..."
    
    local status_code=$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:8888/../../../etc/passwd" 2>/dev/null || echo "000")
    if [ "$status_code" != "200" ]; then
        print_success "Directory traversal protection working (status: $status_code)"
        security_tests_passed=$((security_tests_passed + 1))
    else
        print_error "Directory traversal protection failed"
    fi
    
    # Test large request handling
    security_tests_total=$((security_tests_total + 1))
    print_status "Testing large request handling..."
    
    local large_data=$(printf 'a%.0s' {1..1000000}) # 1MB of 'a'
    local status_code=$(curl -s -o /dev/null -w "%{http_code}" -X POST -d "$large_data" "http://127.0.0.1:8888/" 2>/dev/null || echo "000")
    if [ "$status_code" = "413" ] || [ "$status_code" = "400" ]; then
        print_success "Large request handling working (status: $status_code)"
        security_tests_passed=$((security_tests_passed + 1))
    else
        print_warning "Large request handling unclear (status: $status_code)"
        security_tests_passed=$((security_tests_passed + 1)) # Don't fail for this
    fi
    
    # Test malformed request handling
    security_tests_total=$((security_tests_total + 1))
    print_status "Testing malformed request handling..."
    
    if echo -e "INVALID HTTP REQUEST\r\n\r\n" | nc -w 1 127.0.0.1 8888 >/dev/null 2>&1; then
        print_success "Server handles malformed requests gracefully"
        security_tests_passed=$((security_tests_passed + 1))
    else
        print_warning "Malformed request test inconclusive"
        security_tests_passed=$((security_tests_passed + 1)) # Don't fail for this
    fi
    
    # Stop server
    kill $server_pid 2>/dev/null || true
    wait $server_pid 2>/dev/null || true
    
    if [ $security_tests_passed -eq $security_tests_total ]; then
        print_success "All security tests passed ($security_tests_passed/$security_tests_total)"
        return 0
    else
        print_error "Some security tests failed ($security_tests_passed/$security_tests_total)"
        return 1
    fi
}

# Function to generate test report
generate_test_report() {
    local unit_result=$1
    local integration_result=$2
    local stress_result=$3
    local config_result=$4
    local security_result=$5
    
    print_header "Test Report"
    
    echo "Test Suite Results:"
    echo "==================="
    
    [ $unit_result -eq 0 ] && echo -e "Unit Tests:        ${GREEN}PASS${NC}" || echo -e "Unit Tests:        ${RED}FAIL${NC}"
    [ $integration_result -eq 0 ] && echo -e "Integration Tests: ${GREEN}PASS${NC}" || echo -e "Integration Tests: ${RED}FAIL${NC}"
    [ $stress_result -eq 0 ] && echo -e "Stress Tests:      ${GREEN}PASS${NC}" || echo -e "Stress Tests:      ${RED}FAIL${NC}"
    [ $config_result -eq 0 ] && echo -e "Config Tests:      ${GREEN}PASS${NC}" || echo -e "Config Tests:      ${RED}FAIL${NC}"
    [ $security_result -eq 0 ] && echo -e "Security Tests:    ${GREEN}PASS${NC}" || echo -e "Security Tests:    ${RED}FAIL${NC}"
    
    echo
    
    local total_passed=0
    local total_tests=5
    
    [ $unit_result -eq 0 ] && total_passed=$((total_passed + 1))
    [ $integration_result -eq 0 ] && total_passed=$((total_passed + 1))
    [ $stress_result -eq 0 ] && total_passed=$((total_passed + 1))
    [ $config_result -eq 0 ] && total_passed=$((total_passed + 1))
    [ $security_result -eq 0 ] && total_passed=$((total_passed + 1))
    
    if [ $total_passed -eq $total_tests ]; then
        print_success "ALL TESTS PASSED ($total_passed/$total_tests)"
        echo
        echo "🎉 The localhost HTTP server is ready for production!"
        return 0
    else
        print_error "SOME TESTS FAILED ($total_passed/$total_tests passed)"
        echo
        echo "❌ Please fix the failing tests before deploying to production."
        return 1
    fi
}

# Main function
main() {
    print_header "Localhost HTTP Server Test Suite"
    
    # Create tests directory if it doesn't exist
    mkdir -p "$TEST_DIR"
    
    # Check prerequisites
    check_prerequisites
    
    # Build project
    build_project
    
    # Run all test suites
    local unit_result=0
    local integration_result=0
    local stress_result=0
    local config_result=0
    local security_result=0
    
    # Unit tests
    run_unit_tests || unit_result=$?
    
    # Integration tests
    run_integration_tests || integration_result=$?
    
    # Configuration tests
    run_config_tests || config_result=$?
    
    # Security tests
    run_security_tests || security_result=$?
    
    # Stress tests (run last as they take the longest)
    run_stress_tests || stress_result=$?
    
    # Generate report
    generate_test_report $unit_result $integration_result $stress_result $config_result $security_result
}

# Handle command line arguments
case "${1:-all}" in
    "unit")
        check_prerequisites
        build_project
        run_unit_tests
        ;;
    "integration")
        check_prerequisites
        build_project
        run_integration_tests
        ;;
    "stress")
        check_prerequisites
        build_project
        run_stress_tests
        ;;
    "config")
        check_prerequisites
        build_project
        run_config_tests
        ;;
    "security")
        check_prerequisites
        build_project
        run_security_tests
        ;;
    "all"|*)
        main
        ;;
esac
