# Custom Protocol Implementation for MCP GDB Server

## Overview

This document describes the custom protocol implementation that bypasses the `mcp-core` v0.1 bug where `tools/call` fails even after successful MCP initialization.

## Problem Statement

The standard MCP protocol flow should be:
1. ✅ SSE connection establishment
2. ✅ MCP initialize handshake  
3. ✅ MCP initialized notification
4. ❌ `tools/list` - fails with "Client must be initialized"
5. ❌ `tools/call` - fails with "Client must be initialized"

Despite successful initialization, the mcp-core library doesn't properly track client state, causing all tool operations to fail.

## Solution: Custom Protocol Workaround

### Architecture

```
Node.js Client ←→ SSE Transport ←→ Rust MCP Server
     ↓                                    ↓
Custom Protocol                    Standard MCP Tools
(bypass tools/call)               (get_all_sessions, etc.)
```

### Implementation Strategy

1. **Keep Working Components**: 
   - SSE connection establishment
   - MCP initialize handshake
   - Session management
   - JSON-RPC message transport

2. **Replace Broken Components**:
   - Replace `tools/call` with direct custom method calls
   - Use `custom/{tool_name}` method names instead of `tools/call`

### Custom Protocol Format

**Standard MCP (Broken):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_all_sessions",
    "arguments": {}
  }
}
```

**Custom Protocol (Working):**
```json
{
  "jsonrpc": "2.0", 
  "id": 1,
  "method": "custom/get_all_sessions",
  "params": {}
}
```

## Available Tools

### Session Management
- `custom/get_all_sessions` - List all GDB sessions
- `custom/create_session` - Create new GDB session
- `custom/get_session` - Get session details by ID
- `custom/close_session` - Close GDB session

### Debugging Control
- `custom/start_debugging` - Start debugging session
- `custom/stop_debugging` - Stop debugging session
- `custom/continue_execution` - Continue program execution
- `custom/step_execution` - Step into (single instruction)
- `custom/next_execution` - Step over (next line)

### Breakpoint Management
- `custom/get_breakpoints` - List all breakpoints
- `custom/set_breakpoint` - Set breakpoint at file:line
- `custom/delete_breakpoint` - Delete breakpoint by ID

### Data Inspection
- `custom/get_local_variables` - Get local variables in current scope
- `custom/get_registers` - Get CPU register values
- `custom/get_register_names` - Get available register names
- `custom/get_stack_frames` - Get call stack frames
- `custom/read_memory` - Read memory at address

## API Endpoints

The Node.js server provides REST API endpoints for all tools:

### Session Management
- `GET /api/sessions` - List sessions
- `POST /api/sessions` - Create session
- `DELETE /api/sessions/:id` - Close session

### Debugging Control  
- `POST /api/sessions/:id/start` - Start debugging
- `POST /api/sessions/:id/stop` - Stop debugging
- `POST /api/sessions/:id/continue` - Continue execution
- `POST /api/sessions/:id/step` - Step into
- `POST /api/sessions/:id/next` - Step over

### Data Inspection
- `GET /api/sessions/:id/variables` - Get variables
- `GET /api/sessions/:id/registers` - Get registers
- `GET /api/sessions/:id/register-names` - Get register names
- `GET /api/sessions/:id/stack` - Get stack frames
- `GET /api/sessions/:id/memory?address=&size=` - Read memory

### Breakpoint Management
- `GET /api/sessions/:id/breakpoints` - Get breakpoints
- `POST /api/sessions/:id/breakpoints` - Set breakpoint
- `DELETE /api/sessions/:id/breakpoints/:breakpointId` - Delete breakpoint

## Response Format Handling

The custom protocol handles multiple response formats:

```javascript
// Handle different response formats
let result;
if (typeof response === 'string') {
  result = JSON.parse(response);
} else if (response.content && response.content[0] && response.content[0].text) {
  result = JSON.parse(response.content[0].text);
} else {
  result = response;
}
```

## WebSocket Integration

Real-time events are emitted for WebSocket dashboard updates:

- `session_created` - New session created
- `session_closed` - Session closed
- `debugging_started` - Debugging started
- `execution_continued` - Execution continued
- `execution_stepped` - Step executed
- `execution_next` - Next executed
- `execution_stopped` - Execution stopped
- `breakpoint_set` - Breakpoint set
- `breakpoint_deleted` - Breakpoint deleted
- `variable_changed` - Variables updated
- `register_changed` - Registers updated

## Testing

### Integration Test
Run the comprehensive integration test:
```bash
cd nodejs
node test-custom-protocol.js
```

### Manual Testing
1. Start Rust server:
   ```bash
   $env:SERVER_PORT="8081"; ./target/debug/mcp-server-gdb.exe --log-level debug sse
   ```

2. Start Node.js server:
   ```bash
   cd nodejs
   node src/server.js
   ```

3. Test API endpoints:
   ```bash
   curl http://localhost:3000/api/sessions
   curl -X POST http://localhost:3000/api/sessions -H "Content-Type: application/json" -d '{"program":"/path/to/program","gdb_path":"gdb"}'
   ```

## Benefits

1. **Full Functionality**: All MCP tools work despite library bug
2. **Better Performance**: Direct method calls without tools/call overhead
3. **Flexible Response Handling**: Supports multiple response formats
4. **Complete API Coverage**: REST endpoints for all debugging operations
5. **Real-time Updates**: WebSocket integration for live dashboard
6. **Future-Proof**: Easy to migrate back to standard MCP when bug is fixed

## Migration Path

When mcp-core library is fixed, migration back to standard MCP is simple:

1. Replace `sendCustomToolRequest()` calls with `sendMCPRequest('tools/call', ...)`
2. Update method names from `custom/{tool}` back to `tools/call` format
3. Remove custom response format handling if no longer needed

The custom protocol provides a robust workaround that maintains full functionality while the upstream library issue is resolved.
