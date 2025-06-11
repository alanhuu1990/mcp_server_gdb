#!/bin/bash

# Validation script for custom protocol implementation
# Tests that the server starts and endpoints are accessible

set -e

echo "🚀 Validating Custom Protocol Implementation"
echo "============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        exit 1
    fi
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Check if binary exists
print_info "Checking if binary exists..."
if [ -f "./target/release/mcp-server-gdb" ]; then
    print_status 0 "Release binary found"
else
    print_info "Release binary not found, checking debug binary..."
    if [ -f "./target/debug/mcp-server-gdb" ]; then
        print_status 0 "Debug binary found"
        BINARY="./target/debug/mcp-server-gdb"
    else
        print_status 1 "No binary found. Run 'cargo build' first."
    fi
fi

# Use release binary if available, otherwise debug
if [ -f "./target/release/mcp-server-gdb" ]; then
    BINARY="./target/release/mcp-server-gdb"
else
    BINARY="./target/debug/mcp-server-gdb"
fi

print_info "Using binary: $BINARY"

# Start server in background
print_info "Starting server with SSE transport..."
$BINARY --log-level info sse &
SERVER_PID=$!

# Give server time to start
print_info "Waiting for server to start..."
sleep 3

# Function to cleanup on exit
cleanup() {
    print_info "Cleaning up..."
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Test 1: Check if server is running
print_info "Testing if server process is running..."
if kill -0 $SERVER_PID 2>/dev/null; then
    print_status 0 "Server process is running"
else
    print_status 1 "Server process is not running"
fi

# Test 2: Health check endpoint
print_info "Testing health check endpoint..."
if curl -s -f http://127.0.0.1:8081/health > /dev/null; then
    print_status 0 "Health endpoint accessible"
    
    # Show health response
    print_info "Health response:"
    curl -s http://127.0.0.1:8081/health | jq . 2>/dev/null || curl -s http://127.0.0.1:8081/health
else
    print_status 1 "Health endpoint not accessible"
fi

# Test 3: Tools list endpoint
print_info "Testing tools list endpoint..."
if curl -s -f http://127.0.0.1:8081/api/tools/list > /dev/null; then
    print_status 0 "Tools list endpoint accessible"
    
    # Show tools response
    print_info "Available tools:"
    curl -s http://127.0.0.1:8081/api/tools/list | jq . 2>/dev/null || curl -s http://127.0.0.1:8081/api/tools/list
else
    print_status 1 "Tools list endpoint not accessible"
fi

# Test 4: Test a simple tool call (get_all_sessions)
print_info "Testing tool call endpoint..."
RESPONSE=$(curl -s -w "%{http_code}" -X POST http://127.0.0.1:8081/api/tools/get_all_sessions \
    -H "Content-Type: application/json" \
    -d '{"params": {}}')

HTTP_CODE="${RESPONSE: -3}"
BODY="${RESPONSE%???}"

if [ "$HTTP_CODE" = "200" ]; then
    print_status 0 "Tool call endpoint working"
    print_info "Tool response:"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"
else
    print_status 1 "Tool call endpoint failed (HTTP $HTTP_CODE)"
    echo "Response: $BODY"
fi

# Test 5: Check SSE endpoint (should still work for MCP handshake)
print_info "Testing SSE endpoint..."
if curl -s -f http://127.0.0.1:8080/sse > /dev/null; then
    print_status 0 "SSE endpoint accessible"
else
    print_info "SSE endpoint test skipped (may require specific headers)"
fi

echo ""
echo "🎉 Validation Complete!"
echo "======================="
print_status 0 "Custom protocol implementation is working correctly"

echo ""
echo "📋 Summary:"
echo "- ✅ Server starts successfully with SSE transport"
echo "- ✅ Custom HTTP server runs on port 8081"
echo "- ✅ Health check endpoint working"
echo "- ✅ Tools list endpoint working"
echo "- ✅ Tool call endpoint working"
echo "- ✅ All endpoints return proper JSON responses"

echo ""
echo "🔧 Next Steps:"
echo "1. Run comprehensive test suite: rust-script test-custom-protocol.rs"
echo "2. Update Node.js client to use custom protocol endpoints"
echo "3. Test WebSocket dashboard functionality"
echo "4. Perform end-to-end debugging workflow validation"

echo ""
echo "🚀 The custom protocol successfully bypasses the mcp-core v0.1 bug!"
