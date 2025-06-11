# Changelog

All notable changes to the MCP Server GDB for STM32 project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2025-01-27

> **🔧 CRITICAL FIX**: Custom Protocol Workaround for mcp-core v0.1 Bug! Complete Node.js client rewrite with custom protocol.

### Fixed
- **MCP Core Library Bug Workaround**:
  - **Root Cause**: `mcp-core` v0.1 has a critical bug where `tools/list` and `tools/call` fail with "Client must be initialized" even after successful MCP initialization
  - **Solution**: Implemented custom protocol that bypasses broken `tools/call` mechanism
  - **Impact**: All debugging tools now work despite upstream library bug
- **Custom Protocol Implementation**:
  - Replaced `tools/call` with direct `custom/{tool_name}` method calls
  - Maintains SSE transport and MCP initialization (working components)
  - Handles multiple response formats (string, JSON, MCP content format)
  - Provides complete feature parity with standard MCP protocol

### Added
- **Custom Tool Request Method**: `sendCustomToolRequest(toolName, params)`
- **Complete Tool Coverage**: All 16 GDB tools implemented with custom protocol
  - Session management: `custom/get_all_sessions`, `custom/create_session`, `custom/get_session`, `custom/close_session`
  - Debugging control: `custom/start_debugging`, `custom/stop_debugging`, `custom/continue_execution`, `custom/step_execution`, `custom/next_execution`
  - Breakpoint management: `custom/get_breakpoints`, `custom/set_breakpoint`, `custom/delete_breakpoint`
  - Data inspection: `custom/get_local_variables`, `custom/get_registers`, `custom/get_register_names`, `custom/get_stack_frames`, `custom/read_memory`
- **Enhanced API Endpoints**: Complete REST API coverage for all debugging operations
  - `DELETE /api/sessions/:id` - Close session
  - `POST /api/sessions/:id/start` - Start debugging
  - `POST /api/sessions/:id/stop` - Stop debugging
  - `POST /api/sessions/:id/continue` - Continue execution
  - `POST /api/sessions/:id/step` - Step into
  - `POST /api/sessions/:id/next` - Step over
  - `GET /api/sessions/:id/breakpoints` - Get breakpoints
  - `POST /api/sessions/:id/breakpoints` - Set breakpoint
  - `DELETE /api/sessions/:id/breakpoints/:breakpointId` - Delete breakpoint
  - `GET /api/sessions/:id/stack` - Get stack frames
  - `GET /api/sessions/:id/register-names` - Get register names
  - `GET /api/sessions/:id/memory?address=&size=` - Read memory
- **Integration Testing**: `test-custom-protocol.js` - Comprehensive test suite for custom protocol
- **Documentation**: `CUSTOM_PROTOCOL_README.md` - Complete custom protocol documentation

### Enhanced
- **Response Format Handling**: Robust handling of different response formats from Rust server
- **Error Handling**: Improved error messages and debugging information
- **Event Management**: Enhanced WebSocket event emission for real-time dashboard updates
- **Health Checks**: Updated health check to use custom protocol instead of broken `tools/list`

### Technical Implementation
- **Protocol Bypass**: Uses working SSE transport while bypassing broken `tools/call`
- **Method Mapping**: Direct mapping from Node.js methods to custom Rust server endpoints
- **Format Flexibility**: Handles string, JSON, and MCP content response formats
- **Event Integration**: Maintains WebSocket event emission for dashboard real-time updates
- **Future Migration**: Easy migration path back to standard MCP when library is fixed

### Files Modified
- `nodejs/src/mcp-client.js` - **MAJOR REWRITE**: Custom protocol implementation
- `nodejs/src/server.js` - **ENHANCED**: Complete API endpoint coverage
- `nodejs/test-custom-protocol.js` - **NEW**: Integration test suite
- `nodejs/test-server-startup.js` - **NEW**: Server startup verification
- `nodejs/CUSTOM_PROTOCOL_README.md` - **NEW**: Custom protocol documentation

### Benefits
- **Full Functionality**: All debugging tools work despite mcp-core bug
- **Better Performance**: Direct method calls without tools/call overhead
- **Enhanced Reliability**: Bypasses unstable library components
- **Complete Coverage**: All 16 GDB tools available via REST API
- **Real-time Updates**: WebSocket integration maintained
- **Future-Proof**: Easy migration when upstream bug is fixed

### Breaking Changes
- Internal protocol changed from `tools/call` to `custom/{tool_name}`
- Response format handling updated for multiple formats
- Health check method changed from `tools/list` to `custom/get_all_sessions`

### Migration Notes
- **For Users**: No changes required - all API endpoints remain the same
- **For Developers**: Custom protocol is transparent to external consumers
- **Future Migration**: When mcp-core is fixed, can easily revert to standard protocol

---

## [0.4.0] - 2025-06-09

> **🚀 NEW FEATURE**: Node.js Real-Time Debugging Integration! Web-based dashboard with live monitoring.

### Added
- **Node.js Integration**: Complete Node.js bridge for real-time debugging capabilities
- **Real-Time Web Dashboard**: Interactive web interface for debugging sessions
  - Live variable monitoring with automatic updates
  - Real-time ARM Cortex-M register visualization
  - Interactive breakpoint management
  - Debug execution controls (continue, step, stop)
  - Live log streaming with filtering
  - Session management interface
- **WebSocket Server**: Real-time communication between web dashboard and debugging sessions
- **MCP Bridge**: Node.js bridge to interface with Rust MCP server
- **Event Management System**: Comprehensive event handling for debugging state changes
- **Startup Script**: `scripts/start-with-nodejs.sh` for easy deployment
- **Web Technologies Stack**:
  - Express.js HTTP server
  - Socket.IO WebSocket implementation
  - Modern responsive web interface
  - Real-time data streaming
- **Configuration System**: Flexible configuration for Node.js components
- **Auto-refresh Capabilities**: Configurable real-time data updates
- **Keyboard Shortcuts**: Hotkeys for common debugging operations

### Enhanced
- **README.md**: Added comprehensive Node.js integration documentation
- **Project Structure**: Added `nodejs/` directory with complete Node.js application
- **Dependencies**: Added Node.js package management with npm
- **Documentation**: Updated setup and usage instructions for dual-server architecture

### Technical Implementation
- **Architecture**: Hybrid Rust + Node.js architecture
  - Rust MCP server: Core GDB/MI protocol handling
  - Node.js bridge: Real-time web interface and WebSocket communication
- **Communication**: HTTP API and WebSocket protocols for real-time updates
- **Frontend**: Modern HTML5/CSS3/JavaScript dashboard
- **Backend**: Express.js server with Socket.IO integration
- **Data Flow**: Rust ↔ Node.js ↔ WebSocket ↔ Web Dashboard

### Files Added
- `nodejs/package.json` - Node.js project configuration
- `nodejs/src/server.js` - Main Node.js server
- `nodejs/src/websocket-server.js` - WebSocket handling
- `nodejs/src/mcp-bridge.js` - Rust MCP server interface
- `nodejs/src/event-manager.js` - Event management system
- `nodejs/public/index.html` - Web dashboard interface
- `nodejs/public/css/dashboard.css` - Dashboard styling
- `nodejs/public/js/dashboard.js` - Dashboard JavaScript
- `nodejs/config/default.json` - Node.js configuration
- `scripts/start-with-nodejs.sh` - Integrated startup script

### Benefits
- **Enhanced User Experience**: Visual debugging interface with real-time updates
- **Improved Productivity**: Faster debugging cycles with live monitoring
- **Better Visualization**: Graphical representation of debugging data
- **Modern Interface**: Web-based dashboard accessible from any browser
- **Real-Time Feedback**: Instant updates without manual refresh
- **Multi-Session Support**: Manage multiple debugging sessions simultaneously

---

## [0.3.0] - 2025-06-08

> **🎯 STATUS**: Ready for Augment AI import! All JSON Schema validation issues resolved.

### Added
- **STM32 Optimization**: Complete optimization for STM32 microcontroller debugging
- **Augment AI Compatibility**: Full compatibility with Augment AI MCP import system
- **Multiple Configuration Files**: 
  - `mcp-stm32-recommended.json` - Recommended configuration for Augment AI
  - `mcp-server-gdb.json` - Comprehensive configuration with full STM32 metadata
  - `mcp-stm32-config.json` - Simple configuration with relative paths
  - `mcp-stm32-absolute.json` - Simple configuration with absolute paths
  - `mcp-stm32-working-dir.json` - Configuration with working directory specified
- **STM32-Specific Documentation**: 
  - ARM Cortex-M register descriptions
  - STM32 memory region mappings
  - Peripheral register access examples
  - Debug probe compatibility information
- **Setup Guide**: `AUGMENT_AI_SETUP.md` with comprehensive integration instructions

### Changed
- **Server Description**: Updated from generic GDB server to STM32-focused debugging server
- **Tool Descriptions**: Enhanced all tool descriptions with STM32-specific context
  - `get_registers`: Now describes ARM Cortex-M registers (R0-R15, PSR, MSP, PSP, etc.)
  - `read_memory`: Updated for STM32 memory regions (Flash, SRAM, peripherals)
  - `disassemble`: Now mentions ARM Thumb/Thumb-2 instruction sets
- **Timeout Settings**: Increased default GDB command timeout from 10 to 30 seconds for embedded debugging
- **Keywords**: Added STM32, embedded, ARM, Cortex-M to project keywords
- **Examples**: Updated all usage examples to focus on STM32 firmware debugging workflows

### Fixed
- **Read-Only Filesystem Issue**:
  - Modified `src/main.rs` logging initialization to gracefully handle read-only filesystems
  - Added panic handling around `RollingFileAppender::new()` with fallback to stderr logging
  - Server now works in containerized environments like Augment AI
- **JSON Schema Validation Errors** (CRITICAL FIX):
  - **Root Cause**: `schemars` library generating invalid format specifiers for `usize`/`isize` types
  - **Solution**: Replaced problematic integer types with `u64`/`i64` for valid JSON Schema generation
  - Fixed `create_session` tool: `bps: Option<usize>` → `bps: Option<u64>` (with `u32` conversion)
  - Fixed `create_session` tool: `proc_id: Option<usize>` → `proc_id: Option<u64>` (with `u32` conversion)
  - Fixed `set_breakpoint` tool: `line: usize` → `line: u64` (with `usize` conversion)
  - Fixed `get_local_variables` tool: `frame_id: Option<usize>` → `frame_id: Option<u64>` (with `usize` conversion)
  - Fixed `read_memory` tool: `count: usize` → `count: u64` (with `usize` conversion)
  - Fixed `read_memory` tool: `offset: Option<isize>` → `offset: Option<i64>` (with `isize` conversion)
  - **Result**: All tools now generate valid JSON Schema with `"format": "uint64"` and `"format": "int64"`
  - Eliminated "unknown format uint32/int32" errors that prevented MCP client integration
- **Type Compatibility**: Added proper type conversions to maintain compatibility with existing GDB manager interface
- **Code Cleanup**: Removed unused imports and resolved compiler warnings
- **MCP Protocol Compliance**: Server now fully validates against MCP JSON Schema requirements

### Technical Details
- **Logging System**: Implemented graceful degradation from file logging to stderr when filesystem is read-only
- **Schema Generation**: Fixed `schemars` compatibility issues with custom type handling
- **Error Handling**: Enhanced error messages and fallback mechanisms
- **Type Safety**: Maintained internal type safety while ensuring MCP compatibility

### Supported STM32 Families
- STM32F0, F1, F2, F3, F4, F7 series
- STM32L0, L1, L4, L5 series (low power)
- STM32G0, G4 series
- STM32H7 series (high performance)
- STM32WB, WL series (wireless)
- STM32U5 series (ultra-low power)

### Supported Debug Probes
- ST-Link V2/V3
- J-Link (Segger)
- OpenOCD compatible probes
- Black Magic Probe

### Memory Regions
- **Flash Memory**: 0x08000000 (varies by device)
- **SRAM**: 0x20000000 (varies by device)  
- **System Memory**: 0x1FFF0000 (bootloader)
- **Option Bytes**: 0x1FFF7800 (configuration)
- **Peripherals**: 0x40000000 - 0x60000000

### Breaking Changes
- Configuration file format updated for STM32 focus
- Tool parameter types changed for JSON Schema compliance
- Server name changed from "MCP Server GDB" to "MCP Server GDB for STM32"

### Migration Guide
For users upgrading from previous versions:

1. **Update Configuration**: Use new STM32-specific configuration files
2. **Update Paths**: Ensure binary path points to `target/release/mcp-server-gdb`
3. **Update Environment**: Set `GDB_COMMAND_TIMEOUT=30` for embedded debugging
4. **Update GDB Path**: Use `arm-none-eabi-gdb` for STM32 debugging

### Dependencies
- Rust 1.87.0+
- arm-none-eabi-gdb (GNU Arm Embedded Toolchain)
- OpenOCD or ST-Link GDB server
- STM32 debug probe hardware

### Testing
- ✅ MCP protocol compliance verified (MCP 2024-11-05)
- ✅ JSON Schema validation passed (all 16 tools generate valid schema)
- ✅ Read-only filesystem compatibility confirmed
- ✅ All 16 debugging tools functional and tested
- ✅ STM32-specific workflows tested
- ✅ Type conversion compatibility verified
- ✅ Build system tested (Rust 1.87.0, cargo build --release)
- ✅ Schema format specifiers validated ("uint64"/"int64" instead of invalid "uint32"/"int32")

### Known Issues
- Some MCP clients may still reject "uint64"/"int64" format specifiers (rare compatibility issue)
- Workaround available: Use `u32`/`i32` types if compatibility issues persist with specific clients

### Contributors
- Alan Hu (ahu@custompower.com) - STM32 optimization and Augment AI integration
- Lix Zhou (xeontz@gmail.com) - Original MCP Server GDB implementation

---

## [0.2.3] - Previous Release

### Features
- Basic GDB/MI protocol server
- MCP protocol implementation
- Generic debugging capabilities
- File logging system
- Basic tool set for debugging

### Issues
- Read-only filesystem compatibility problems
- JSON Schema validation errors
- Generic (non-STM32 specific) descriptions
- Timeout settings not optimized for embedded debugging

---

For more information about this release, see the [Augment AI Setup Guide](AUGMENT_AI_SETUP.md).
