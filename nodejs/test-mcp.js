const EventSource = require('eventsource');
const fetch = require('node-fetch');

async function testMCPConnection() {
  console.log('Testing MCP connection...');
  
  try {
    // Test SSE connection
    console.log('1. Testing SSE connection to http://127.0.0.1:8081/sse');
    
    const eventSource = new EventSource('http://127.0.0.1:8081/sse');
    
    eventSource.onopen = () => {
      console.log('✅ SSE connection established');
    };
    
    eventSource.onerror = (error) => {
      console.error('❌ SSE connection error:', error);
    };
    
    eventSource.addEventListener('endpoint', (event) => {
      console.log('✅ Received endpoint event:', event.data);
      const endpointPath = event.data.trim();
      const messageEndpoint = `http://127.0.0.1:8081${endpointPath}`;
      console.log('Message endpoint:', messageEndpoint);
      
      // Extract session ID
      const sessionMatch = endpointPath.match(/sessionId=([^&]+)/);
      if (sessionMatch) {
        const sessionId = sessionMatch[1];
        console.log('Session ID:', sessionId);
        
        // Test initialize request
        testInitialize(messageEndpoint, sessionId);
      }
    });
    
    // Keep the connection alive for a bit
    setTimeout(() => {
      eventSource.close();
      console.log('Test completed');
    }, 10000);
    
  } catch (error) {
    console.error('❌ Test failed:', error);
  }
}

async function testInitialize(messageEndpoint, sessionId) {
  try {
    console.log('2. Testing initialize request...');
    
    const request = {
      jsonrpc: '2.0',
      id: 1,
      method: 'initialize',
      params: {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: {
          name: 'test-client',
          version: '1.0.0'
        }
      }
    };
    
    const response = await fetch(messageEndpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Session-Id': sessionId
      },
      body: JSON.stringify(request)
    });
    
    console.log('Initialize response status:', response.status);
    
    if (response.ok) {
      console.log('✅ Initialize request sent successfully');
    } else {
      console.error('❌ Initialize request failed:', response.statusText);
    }
    
  } catch (error) {
    console.error('❌ Initialize test failed:', error);
  }
}

testMCPConnection();
