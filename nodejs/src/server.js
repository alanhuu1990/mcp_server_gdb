#!/usr/bin/env node

const express = require('express');
const http = require('http');
const path = require('path');
const cors = require('cors');
const helmet = require('helmet');
const morgan = require('morgan');
require('dotenv').config();

const WebSocketServer = require('./websocket-server');
const MCPClient = require('./mcp-client');
const EventManager = require('./event-manager');
const config = require('../config/default.json');

class MCPGDBNodeServer {
  constructor() {
    this.app = express();
    this.server = http.createServer(this.app);
    this.config = config;
    this.mcpClient = null;
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
          mcp_client: this.mcpClient ? this.mcpClient.isConnected() : false,
          websocket: this.wsServer ? this.wsServer.isRunning() : false
        }
      });
    });

    // API routes
    this.app.get('/api/sessions', async (req, res) => {
      try {
        const sessions = await this.mcpClient.getSessions();
        res.json(sessions);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.post('/api/sessions', async (req, res) => {
      try {
        const session = await this.mcpClient.createSession(req.body);
        res.json(session);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.get('/api/sessions/:id/variables', async (req, res) => {
      try {
        const variables = await this.mcpClient.getVariables(req.params.id);
        res.json(variables);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.get('/api/sessions/:id/registers', async (req, res) => {
      try {
        const registers = await this.mcpClient.getRegisters(req.params.id);
        res.json(registers);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    // Additional API endpoints for complete GDB functionality
    this.app.delete('/api/sessions/:id', async (req, res) => {
      try {
        const result = await this.mcpClient.closeSession(req.params.id);
        res.json({ message: result });
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.post('/api/sessions/:id/start', async (req, res) => {
      try {
        const result = await this.mcpClient.startDebugging(req.params.id);
        res.json({ message: result });
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.post('/api/sessions/:id/stop', async (req, res) => {
      try {
        const result = await this.mcpClient.stopExecution(req.params.id);
        res.json({ message: result });
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.post('/api/sessions/:id/continue', async (req, res) => {
      try {
        const result = await this.mcpClient.continueExecution(req.params.id);
        res.json({ message: result });
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.post('/api/sessions/:id/step', async (req, res) => {
      try {
        const result = await this.mcpClient.stepExecution(req.params.id);
        res.json({ message: result });
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.post('/api/sessions/:id/next', async (req, res) => {
      try {
        const result = await this.mcpClient.nextExecution(req.params.id);
        res.json({ message: result });
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.get('/api/sessions/:id/breakpoints', async (req, res) => {
      try {
        const breakpoints = await this.mcpClient.getBreakpoints(req.params.id);
        res.json(breakpoints);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.post('/api/sessions/:id/breakpoints', async (req, res) => {
      try {
        const breakpoint = await this.mcpClient.setBreakpoint(req.params.id, req.body);
        res.json(breakpoint);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.delete('/api/sessions/:id/breakpoints/:breakpointId', async (req, res) => {
      try {
        const result = await this.mcpClient.deleteBreakpoint(req.params.id, req.params.breakpointId);
        res.json({ message: result });
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.get('/api/sessions/:id/stack', async (req, res) => {
      try {
        const stackFrames = await this.mcpClient.getStackFrames(req.params.id);
        res.json(stackFrames);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.get('/api/sessions/:id/register-names', async (req, res) => {
      try {
        const registerNames = await this.mcpClient.getRegisterNames(req.params.id);
        res.json(registerNames);
      } catch (error) {
        res.status(500).json({ error: error.message });
      }
    });

    this.app.get('/api/sessions/:id/memory', async (req, res) => {
      try {
        const { address, size } = req.query;
        if (!address || !size) {
          return res.status(400).json({ error: 'address and size parameters are required' });
        }
        const memoryData = await this.mcpClient.readMemory(req.params.id, address, parseInt(size, 10));
        res.json(memoryData);
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

      // Initialize MCP Client (but don't fail if connection fails)
      this.mcpClient = new MCPClient(this.config.mcp, this.eventManager);
      try {
        await this.mcpClient.connect();
        console.log('MCP Client connected successfully');
      } catch (error) {
        console.warn('MCP Client connection failed (will retry automatically):', error.message);
        // Don't exit - let the client handle reconnection
      }

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
    
    if (this.mcpClient) {
      await this.mcpClient.disconnect();
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
