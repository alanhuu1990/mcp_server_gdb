#!/bin/bash

# MCP GDB Server with Node.js Real-Time Debugging
# This script starts both the Rust MCP server and Node.js bridge

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RUST_SERVER_PORT=${RUST_SERVER_PORT:-8080}
NODEJS_PORT=${NODEJS_PORT:-3000}
WEBSOCKET_PORT=${WEBSOCKET_PORT:-3001}

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
NODEJS_DIR="$PROJECT_ROOT/nodejs"
RUST_BINARY="$PROJECT_ROOT/target/release/mcp-server-gdb"

echo -e "${BLUE}=== MCP GDB Server with Node.js Real-Time Debugging ===${NC}"
echo -e "${BLUE}Project Root: $PROJECT_ROOT${NC}"
echo -e "${BLUE}Node.js Directory: $NODEJS_DIR${NC}"
echo ""

# Function to check if port is available
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo -e "${RED}Error: Port $port is already in use${NC}"
        return 1
    fi
    return 0
}

# Function to cleanup processes on exit
cleanup() {
    echo -e "\n${YELLOW}Shutting down servers...${NC}"
    
    if [ ! -z "$RUST_PID" ]; then
        echo -e "${YELLOW}Stopping Rust MCP server (PID: $RUST_PID)${NC}"
        kill $RUST_PID 2>/dev/null || true
    fi
    
    if [ ! -z "$NODEJS_PID" ]; then
        echo -e "${YELLOW}Stopping Node.js bridge (PID: $NODEJS_PID)${NC}"
        kill $NODEJS_PID 2>/dev/null || true
    fi
    
    echo -e "${GREEN}Cleanup complete${NC}"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Check prerequisites
echo -e "${BLUE}Checking prerequisites...${NC}"

# Check if Rust binary exists
if [ ! -f "$RUST_BINARY" ]; then
    echo -e "${RED}Error: Rust binary not found at $RUST_BINARY${NC}"
    echo -e "${YELLOW}Please build the project first: cargo build --release${NC}"
    exit 1
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js is not installed${NC}"
    echo -e "${YELLOW}Please install Node.js (version 16 or higher)${NC}"
    exit 1
fi

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo -e "${RED}Error: npm is not installed${NC}"
    exit 1
fi

# Check Node.js directory
if [ ! -d "$NODEJS_DIR" ]; then
    echo -e "${RED}Error: Node.js directory not found at $NODEJS_DIR${NC}"
    exit 1
fi

# Check if package.json exists
if [ ! -f "$NODEJS_DIR/package.json" ]; then
    echo -e "${RED}Error: package.json not found in $NODEJS_DIR${NC}"
    exit 1
fi

echo -e "${GREEN}Prerequisites check passed${NC}"

# Check ports availability
echo -e "${BLUE}Checking port availability...${NC}"
check_port $RUST_SERVER_PORT || exit 1
check_port $NODEJS_PORT || exit 1
check_port $WEBSOCKET_PORT || exit 1
echo -e "${GREEN}All ports are available${NC}"

# Install Node.js dependencies if needed
echo -e "${BLUE}Checking Node.js dependencies...${NC}"
cd "$NODEJS_DIR"

if [ ! -d "node_modules" ] || [ ! -f "package-lock.json" ]; then
    echo -e "${YELLOW}Installing Node.js dependencies...${NC}"
    npm install
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to install Node.js dependencies${NC}"
        exit 1
    fi
    echo -e "${GREEN}Node.js dependencies installed${NC}"
else
    echo -e "${GREEN}Node.js dependencies already installed${NC}"
fi

# Create logs directory
mkdir -p "$PROJECT_ROOT/logs"

# Start Rust MCP server
echo -e "${BLUE}Starting Rust MCP server on port $RUST_SERVER_PORT...${NC}"
cd "$PROJECT_ROOT"

export SERVER_IP="127.0.0.1"
export SERVER_PORT="$RUST_SERVER_PORT"
export GDB_COMMAND_TIMEOUT="30"
export RUST_LOG="info"

"$RUST_BINARY" --log-level info --transport sse > logs/rust-server.log 2>&1 &
RUST_PID=$!

# Wait a moment for the server to start
sleep 2

# Check if Rust server is running
if ! kill -0 $RUST_PID 2>/dev/null; then
    echo -e "${RED}Error: Failed to start Rust MCP server${NC}"
    echo -e "${YELLOW}Check logs/rust-server.log for details${NC}"
    exit 1
fi

echo -e "${GREEN}Rust MCP server started (PID: $RUST_PID)${NC}"

# Start Node.js bridge
echo -e "${BLUE}Starting Node.js bridge on port $NODEJS_PORT...${NC}"
cd "$NODEJS_DIR"

export PORT="$NODEJS_PORT"
export HOST="127.0.0.1"

npm start > ../logs/nodejs-bridge.log 2>&1 &
NODEJS_PID=$!

# Wait a moment for the Node.js server to start
sleep 3

# Check if Node.js server is running
if ! kill -0 $NODEJS_PID 2>/dev/null; then
    echo -e "${RED}Error: Failed to start Node.js bridge${NC}"
    echo -e "${YELLOW}Check logs/nodejs-bridge.log for details${NC}"
    cleanup
    exit 1
fi

echo -e "${GREEN}Node.js bridge started (PID: $NODEJS_PID)${NC}"

# Display status
echo ""
echo -e "${GREEN}=== Servers Started Successfully ===${NC}"
echo -e "${GREEN}Rust MCP Server:    http://127.0.0.1:$RUST_SERVER_PORT${NC}"
echo -e "${GREEN}Node.js Dashboard:  http://127.0.0.1:$NODEJS_PORT${NC}"
echo -e "${GREEN}WebSocket Server:   ws://127.0.0.1:$WEBSOCKET_PORT${NC}"
echo ""
echo -e "${BLUE}Dashboard URL: ${GREEN}http://127.0.0.1:$NODEJS_PORT${NC}"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all servers${NC}"

# Wait for processes
wait
