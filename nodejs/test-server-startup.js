#!/usr/bin/env node

/**
 * Simple test to verify Node.js server starts correctly with custom protocol
 */

const http = require('http');

async function testServerStartup() {
  console.log('🚀 Testing Node.js Server Startup...\n');

  try {
    // Test health endpoint
    console.log('📋 Testing health endpoint...');
    const healthResponse = await makeRequest('http://localhost:3000/health');
    console.log('✅ Health check successful:', healthResponse);

    // Test sessions endpoint
    console.log('\n📋 Testing sessions endpoint...');
    try {
      const sessionsResponse = await makeRequest('http://localhost:3000/api/sessions');
      console.log('✅ Sessions endpoint successful:', sessionsResponse);
    } catch (error) {
      console.log('⚠️  Sessions endpoint failed (expected if Rust server not running):', error.message);
    }

    console.log('\n🎉 Node.js server is running correctly!');
    console.log('\n📝 Next steps:');
    console.log('1. Start Rust server: $env:SERVER_PORT="8081"; ./target/debug/mcp-server-gdb.exe --log-level debug sse');
    console.log('2. Run integration test: node test-custom-protocol.js');
    console.log('3. Open dashboard: http://localhost:3000');

  } catch (error) {
    console.error('❌ Server startup test failed:', error.message);
    console.log('\n📝 Make sure to start the Node.js server first:');
    console.log('cd nodejs && node src/server.js');
  }
}

function makeRequest(url) {
  return new Promise((resolve, reject) => {
    const request = http.get(url, (response) => {
      let data = '';

      // Check HTTP status code
      if (response.statusCode < 200 || response.statusCode >= 300) {
        reject(new Error(`HTTP ${response.statusCode}: ${response.statusMessage}`));
        return;
      }

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
    
    request.setTimeout(5000, () => {
      request.destroy();
      reject(new Error('Request timeout'));
    });
  });
}

testServerStartup().catch(console.error);
