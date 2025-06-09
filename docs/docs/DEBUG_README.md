# STM32F429 Debugging Setup

This document describes the comprehensive debugging setup for STM32F429 using both ST-Link GDB Server and MCP GDB Server.

## 📚 Documentation Overview

This project includes multiple debugging documentation files:

- **`DEBUGGING_GUIDE.md`** - Comprehensive debugging guide with step-by-step instructions
- **`DEBUG_QUICK_REFERENCE.md`** - Quick reference card with copy-paste commands
- **`debug_automated.sh`** - Automated debugging script for common tasks
- **`mcp_debug_config.json`** - Configuration file for MCP debugging sessions
- **`DEBUG_README.md`** - This file (overview and setup instructions)

## 🚀 Quick Start (New Users)

1. **Read the Quick Reference**: Start with `DEBUG_QUICK_REFERENCE.md`
2. **Use Automated Script**: Run `./debug_automated.sh help`
3. **For Detailed Info**: See `DEBUGGING_GUIDE.md`

## ⚡ Instant Debug Commands

```bash
# Get counter value right now
./debug_automated.sh counter

# Monitor counter for 30 seconds
./debug_automated.sh monitor 30

# Check system health
./debug_automated.sh health

# Run comprehensive test
./debug_automated.sh test
```

## Prerequisites

### Hardware
- STM32F429 Discovery board or compatible
- ST-Link/V2 debugger (built-in on Discovery board)
- USB cable for ST-Link connection

### Software
- macOS with Homebrew
- STM32CubeIDE or compatible toolchain
- ARM GDB toolchain
- ST-Link utilities
- MCP GDB Server

## Installation

### 1. Install ST-Link Utilities
```bash
brew install stlink
```

### 2. Install ARM GDB Toolchain
```bash
brew install arm-none-eabi-gdb
```

### 3. Verify Installation
```bash
# Check ST-Link
st-info --probe

# Check ARM GDB
arm-none-eabi-gdb --version

# Check MCP GDB Server
which mcp-server-gdb
```

## Debugging Methods

### Method 1: ST-Link GDB Server

#### Starting the Server
```bash
./start_stlink_server.sh
```

This starts the ST-Link GDB server on port 4242.

#### Connecting with GDB
```bash
./debug_with_stlink.sh
```

Or manually:
```bash
arm-none-eabi-gdb -x .gdbinit
(gdb) connect_stlink
```

#### Available GDB Commands
- `connect_stlink` - Connect to ST-Link GDB server
- `reset` - Reset and halt target
- `flash_program` - Program flash and run
- `setup_memory` - Configure memory regions
- `show_cortex_regs` - Display Cortex-M registers
- `bp_main` - Set breakpoint at main
- `bp_hardfault` - Set breakpoint at HardFault

### Method 2: MCP GDB Server

The MCP GDB Server provides a modern API-based interface for debugging.

#### Key Features
- Session management
- Structured API calls
- Programmatic control
- Integration capabilities

#### Basic Usage
The MCP GDB server is controlled through API calls rather than interactive commands.

## File Structure

```
.
├── .gdbinit                 # GDB initialization script
├── start_stlink_server.sh   # ST-Link server startup script
├── debug_with_stlink.sh     # ST-Link debugging script
├── debug_with_mcp.sh        # MCP debugging script
├── debug_demo.sh            # Interactive demonstration
├── DEBUG_README.md          # This documentation
└── Debug/
    └── stm32-f429.elf       # Compiled firmware
```

## Configuration Files

### .gdbinit
Contains GDB initialization commands:
- ARM architecture setup
- Memory region configuration
- Helper functions for STM32 debugging
- Pretty printing settings

### start_stlink_server.sh
Starts the ST-Link GDB server with appropriate settings for STM32F429.

## Debugging Workflow

### 1. Build the Project
Ensure your STM32 project is built and the ELF file exists in `Debug/stm32-f429.elf`.

### 2. Connect Hardware
Connect your STM32F429 board via USB (ST-Link).

### 3. Choose Debugging Method

#### For ST-Link GDB Server:
1. Start the server: `./start_stlink_server.sh`
2. In another terminal: `./debug_with_stlink.sh`
3. Use GDB commands to debug

#### For MCP GDB Server:
Use the MCP API tools to create sessions and control debugging.

### 4. Common Debugging Tasks

#### Set Breakpoints
```gdb
(gdb) break main
(gdb) break 93
```

#### Step Through Code
```gdb
(gdb) step      # Step into
(gdb) next      # Step over
(gdb) continue  # Continue execution
```

#### Examine Variables
```gdb
(gdb) print variable_name
(gdb) info locals
(gdb) info registers
```

#### Memory Inspection
```gdb
(gdb) x/16x 0x20000000  # Examine 16 words at SRAM start
(gdb) x/s string_ptr    # Examine string
```

## Troubleshooting

### ST-Link Not Detected
```bash
# Check USB connection
st-info --probe

# Reset ST-Link
st-info --reset
```

### GDB Connection Issues
```bash
# Ensure ST-Link server is running
ps aux | grep st-util

# Check port availability
lsof -i :4242
```

### Symbol Loading Issues
```bash
# Manually load symbols
(gdb) file Debug/stm32-f429.elf
(gdb) symbol-file Debug/stm32-f429.elf
```

## Advanced Features

### Flash Programming
```gdb
(gdb) flash_program
```

### Memory Regions
The configuration includes proper memory regions for STM32F429:
- Flash: 0x08000000 - 0x081FFFFF (2MB)
- SRAM: 0x20000000 - 0x2002FFFF (192KB)
- CCM RAM: 0x10000000 - 0x1000FFFF (64KB)

### Hardware Breakpoints
STM32F429 supports up to 6 hardware breakpoints and 4 watchpoints.

## Comparison: ST-Link vs MCP

| Feature | ST-Link GDB Server | MCP GDB Server |
|---------|-------------------|----------------|
| Interface | Interactive GDB | API-based |
| Target Connection | Direct hardware | Configurable |
| Session Management | Manual | Automated |
| Integration | Standard GDB | Modern tooling |
| Real-time Debugging | Yes | Yes |
| Flash Programming | Yes | Yes |

## Demo Script

Run the interactive demonstration:
```bash
./debug_demo.sh
```

This script provides a guided tour of both debugging methods.

## Support

For issues with:
- ST-Link: Check hardware connections and drivers
- ARM GDB: Verify toolchain installation
- MCP GDB Server: Check server configuration and API access

## References

- [STM32F429 Reference Manual](https://www.st.com/resource/en/reference_manual/dm00031020.pdf)
- [ARM GDB Documentation](https://sourceware.org/gdb/documentation/)
- [ST-Link Utilities](https://github.com/stlink-org/stlink)
