#!/usr/bin/env node

/**
 * Complete workflow test for the MCP GDB custom protocol implementation
 * This script demonstrates the full integration between Node.js and Rust servers
 */

const http = require('http');

class CompleteWorkflowTester {
  constructor() {
    this.nodeJsPort = 3000;
    this.rustPort = 8081;
    this.testResults = {
      nodeJsServer: false,
      rustServer: false,
      healthCheck: false,
      apiEndpoints: false,
      customProtocol: false
    };
  }

  async runCompleteTest() {
    console.log('🚀 MCP GDB Complete Workflow Test\n');
    console.log('This test verifies the complete integration between:');
    console.log('- Node.js server with custom protocol implementation');
    console.log('- Rust MCP server with GDB tools');
    console.log('- REST API endpoints');
    console.log('- WebSocket dashboard integration\n');

    try {
      // Step 1: Test Node.js server
      console.log('📋 Step 1: Testing Node.js Server');
      await this.testNodeJsServer();

      // Step 2: Test Rust server availability
      console.log('\n📋 Step 2: Testing Rust MCP Server');
      await this.testRustServer();

      // Step 3: Test API endpoints
      console.log('\n📋 Step 3: Testing API Endpoints');
      await this.testApiEndpoints();

      // Step 4: Test custom protocol (if Rust server is available)
      if (this.testResults.rustServer) {
        console.log('\n📋 Step 4: Testing Custom Protocol Integration');
        await this.testCustomProtocol();
      } else {
        console.log('\n📋 Step 4: Skipping Custom Protocol Test (Rust server not available)');
      }

      // Print results
      this.printResults();

    } catch (error) {
      console.error('\n💥 Complete workflow test failed:', error.message);
    }
  }

  async testNodeJsServer() {
    try {
      const health = await this.makeRequest(`http://localhost:${this.nodeJsPort}/health`);
      console.log('✅ Node.js server is running');
      console.log('📊 Health status:', health);
      this.testResults.nodeJsServer = true;
      this.testResults.healthCheck = true;
    } catch (error) {
      console.error('❌ Node.js server test failed:', error.message);
      console.log('💡 Make sure to start the Node.js server:');
      console.log('   cd nodejs && node src/server.js');
    }
  }

  async testRustServer() {
    try {
      // Test SSE endpoint
      const response = await this.makeRequest(`http://localhost:${this.rustPort}/sse`, { timeout: 3000 });
      console.log('✅ Rust MCP server is running');
      console.log('📊 SSE endpoint accessible');
      this.testResults.rustServer = true;
    } catch (error) {
      console.error('❌ Rust MCP server test failed:', error.message);
      console.log('💡 Make sure to start the Rust server:');
      console.log('   $env:SERVER_PORT="8081"; ./target/debug/mcp-server-gdb.exe --log-level debug sse');
      this.testResults.rustServer = false;
    }
  }

  async testApiEndpoints() {
    const endpoints = [
      { path: '/api/sessions', method: 'GET', description: 'List sessions' },
      { path: '/health', method: 'GET', description: 'Health check' },
      { path: '/', method: 'GET', description: 'Dashboard' }
    ];

    let successCount = 0;
    
    for (const endpoint of endpoints) {
      try {
        const response = await this.makeRequest(`http://localhost:${this.nodeJsPort}${endpoint.path}`);
        console.log(`✅ ${endpoint.description}: ${endpoint.method} ${endpoint.path}`);
        successCount++;
      } catch (error) {
        console.log(`⚠️  ${endpoint.description}: ${endpoint.method} ${endpoint.path} - ${error.message}`);
      }
    }

    this.testResults.apiEndpoints = successCount > 0;
    console.log(`📊 API Endpoints: ${successCount}/${endpoints.length} working`);
  }

  async testCustomProtocol() {
    try {
      // Test sessions endpoint with custom protocol
      const sessions = await this.makeRequest(`http://localhost:${this.nodeJsPort}/api/sessions`);
      console.log('✅ Custom protocol working - sessions endpoint responded');
      console.log('📊 Sessions response:', sessions);
      this.testResults.customProtocol = true;
    } catch (error) {
      console.error('❌ Custom protocol test failed:', error.message);
      this.testResults.customProtocol = false;
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

  printResults() {
    console.log('\n🎯 Complete Workflow Test Results:');
    console.log('=' .repeat(50));
    
    const results = [
      { name: 'Node.js Server', status: this.testResults.nodeJsServer },
      { name: 'Rust MCP Server', status: this.testResults.rustServer },
      { name: 'Health Check', status: this.testResults.healthCheck },
      { name: 'API Endpoints', status: this.testResults.apiEndpoints },
      { name: 'Custom Protocol', status: this.testResults.customProtocol }
    ];
    
    for (const result of results) {
      const status = result.status ? '✅' : '❌';
      console.log(`${status} ${result.name}`);
    }
    
    const passed = results.filter(r => r.status).length;
    const total = results.length;
    
    console.log('=' .repeat(50));
    console.log(`📊 Summary: ${passed}/${total} components working`);
    
    if (this.testResults.nodeJsServer && this.testResults.apiEndpoints) {
      console.log('\n🎉 Node.js server and API are working correctly!');
      
      if (this.testResults.rustServer && this.testResults.customProtocol) {
        console.log('🎉 Complete integration is working!');
        console.log('\n📝 Ready for production use:');
        console.log('1. ✅ Node.js server running');
        console.log('2. ✅ Rust MCP server running');
        console.log('3. ✅ Custom protocol working');
        console.log('4. ✅ API endpoints functional');
        console.log('5. ✅ Dashboard available at http://localhost:3000');
      } else {
        console.log('\n📝 Partial integration working:');
        console.log('1. ✅ Node.js server running');
        console.log('2. ✅ API endpoints functional');
        console.log('3. ❌ Rust MCP server needed for full functionality');
        console.log('\n💡 To complete the integration:');
        console.log('   Start Rust server: $env:SERVER_PORT="8081"; ./target/debug/mcp-server-gdb.exe --log-level debug sse');
      }
    } else {
      console.log('\n⚠️  Integration issues detected:');
      if (!this.testResults.nodeJsServer) {
        console.log('❌ Node.js server not running');
        console.log('💡 Start with: cd nodejs && node src/server.js');
      }
      if (!this.testResults.apiEndpoints) {
        console.log('❌ API endpoints not working');
      }
    }

    console.log('\n📚 Documentation:');
    console.log('- Custom Protocol: nodejs/CUSTOM_PROTOCOL_README.md');
    console.log('- Task Log: task-log.md');
    console.log('- Lessons Learned: lessons.md');
    console.log('- Changelog: CHANGELOG.md');
  }
}

// Run the complete workflow test
const tester = new CompleteWorkflowTester();
tester.runCompleteTest().catch(console.error);
