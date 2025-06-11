# MCP Server GDB for STM32

A GDB/MI protocol server based on the MCP protocol, optimized for STM32 microcontroller debugging and providing remote embedded debugging capabilities with AI assistants.

## Features

- **STM32-Optimized Debugging**: Specialized for ARM Cortex-M microcontrollers
- **Multi-Family Support**: STM32F0/F1/F2/F3/F4/F7, STM32L0/L1/L4/L5, STM32G0/G4, STM32H7, STM32WB/WL, STM32U5
- **Debug Probe Integration**: ST-Link V2/V3, J-Link, OpenOCD, Black Magic Probe
- **ARM Cortex-M Registers**: Access to R0-R15, PSR, MSP, PSP, PRIMASK, FAULTMASK, BASEPRI, CONTROL
- **Memory Region Access**: Flash (0x08000000), SRAM (0x20000000), Peripherals (0x40000000-0x60000000)
- **Peripheral Register Inspection**: Real-time access to STM32 peripheral registers
- **ARM Thumb/Thumb-2 Disassembly**: Native support for ARM instruction sets
- **Embedded Breakpoint Management**: Optimized for firmware debugging workflows
- **Multi-Session Support**: Debug multiple STM32 devices simultaneously
- **Built-in TUI**: Inspect embedded debugging workflows and improve AI prompts
- **Real-Time Web Dashboard**: Node.js-powered web interface for live debugging
- **WebSocket Integration**: Real-time variable and register monitoring
- **Interactive Debugging**: Web-based controls for breakpoints and execution

## Installation

### Pre-built Binaries
Find the binaries in the release page, choose one per your working platform, then you can run it directly.

### Build From Source
Clone the repository and build it by cargo
```
cargo build --release
cargo run
```

## Usage

### Standalone Usage

1. Just run it directly: `./mcp-server-gdb`
2. The server supports two transport modes:
   - Stdio (default): Standard input/output transport
   - SSE: Server-Sent Events transport, default at `http://127.0.0.1:8080`

### Real-Time Debugging with Node.js Dashboard

For enhanced debugging experience with real-time web interface:

1. **Quick Start with Node.js Integration:**
   ```bash
   # Start both Rust server and Node.js dashboard
   ./scripts/start-with-nodejs.sh
   ```

2. **Manual Setup:**
   ```bash
   # Install Node.js dependencies
   cd nodejs
   npm install

   # Start Rust MCP server (in one terminal)
   ./target/release/mcp-server-gdb --transport sse

   # Start Node.js bridge (in another terminal)
   cd nodejs
   npm start
   ```

3. **Access the Dashboard:**
   - Open your browser to `http://localhost:3000`
   - Real-time debugging interface with live variable monitoring
   - WebSocket-powered updates for immediate feedback
   - Interactive controls for breakpoints and execution

#### Node.js Dashboard Features:
- **Live Variable Monitoring**: Real-time updates of variable values during debugging
- **Register Visualization**: Live ARM Cortex-M register display
- **Interactive Breakpoints**: Set/remove breakpoints through web interface
- **Debug Controls**: Continue, step, stop execution via web buttons
- **Real-time Logs**: Live streaming of debug messages and events
- **Session Management**: Create and manage multiple debugging sessions
- **WebSocket Communication**: Instant updates without page refresh

### Integration with Augment AI

This MCP server can be imported into Augment AI using the provided configuration files:

#### Option 1: Comprehensive STM32 Configuration
Use `mcp-server-gdb.json` for a full-featured STM32 debugging configuration:

```json
{
  "mcpServers": {
    "mcp-server-gdb": {
      "name": "MCP Server GDB for STM32",
      "description": "STM32 microcontroller debugging capabilities for AI assistants",
      "command": "mcp-server-gdb",
      "args": ["--log-level", "info"],
      "transport": "stdio",
      "env": {
        "GDB_COMMAND_TIMEOUT": "30"
      }
    }
  }
}
```

#### Option 2: Simple STM32 Configuration
Use `mcp-stm32-config.json` for a minimal STM32 setup:

```json
{
  "mcpServers": {
    "stm32-gdb": {
      "command": "mcp-server-gdb",
      "args": ["--log-level", "info"],
      "env": {
        "GDB_COMMAND_TIMEOUT": "30"
      }
    }
  }
}
```

#### Import Steps:
1. **Ensure the binary exists**:
   ```bash
   # Check if binary exists
   ls -la build/mcp-server-gdb
   # If not, build it (requires Rust toolchain)
   cargo build --release
   ```

2. **Choose a configuration file**:
   - `mcp-stm32-recommended.json` - **RECOMMENDED** - Full path with working directory
   - `mcp-server-gdb.json` - Comprehensive configuration with STM32 metadata
   - `mcp-stm32-config.json` - Simple configuration with relative path
   - `mcp-stm32-absolute.json` - Simple configuration with absolute path

3. **Update paths** in your chosen configuration file:
   - Replace `/Users/alanhu/development/mcp_server_gdb` with your actual path
   - Ensure the `command` points to the correct binary location
   - Set `cwd` to the repository root directory (important for log file creation)

4. **Import the configuration** into Augment AI

5. **Set up STM32 debugging environment**:
   - Install `arm-none-eabi-gdb` (GNU Arm Embedded Toolchain)
   - Set up your debug probe (ST-Link, J-Link, etc.)
   - Start OpenOCD or ST-Link GDB server for your target

#### STM32 Setup Requirements:

**Required Tools:**
```bash
# Install ARM GCC toolchain (includes arm-none-eabi-gdb)
# On macOS:
brew install --cask gcc-arm-embedded

# On Ubuntu/Debian:
sudo apt-get install gcc-arm-none-eabi gdb-arm-none-eabi

# Install OpenOCD (for debug probe support)
# On macOS:
brew install openocd

# On Ubuntu/Debian:
sudo apt-get install openocd
```

**Example OpenOCD command for STM32:**
```bash
# For STM32F4 with ST-Link V2:
openocd -f interface/stlink-v2.cfg -f target/stm32f4x.cfg

# For STM32L4 with ST-Link V3:
openocd -f interface/stlink-v3.cfg -f target/stm32l4x.cfg
```

**Example GDB session creation:**
```json
{
  "program": "/path/to/your/firmware.elf",
  "gdb_path": "arm-none-eabi-gdb"
}
```

## STM32 Debugging Workflows

### Common STM32 Debugging Tasks:

1. **Firmware Loading and Debugging:**
   ```
   create_session(program="/path/to/firmware.elf", gdb_path="arm-none-eabi-gdb")
   set_breakpoint(file="main.c", line=100)
   start_debugging()
   continue_execution()
   ```

2. **Peripheral Register Inspection:**
   ```
   read_memory(address="0x40020000", size=32)  # GPIOA registers
   read_memory(address="0x40023800", size=32)  # RCC registers
   read_memory(address="0x40013800", size=32)  # USART1 registers
   ```

3. **Memory Analysis:**
   ```
   read_memory(address="0x08000000", size=1024)  # Flash memory
   read_memory(address="0x20000000", size=1024)  # SRAM
   get_memory_mappings()  # View memory layout
   ```

4. **ARM Cortex-M Register Inspection:**
   ```
   get_registers(["R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7"])
   get_registers(["R8", "R9", "R10", "R11", "R12", "SP", "LR", "PC"])
   get_registers(["PSR", "MSP", "PSP", "PRIMASK", "FAULTMASK", "BASEPRI"])
   ```

## Configuration

You can adjust server configuration by modifying the `src/config.rs` file or by environment variables:

- **SERVER_IP**: Server IP address for SSE transport (default: 127.0.0.1)
- **SERVER_PORT**: Server port for SSE transport (default: 8080)
- **GDB_COMMAND_TIMEOUT**: GDB command timeout in seconds (default: 30 for STM32)

## Supported MCP Tools

### Session Management

- `create_session` - Create a new GDB debugging session
- `get_session` - Get specific session information
- `get_all_sessions` - Get all sessions
- `close_session` - Close session

### Debug Control

- `start_debugging` - Start debugging
- `stop_debugging` - Stop debugging
- `continue_execution` - Continue execution
- `step_execution` - Step into next line
- `next_execution` - Step over next line

### Breakpoint Management

- `get_breakpoints` - Get breakpoint list
- `set_breakpoint` - Set breakpoint
- `delete_breakpoint` - Delete breakpoint

### Debug Information

- `get_stack_frames` - Get stack frame information
- `get_local_variables` - Get local variables
- `get_registers` - Get registers
- `read_memory` - Read memory contents

## License

MIT
