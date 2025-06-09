#!/bin/bash

# MCP Server GDB Wrapper Script
# This script works around the read-only filesystem issue by creating logs in /tmp

# Create a temporary logs directory
TEMP_LOGS_DIR="/tmp/mcp-gdb-logs-$$"
mkdir -p "$TEMP_LOGS_DIR"

# Create a symbolic link to the temp logs directory
cd "$(dirname "$0")"
if [ ! -e "logs" ]; then
    ln -sf "$TEMP_LOGS_DIR" logs
fi

# Run the actual MCP server
exec ./build/mcp-server-gdb "$@"
