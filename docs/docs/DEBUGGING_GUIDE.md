# STM32F429 Debugging Guide

## Quick Start Guide for Streamlined Debugging

This guide provides step-by-step instructions for efficient debugging of the STM32F429 project using both MCP GDB Server and traditional ST-Link debugging methods.

## 🚀 Quick Debug Session Setup

### Prerequisites Checklist
- [ ] STM32F429 board connected via USB (ST-Link)
- [ ] Project built successfully (`Debug/stm32-f429.elf` exists)
- [ ] ST-Link utilities installed (`brew install stlink`)
- [ ] ARM GDB toolchain installed (`brew install arm-none-eabi-gdb`)
- [ ] MCP GDB Server available (if using MCP method)

### Method 1: Quick Variable Inspection (Recommended)

**Use Case**: Get current values of variables like `counter_1000ms`

```bash
# 1. Start ST-Link server (in background)
cd stm32-f429
./start_stlink_server.sh &

# 2. Quick variable check
arm-none-eabi-gdb Debug/stm32-f429.elf \
  -ex "target extended-remote localhost:4242" \
  -ex "break main.c:112" \
  -ex "continue" \
  -ex "print counter_1000ms" \
  -ex "print last_tick_time" \
  -ex "print HAL_GetTick()" \
  -ex "quit"
```

### Method 2: Interactive Debugging Session

**Use Case**: Step-by-step debugging with breakpoints

```bash
# 1. Start ST-Link server
./start_stlink_server.sh

# 2. Start interactive GDB session
arm-none-eabi-gdb Debug/stm32-f429.elf -x .gdbinit

# 3. In GDB prompt:
(gdb) connect_stlink
(gdb) break main
(gdb) continue
(gdb) print counter_1000ms
```

### Method 3: MCP GDB Server (API-based)

**Use Case**: Programmatic debugging control

```bash
# Use MCP tools to create session and control debugging
# Session management through API calls
```

## 📋 Common Debugging Scenarios

### Scenario 1: Check Counter Value After Time Period

```bash
# Let program run for specific time, then check counter
echo "Letting program run for 20 seconds..."
sleep 20

arm-none-eabi-gdb Debug/stm32-f429.elf \
  -ex "target extended-remote localhost:4242" \
  -ex "break main.c:112" \
  -ex "continue" \
  -ex "print counter_1000ms" \
  -ex "continue" \
  -ex "print counter_1000ms" \
  -ex "quit"
```

### Scenario 2: Monitor Counter Increments

```bash
# Set breakpoint at counter increment and monitor
arm-none-eabi-gdb Debug/stm32-f429.elf \
  -ex "target extended-remote localhost:4242" \
  -ex "break main.c:112" \
  -ex "commands 1" \
  -ex "print counter_1000ms" \
  -ex "continue" \
  -ex "end" \
  -ex "continue"
```

### Scenario 3: System Health Check

```bash
# Check system timing and variables
arm-none-eabi-gdb Debug/stm32-f429.elf \
  -ex "target extended-remote localhost:4242" \
  -ex "interrupt" \
  -ex "print HAL_GetTick()" \
  -ex "break main" \
  -ex "continue" \
  -ex "info locals" \
  -ex "show_cortex_regs" \
  -ex "quit"
```

## 🎯 Key Breakpoint Locations

### Strategic Breakpoints for STM32F429 Project

| Line | Location | Purpose | Command |
|------|----------|---------|---------|
| 77 | `HAL_Init()` | System initialization | `break main.c:77` |
| 84 | `SystemClock_Config()` | Clock setup | `break main.c:84` |
| 94 | Timer initialization | Variable setup | `break main.c:94` |
| 108 | `HAL_GetTick()` call | Timing check | `break main.c:108` |
| 112 | `counter_1000ms++` | Counter increment | `break main.c:112` |
| 174 | `Error_Handler()` | Error conditions | `break main.c:174` |

### Pre-configured Breakpoint Functions

```bash
# Available in .gdbinit
(gdb) bp_main          # Break at main function
(gdb) bp_hardfault     # Break at HardFault handler
(gdb) bp_systick       # Break at SysTick handler
```

## 🔍 Variable Inspection Commands

### Core Variables in STM32F429 Project

```bash
# Main application variables
print counter_1000ms     # 1000ms counter value
print last_tick_time     # Last recorded tick time
print current_tick       # Current HAL tick value

# System timing
print HAL_GetTick()      # Current system tick (milliseconds)

# Local variables in main()
info locals              # All local variables in current scope

# Memory inspection
x/16x 0x20000000        # Examine SRAM start
x/16x 0x08000000        # Examine Flash start
```

## ⚡ One-Line Debug Commands

### Quick Status Checks

```bash
# Get counter value immediately
arm-none-eabi-gdb Debug/stm32-f429.elf -batch -ex "target extended-remote localhost:4242" -ex "break main.c:112" -ex "continue" -ex "print counter_1000ms" -ex "quit"

# Check system uptime
arm-none-eabi-gdb Debug/stm32-f429.elf -batch -ex "target extended-remote localhost:4242" -ex "print HAL_GetTick()" -ex "quit"

# Verify program is running
arm-none-eabi-gdb Debug/stm32-f429.elf -batch -ex "target extended-remote localhost:4242" -ex "interrupt" -ex "bt" -ex "quit"
```

## 🛠️ Troubleshooting Quick Fixes

### Common Issues and Solutions

| Issue | Symptom | Solution |
|-------|---------|----------|
| ST-Link not found | `Error: no device found` | `st-info --probe` then reconnect USB |
| GDB connection failed | `Connection refused` | Restart ST-Link server: `./start_stlink_server.sh` |
| Variables not accessible | `No symbol in current context` | Ensure breakpoint is in main() function |
| Program not responding | No breakpoint hits | Check if program is loaded: `load` command |
| Counter always 0 | Counter not incrementing | Verify HAL_GetTick() is working |

### Emergency Reset Commands

```bash
# Hard reset and reload
(gdb) monitor reset halt
(gdb) load
(gdb) monitor reset halt

# Or use helper function
(gdb) reset
```

## 📊 Expected Values and Timing

### Normal Operation Values

| Variable | Expected Range | Notes |
|----------|----------------|-------|
| `counter_1000ms` | 0 to ∞ | Increments every 1000ms |
| `last_tick_time` | 0 to ∞ | HAL tick value at last increment |
| `current_tick` | 0 to ∞ | Current HAL tick (milliseconds) |
| `HAL_GetTick()` | 0 to ∞ | System uptime in milliseconds |

### Timing Verification

```bash
# Verify 1000ms timing
# counter_1000ms should increment every 1000ms
# (current_tick - last_tick_time) should be ≥ 1000 when counter increments
```

## 🔄 Automated Debug Scripts

### Create Custom Debug Scripts

```bash
# Save as debug_counter.sh
#!/bin/bash
echo "=== STM32F429 Counter Debug ==="
arm-none-eabi-gdb Debug/stm32-f429.elf \
  -ex "target extended-remote localhost:4242" \
  -ex "break main.c:112" \
  -ex "continue" \
  -ex "printf \"Counter: %d, Tick: %d\\n\", counter_1000ms, HAL_GetTick()" \
  -ex "continue" \
  -ex "printf \"Counter: %d, Tick: %d\\n\", counter_1000ms, HAL_GetTick()" \
  -ex "quit"
```

## 📝 Debug Session Templates

### Template 1: Quick Health Check

```bash
# Copy and modify as needed
arm-none-eabi-gdb Debug/stm32-f429.elf \
  -ex "target extended-remote localhost:4242" \
  -ex "interrupt" \
  -ex "print HAL_GetTick()" \
  -ex "break main.c:112" \
  -ex "continue" \
  -ex "print counter_1000ms" \
  -ex "quit"
```

### Template 2: Extended Monitoring

```bash
# Monitor for multiple increments
arm-none-eabi-gdb Debug/stm32-f429.elf \
  -ex "target extended-remote localhost:4242" \
  -ex "break main.c:112" \
  -ex "continue" \
  -ex "print counter_1000ms" \
  -ex "continue" \
  -ex "print counter_1000ms" \
  -ex "continue" \
  -ex "print counter_1000ms" \
  -ex "quit"
```

## 🎯 Best Practices

1. **Always start ST-Link server first**: `./start_stlink_server.sh`
2. **Use batch mode for quick checks**: Add `-batch` flag
3. **Set breakpoints at strategic locations**: Line 112 for counter, line 77 for main
4. **Verify timing with HAL_GetTick()**: Should increment by ~1000ms between counter increments
5. **Use helper functions from .gdbinit**: `connect_stlink`, `reset`, `show_cortex_regs`
6. **Monitor system health**: Check HAL_GetTick() for system responsiveness

## 📚 Reference

- **Project Structure**: See `DEBUG_README.md` for detailed setup
- **GDB Commands**: See `.gdbinit` for available helper functions
- **Hardware Setup**: Ensure USB connection and ST-Link drivers
- **Build Status**: Verify `Debug/stm32-f429.elf` exists and is recent

---

*This guide is optimized for the STM32F429 counter application. Modify breakpoint locations and variable names for other projects.*
