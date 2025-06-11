const axios = require('axios');
const EventEmitter = require('events');

class MCPBridge extends EventEmitter {
  constructor(config, eventManager) {
    super();
    this.config = config;
    this.eventManager = eventManager;
    this.baseUrl = `${config.rust_server.protocol}://${config.rust_server.host}:${config.rust_server.port}`;
    this.connected = false;
    this.reconnectTimer = null;
    this.reconnectAttempts = 0;
    this.sessions = new Map();
    
    // Setup axios instance with timeout
    this.client = axios.create({
      baseURL: this.baseUrl,
      timeout: config.bridge.timeout,
      headers: {
        'Content-Type': 'application/json'
      }
    });

    this.setupAxiosInterceptors();
  }

  setupAxiosInterceptors() {
    // Request interceptor
    this.client.interceptors.request.use(
      (config) => {
        console.log(`MCP Request: ${config.method?.toUpperCase()} ${config.url}`);
        return config;
      },
      (error) => {
        console.error('MCP Request Error:', error);
        return Promise.reject(error);
      }
    );

    // Response interceptor
    this.client.interceptors.response.use(
      (response) => {
        console.log(`MCP Response: ${response.status} ${response.config.url}`);
        return response;
      },
      (error) => {
        console.error('MCP Response Error:', error.message);
        if (error.code === 'ECONNREFUSED') {
          this.handleConnectionError();
        }
        return Promise.reject(error);
      }
    );
  }

  async connect() {
    try {
      // Test connection to Rust MCP server
      const response = await this.client.get('/health');
      this.connected = true;
      this.reconnectAttempts = 0;
      
      console.log('Connected to MCP Rust server');
      this.eventManager.emit('mcp_connected', { timestamp: new Date().toISOString() });
      
      // Start periodic health checks
      this.startHealthCheck();
      
      return response.data;
    } catch (error) {
      console.error('Failed to connect to MCP server:', error.message);
      this.connected = false;
      this.scheduleReconnect();
      throw error;
    }
  }

  async disconnect() {
    this.connected = false;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
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
          await this.client.get('/health');
        } catch (error) {
          console.error('Health check failed:', error.message);
          this.handleConnectionError();
        }
      }
    }, 30000); // Check every 30 seconds
  }

  // MCP API Methods
  async getSessions() {
    try {
      const response = await this.client.get('/api/sessions');
      return response.data;
    } catch (error) {
      throw new Error(`Failed to get sessions: ${error.message}`);
    }
  }

  async createSession(sessionData) {
    try {
      const response = await this.client.post('/api/sessions', sessionData);
      const session = response.data;
      
      this.sessions.set(session.id, session);
      this.eventManager.emit('session_created', session);
      
      return session;
    } catch (error) {
      throw new Error(`Failed to create session: ${error.message}`);
    }
  }

  async getSession(sessionId) {
    try {
      const response = await this.client.get(`/api/sessions/${sessionId}`);
      return response.data;
    } catch (error) {
      throw new Error(`Failed to get session: ${error.message}`);
    }
  }

  async getVariables(sessionId) {
    try {
      const response = await this.client.get(`/api/sessions/${sessionId}/variables`);
      const variables = response.data;
      
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
      const response = await this.client.get(`/api/sessions/${sessionId}/registers`);
      const registers = response.data;
      
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
      const response = await this.client.post(`/api/sessions/${sessionId}/breakpoints`, breakpointData);
      const breakpoint = response.data;
      
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
      const response = await this.client.post(`/api/sessions/${sessionId}/continue`);
      
      this.eventManager.emit('execution_continued', {
        sessionId,
        timestamp: new Date().toISOString()
      });
      
      return response.data;
    } catch (error) {
      throw new Error(`Failed to continue execution: ${error.message}`);
    }
  }

  async stepExecution(sessionId) {
    try {
      const response = await this.client.post(`/api/sessions/${sessionId}/step`);
      
      this.eventManager.emit('execution_stepped', {
        sessionId,
        timestamp: new Date().toISOString()
      });
      
      return response.data;
    } catch (error) {
      throw new Error(`Failed to step execution: ${error.message}`);
    }
  }

  async stopExecution(sessionId) {
    try {
      const response = await this.client.post(`/api/sessions/${sessionId}/stop`);
      
      this.eventManager.emit('execution_stopped', {
        sessionId,
        timestamp: new Date().toISOString()
      });
      
      return response.data;
    } catch (error) {
      throw new Error(`Failed to stop execution: ${error.message}`);
    }
  }

  isConnected() {
    return this.connected;
  }
}

module.exports = MCPBridge;
