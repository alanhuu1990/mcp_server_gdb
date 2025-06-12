const { Server } = require('socket.io');

class WebSocketServer {
  constructor(httpServer, config, eventManager) {
    this.httpServer = httpServer;
    this.config = config;
    this.eventManager = eventManager;
    this.io = null;
    this.clients = new Map();
    this.running = false;
  }

  start() {
    this.io = new Server(this.httpServer, {
      cors: this.config.cors,
      path: this.config.path || '/socket.io/'
    });

    this.setupEventHandlers();
    this.setupEventManagerListeners();
    this.running = true;
    
    console.log('WebSocket server started');
  }

  setupEventHandlers() {
    this.io.on('connection', (socket) => {
      console.log(`Client connected: ${socket.id}`);
      
      // Store client info
      this.clients.set(socket.id, {
        socket,
        subscriptions: new Set(),
        lastActivity: Date.now()
      });

      // Handle client events
      socket.on('subscribe', (data) => {
        this.handleSubscribe(socket, data);
      });

      socket.on('unsubscribe', (data) => {
        this.handleUnsubscribe(socket, data);
      });

      socket.on('debug_command', async (data) => {
        await this.handleDebugCommand(socket, data);
      });

      socket.on('get_sessions', async () => {
        await this.handleGetSessions(socket);
      });

      socket.on('get_variables', async (data) => {
        await this.handleGetVariables(socket, data);
      });

      socket.on('get_registers', async (data) => {
        await this.handleGetRegisters(socket, data);
      });

      socket.on('set_breakpoint', async (data) => {
        await this.handleSetBreakpoint(socket, data);
      });

      socket.on('disconnect', () => {
        console.log(`Client disconnected: ${socket.id}`);
        this.clients.delete(socket.id);
      });

      // Send initial connection confirmation
      socket.emit('connected', {
        id: socket.id,
        timestamp: new Date().toISOString()
      });
    });
  }

  setupEventManagerListeners() {
    // Listen for debugging events from the event manager
    this.eventManager.on('session_created', (data) => {
      this.broadcast('session_created', data);
    });

    this.eventManager.on('session_updated', (data) => {
      this.broadcast('session_updated', data);
    });

    this.eventManager.on('breakpoint_hit', (data) => {
      this.broadcast('breakpoint_hit', data);
    });

    this.eventManager.on('variable_changed', (data) => {
      this.broadcastToSubscribers('variables', data);
    });

    this.eventManager.on('register_changed', (data) => {
      this.broadcastToSubscribers('registers', data);
    });

    this.eventManager.on('execution_stopped', (data) => {
      this.broadcast('execution_stopped', data);
    });

    this.eventManager.on('execution_continued', (data) => {
      this.broadcast('execution_continued', data);
    });

    this.eventManager.on('log_message', (data) => {
      this.broadcastToSubscribers('logs', data);
    });
  }

  handleSubscribe(socket, data) {
    const client = this.clients.get(socket.id);
    if (client && data.type) {
      client.subscriptions.add(data.type);
      socket.emit('subscribed', { type: data.type });
      console.log(`Client ${socket.id} subscribed to ${data.type}`);
    }
  }

  handleUnsubscribe(socket, data) {
    const client = this.clients.get(socket.id);
    if (client && data.type) {
      client.subscriptions.delete(data.type);
      socket.emit('unsubscribed', { type: data.type });
      console.log(`Client ${socket.id} unsubscribed from ${data.type}`);
    }
  }

  async handleDebugCommand(socket, data) {
    try {
      const result = await this.eventManager.executeDebugCommand(data);
      socket.emit('debug_command_result', {
        id: data.id,
        success: true,
        result
      });
    } catch (error) {
      socket.emit('debug_command_result', {
        id: data.id,
        success: false,
        error: error.message
      });
    }
  }

  async handleGetSessions(socket) {
    try {
      const sessions = await this.eventManager.getSessions();
      socket.emit('sessions_data', sessions);
    } catch (error) {
      socket.emit('error', { message: error.message });
    }
  }

  async handleGetVariables(socket, data) {
    try {
      const variables = await this.eventManager.getVariables(data.sessionId);
      socket.emit('variables_data', {
        sessionId: data.sessionId,
        variables
      });
    } catch (error) {
      socket.emit('error', { message: error.message });
    }
  }

  async handleGetRegisters(socket, data) {
    try {
      const registers = await this.eventManager.getRegisters(data.sessionId);
      socket.emit('registers_data', {
        sessionId: data.sessionId,
        registers
      });
    } catch (error) {
      socket.emit('error', { message: error.message });
    }
  }

  async handleSetBreakpoint(socket, data) {
    try {
      const result = await this.eventManager.setBreakpoint(data);
      socket.emit('breakpoint_set', result);
    } catch (error) {
      socket.emit('error', { message: error.message });
    }
  }

  broadcast(event, data) {
    this.io.emit(event, data);
  }

  broadcastToSubscribers(subscriptionType, data) {
    this.clients.forEach((client, socketId) => {
      if (client.subscriptions.has(subscriptionType)) {
        client.socket.emit(subscriptionType, data);
      }
    });
  }

  async stop() {
    if (this.io) {
      this.io.close();
      this.running = false;
      console.log('WebSocket server stopped');
    }
  }

  isRunning() {
    return this.running;
  }

  getClientCount() {
    return this.clients.size;
  }
}

module.exports = WebSocketServer;
