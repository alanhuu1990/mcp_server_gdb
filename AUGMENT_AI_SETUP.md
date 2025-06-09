# MCP Server GDB for STM32 - Augment AI Setup Guide

## Overview

This repository contains an MCP (Model Context Protocol) server optimized for STM32 microcontroller debugging. The server has been modified to work seamlessly with Augment AI by fixing read-only filesystem issues and providing comprehensive STM32-specific configuration.

## What Was Fixed

### 1. Read-Only Filesystem Issue
- **Problem**: The original server tried to create a `logs` directory, causing crashes in read-only environments like Augment AI
- **Solution**: Modified `src/main.rs` to gracefully fall back to stderr logging when file logging fails
- **Code Changes**: Added panic handling around `RollingFileAppender::new()` with fallback to stderr

### 2. STM32 Optimization
- Updated descriptions and metadata to focus on STM32 debugging
- Extended timeout from 10 to 30 seconds for embedded debugging
- Added STM32-specific tool descriptions and examples
- Included ARM Cortex-M register and memory region information

## Available Configuration Files

### Recommended Configuration
**File**: `mcp-stm32-recommended.json`
```json
{
  "mcpServers": {
    "stm32-gdb": {
      "command": "/Users/alanhu/development/mcp_server_gdb/target/release/mcp-server-gdb",
      "args": ["--log-level", "info"],
      "cwd": "/Users/alanhu/development/mcp_server_gdb",
      "env": {
        "GDB_COMMAND_TIMEOUT": "30",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Other Options
1. **`mcp-server-gdb.json`** - Comprehensive configuration with full STM32 metadata
2. **`mcp-stm32-config.json`** - Simple configuration with relative paths
3. **`mcp-stm32-absolute.json`** - Simple configuration with absolute paths
4. **`mcp-stm32-working-dir.json`** - Configuration with working directory specified

## Setup Instructions

### 1. Build the Fixed Binary
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Build the release binary
cargo build --release
```

### 2. Update Configuration Paths
Edit your chosen configuration file and replace `/Users/alanhu/development/mcp_server_gdb` with your actual repository path.

### 3. Import into Augment AI
1. Choose one of the configuration files (recommend `mcp-stm32-recommended.json`)
2. Update the paths to match your system
3. Import the configuration into Augment AI

## STM32 Debugging Features

### Supported STM32 Families
- STM32F0, F1, F2, F3, F4, F7
- STM32L0, L1, L4, L5
- STM32G0, G4
- STM32H7
- STM32WB, WL
- STM32U5

### Debug Probe Support
- ST-Link V2/V3
- J-Link
- OpenOCD compatible probes
- Black Magic Probe

### Memory Regions
- **Flash**: 0x08000000 (varies by device)
- **SRAM**: 0x20000000 (varies by device)
- **Peripherals**: 0x40000000 - 0x60000000
- **System Memory**: 0x1FFF0000 (bootloader)

### ARM Cortex-M Registers
- Core: R0-R15, PSR, MSP, PSP, PRIMASK, FAULTMASK, BASEPRI, CONTROL
- Debug: DHCSR, DCRSR, DCRDR, DEMCR
- SysTick: SYST_CSR, SYST_RVR, SYST_CVR, SYST_CALIB

## Available Tools

### Session Management
- `create_session` - Create GDB session with STM32 firmware
- `get_session` - Get session information
- `get_all_sessions` - List all sessions
- `close_session` - Close debugging session

### Debug Control
- `start_debugging` - Connect to STM32 target
- `stop_debugging` - Disconnect from target
- `continue_execution` - Resume execution
- `step_execution` - Step into next instruction
- `next_execution` - Step over next instruction

### Breakpoint Management
- `set_breakpoint` - Set breakpoints in firmware
- `delete_breakpoint` - Remove breakpoints
- `get_breakpoints` - List all breakpoints

### STM32 Inspection
- `get_registers` - Read ARM Cortex-M registers
- `get_local_variables` - View current variables
- `read_memory` - Access Flash/SRAM/peripheral registers
- `get_memory_mappings` - View STM32 memory layout
- `disassemble` - View ARM Thumb/Thumb-2 assembly

## Example Usage

### Basic STM32 Debugging Session
```
1. create_session(program="/path/to/firmware.elf", gdb_path="arm-none-eabi-gdb")
2. set_breakpoint(file="main.c", line=100)
3. start_debugging()
4. continue_execution()
```

### Peripheral Register Inspection
```
1. read_memory(address="0x40020000", size=32)  # GPIOA registers
2. read_memory(address="0x40023800", size=32)  # RCC registers
3. read_memory(address="0x40013800", size=32)  # USART1 registers
```

### Memory Analysis
```
1. read_memory(address="0x08000000", size=1024)  # Flash memory
2. read_memory(address="0x20000000", size=1024)  # SRAM
3. get_memory_mappings()  # View memory layout
```

## Prerequisites

### Required Tools
```bash
# ARM GCC toolchain (includes arm-none-eabi-gdb)
# On macOS:
brew install --cask gcc-arm-embedded

# On Ubuntu/Debian:
sudo apt-get install gcc-arm-none-eabi gdb-arm-none-eabi

# OpenOCD (for debug probe support)
# On macOS:
brew install openocd

# On Ubuntu/Debian:
sudo apt-get install openocd
```

### Example OpenOCD Commands
```bash
# For STM32F4 with ST-Link V2:
openocd -f interface/stlink-v2.cfg -f target/stm32f4x.cfg

# For STM32L4 with ST-Link V3:
openocd -f interface/stlink-v3.cfg -f target/stm32l4x.cfg
```

## Troubleshooting

### Common Issues
1. **Binary not found**: Ensure the path in the configuration file is correct
2. **Permission denied**: Make sure the binary is executable (`chmod +x`)
3. **GDB not found**: Install arm-none-eabi-gdb and ensure it's in PATH
4. **Connection failed**: Check that OpenOCD or ST-Link server is running

### Logging
The server now gracefully handles read-only filesystems by falling back to stderr logging when file logging fails. You'll see a warning message if this happens.

## Version Information
- **MCP Server GDB**: v0.3.0
- **Built with**: Rust 1.87.0
- **Optimized for**: STM32 microcontrollers and ARM Cortex-M debugging
