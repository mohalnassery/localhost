#!/bin/bash

# Performance benchmark script for localhost HTTP server
# Measures throughput, latency, and resource usage

set -e

# Configuration
SERVER_HOST="127.0.0.1"
SERVER_PORT="8889"
CONFIG_FILE="config/test-listing.conf"
SERVER_BINARY="./target/release/localhost-server"
BENCHMARK_DURATION="60s"
WARMUP_DURATION="10s"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

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
    print_status "Starting localhost HTTP server for benchmarking..."
    
    if [ ! -f "$SERVER_BINARY" ]; then
        print_error "Server binary not found: $SERVER_BINARY"
        print_status "Building server..."
        cargo build --release
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

# Function to run warmup
run_warmup() {
    print_status "Running warmup for $WARMUP_DURATION..."
    
    local url="http://${SERVER_HOST}:${SERVER_PORT}/static/"
    
    # Simple warmup with curl
    timeout $WARMUP_DURATION bash -c "
        while true; do
            curl -s '$url' > /dev/null 2>&1 || true
            sleep 0.1
        done
    " || true
    
    print_success "Warmup completed"
}

# Function to measure baseline performance
measure_baseline() {
    print_header "Baseline Performance Measurement"
    
    local url="http://${SERVER_HOST}:${SERVER_PORT}/static/"
    local requests=1000
    local start_time=$(date +%s.%N)
    local success_count=0
    
    print_status "Measuring baseline with $requests sequential requests..."
    
    for ((i=1; i<=requests; i++)); do
        if curl -s "$url" > /dev/null 2>&1; then
            success_count=$((success_count + 1))
        fi
        
        if (( i % 100 == 0 )); then
            echo -n "."
        fi
    done
    echo
    
    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc -l)
    local rps=$(echo "scale=2; $success_count / $duration" | bc -l)
    local avg_latency=$(echo "scale=2; $duration * 1000 / $success_count" | bc -l)
    
    print_status "Baseline Results:"
    echo "  Total requests: $requests"
    echo "  Successful: $success_count"
    echo "  Duration: ${duration}s"
    echo "  Requests/sec: $rps"
    echo "  Avg latency: ${avg_latency}ms"
}

# Function to run concurrent benchmark
run_concurrent_benchmark() {
    print_header "Concurrent Load Benchmark"
    
    local url="http://${SERVER_HOST}:${SERVER_PORT}/static/"
    local concurrent_levels=(1 5 10 25 50 100)
    
    echo "Concurrency Level | Requests/sec | Avg Latency (ms) | Success Rate (%)"
    echo "------------------|--------------|------------------|------------------"
    
    for concurrency in "${concurrent_levels[@]}"; do
        print_status "Testing with $concurrency concurrent connections..."
        
        local temp_dir=$(mktemp -d)
        local total_requests=$((concurrency * 20)) # 20 requests per connection
        local success_count=0
        local total_time=0
        
        local start_time=$(date +%s.%N)
        
        # Run concurrent requests
        for ((i=1; i<=concurrency; i++)); do
            {
                local conn_success=0
                local conn_start=$(date +%s.%N)
                
                for ((j=1; j<=20; j++)); do
                    if curl -s "$url" > /dev/null 2>&1; then
                        conn_success=$((conn_success + 1))
                    fi
                done
                
                local conn_end=$(date +%s.%N)
                local conn_time=$(echo "$conn_end - $conn_start" | bc -l)
                
                echo "$conn_success" > "$temp_dir/success_$i"
                echo "$conn_time" > "$temp_dir/time_$i"
            } &
        done
        
        # Wait for all connections to complete
        wait
        
        local end_time=$(date +%s.%N)
        local total_duration=$(echo "$end_time - $start_time" | bc -l)
        
        # Collect results
        for ((i=1; i<=concurrency; i++)); do
            if [ -f "$temp_dir/success_$i" ]; then
                local conn_success=$(cat "$temp_dir/success_$i")
                success_count=$((success_count + conn_success))
            fi
        done
        
        # Calculate metrics
        local rps=$(echo "scale=2; $success_count / $total_duration" | bc -l)
        local avg_latency=$(echo "scale=2; $total_duration * 1000 / $success_count" | bc -l)
        local success_rate=$(echo "scale=1; $success_count * 100 / $total_requests" | bc -l)
        
        printf "%17s | %12s | %16s | %16s\n" "$concurrency" "$rps" "$avg_latency" "$success_rate"
        
        # Cleanup
        rm -rf "$temp_dir"
        
        # Brief pause between tests
        sleep 2
    done
}

# Function to benchmark different endpoints
benchmark_endpoints() {
    print_header "Endpoint Performance Comparison"
    
    local endpoints=(
        "http://${SERVER_HOST}:${SERVER_PORT}/static/"
        "http://${SERVER_HOST}:${SERVER_PORT}/static/test.txt"
        "http://${SERVER_HOST}:${SERVER_PORT}/cgi-bin/hello.py"
    )
    
    local endpoint_names=("Directory Listing" "Static File" "CGI Script")
    
    echo "Endpoint          | Requests/sec | Avg Latency (ms) | Success Rate (%)"
    echo "------------------|--------------|------------------|------------------"
    
    for i in "${!endpoints[@]}"; do
        local url="${endpoints[$i]}"
        local name="${endpoint_names[$i]}"
        
        print_status "Benchmarking: $name"
        
        local requests=200
        local concurrency=10
        local success_count=0
        local temp_dir=$(mktemp -d)
        
        local start_time=$(date +%s.%N)
        
        # Run concurrent requests
        for ((j=1; j<=concurrency; j++)); do
            {
                local conn_success=0
                for ((k=1; k<=$((requests/concurrency)); k++)); do
                    if curl -s "$url" > /dev/null 2>&1; then
                        conn_success=$((conn_success + 1))
                    fi
                done
                echo "$conn_success" > "$temp_dir/success_$j"
            } &
        done
        
        wait
        
        local end_time=$(date +%s.%N)
        local duration=$(echo "$end_time - $start_time" | bc -l)
        
        # Collect results
        for ((j=1; j<=concurrency; j++)); do
            if [ -f "$temp_dir/success_$j" ]; then
                local conn_success=$(cat "$temp_dir/success_$j")
                success_count=$((success_count + conn_success))
            fi
        done
        
        # Calculate metrics
        local rps=$(echo "scale=2; $success_count / $duration" | bc -l)
        local avg_latency=$(echo "scale=2; $duration * 1000 / $success_count" | bc -l)
        local success_rate=$(echo "scale=1; $success_count * 100 / $requests" | bc -l)
        
        printf "%17s | %12s | %16s | %16s\n" "$name" "$rps" "$avg_latency" "$success_rate"
        
        # Cleanup
        rm -rf "$temp_dir"
        
        sleep 1
    done
}

# Function to monitor resource usage
monitor_resources() {
    print_header "Resource Usage Monitoring"
    
    if [ -z "$SERVER_PID" ]; then
        print_error "Server PID not available for monitoring"
        return 1
    fi
    
    print_status "Monitoring server resources during load test..."
    
    # Monitor for 30 seconds during load
    local monitor_duration=30
    local url="http://${SERVER_HOST}:${SERVER_PORT}/static/"
    
    # Start resource monitoring in background
    {
        local max_memory=0
        local max_cpu=0
        local samples=0
        
        for ((i=1; i<=monitor_duration; i++)); do
            if ps -p $SERVER_PID > /dev/null 2>&1; then
                local memory=$(ps -o rss= -p $SERVER_PID 2>/dev/null || echo "0")
                local cpu=$(ps -o %cpu= -p $SERVER_PID 2>/dev/null || echo "0")
                
                if [ "$memory" -gt "$max_memory" ]; then
                    max_memory=$memory
                fi
                
                if (( $(echo "$cpu > $max_cpu" | bc -l) )); then
                    max_cpu=$cpu
                fi
                
                samples=$((samples + 1))
            fi
            
            sleep 1
        done
        
        echo "$max_memory $max_cpu $samples" > /tmp/resource_stats
    } &
    
    local monitor_pid=$!
    
    # Generate load during monitoring
    for ((i=1; i<=5; i++)); do
        {
            for ((j=1; j<=100; j++)); do
                curl -s "$url" > /dev/null 2>&1 || true
            done
        } &
    done
    
    # Wait for monitoring to complete
    wait $monitor_pid
    
    # Read results
    if [ -f "/tmp/resource_stats" ]; then
        local stats=$(cat /tmp/resource_stats)
        local max_memory=$(echo $stats | cut -d' ' -f1)
        local max_cpu=$(echo $stats | cut -d' ' -f2)
        local samples=$(echo $stats | cut -d' ' -f3)
        
        print_status "Resource Usage Results:"
        echo "  Peak Memory: ${max_memory} KB"
        echo "  Peak CPU: ${max_cpu}%"
        echo "  Monitoring samples: $samples"
        
        rm -f /tmp/resource_stats
    else
        print_warning "Could not collect resource usage data"
    fi
}

# Main function
main() {
    print_header "Localhost HTTP Server Performance Benchmark"
    
    # Cleanup function
    cleanup() {
        print_status "Cleaning up..."
        stop_server
    }
    
    # Set trap for cleanup
    trap cleanup EXIT
    
    # Check prerequisites
    if ! command -v bc &> /dev/null; then
        print_error "bc (calculator) not found. Please install bc for benchmarking."
        exit 1
    fi
    
    # Start server
    start_server
    
    # Run warmup
    run_warmup
    
    # Run benchmarks
    measure_baseline
    echo
    run_concurrent_benchmark
    echo
    benchmark_endpoints
    echo
    monitor_resources
    
    print_header "Benchmark Summary"
    print_success "Performance benchmarking completed successfully!"
    echo
    echo "📊 Key Takeaways:"
    echo "  • Static file serving should achieve 1000+ req/sec"
    echo "  • Directory listing should achieve 500+ req/sec"
    echo "  • CGI scripts will be slower but should handle 50+ req/sec"
    echo "  • Memory usage should remain stable under load"
    echo "  • CPU usage should scale with concurrent connections"
    echo
    echo "🔧 For production optimization:"
    echo "  • Consider connection pooling for high-concurrency scenarios"
    echo "  • Monitor memory usage with long-running processes"
    echo "  • Implement caching for frequently accessed content"
}

# Run main function
main "$@"
