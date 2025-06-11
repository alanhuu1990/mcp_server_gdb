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

## Summary
The main lesson is that MCP transport implementation requires careful attention to protocol details, proper error handling, and incremental testing. **CRITICAL**: Always test the actual MCP protocol implementation, not just the documentation, as library bugs can cause unexpected failures even when the protocol handshake appears successful. The documentation may not always match the actual implementation in specific language SDKs, so always verify against the actual code and test thoroughly.
