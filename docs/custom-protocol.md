# Custom Protocol Implementation

## Overview

This document describes the custom SSE-based tool routing implementation that bypasses the mcp-core v0.1 bug preventing tool execution despite successful MCP handshake.

## Problem Statement

The mcp-core v0.1 crate has a critical bug where:
- ✅ SSE connection works perfectly
- ✅ MCP initialize handshake works perfectly  
- ✅ MCP initialized notification sent successfully
- ❌ `tools/list` fails with "Client must be initialized before using tools/list"
- ❌ `tools/call` fails with "Client must be initialized before using tools/call"

## Solution Architecture

The custom protocol implementation provides a workaround by:

1. **Maintaining existing SSE transport** for MCP handshake compatibility
2. **Adding custom HTTP routes** for direct tool invocation
3. **Bypassing mcp-core's tools/call mechanism** entirely
4. **Preserving all existing GDB functionality**

## Implementation Details

### Components

#### 1. Custom Protocol Module (`src/custom_protocol.rs`)
- **Purpose**: Custom tool routing handlers
- **Features**:
  - JSON request/response structures
  - Direct tool invocation
  - Comprehensive error handling
  - All 13+ GDB tools supported

#### 2. HTTP Server Integration (`src/main.rs`)
- **Purpose**: HTTP server alongside SSE transport
- **Features**:
  - Runs on SSE port + 1 (e.g., 8081 if SSE is 8080)
  - CORS enabled for web client compatibility
  - Graceful shutdown handling

#### 3. Direct Tool Invocation (`src/tools.rs`)
- **Purpose**: Direct access to tool functions
- **Features**:
  - Bypasses mcp-core tool registration
  - Maintains existing tool signatures
  - Preserves all GDB functionality

### API Endpoints

#### Health Check
```
GET /health
```
Returns server status and version information.

#### List Tools
```
GET /api/tools/list
```
Returns all available GDB tools.

#### Tool Invocation
```
POST /api/tools/{tool_name}
Content-Type: application/json

{
  "params": {
    "session_id": "uuid",
    "other_params": "values"
  }
}
```

### Supported Tools

All 13+ GDB tools are supported via custom protocol:

#### Session Management
- `create_session` - Create new GDB session
- `get_session` - Get session by ID  
- `get_all_sessions` - List all sessions
- `close_session` - Close session

#### Debug Control
- `start_debugging` - Start debugging
- `stop_debugging` - Stop debugging

#### Breakpoint Management
- `get_breakpoints` - List breakpoints
- `set_breakpoint` - Set breakpoint
- `delete_breakpoint` - Delete breakpoint

#### Execution Control
- `continue_execution` - Continue execution
- `step_execution` - Step into
- `next_execution` - Step over

#### Information Retrieval
- `get_stack_frames` - Get call stack
- `get_local_variables` - Get local variables
- `get_registers` - Get CPU registers
- `get_register_names` - Get register names
- `read_memory` - Read memory

## Usage

### Starting the Server

```bash
# Start with SSE transport (enables custom protocol)
./target/debug/mcp-server-gdb --transport sse --log-level debug

# Server will start:
# - SSE transport on port 8080 (for MCP handshake)
# - Custom HTTP server on port 8081 (for tool calls)
```

### Client Integration

#### Node.js Example
```javascript
const axios = require('axios');

const client = axios.create({
  baseURL: 'http://127.0.0.1:8081',
  timeout: 30000
});

// Create session
const response = await client.post('/api/tools/create_session', {
  params: {
    program: '/path/to/executable'
  }
});

console.log(response.data);
```

#### cURL Example
```bash
# Health check
curl http://127.0.0.1:8081/health

# List tools
curl http://127.0.0.1:8081/api/tools/list

# Create session
curl -X POST http://127.0.0.1:8081/api/tools/create_session \
  -H "Content-Type: application/json" \
  -d '{"params": {}}'
```

## Testing

### Test Script
Run the comprehensive test suite:

```bash
# Install rust-script if needed
cargo install rust-script

# Run test suite
rust-script test-custom-protocol.rs
```

### Manual Testing
```bash
# Start server
./target/debug/mcp-server-gdb --transport sse

# Test health endpoint
curl http://127.0.0.1:8081/health

# Test tool list
curl http://127.0.0.1:8081/api/tools/list

# Test tool call
curl -X POST http://127.0.0.1:8081/api/tools/get_all_sessions \
  -H "Content-Type: application/json" \
  -d '{"params": {}}'
```

## Performance

The custom protocol provides:
- **Equal or better performance** than original MCP tools/call
- **Lower latency** due to direct HTTP calls vs MCP message routing
- **Better error handling** with proper HTTP status codes
- **Improved debugging** with detailed logging

## Error Handling

### HTTP Status Codes
- `200 OK` - Successful tool execution
- `400 Bad Request` - Invalid parameters
- `500 Internal Server Error` - Tool execution error

### Response Format
```json
{
  "success": true,
  "data": {
    "message": "Tool response content"
  },
  "error": null
}
```

### Error Response
```json
{
  "success": false,
  "data": null,
  "error": "Error description"
}
```

## Compatibility

### MCP Protocol
- ✅ Maintains MCP handshake compatibility
- ✅ SSE transport continues to work
- ✅ Existing Node.js client can be updated
- ✅ WebSocket dashboard functionality preserved

### GDB Functionality
- ✅ All 13+ tools work identically
- ✅ Session management preserved
- ✅ Debugging workflows unchanged
- ✅ TUI functionality unaffected

## Future Considerations

### mcp-core Updates
When mcp-core fixes the initialization bug:
1. Custom protocol can be disabled via feature flag
2. Fallback to standard MCP tools/call
3. Gradual migration path available

### Extensions
The custom protocol can be extended to:
- Add new debugging tools
- Implement custom authentication
- Add rate limiting
- Provide metrics and monitoring

## Conclusion

The custom protocol implementation successfully works around the mcp-core v0.1 bug while:
- ✅ Maintaining all existing functionality
- ✅ Providing better performance and error handling
- ✅ Ensuring compatibility with existing clients
- ✅ Offering a clear migration path for future updates

This solution enables full GDB debugging capabilities via the custom HTTP protocol while preserving the MCP handshake for compatibility.
