#!/usr/bin/env node

/**
 * Test script to verify direct tool invocation bypassing tools/list
 * This tests if we can call tools directly even if tools/list fails
 */

const { EventSource } = require('eventsource');
const fetch = require('node-fetch');

class DirectToolTester {
  constructor() {
    this.baseUrl = 'http://127.0.0.1:8081';
    this.eventSource = null;
    this.messageEndpoint = null;
    this.sessionId = null;
    this.requestId = 1;
    this.pendingRequests = new Map();
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

      console.log(`\n🔄 MCP Request: ${method}`, JSON.stringify(params, null, 2));

      // Store the pending request
      this.pendingRequests.set(requestId, { resolve, reject, timestamp: Date.now() });

      if (!this.messageEndpoint) {
        reject(new Error('No message endpoint available'));
        return;
      }

      const headers = {
        'Content-Type': 'application/json',
      };

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
        // Response will come via SSE
      })
      .catch(error => {
        console.error(`❌ MCP Error: ${method}`, error.message);
        reject(error);
        this.pendingRequests.delete(requestId);
      });

      // Timeout handling
      setTimeout(() => {
        if (this.pendingRequests.has(requestId)) {
          this.pendingRequests.delete(requestId);
          reject(new Error('Request timeout'));
        }
      }, 10000);
    });
  }

  async sendMCPNotification(method, params = {}) {
    const notification = {
      jsonrpc: '2.0',
      method: method,
      params: params
    };

    console.log(`\n📢 MCP Notification: ${method}`, JSON.stringify(params, null, 2));

    const headers = {
      'Content-Type': 'application/json',
    };

    if (this.sessionId) {
      headers['X-Session-Id'] = this.sessionId;
    }

    const response = await fetch(this.messageEndpoint, {
      method: 'POST',
      headers: headers,
      body: JSON.stringify(notification)
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    console.log(`✅ MCP Notification sent: ${method}`);
  }

  async establishSSEConnection() {
    return new Promise((resolve, reject) => {
      console.log('🔗 Establishing SSE connection...');

      this.eventSource = new EventSource(`${this.baseUrl}/sse`);

      this.eventSource.onopen = () => {
        console.log('✅ SSE connection established');
      };

      this.eventSource.onerror = (error) => {
        console.error('❌ SSE connection error:', error);
        if (!this.messageEndpoint) {
          reject(new Error('Failed to establish SSE connection'));
        }
      };

      this.eventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.handleSSEMessage(data);
        } catch (error) {
          console.error('❌ Failed to parse SSE message:', error);
        }
      };

      this.eventSource.addEventListener('endpoint', (event) => {
        try {
          console.log('📍 Received endpoint event:', event.data);
          const endpointPath = event.data.trim();
          this.messageEndpoint = `${this.baseUrl}${endpointPath}`;
          console.log('✅ Message endpoint set to:', this.messageEndpoint);

          const sessionMatch = endpointPath.match(/sessionId=([^&]+)/);
          if (sessionMatch) {
            this.sessionId = sessionMatch[1];
            console.log('🆔 Session ID extracted:', this.sessionId);
          }

          resolve();
        } catch (error) {
          console.error('❌ Failed to parse endpoint event:', error);
          reject(error);
        }
      });

      setTimeout(() => {
        if (!this.messageEndpoint) {
          this.eventSource.close();
          reject(new Error('SSE connection timeout'));
        }
      }, 10000);
    });
  }

  handleSSEMessage(data) {
    console.log('📨 SSE Message received:', JSON.stringify(data, null, 2));

    if (data.id && this.pendingRequests.has(data.id)) {
      const { resolve, reject } = this.pendingRequests.get(data.id);
      this.pendingRequests.delete(data.id);

      if (data.error) {
        console.error('❌ MCP Response Error:', data.error);
        reject(new Error(data.error.message || 'MCP Error'));
      } else {
        console.log('✅ MCP Response Success:', JSON.stringify(data.result, null, 2));
        resolve(data.result);
      }
    }
  }

  async runTests() {
    try {
      console.log('🚀 Starting Direct Tool Tests...\n');

      // Step 1: Establish SSE connection
      await this.establishSSEConnection();

      // Step 2: Initialize MCP session
      console.log('\n📋 Step 2: Initialize MCP session');
      const initResult = await this.sendMCPRequest('initialize', {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: {
          name: 'direct-tool-tester',
          version: '1.0.0'
        }
      });
      console.log('✅ MCP initialization successful');

      // Step 3: Send initialized notification
      console.log('\n📋 Step 3: Send initialized notification');
      await this.sendMCPNotification('initialized', {});
      
      // Wait for server to process
      await new Promise(resolve => setTimeout(resolve, 500));

      // Step 4: Test tools/list (expected to fail)
      console.log('\n📋 Step 4: Test tools/list (expected to fail)');
      try {
        const tools = await this.sendMCPRequest('tools/list');
        console.log('🎉 Unexpected success! tools/list worked:', tools);
      } catch (error) {
        console.log('⚠️  Expected failure - tools/list error:', error.message);
      }

      // Step 5: Test direct tool calls
      console.log('\n📋 Step 5: Test direct tool calls');
      
      // Test get_all_sessions
      console.log('\n🔧 Testing get_all_sessions tool...');
      try {
        const sessionsResult = await this.sendMCPRequest('tools/call', {
          name: 'get_all_sessions',
          arguments: {}
        });
        console.log('✅ get_all_sessions SUCCESS!');
        console.log('📊 Sessions data:', sessionsResult);
      } catch (error) {
        console.error('❌ get_all_sessions FAILED:', error.message);
      }

      // Test create_session
      console.log('\n🔧 Testing create_session tool...');
      try {
        const createResult = await this.sendMCPRequest('tools/call', {
          name: 'create_session',
          arguments: {
            program: '/path/to/test/program',
            gdb_path: 'gdb'
          }
        });
        console.log('✅ create_session SUCCESS!');
        console.log('📊 Create result:', createResult);
      } catch (error) {
        console.error('❌ create_session FAILED:', error.message);
      }

      console.log('\n🎯 Direct Tool Tests Complete!');
      console.log('\n📊 Summary:');
      console.log('- SSE Connection: ✅ Working');
      console.log('- MCP Initialize: ✅ Working');
      console.log('- tools/list: ❌ Expected failure (known issue)');
      console.log('- Direct tool calls: Check results above');

    } catch (error) {
      console.error('\n💥 Test failed:', error.message);
    } finally {
      if (this.eventSource) {
        this.eventSource.close();
      }
      process.exit(0);
    }
  }
}

// Run the tests
const tester = new DirectToolTester();
tester.runTests().catch(console.error);
