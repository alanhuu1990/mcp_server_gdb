#!/usr/bin/env node

const express = require('express');
const http = require('http');
const path = require('path');
const cors = require('cors');
const helmet = require('helmet');
const morgan = require('morgan');
require('dotenv').config();

const WebSocketServer = require('./websocket-server');
const MCPBridge = require('./mcp-bridge');
const EventManager = require('./event-manager');
const config = require('../config/default.json');

class MCPGDBNodeServer {
  constructor() {
    this.app = express();
    this.server = http.createServer(this.app);
    this.config = config;
    this.mcpBridge = null;
    this.wsServer = null;
    this.eventManager = null;
    
    this.setupMiddleware();
    this.setupRoutes();
    this.setupServices();
  }

  setupMiddleware() {
    // Security middleware
    if (this.config.security.enable_helmet) {
      this.app.use(helmet({
        contentSecurityPolicy: {
          directives: {
            defaultSrc: ["'self'"],
            scriptSrc: ["'self'", "'unsafe-inline'"],
            styleSrc: ["'self'", "'unsafe-inline'"],
            connectSrc: ["'self'", `ws://localhost:${this.config.websocket.port}`]
          }
        }
      }));
    }

    // CORS
    if (this.config.security.enable_cors) {
      this.app.use(cors(this.config.websocket.cors));
    }

    // Logging
    this.app.use(morgan('combined'));

    // Body parsing
    this.app.use(express.json());
    this.app.use(express.urlencoded({ extended: true }));

    // Static files
    this.app.use(express.static(path.join(__dirname, '../public')));
  }

  setupRoutes() {
    // Health check
    this.app.get('/health', (req, res) => {
      res.json({
        status: 'ok',
        timestamp: new Date().toISOString(),
        services: {
          mcp_bridge: this.mcpBridge ? this.mcpBridge.isConnected() : false,
          websocket: this.wsServer ? this.wsServer.isRunning() : false
        }
      });
    });

    // API routes
    this.app.get('/api/sessions', async (req, res) => {
      try {
        const sessions = await this.mcpBridge.getSessions();
        res.json(sessions);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.post('/api/sessions', async (req, res) => {
      try {
        const session = await this.mcpBridge.createSession(req.body);
        res.json(session);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.get('/api/sessions/:id/variables', async (req, res) => {
      try {
        const variables = await this.mcpBridge.getVariables(req.params.id);
        res.json(variables);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.get('/api/sessions/:id/registers', async (req, res) => {
      try {
        const registers = await this.mcpBridge.getRegisters(req.params.id);
        res.json(registers);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    // Dashboard route
    this.app.get('/', (req, res) => {
      res.sendFile(path.join(__dirname, '../public/index.html'));
    });

    // 404 handler
    this.app.use((req, res) => {
      res.status(404).json({ error: 'Not found' });
    });

    // Error handler
    this.app.use((err, req, res, next) => {
      console.error('Server error:', err);
      res.status(500).json({ error: 'Internal server error' });
    });
  }

  async setupServices() {
    try {
      // Initialize Event Manager
      this.eventManager = new EventManager();

      // Initialize MCP Bridge
      this.mcpBridge = new MCPBridge(this.config.mcp, this.eventManager);
      await this.mcpBridge.connect();

      // Initialize WebSocket Server
      this.wsServer = new WebSocketServer(this.server, this.config.websocket, this.eventManager);
      this.wsServer.start();

      console.log('All services initialized successfully');
    } catch (error) {
      console.error('Failed to initialize services:', error);
      process.exit(1);
    }
  }

  async start() {
    const port = process.env.PORT || this.config.server.port;
    const host = process.env.HOST || this.config.server.host;

    this.server.listen(port, host, () => {
      console.log(`MCP GDB Node.js Bridge running on http://${host}:${port}`);
      console.log(`WebSocket server running on ws://${host}:${this.config.websocket.port}`);
      console.log('Dashboard available at http://' + host + ':' + port);
    });

    // Graceful shutdown
    process.on('SIGTERM', () => this.shutdown());
    process.on('SIGINT', () => this.shutdown());
  }

  async shutdown() {
    console.log('Shutting down server...');
    
    if (this.wsServer) {
      await this.wsServer.stop();
    }
    
    if (this.mcpBridge) {
      await this.mcpBridge.disconnect();
    }
    
    this.server.close(() => {
      console.log('Server shut down gracefully');
      process.exit(0);
    });
  }
}

// Start server if this file is run directly
if (require.main === module) {
  const server = new MCPGDBNodeServer();
  server.start().catch(console.error);
}

module.exports = MCPGDBNodeServer;
