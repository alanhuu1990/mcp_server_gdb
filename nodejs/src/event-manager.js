const EventEmitter = require('events');

class EventManager extends EventEmitter {
  constructor() {
    super();
    this.sessions = new Map();
    this.activePolling = new Map();
    this.logBuffer = [];
    this.maxLogEntries = 1000;
    
    this.setupEventHandlers();
  }

  setupEventHandlers() {
    // Handle session events
    this.on('session_created', (session) => {
      this.sessions.set(session.id, {
        ...session,
        lastUpdate: Date.now(),
        variables: new Map(),
        registers: new Map(),
        breakpoints: new Map()
      });
      
      this.log('info', `Session created: ${session.id}`);
    });

    this.on('session_updated', (session) => {
      if (this.sessions.has(session.id)) {
        this.sessions.set(session.id, {
          ...this.sessions.get(session.id),
          ...session,
          lastUpdate: Date.now()
        });
      }
      
      this.log('info', `Session updated: ${session.id}`);
    });

    // Handle variable changes
    this.on('variable_changed', (data) => {
      const session = this.sessions.get(data.sessionId);
      if (session) {
        // Update variable cache
        if (data.variables) {
          data.variables.forEach(variable => {
            session.variables.set(variable.name, variable);
          });
        }
        
        session.lastUpdate = Date.now();
      }
    });

    // Handle register changes
    this.on('register_changed', (data) => {
      const session = this.sessions.get(data.sessionId);
      if (session) {
        // Update register cache
        if (data.registers) {
          data.registers.forEach(register => {
            session.registers.set(register.name, register);
          });
        }
        
        session.lastUpdate = Date.now();
      }
    });

    // Handle breakpoint events
    this.on('breakpoint_set', (data) => {
      const session = this.sessions.get(data.sessionId);
      if (session && data.breakpoint) {
        session.breakpoints.set(data.breakpoint.id, data.breakpoint);
      }
      
      this.log('info', `Breakpoint set in session ${data.sessionId}`);
    });

    this.on('breakpoint_hit', (data) => {
      this.log('info', `Breakpoint hit in session ${data.sessionId}`);
      
      // Automatically fetch updated variables and registers when breakpoint is hit
      this.fetchSessionData(data.sessionId);
    });

    // Handle execution state changes
    this.on('execution_stopped', (data) => {
      this.log('info', `Execution stopped in session ${data.sessionId}`);
      this.fetchSessionData(data.sessionId);
    });

    this.on('execution_continued', (data) => {
      this.log('info', `Execution continued in session ${data.sessionId}`);
    });

    this.on('execution_stepped', (data) => {
      this.log('info', `Execution stepped in session ${data.sessionId}`);
      this.fetchSessionData(data.sessionId);
    });
  }

  async executeDebugCommand(command) {
    try {
      this.log('debug', `Executing debug command: ${command.type}`);
      
      switch (command.type) {
        case 'continue':
          return await this.continueExecution(command.sessionId);
        case 'step':
          return await this.stepExecution(command.sessionId);
        case 'stop':
          return await this.stopExecution(command.sessionId);
        case 'set_breakpoint':
          return await this.setBreakpoint(command);
        case 'get_variables':
          return await this.getVariables(command.sessionId);
        case 'get_registers':
          return await this.getRegisters(command.sessionId);
        default:
          throw new Error(`Unknown command type: ${command.type}`);
      }
    } catch (error) {
      this.log('error', `Debug command failed: ${error.message}`);
      throw error;
    }
  }

  async fetchSessionData(sessionId) {
    try {
      // This would typically call the MCP bridge to fetch fresh data
      // For now, we'll emit events to trigger data refresh
      this.emit('data_refresh_requested', { sessionId });
    } catch (error) {
      this.log('error', `Failed to fetch session data: ${error.message}`);
    }
  }

  startVariablePolling(sessionId, interval = 1000) {
    if (this.activePolling.has(sessionId)) {
      return; // Already polling
    }

    const pollTimer = setInterval(async () => {
      try {
        this.emit('poll_variables', { sessionId });
      } catch (error) {
        this.log('error', `Variable polling error: ${error.message}`);
      }
    }, interval);

    this.activePolling.set(sessionId, pollTimer);
    this.log('info', `Started variable polling for session ${sessionId}`);
  }

  stopVariablePolling(sessionId) {
    const pollTimer = this.activePolling.get(sessionId);
    if (pollTimer) {
      clearInterval(pollTimer);
      this.activePolling.delete(sessionId);
      this.log('info', `Stopped variable polling for session ${sessionId}`);
    }
  }

  // Placeholder methods that would interact with MCP bridge
  async getSessions() {
    return Array.from(this.sessions.values());
  }

  async getVariables(sessionId) {
    const session = this.sessions.get(sessionId);
    if (session) {
      return Array.from(session.variables.values());
    }
    return [];
  }

  async getRegisters(sessionId) {
    const session = this.sessions.get(sessionId);
    if (session) {
      return Array.from(session.registers.values());
    }
    return [];
  }

  async setBreakpoint(data) {
    // This would call the MCP bridge
    this.emit('breakpoint_set', data);
    return { success: true, id: Date.now().toString() };
  }

  async continueExecution(sessionId) {
    this.emit('execution_continued', { sessionId });
    return { success: true };
  }

  async stepExecution(sessionId) {
    this.emit('execution_stepped', { sessionId });
    return { success: true };
  }

  async stopExecution(sessionId) {
    this.emit('execution_stopped', { sessionId });
    return { success: true };
  }

  log(level, message) {
    const logEntry = {
      timestamp: new Date().toISOString(),
      level,
      message
    };

    this.logBuffer.push(logEntry);
    
    // Keep buffer size manageable
    if (this.logBuffer.length > this.maxLogEntries) {
      this.logBuffer.shift();
    }

    // Emit log event for real-time streaming
    this.emit('log_message', logEntry);

    // Also log to console
    console.log(`[${level.toUpperCase()}] ${message}`);
  }

  getLogs(limit = 100) {
    return this.logBuffer.slice(-limit);
  }

  getSessionInfo(sessionId) {
    return this.sessions.get(sessionId);
  }

  getAllSessions() {
    return Array.from(this.sessions.values());
  }

  cleanup() {
    // Stop all polling
    this.activePolling.forEach((timer, sessionId) => {
      clearInterval(timer);
    });
    this.activePolling.clear();
    
    // Clear sessions
    this.sessions.clear();
    
    // Clear logs
    this.logBuffer = [];
    
    this.log('info', 'Event manager cleaned up');
  }
}

module.exports = EventManager;
