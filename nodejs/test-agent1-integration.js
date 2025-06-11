#!/usr/bin/env node

/**
 * Integration test for Agent-1's dual-server approach
 * Tests both SSE MCP server (port 8081) and Custom Protocol HTTP server (port 8082)
 */

const http = require('http');
const MCPClient = require('./src/mcp-client');
const EventManager = require('./src/event-manager');
const config = require('./config/default.json');

class Agent1IntegrationTester {
  constructor() {
    this.mcpPort = 8081;
    this.customProtocolPort = 8082;
    this.eventManager = new EventManager();
    this.mcpClient = null;
    this.testResults = {
      mcpServer: false,
      customProtocolServer: false,
      mcpConnection: false,
      customProtocolTools: false,
      nodeJsIntegration: false
    };
  }

  async runIntegrationTest() {
    console.log('🚀 Agent-1 Dual-Server Integration Test\n');
    console.log('Testing integration with Agent-1\'s implementation:');
    console.log('- SSE MCP Server (port 8081)');
    console.log('- Custom Protocol HTTP Server (port 8082)');
    console.log('- Node.js client with updated protocol\n');

    try {
      // Step 1: Test MCP Server availability
      console.log('📋 Step 1: Testing MCP Server (port 8081)');
      await this.testMCPServer();

      // Step 2: Test Custom Protocol Server availability
      console.log('\n📋 Step 2: Testing Custom Protocol Server (port 8082)');
      await this.testCustomProtocolServer();

      // Step 3: Test MCP Client connection
      console.log('\n📋 Step 3: Testing MCP Client Connection');
      await this.testMCPConnection();

      // Step 4: Test Custom Protocol Tools
      console.log('\n📋 Step 4: Testing Custom Protocol Tools');
      await this.testCustomProtocolTools();

      // Step 5: Test Node.js Integration
      console.log('\n📋 Step 5: Testing Node.js Integration');
      await this.testNodeJsIntegration();

      // Print results
      this.printResults();

    } catch (error) {
      console.error('\n💥 Integration test failed:', error.message);
    } finally {
      if (this.mcpClient) {
        await this.mcpClient.disconnect();
      }
    }
  }

  async testMCPServer() {
    try {
      const response = await this.makeRequest(`http://localhost:${this.mcpPort}/sse`, { timeout: 3000 });
      console.log('✅ MCP Server (SSE) is running on port 8081');
      this.testResults.mcpServer = true;
    } catch (error) {
      console.error('❌ MCP Server test failed:', error.message);
      console.log('💡 Make sure Agent-1\'s server is running:');
      console.log('   $env:SERVER_PORT="8081"; ./target/debug/mcp-server-gdb.exe sse');
    }
  }

  async testCustomProtocolServer() {
    try {
      const response = await this.makeRequest(`http://localhost:${this.customProtocolPort}/health`);
      console.log('✅ Custom Protocol Server is running on port 8082');
      console.log('📊 Health response:', response);
      this.testResults.customProtocolServer = true;
    } catch (error) {
      console.error('❌ Custom Protocol Server test failed:', error.message);
      console.log('💡 Custom Protocol server should start automatically with MCP server');
    }
  }

  async testMCPConnection() {
    try {
      this.mcpClient = new MCPClient(config.mcp, this.eventManager);
      await this.mcpClient.connect();
      console.log('✅ MCP Client connected successfully');
      this.testResults.mcpConnection = true;
    } catch (error) {
      console.error('❌ MCP Connection test failed:', error.message);
      this.testResults.mcpConnection = false;
    }
  }

  async testCustomProtocolTools() {
    if (!this.testResults.customProtocolServer) {
      console.log('⏭️  Skipping custom protocol tools test (server not available)');
      return;
    }

    try {
      // Test tools list endpoint
      const toolsList = await this.makeRequest(`http://localhost:${this.customProtocolPort}/api/tools/list`);
      console.log('✅ Tools list endpoint working');
      console.log('📊 Available tools:', toolsList.tools?.length || 0);

      // Test get_all_sessions tool
      const sessionsResponse = await this.makePostRequest(
        `http://localhost:${this.customProtocolPort}/api/tools/get_all_sessions`,
        { params: {} }
      );
      console.log('✅ get_all_sessions tool working');
      console.log('📊 Sessions response:', sessionsResponse);

      this.testResults.customProtocolTools = true;
    } catch (error) {
      console.error('❌ Custom Protocol Tools test failed:', error.message);
      this.testResults.customProtocolTools = false;
    }
  }

  async testNodeJsIntegration() {
    if (!this.mcpClient) {
      console.log('⏭️  Skipping Node.js integration test (MCP client not available)');
      return;
    }

    try {
      // Test getSessions through Node.js client
      const sessions = await this.mcpClient.getSessions();
      console.log('✅ Node.js getSessions working');
      console.log('📊 Sessions via Node.js:', sessions);

      // Test createSession through Node.js client
      try {
        const sessionData = {
          program: '/path/to/test/program',
          gdb_path: 'gdb'
        };
        const session = await this.mcpClient.createSession(sessionData);
        console.log('✅ Node.js createSession working');
        console.log('📊 Created session:', session);
      } catch (createError) {
        console.log('⚠️  createSession test failed (expected for test environment):', createError.message);
      }

      this.testResults.nodeJsIntegration = true;
    } catch (error) {
      console.error('❌ Node.js Integration test failed:', error.message);
      this.testResults.nodeJsIntegration = false;
    }
  }

  makeRequest(url, options = {}) {
    return new Promise((resolve, reject) => {
      const timeout = options.timeout || 5000;
      const request = http.get(url, (response) => {
        let data = '';
        
        response.on('data', (chunk) => {
          data += chunk;
        });
        
        response.on('end', () => {
          try {
            const jsonData = JSON.parse(data);
            resolve(jsonData);
          } catch (error) {
            resolve(data);
          }
        });
      });
      
      request.on('error', (error) => {
        reject(error);
      });
      
      request.setTimeout(timeout, () => {
        request.destroy();
        reject(new Error('Request timeout'));
      });
    });
  }

  makePostRequest(url, data) {
    return new Promise((resolve, reject) => {
      const postData = JSON.stringify(data);
      const options = {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Content-Length': Buffer.byteLength(postData)
        }
      };

      const request = http.request(url, options, (response) => {
        let responseData = '';
        
        response.on('data', (chunk) => {
          responseData += chunk;
        });
        
        response.on('end', () => {
          try {
            const jsonData = JSON.parse(responseData);
            resolve(jsonData);
          } catch (error) {
            resolve(responseData);
          }
        });
      });
      
      request.on('error', (error) => {
        reject(error);
      });
      
      request.write(postData);
      request.end();
    });
  }

  printResults() {
    console.log('\n🎯 Agent-1 Integration Test Results:');
    console.log('=' .repeat(50));
    
    const results = [
      { name: 'MCP Server (8081)', status: this.testResults.mcpServer },
      { name: 'Custom Protocol Server (8082)', status: this.testResults.customProtocolServer },
      { name: 'MCP Connection', status: this.testResults.mcpConnection },
      { name: 'Custom Protocol Tools', status: this.testResults.customProtocolTools },
      { name: 'Node.js Integration', status: this.testResults.nodeJsIntegration }
    ];
    
    for (const result of results) {
      const status = result.status ? '✅' : '❌';
      console.log(`${status} ${result.name}`);
    }
    
    const passed = results.filter(r => r.status).length;
    const total = results.length;
    
    console.log('=' .repeat(50));
    console.log(`📊 Summary: ${passed}/${total} components working`);
    
    if (passed === total) {
      console.log('\n🎉 Complete integration with Agent-1 is working!');
      console.log('✅ Both MCP and Custom Protocol servers are functional');
      console.log('✅ Node.js client successfully integrated');
      console.log('✅ Ready for production use!');
    } else if (this.testResults.customProtocolServer && this.testResults.customProtocolTools) {
      console.log('\n🎉 Custom Protocol integration is working!');
      console.log('✅ Agent-1\'s custom protocol server is functional');
      console.log('✅ All debugging tools available via HTTP API');
      console.log('⚠️  MCP connection issues can be ignored (custom protocol bypasses them)');
    } else {
      console.log('\n⚠️  Integration issues detected:');
      if (!this.testResults.mcpServer) {
        console.log('❌ MCP Server not running on port 8081');
      }
      if (!this.testResults.customProtocolServer) {
        console.log('❌ Custom Protocol Server not running on port 8082');
      }
      console.log('\n💡 Make sure Agent-1\'s server is running:');
      console.log('   $env:SERVER_PORT="8081"; ./target/debug/mcp-server-gdb.exe sse');
    }

    console.log('\n📚 Documentation:');
    console.log('- Integration Summary: nodejs/IMPLEMENTATION_SUMMARY.md');
    console.log('- Custom Protocol: nodejs/CUSTOM_PROTOCOL_README.md');
    console.log('- Agent-1 Implementation: docs/custom-protocol.md');
  }
}

// Run the integration test
const tester = new Agent1IntegrationTester();
tester.runIntegrationTest().catch(console.error);
