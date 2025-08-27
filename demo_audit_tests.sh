#!/bin/bash

# Demo script to showcase audit test capabilities
# This script demonstrates key features and runs selected tests

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${PURPLE}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PURPLE}║                    LOCALHOST HTTP SERVER AUDIT DEMO                         ║${NC}"
echo -e "${PURPLE}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo

echo -e "${BLUE}This demo showcases the comprehensive audit testing capabilities.${NC}"
echo -e "${BLUE}All tests correspond to specific audit requirements and provide actionable proof.${NC}"
echo

echo -e "${YELLOW}Available Test Categories:${NC}"
echo "1. HTTP Server Basics      - Protocol compliance, request/response handling"
echo "2. I/O Multiplexing       - Epoll usage verification"
echo "3. Single Thread          - Thread count validation under load"
echo "4. HTTP Methods           - GET, POST, DELETE support"
echo "5. Error Handling         - Custom error pages, malformed requests"
echo "6. Configuration          - Multi-port, virtual hosts"
echo "7. CGI Support            - Python CGI execution"
echo "8. Sessions/Cookies       - Session management"
echo "9. File Uploads           - Upload handling with size limits"
echo "10. Stress Testing        - Performance and availability"
echo "11. Browser Compatibility - Real browser request handling"
echo

echo -e "${GREEN}Quick Demo Options:${NC}"
echo "1. Run basic functionality tests (fast)"
echo "2. Run comprehensive audit tests (complete)"
echo "3. Show individual test category"
echo "4. View audit answers with proofs"
echo "5. Exit"
echo

read -p "Choose an option (1-5): " choice

case $choice in
    1)
        echo -e "${BLUE}Running basic functionality tests...${NC}"
        echo
        ./test_all_audit_requirements.sh basic
        echo
        ./test_all_audit_requirements.sh methods
        echo
        echo -e "${GREEN}Basic tests completed! ✓${NC}"
        ;;
    2)
        echo -e "${BLUE}Running comprehensive audit tests...${NC}"
        echo -e "${YELLOW}This will test ALL audit requirements (may take 5-10 minutes)${NC}"
        read -p "Continue? (y/N): " confirm
        if [[ $confirm =~ ^[Yy]$ ]]; then
            ./test_all_audit_requirements.sh
        else
            echo "Cancelled."
        fi
        ;;
    3)
        echo -e "${BLUE}Available test categories:${NC}"
        echo "basic, epoll, thread, methods, errors, config, cgi, session, upload, stress, browser"
        echo
        read -p "Enter category name: " category
        ./test_all_audit_requirements.sh "$category"
        ;;
    4)
        echo -e "${BLUE}Opening audit answers document...${NC}"
        if command -v less &> /dev/null; then
            less auditanswers.md
        else
            cat auditanswers.md
        fi
        ;;
    5)
        echo -e "${GREEN}Goodbye!${NC}"
        exit 0
        ;;
    *)
        echo -e "${YELLOW}Invalid option. Please choose 1-5.${NC}"
        exit 1
        ;;
esac

echo
echo -e "${PURPLE}Demo completed!${NC}"
echo
echo -e "${BLUE}For more information:${NC}"
echo "• Full test suite: ./test_all_audit_requirements.sh"
echo "• Help: ./test_all_audit_requirements.sh help"
echo "• Audit answers: cat auditanswers.md"
echo "• Server config: cat config/test.conf"
echo
echo -e "${GREEN}The localhost HTTP server is ready for audit! 🎉${NC}"
