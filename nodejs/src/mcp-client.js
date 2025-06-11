const EventEmitter = require('events');
const fetch = require('node-fetch');
const { EventSource } = require('eventsource');

class MCPClient extends EventEmitter {
  constructor(config, eventManager) {
    super();
    this.config = config;
    this.eventManager = eventManager;
    this.baseUrl = `${config.rust_server.protocol}://${config.rust_server.host}:${config.rust_server.port}`;
    this.customProtocolUrl = `${config.rust_server.protocol}://${config.rust_server.host}:${config.rust_server.port + 1}`;
    this.connected = false;
    this.reconnectTimer = null;
    this.reconnectAttempts = 0;
    this.sessions = new Map();
    this.requestId = 1;
    this.pendingRequests = new Map();
    this.eventSource = null;
    this.messageEndpoint = null;
    this.sessionId = null;

    console.log('MCP Client initialized for:', this.baseUrl);
    console.log('Custom Protocol URL:', this.customProtocolUrl);
  }

  async sendMCPRequest(method, params = {}) {
    return new Promise((resolve, reject) => {
      const requestId = this.requestId++;
      const request = {
        jsonrpc: '2.0',
        id: requestId,
        method: method,
        params: params
      };

      console.log(`MCP Request: ${method}`, params);

      // Store the pending request
      this.pendingRequests.set(requestId, { resolve, reject, timestamp: Date.now() });

      // For SSE transport, send request via POST to the message endpoint
      if (!this.messageEndpoint) {
        reject(new Error('No message endpoint available - SSE connection not established'));
        return;
      }

      const headers = {
        'Content-Type': 'application/json',
      };

      // Add session ID header if available
      if (this.sessionId) {
        headers['X-Session-Id'] = this.sessionId;
      }

      fetch(this.messageEndpoint, {
        method: 'POST',
        headers: headers,
        body: JSON.stringify(request)
      })
      .then(response => {
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        // For SSE transport, the response might be empty as the actual response comes via SSE
        if (response.headers.get('content-type')?.includes('application/json')) {
          return response.json();
        } else {
          return null; // Response will come via SSE
        }
      })
      .then(data => {
        if (data) {
          console.log(`MCP Response: ${method}`, data);
          if (data.error) {
            reject(new Error(data.error.message || 'MCP Error'));
          } else {
            resolve(data.result);
          }
          this.pendingRequests.delete(requestId);
        }
        // If no immediate response, wait for SSE response
      })
      .catch(error => {
        console.error(`MCP Error: ${method}`, error);
        reject(error);
        this.pendingRequests.delete(requestId);
      });

      // Timeout handling
      setTimeout(() => {
        if (this.pendingRequests.has(requestId)) {
          this.pendingRequests.delete(requestId);
          reject(new Error('Request timeout'));
        }
      }, this.config.bridge.timeout);
    });
  }

  async sendMCPNotification(method, params = {}) {
    const notification = {
      jsonrpc: '2.0',
      method: method,
      params: params
    };

    console.log(`MCP Notification: ${method}`, params);

    // For SSE transport, send notification via POST to the message endpoint
    if (!this.messageEndpoint) {
      throw new Error('No message endpoint available - SSE connection not established');
    }

    const headers = {
      'Content-Type': 'application/json',
    };

    // Add session ID header if available
    if (this.sessionId) {
      headers['X-Session-Id'] = this.sessionId;
    }

    try {
      const response = await fetch(this.messageEndpoint, {
        method: 'POST',
        headers: headers,
        body: JSON.stringify(notification)
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      console.log(`MCP Notification sent: ${method}`);
    } catch (error) {
      console.error(`MCP Notification Error: ${method}`, error);
      throw error;
    }
  }

  async connect() {
    try {
      // First, establish SSE connection to get the message endpoint
      await this.establishSSEConnection();

      // Initialize the MCP session
      const initResult = await this.sendMCPRequest('initialize', {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: {
          name: 'mcp-gdb-nodejs-bridge',
          version: '1.0.0'
        }
      });

      console.log('MCP initialization successful:', initResult);

      // Send initialized notification (no response expected)
      await this.sendMCPNotification('initialized', {});

      // Wait a moment for the server to process the initialized notification
      await new Promise(resolve => setTimeout(resolve, 100));

      // Try to get available tools, but don't fail if it doesn't work
      let tools = null;
      try {
        tools = await this.sendMCPRequest('tools/list');
        console.log('Available tools:', tools.tools?.map(t => t.name) || []);
      } catch (error) {
        console.warn('Could not get tools list:', error.message);
        // Continue anyway - we know the tools from the registration
        tools = { tools: [] };
      }

      this.connected = true;
      this.reconnectAttempts = 0;

      console.log('Connected to MCP Rust server');
      this.eventManager.emit('mcp_connected', { timestamp: new Date().toISOString() });

      // Start periodic health checks
      this.startHealthCheck();

      return tools;
    } catch (error) {
      console.error('Failed to connect to MCP server:', error.message);
      this.connected = false;
      this.scheduleReconnect();
      throw error;
    }
  }

  async establishSSEConnection() {
    return new Promise((resolve, reject) => {
      console.log('Establishing SSE connection to:', `${this.baseUrl}/sse`);

      this.eventSource = new EventSource(`${this.baseUrl}/sse`);

      this.eventSource.onopen = () => {
        console.log('SSE connection established');
        // Don't resolve immediately, wait for the endpoint event
      };

      this.eventSource.onerror = (error) => {
        console.error('SSE connection error:', error);
        if (!this.connected) {
          reject(new Error('Failed to establish SSE connection'));
        }
      };

      this.eventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.handleSSEMessage(data);
        } catch (error) {
          console.error('Failed to parse SSE message:', error);
        }
      };

      // Handle specific SSE events
      this.eventSource.addEventListener('endpoint', (event) => {
        try {
          console.log('Received endpoint event:', event.data);
          // The data is just the endpoint path, not JSON
          const endpointPath = event.data.trim();
          this.messageEndpoint = `${this.baseUrl}${endpointPath}`;
          console.log('Message endpoint set to:', this.messageEndpoint);

          // Extract session ID from the endpoint URL
          const sessionMatch = endpointPath.match(/sessionId=([^&]+)/);
          if (sessionMatch) {
            this.sessionId = sessionMatch[1];
            console.log('Session ID extracted:', this.sessionId);
          }

          // Now we can resolve the connection
          resolve();
        } catch (error) {
          console.error('Failed to parse endpoint event:', error);
          reject(error);
        }
      });

      // Timeout for connection establishment
      setTimeout(() => {
        if (!this.messageEndpoint) {
          this.eventSource.close();
          reject(new Error('SSE connection timeout - no endpoint received'));
        }
      }, 10000);
    });
  }

  handleSSEMessage(data) {
    console.log('SSE Message received:', data);

    // Handle JSON-RPC responses
    if (data.id && this.pendingRequests.has(data.id)) {
      const { resolve, reject } = this.pendingRequests.get(data.id);
      this.pendingRequests.delete(data.id);

      if (data.error) {
        reject(new Error(data.error.message || 'MCP Error'));
      } else {
        resolve(data.result);
      }
    }

    // Handle notifications and other messages
    if (data.method) {
      this.emit('notification', data);
    }
  }

  async disconnect() {
    this.connected = false;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.eventSource) {
      try {
        this.eventSource.close();
      } catch (error) {
        // Ignore errors during close
      }
      this.eventSource = null;
    }
    console.log('Disconnected from MCP server');
  }

  handleConnectionError() {
    if (this.connected) {
      this.connected = false;
      this.eventManager.emit('mcp_disconnected', { timestamp: new Date().toISOString() });
      this.scheduleReconnect();
    }
  }

  scheduleReconnect() {
    if (this.reconnectAttempts >= this.config.bridge.max_reconnect_attempts) {
      console.error('Max reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = this.config.bridge.reconnect_interval * this.reconnectAttempts;
    
    console.log(`Scheduling reconnection attempt ${this.reconnectAttempts} in ${delay}ms`);
    
    this.reconnectTimer = setTimeout(async () => {
      try {
        await this.connect();
      } catch (error) {
        console.error('Reconnection failed:', error.message);
      }
    }, delay);
  }

  startHealthCheck() {
    setInterval(async () => {
      if (this.connected) {
        try {
          // Test Agent-1's custom protocol health endpoint
          const response = await fetch(`${this.customProtocolUrl}/health`);
          if (!response.ok) {
            throw new Error(`Health check failed: ${response.status}`);
          }
        } catch (error) {
          console.error('Health check failed:', error.message);
          this.handleConnectionError();
        }
      }
    }, 30000); // Check every 30 seconds
  }

  // Custom Protocol Tool Methods (Workaround for mcp-core v0.1 bug)
  // Instead of using tools/call, we use Agent-1's HTTP REST API

  // Helper function to handle Agent-1's response format
  handleCustomProtocolResponse(result, expectJson = false) {
    if (result && result.message) {
      if (expectJson) {
        try {
          return JSON.parse(result.message);
        } catch (parseError) {
          // If not JSON, return as text wrapped in object
          return { data: result.message };
        }
      }
      return result.message;
    }
    return result;
  }

  async sendCustomToolRequest(toolName, params = {}) {
    try {
      console.log(`🔧 Custom Tool Request: ${toolName}`, params);

      // Use Agent-1's HTTP REST API on port 8082
      const response = await fetch(`${this.customProtocolUrl}/api/tools/${toolName}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ params: params })
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      console.log(`✅ Custom Tool Response: ${toolName}`, result);

      if (!result.success) {
        throw new Error(result.error || 'Tool execution failed');
      }

      return result.data;
    } catch (error) {
      console.error(`❌ Custom Tool Error: ${toolName}`, error.message);
      throw new Error(`Failed to call ${toolName}: ${error.message}`);
    }
  }

  async getSessions() {
    try {
      const result = await this.sendCustomToolRequest('get_all_sessions', {});
      const sessions = this.handleCustomProtocolResponse(result, true);
      return sessions || { sessions: [] };
    } catch (error) {
      throw new Error(`Failed to get sessions: ${error.message}`);
    }
  }

  async createSession(sessionData) {
    try {
      const result = await this.sendCustomToolRequest('create_session', sessionData);
      const sessionText = this.handleCustomProtocolResponse(result);

      const sessionId = sessionText.match(/Created GDB session: (.+)/)?.[1];

      if (sessionId) {
        const session = { id: sessionId, ...sessionData };
        this.sessions.set(sessionId, session);
        this.eventManager.emit('session_created', session);
        return session;
      }

      throw new Error('Failed to extract session ID from response');
    } catch (error) {
      throw new Error(`Failed to create session: ${error.message}`);
    }
  }

  async getSession(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('get_session', { session_id: sessionId });
      // Handle both direct JSON response and text content response
      if (typeof result === 'string') {
        return JSON.parse(result);
      } else if (result.content && result.content[0] && result.content[0].text) {
        return JSON.parse(result.content[0].text);
      } else {
        return result;
      }
    } catch (error) {
      throw new Error(`Failed to get session: ${error.message}`);
    }
  }

  async getVariables(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('get_local_variables', { session_id: sessionId });

      // Handle different response formats
      let variables;
      if (typeof result === 'string') {
        variables = JSON.parse(result);
      } else if (result.content && result.content[0] && result.content[0].text) {
        variables = JSON.parse(result.content[0].text);
      } else {
        variables = result;
      }

      // Emit variable update event
      this.eventManager.emit('variable_changed', {
        sessionId,
        variables,
        timestamp: new Date().toISOString()
      });

      return variables;
    } catch (error) {
      throw new Error(`Failed to get variables: ${error.message}`);
    }
  }

  async getRegisters(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('get_registers', { session_id: sessionId });

      // Handle different response formats
      let registers;
      if (typeof result === 'string') {
        registers = JSON.parse(result);
      } else if (result.content && result.content[0] && result.content[0].text) {
        registers = JSON.parse(result.content[0].text);
      } else {
        registers = result;
      }

      // Emit register update event
      this.eventManager.emit('register_changed', {
        sessionId,
        registers,
        timestamp: new Date().toISOString()
      });

      return registers;
    } catch (error) {
      throw new Error(`Failed to get registers: ${error.message}`);
    }
  }

  async setBreakpoint(sessionId, breakpointData) {
    try {
      const result = await this.sendCustomToolRequest('set_breakpoint', {
        session_id: sessionId,
        file: breakpointData.file,
        line: breakpointData.line
      });

      // Handle different response formats
      let breakpoint;
      if (typeof result === 'string') {
        breakpoint = JSON.parse(result);
      } else if (result.content && result.content[0] && result.content[0].text) {
        breakpoint = JSON.parse(result.content[0].text);
      } else {
        breakpoint = result;
      }

      this.eventManager.emit('breakpoint_set', {
        sessionId,
        breakpoint,
        timestamp: new Date().toISOString()
      });

      return breakpoint;
    } catch (error) {
      throw new Error(`Failed to set breakpoint: ${error.message}`);
    }
  }

  async continueExecution(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('continue_execution', { session_id: sessionId });

      this.eventManager.emit('execution_continued', {
        sessionId,
        timestamp: new Date().toISOString()
      });

      // Handle different response formats
      if (typeof result === 'string') {
        return result;
      } else if (result.content && result.content[0] && result.content[0].text) {
        return result.content[0].text;
      } else {
        return JSON.stringify(result);
      }
    } catch (error) {
      throw new Error(`Failed to continue execution: ${error.message}`);
    }
  }

  async stepExecution(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('step_execution', { session_id: sessionId });

      this.eventManager.emit('execution_stepped', {
        sessionId,
        timestamp: new Date().toISOString()
      });

      // Handle different response formats
      if (typeof result === 'string') {
        return result;
      } else if (result.content && result.content[0] && result.content[0].text) {
        return result.content[0].text;
      } else {
        return JSON.stringify(result);
      }
    } catch (error) {
      throw new Error(`Failed to step execution: ${error.message}`);
    }
  }

  async stopExecution(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('stop_debugging', { session_id: sessionId });

      this.eventManager.emit('execution_stopped', {
        sessionId,
        timestamp: new Date().toISOString()
      });

      // Handle different response formats
      if (typeof result === 'string') {
        return result;
      } else if (result.content && result.content[0] && result.content[0].text) {
        return result.content[0].text;
      } else {
        return JSON.stringify(result);
      }
    } catch (error) {
      throw new Error(`Failed to stop execution: ${error.message}`);
    }
  }

  // Additional tool methods for complete GDB functionality
  async closeSession(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('close_session', { session_id: sessionId });

      // Remove from local sessions map
      this.sessions.delete(sessionId);

      this.eventManager.emit('session_closed', {
        sessionId,
        timestamp: new Date().toISOString()
      });

      // Handle different response formats
      if (typeof result === 'string') {
        return result;
      } else if (result.content && result.content[0] && result.content[0].text) {
        return result.content[0].text;
      } else {
        return JSON.stringify(result);
      }
    } catch (error) {
      throw new Error(`Failed to close session: ${error.message}`);
    }
  }

  async startDebugging(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('start_debugging', { session_id: sessionId });

      this.eventManager.emit('debugging_started', {
        sessionId,
        timestamp: new Date().toISOString()
      });

      // Handle different response formats
      if (typeof result === 'string') {
        return result;
      } else if (result.content && result.content[0] && result.content[0].text) {
        return result.content[0].text;
      } else {
        return JSON.stringify(result);
      }
    } catch (error) {
      throw new Error(`Failed to start debugging: ${error.message}`);
    }
  }

  async getBreakpoints(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('get_breakpoints', { session_id: sessionId });

      // Handle different response formats
      let breakpoints;
      if (typeof result === 'string') {
        breakpoints = JSON.parse(result);
      } else if (result.content && result.content[0] && result.content[0].text) {
        breakpoints = JSON.parse(result.content[0].text);
      } else {
        breakpoints = result;
      }

      return breakpoints;
    } catch (error) {
      throw new Error(`Failed to get breakpoints: ${error.message}`);
    }
  }

  async deleteBreakpoint(sessionId, breakpointId) {
    try {
      const result = await this.sendCustomToolRequest('delete_breakpoint', {
        session_id: sessionId,
        breakpoint_id: breakpointId
      });

      this.eventManager.emit('breakpoint_deleted', {
        sessionId,
        breakpointId,
        timestamp: new Date().toISOString()
      });

      // Handle different response formats
      if (typeof result === 'string') {
        return result;
      } else if (result.content && result.content[0] && result.content[0].text) {
        return result.content[0].text;
      } else {
        return JSON.stringify(result);
      }
    } catch (error) {
      throw new Error(`Failed to delete breakpoint: ${error.message}`);
    }
  }

  async getStackFrames(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('get_stack_frames', { session_id: sessionId });

      // Handle different response formats
      let stackFrames;
      if (typeof result === 'string') {
        stackFrames = JSON.parse(result);
      } else if (result.content && result.content[0] && result.content[0].text) {
        stackFrames = JSON.parse(result.content[0].text);
      } else {
        stackFrames = result;
      }

      return stackFrames;
    } catch (error) {
      throw new Error(`Failed to get stack frames: ${error.message}`);
    }
  }

  async nextExecution(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('next_execution', { session_id: sessionId });

      this.eventManager.emit('execution_next', {
        sessionId,
        timestamp: new Date().toISOString()
      });

      // Handle different response formats
      if (typeof result === 'string') {
        return result;
      } else if (result.content && result.content[0] && result.content[0].text) {
        return result.content[0].text;
      } else {
        return JSON.stringify(result);
      }
    } catch (error) {
      throw new Error(`Failed to step over: ${error.message}`);
    }
  }

  async getRegisterNames(sessionId) {
    try {
      const result = await this.sendCustomToolRequest('get_register_names', { session_id: sessionId });

      // Handle different response formats
      let registerNames;
      if (typeof result === 'string') {
        registerNames = JSON.parse(result);
      } else if (result.content && result.content[0] && result.content[0].text) {
        registerNames = JSON.parse(result.content[0].text);
      } else {
        registerNames = result;
      }

      return registerNames;
    } catch (error) {
      throw new Error(`Failed to get register names: ${error.message}`);
    }
  }

  async readMemory(sessionId, address, size) {
    try {
      const result = await this.sendCustomToolRequest('read_memory', {
        session_id: sessionId,
        address: address,
        size: size
      });

      // Handle different response formats
      let memoryData;
      if (typeof result === 'string') {
        memoryData = JSON.parse(result);
      } else if (result.content && result.content[0] && result.content[0].text) {
        memoryData = JSON.parse(result.content[0].text);
      } else {
        memoryData = result;
      }

      return memoryData;
    } catch (error) {
      throw new Error(`Failed to read memory: ${error.message}`);
    }
  }

  isConnected() {
    return this.connected;
  }
}

module.exports = MCPClient;
