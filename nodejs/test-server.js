console.log('Starting test server...');

try {
  const MCPGDBNodeServer = require('./src/server.js');
  console.log('Server class loaded successfully');
  
  const server = new MCPGDBNodeServer();
  console.log('Server instance created');
  
  server.start().then(() => {
    console.log('Server started successfully');
    process.exit(0);
  }).catch(error => {
    console.error('Server start failed:', error);
    process.exit(1);
  });
  
} catch (error) {
  console.error('Failed to load server:', error);
}
