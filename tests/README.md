# STM32 GDB Debug Test Suite

Comprehensive test suite for STM32 debugging functionality using the MCP GDB server.

## 🎯 Overview

This test suite validates the complete STM32 debugging workflow, from basic MCP server functionality to real hardware debugging scenarios. Tests are organized into categories based on complexity and hardware requirements.

## 📁 Test Structure

```
tests/
├── common/                     # Shared test utilities
│   └── mod.rs                 # Common functions and macros
├── unit/                      # Unit tests (no hardware required)
│   └── test_stm32_gdb_mcp.rs  # MCP server functionality tests
├── integration/               # Integration tests (hardware recommended)
│   └── test_stm32_debug_session.rs  # Debugging session tests
├── hardware/                  # Hardware-specific tests (hardware required)
│   └── test_stm32_hardware_debug.rs  # Real hardware debugging
├── e2e/                       # End-to-end tests (hardware required)
│   └── test_complete_debug_workflow.rs  # Complete workflows
├── performance/               # Performance tests (hardware required)
│   └── test_debug_performance.rs  # Timing and efficiency tests
├── fixtures/                  # Test configuration and data
│   └── stm32_test_config.json # Test configuration
├── run_stm32_tests.sh         # Test runner script
└── README.md                  # This file
```

## 🚀 Quick Start

### Prerequisites

1. **Rust toolchain** (1.87.0 or later)
2. **ARM GDB toolchain**:
   ```bash
   brew install arm-none-eabi-gdb
   ```
3. **ST-Link utilities** (for hardware tests):
   ```bash
   brew install stlink
   ```
4. **STM32F429 board** (optional, for hardware tests)

### Running Tests

```bash
# Run all tests
./tests/run_stm32_tests.sh all

# Run specific test categories
./tests/run_stm32_tests.sh unit          # Unit tests only
./tests/run_stm32_tests.sh integration   # Integration tests
./tests/run_stm32_tests.sh hardware      # Hardware tests
./tests/run_stm32_tests.sh e2e           # End-to-end tests
./tests/run_stm32_tests.sh performance   # Performance tests

# Check prerequisites
./tests/run_stm32_tests.sh check

# Build project only
./tests/run_stm32_tests.sh build
```

### Using Cargo Directly

```bash
# Run all tests
cargo test

# Run specific test files
cargo test --test test_stm32_gdb_mcp
cargo test --test test_stm32_debug_session

# Run with debug output
RUST_LOG=debug cargo test -- --nocapture
```

## 📋 Test Categories

### 1. Unit Tests (`tests/unit/`)
- **Purpose**: Test MCP GDB server functions in isolation
- **Hardware Required**: No
- **Duration**: ~30 seconds
- **Tests**:
  - Session creation and management
  - Breakpoint setting
  - Register access
  - Memory operations
  - Error handling

### 2. Integration Tests (`tests/integration/`)
- **Purpose**: Test debugging sessions with STM32 project
- **Hardware Required**: Yes (recommended)
- **Duration**: ~60 seconds
- **Tests**:
  - Complete debugging sessions
  - Counter monitoring
  - Memory region access
  - Register operations
  - Breakpoint functionality

### 3. Hardware Tests (`tests/hardware/`)
- **Purpose**: Test with real STM32 hardware
- **Hardware Required**: Yes (STM32F429 board)
- **Duration**: ~120 seconds
- **Tests**:
  - Hardware detection
  - Real-time counter monitoring
  - Timing accuracy
  - Reset and reload
  - Flash programming
  - Peripheral access

### 4. End-to-End Tests (`tests/e2e/`)
- **Purpose**: Test complete debugging workflows
- **Hardware Required**: Yes
- **Duration**: ~180 seconds
- **Tests**:
  - Complete workflow from start to finish
  - Automated script integration
  - MCP server integration
  - Error recovery and robustness

### 5. Performance Tests (`tests/performance/`)
- **Purpose**: Measure debugging performance and efficiency
- **Hardware Required**: Yes
- **Duration**: ~300 seconds
- **Tests**:
  - Connection performance
  - Breakpoint setting speed
  - Memory read throughput
  - Counter monitoring efficiency
  - Session overhead

## 🔧 Configuration

### Test Configuration File
The test suite uses `tests/fixtures/stm32_test_config.json` for configuration:

```json
{
  "test_configuration": {
    "name": "STM32F429 GDB Debug Tests",
    "version": "1.0.0"
  },
  "hardware_requirements": {
    "board": "STM32F429 Discovery or compatible",
    "connection": "USB ST-Link"
  },
  "software_requirements": {
    "gdb": "arm-none-eabi-gdb",
    "stlink": "st-util"
  }
}
```

### Environment Variables
- `STM32_HARDWARE_AVAILABLE`: Set to 1 to force hardware tests
- `RUST_LOG`: Set log level (debug, info, warn, error)
- `GDB_COMMAND_TIMEOUT`: GDB command timeout in seconds

## 🎯 Test Scenarios

### Basic Functionality
- MCP server session management
- Breakpoint operations
- Register access
- Error handling

### Hardware Debugging
- Real hardware detection
- Counter monitoring
- Memory access
- Timing verification

### Workflow Testing
- Complete debugging sessions
- Script integration
- Error recovery

### Performance Testing
- Connection speed
- Operation efficiency
- Throughput measurement

## 📊 Expected Results

### Counter Behavior
- **Increment Interval**: 1000ms ±100ms
- **Reset Value**: 0
- **Maximum Reasonable Value**: 1,000,000

### Performance Benchmarks
- **Connection Time**: < 5 seconds
- **Breakpoint Setting**: < 500ms
- **Memory Read**: < 2 seconds
- **Counter Read**: < 10 seconds

### Memory Regions
- **Flash (0x08000000)**: Readable
- **SRAM (0x20000000)**: Readable/Writable
- **CCM RAM (0x10000000)**: Readable/Writable

## 🔍 Troubleshooting

### Common Issues

1. **Hardware not detected**
   ```bash
   st-info --probe
   ```
   Solution: Connect STM32 board, install ST-Link drivers

2. **GDB not found**
   ```bash
   arm-none-eabi-gdb --version
   ```
   Solution: Install ARM GDB toolchain

3. **ELF file not found**
   ```bash
   ls tests/stm32-f0-disco/stm32-f429/Debug/stm32-f429.elf
   ```
   Solution: Build STM32 project first

4. **ST-Link server fails**
   ```bash
   st-util --version
   ```
   Solution: Install ST-Link utilities

### Test Skipping
- Hardware tests are automatically skipped if hardware is not detected
- Environment tests are skipped if required tools are not installed
- Tests have individual timeouts to prevent hanging

## 🚀 CI/CD Integration

### GitHub Actions
- **Unit tests**: Always run
- **Integration tests**: Run with simulation where possible
- **Hardware tests**: Skip in CI (no hardware available)

### Local Development
- **Full suite**: Run all tests if hardware available
- **Quick check**: Run unit tests only
- **Hardware validation**: Run hardware tests to verify setup

## 📝 Adding New Tests

### 1. Create Test File
```rust
// tests/new_category/test_new_functionality.rs
mod common;
use common::*;

#[tokio::test]
async fn test_new_functionality() {
    let config = STM32TestConfig::default();
    // Test implementation
}
```

### 2. Update Test Runner
Add new category to `tests/run_stm32_tests.sh`:
```bash
"new_category"]="New functionality tests|false"
```

### 3. Update Configuration
Add test details to `tests/fixtures/stm32_test_config.json`

## 📞 Support

For issues with the test suite:
1. Check prerequisites with `./tests/run_stm32_tests.sh check`
2. Review test output for specific error messages
3. Consult troubleshooting section above
4. Check STM32 project documentation in `tests/stm32-f0-disco/stm32-f429/docs/`

## 🔮 Future Enhancements

- Web-based test dashboard
- Automated hardware-in-the-loop testing
- Performance regression detection
- Test result visualization
- Remote debugging test scenarios
