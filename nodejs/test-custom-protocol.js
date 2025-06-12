#!/usr/bin/env node

/**
 * Integration test script for the custom protocol workaround
 * Tests the complete Node.js client with custom protocol bypassing tools/call
 */

const MCPClient = require('./src/mcp-client');
const EventManager = require('./src/event-manager');
const config = require('./config/default.json');

class CustomProtocolTester {
  constructor() {
    this.eventManager = new EventManager();
    this.mcpClient = null;
    this.testResults = {
      connection: false,
      getSessions: false,
      createSession: false,
      getSession: false,
      getVariables: false,
      getRegisters: false,
      setBreakpoint: false,
      continueExecution: false,
      stepExecution: false,
      nextExecution: false,
      stopExecution: false,
      getBreakpoints: false,
      getStackFrames: false,
      getRegisterNames: false,
      readMemory: false,
      closeSession: false
    };
  }

  async runTests() {
    console.log('🚀 Starting Custom Protocol Integration Tests...\n');

    try {
      // Step 1: Initialize MCP Client
      console.log('📋 Step 1: Initialize MCP Client');
      this.mcpClient = new MCPClient(config.mcp, this.eventManager);
      
      // Step 2: Connect to MCP server
      console.log('\n📋 Step 2: Connect to MCP server');
      await this.mcpClient.connect();
      this.testResults.connection = true;
      console.log('✅ Connection successful');

      // Step 3: Test getSessions
      console.log('\n📋 Step 3: Test getSessions');
      try {
        const sessions = await this.mcpClient.getSessions();
        console.log('✅ getSessions successful:', sessions);
        this.testResults.getSessions = true;
      } catch (error) {
        console.error('❌ getSessions failed:', error.message);
      }

      // Step 4: Test createSession
      console.log('\n📋 Step 4: Test createSession');
      let sessionId = null;
      try {
        const sessionData = {
          program: '/path/to/test/program',
          gdb_path: 'gdb'
        };
        const session = await this.mcpClient.createSession(sessionData);
        console.log('✅ createSession successful:', session);
        sessionId = session.id;
        this.testResults.createSession = true;
      } catch (error) {
        console.error('❌ createSession failed:', error.message);
      }

      // Only continue with session-specific tests if we have a session
      if (sessionId) {
        // Step 5: Test getSession
        console.log('\n📋 Step 5: Test getSession');
        try {
          const session = await this.mcpClient.getSession(sessionId);
          console.log('✅ getSession successful:', session);
          this.testResults.getSession = true;
        } catch (error) {
          console.error('❌ getSession failed:', error.message);
        }

        // Step 6: Test startDebugging
        console.log('\n📋 Step 6: Test startDebugging');
        try {
          const result = await this.mcpClient.startDebugging(sessionId);
          console.log('✅ startDebugging successful:', result);
        } catch (error) {
          console.error('❌ startDebugging failed:', error.message);
        }

        // Step 7: Test getVariables
        console.log('\n📋 Step 7: Test getVariables');
        try {
          const variables = await this.mcpClient.getVariables(sessionId);
          console.log('✅ getVariables successful:', variables);
          this.testResults.getVariables = true;
        } catch (error) {
          console.error('❌ getVariables failed:', error.message);
        }

        // Step 8: Test getRegisters
        console.log('\n📋 Step 8: Test getRegisters');
        try {
          const registers = await this.mcpClient.getRegisters(sessionId);
          console.log('✅ getRegisters successful:', registers);
          this.testResults.getRegisters = true;
        } catch (error) {
          console.error('❌ getRegisters failed:', error.message);
        }

        // Step 9: Test getRegisterNames
        console.log('\n📋 Step 9: Test getRegisterNames');
        try {
          const registerNames = await this.mcpClient.getRegisterNames(sessionId);
          console.log('✅ getRegisterNames successful:', registerNames);
          this.testResults.getRegisterNames = true;
        } catch (error) {
          console.error('❌ getRegisterNames failed:', error.message);
        }

        // Step 10: Test setBreakpoint
        console.log('\n📋 Step 10: Test setBreakpoint');
        try {
          const breakpointData = {
            file: 'main.c',
            line: 10
          };
          const breakpoint = await this.mcpClient.setBreakpoint(sessionId, breakpointData);
          console.log('✅ setBreakpoint successful:', breakpoint);
          this.testResults.setBreakpoint = true;
        } catch (error) {
          console.error('❌ setBreakpoint failed:', error.message);
        }

        // Step 11: Test getBreakpoints
        console.log('\n📋 Step 11: Test getBreakpoints');
        try {
          const breakpoints = await this.mcpClient.getBreakpoints(sessionId);
          console.log('✅ getBreakpoints successful:', breakpoints);
          this.testResults.getBreakpoints = true;
        } catch (error) {
          console.error('❌ getBreakpoints failed:', error.message);
        }

        // Step 12: Test getStackFrames
        console.log('\n📋 Step 12: Test getStackFrames');
        try {
          const stackFrames = await this.mcpClient.getStackFrames(sessionId);
          console.log('✅ getStackFrames successful:', stackFrames);
          this.testResults.getStackFrames = true;
        } catch (error) {
          console.error('❌ getStackFrames failed:', error.message);
        }

        // Step 13: Test readMemory
        console.log('\n📋 Step 13: Test readMemory');
        try {
          const memoryData = await this.mcpClient.readMemory(sessionId, '0x08000000', 64);
          console.log('✅ readMemory successful:', memoryData);
          this.testResults.readMemory = true;
        } catch (error) {
          console.error('❌ readMemory failed:', error.message);
        }

        // Step 14: Test execution control
        console.log('\n📋 Step 14: Test execution control');
        try {
          const continueResult = await this.mcpClient.continueExecution(sessionId);
          console.log('✅ continueExecution successful:', continueResult);
          this.testResults.continueExecution = true;
        } catch (error) {
          console.error('❌ continueExecution failed:', error.message);
        }

        try {
          const stepResult = await this.mcpClient.stepExecution(sessionId);
          console.log('✅ stepExecution successful:', stepResult);
          this.testResults.stepExecution = true;
        } catch (error) {
          console.error('❌ stepExecution failed:', error.message);
        }

        try {
          const nextResult = await this.mcpClient.nextExecution(sessionId);
          console.log('✅ nextExecution successful:', nextResult);
          this.testResults.nextExecution = true;
        } catch (error) {
          console.error('❌ nextExecution failed:', error.message);
        }

        try {
          const stopResult = await this.mcpClient.stopExecution(sessionId);
          console.log('✅ stopExecution successful:', stopResult);
          this.testResults.stopExecution = true;
        } catch (error) {
          console.error('❌ stopExecution failed:', error.message);
        }

        // Step 15: Test closeSession
        console.log('\n📋 Step 15: Test closeSession');
        try {
          const result = await this.mcpClient.closeSession(sessionId);
          console.log('✅ closeSession successful:', result);
          this.testResults.closeSession = true;
        } catch (error) {
          console.error('❌ closeSession failed:', error.message);
        }
      }

      // Print test summary
      this.printTestSummary();

    } catch (error) {
      console.error('\n💥 Test suite failed:', error.message);
    } finally {
      if (this.mcpClient) {
        await this.mcpClient.disconnect();
      }
      process.exit(0);
    }
  }

  printTestSummary() {
    console.log('\n🎯 Custom Protocol Integration Test Results:');
    console.log('=' .repeat(50));
    
    const passed = Object.values(this.testResults).filter(result => result).length;
    const total = Object.keys(this.testResults).length;
    
    for (const [test, result] of Object.entries(this.testResults)) {
      const status = result ? '✅' : '❌';
      console.log(`${status} ${test}`);
    }
    
    console.log('=' .repeat(50));
    console.log(`📊 Summary: ${passed}/${total} tests passed`);
    
    if (passed === total) {
      console.log('🎉 All tests passed! Custom protocol workaround is working!');
    } else {
      console.log('⚠️  Some tests failed. Check the logs above for details.');
    }
  }
}

// Run the tests
const tester = new CustomProtocolTester();
tester.runTests().catch(console.error);
