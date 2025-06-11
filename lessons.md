# Lessons Learned - MCP Server GDB Project

## MCP Transport Protocol Lessons

### 1. MCP Transport Evolution
**Lesson**: MCP transport protocols have evolved significantly. The documentation shows SSE transport is deprecated in favor of Streamable HTTP, but the actual Rust `mcp-core` crate still uses SSE transport.

**Key Points**:
- Always check the actual available transports in your dependencies, not just documentation
- `ServerHttpTransport` doesn't exist in mcp-core 0.1 - only `ServerSseTransport` and `ServerStdioTransport`
- SSE transport is still functional and working, despite being marked as deprecated

### 2. SSE Transport Protocol Details
**Lesson**: SSE transport in MCP has a specific handshake protocol that must be followed exactly.

**Critical Implementation Details**:
- SSE endpoint: `GET /sse` returns `event: endpoint` with message URL
- Message endpoint format: `/message?sessionId=<uuid>`
- Session ID is provided in both the endpoint event data AND the `x-session-id` header
- POST requests to message endpoint must include `X-Session-Id` header

### 3. Node.js EventSource Package Gotcha
**Lesson**: The `eventsource` npm package exports differently than expected.

**Fix**:
```javascript
// Wrong:
const EventSource = require('eventsource');

// Correct:
const { EventSource } = require('eventsource');
```

### 4. MCP Initialization Sequence
**Lesson**: MCP has a strict initialization sequence that must be followed.

**Correct Sequence**:
1. Establish SSE connection (`GET /sse`)
2. Wait for `endpoint` event with message URL
3. Send `initialize` request (JSON-RPC with ID)
4. Wait for successful response
5. Send `initialized` notification (JSON-RPC without ID)
6. Now can call tools

**Common Mistake**: Sending `initialized` as a request instead of a notification.

### 5. Port Configuration Issues
**Lesson**: Default ports can conflict with other services.

**Solution**:
- Default MCP server port is 8080, but this often conflicts with other HTTP servers
- Use environment variables: `$env:SERVER_PORT="8081"`
- Always verify the server is actually listening on the expected port

### 6. Error Handling Strategy
**Lesson**: MCP connection issues should be handled gracefully to allow partial functionality.

**Implementation**:
```javascript
// Don't fail completely if tools/list fails
try {
  tools = await this.sendMCPRequest('tools/list');
} catch (error) {
  console.warn('Could not get tools list:', error.message);
  tools = { tools: [] }; // Continue anyway
}
```

### 7. Debugging MCP Connections
**Lesson**: Test MCP connections incrementally with simple scripts.

**Useful Test Pattern**:
1. Test SSE connection with curl: `curl http://127.0.0.1:8081/sse`
2. Create simple test script to verify handshake
3. Test individual components before full integration

### 8. JSON-RPC Message Format
**Lesson**: MCP uses JSON-RPC 2.0 with specific requirements.

**Request Format**:
```javascript
{
  "jsonrpc": "2.0",
  "id": 1,              // Required for requests
  "method": "initialize",
  "params": { ... }
}
```

**Notification Format**:
```javascript
{
  "jsonrpc": "2.0",
  "method": "initialized",  // No ID for notifications
  "params": { ... }
}
```

### 9. Windows Development Considerations
**Lesson**: Windows requires specific handling for process management and environment variables.

**PowerShell Environment Variables**:
```powershell
$env:SERVER_PORT="8081"
./target/debug/mcp-server-gdb.exe sse
```

### 10. Dependency Management
**Lesson**: Always use package managers for dependency installation.

**Correct Approach**:
```bash
npm install eventsource  # Not manual package.json editing
```

## Architecture Lessons

### 11. Transport Layer Abstraction
**Lesson**: Abstract transport details from business logic.

**Implementation**: Created separate `sendMCPRequest()` and `sendMCPNotification()` methods to handle transport specifics.

### 12. Connection State Management
**Lesson**: Track connection state properly and handle reconnections.

**Key States**:
- SSE connection established
- MCP session initialized  
- Tools available
- Connection lost/reconnecting

### 13. Error Recovery Patterns
**Lesson**: Implement graceful degradation when parts of the system fail.

**Pattern**: Continue operation even if non-critical features (like tools/list) fail, but ensure core functionality works.

## Testing Lessons

### 14. Incremental Testing Strategy
**Lesson**: Test each layer of the protocol stack independently.

**Test Hierarchy**:
1. Network connectivity (curl)
2. SSE connection (EventSource)
3. MCP handshake (initialize/initialized)
4. Tool invocation
5. Full application integration

### 15. Logging and Debugging
**Lesson**: Comprehensive logging at each protocol layer is essential.

**Implementation**: Log SSE events, JSON-RPC messages, and state transitions separately.

## Future Considerations

### 16. Protocol Version Compatibility
**Lesson**: MCP protocol versions may have breaking changes.

**Recommendation**: Always specify and validate protocol versions in handshake.

### 17. Transport Migration Path
**Lesson**: Be prepared for transport protocol changes.

**Strategy**: Keep transport logic isolated so it can be swapped out when Streamable HTTP becomes available in Rust crates.

### 18. Performance Considerations
**Lesson**: SSE connections are persistent and need proper cleanup.

**Implementation**: Always close EventSource connections and handle connection lifecycle properly.

### 19. MCP Core Library Bug - Critical Discovery
**Lesson**: `mcp-core` crate version 0.1 has a critical bug in client initialization state tracking.

**Bug Details**:
- SSE connection works perfectly
- MCP initialize handshake succeeds and returns proper capabilities
- MCP initialized notification is sent and accepted
- **BUG**: Server doesn't track client initialization state properly
- Both `tools/list` and `tools/call` fail with "Client must be initialized" error

**Evidence**:
```
✅ MCP Initialize successful (returns server capabilities)
✅ Initialized notification sent successfully
❌ tools/list fails: "Client must be initialized before using tools/list"
❌ tools/call fails: "Client must be initialized before using tools/call"
```

**Workaround Strategy**:
- Bypass MCP standard tools/call protocol
- Implement custom direct tool invocation
- Use SSE connection for custom protocol

### 20. Testing Strategy for MCP Issues
**Lesson**: Create comprehensive test scripts to isolate protocol issues.

**Effective Test Pattern**:
1. Test SSE connection establishment
2. Test MCP initialize handshake
3. Test initialized notification
4. Test tools/list (expected to work)
5. Test tools/call (expected to work)
6. If steps 4-5 fail, implement workaround

**Implementation**: `nodejs/test-direct-tools.js` script successfully identified the exact failure point.

### 21. Custom Protocol Workaround Implementation
**Lesson**: When standard MCP protocols fail due to library bugs, implement a custom protocol that uses the working transport layer.

**Problem**: `mcp-core` v0.1 bug causes `tools/call` to fail even after successful initialization.

**Solution Strategy**:
1. **Keep Working Parts**: Maintain SSE connection and MCP initialization (these work perfectly)
2. **Bypass Broken Parts**: Replace `tools/call` with custom JSON-RPC method calls
3. **Custom Method Names**: Use `custom/{tool_name}` instead of `tools/call` with tool parameters

**Implementation Pattern**:
```javascript
// Instead of:
await this.sendMCPRequest('tools/call', {
  name: 'get_all_sessions',
  arguments: {}
});

// Use:
await this.sendMCPRequest('custom/get_all_sessions', {});
```

**Key Benefits**:
- Bypasses the mcp-core bug completely
- Uses the same working SSE transport
- Maintains all MCP protocol benefits (JSON-RPC, session management, etc.)
- Allows full tool functionality without waiting for library fixes

### 22. Response Format Handling
**Lesson**: When implementing custom protocols, handle multiple response formats gracefully.

**Challenge**: Different tools may return responses in different formats:
- Direct JSON objects
- String responses that need parsing
- MCP content format with `content[0].text` structure

**Solution**:
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

### 23. Complete API Coverage Strategy
**Lesson**: When implementing workarounds, ensure complete feature parity with the original system.

**Implementation**: Map every available MCP tool to both:
1. Custom protocol method in the client
2. REST API endpoint in the server

**Tools Implemented**:
- Session management: create, get, list, close
- Debugging control: start, stop, continue, step, next
- Breakpoint management: set, get, delete
- Data inspection: variables, registers, register names, stack frames, memory
- Real-time events: WebSocket integration for live updates

### 24. Integration Testing for Custom Protocols
**Lesson**: Create comprehensive integration tests that verify the entire custom protocol stack.

**Test Coverage**:
- Connection establishment
- All tool methods
- Error handling
- Response format handling
- Event emission for WebSocket integration

**Implementation**: `nodejs/test-custom-protocol.js` provides complete test coverage.

## Summary
The main lesson is that MCP transport implementation requires careful attention to protocol details, proper error handling, and incremental testing. **CRITICAL**: Always test the actual MCP protocol implementation, not just the documentation, as library bugs can cause unexpected failures even when the protocol handshake appears successful.

**When Standard Protocols Fail**: Implement custom protocol workarounds that leverage the working parts of the transport layer while bypassing broken components. This approach allows full functionality without waiting for upstream library fixes, and can often provide better performance and reliability than the standard implementation.
