# STM32F429 Debug Quick Reference Card

## 🚀 Essential Commands (Copy & Paste Ready)

### Start Debugging Session
```bash
# 1. Start ST-Link server (run once)
cd stm32-f429 && ./start_stlink_server.sh &

# 2. Quick counter check
arm-none-eabi-gdb Debug/stm32-f429.elf -ex "target extended-remote localhost:4242" -ex "break main.c:112" -ex "continue" -ex "print counter_1000ms" -ex "quit"
```

### Get Counter Value NOW
```bash
arm-none-eabi-gdb Debug/stm32-f429.elf -batch -ex "target extended-remote localhost:4242" -ex "break main.c:112" -ex "continue" -ex "print counter_1000ms" -ex "print HAL_GetTick()" -ex "quit"
```

### Test Counter with 5-Second Analysis (RECOMMENDED)
```bash
./debug_automated.sh counter5
```

### Monitor Counter for 3 Increments
```bash
arm-none-eabi-gdb Debug/stm32-f429.elf -ex "target extended-remote localhost:4242" -ex "break main.c:112" -ex "continue" -ex "print counter_1000ms" -ex "continue" -ex "print counter_1000ms" -ex "continue" -ex "print counter_1000ms" -ex "quit"
```

### Check System Health
```bash
arm-none-eabi-gdb Debug/stm32-f429.elf -batch -ex "target extended-remote localhost:4242" -ex "interrupt" -ex "print HAL_GetTick()" -ex "bt" -ex "quit"
```

## 🎯 Key Breakpoints

| Command | Location | Purpose |
|---------|----------|---------|
| `break main.c:77` | HAL_Init() | System start |
| `break main.c:112` | counter_1000ms++ | Counter increment |
| `break main.c:174` | Error_Handler() | Error detection |

## 📊 Essential Variables

| Variable | Command | Expected |
|----------|---------|----------|
| Counter | `print counter_1000ms` | 0, 1, 2, 3... |
| System Time | `print HAL_GetTick()` | Milliseconds since boot |
| Last Tick | `print last_tick_time` | Previous counter time |

## 🔧 Troubleshooting

| Problem | Solution |
|---------|----------|
| Connection refused | `./start_stlink_server.sh` |
| No symbol found | Add `break main.c:112` first |
| Counter always 0 | Check if program is running |
| ST-Link not found | `st-info --probe` |

## ⚡ One-Liners

```bash
# Counter value right now
arm-none-eabi-gdb Debug/stm32-f429.elf -batch -ex "target extended-remote localhost:4242" -ex "break main.c:112" -ex "continue" -ex "print counter_1000ms" -ex "quit"

# Counter with 5-second analysis (BEST)
./debug_automated.sh counter5

# System uptime
arm-none-eabi-gdb Debug/stm32-f429.elf -batch -ex "target extended-remote localhost:4242" -ex "print HAL_GetTick()" -ex "quit"

# Reset and reload
arm-none-eabi-gdb Debug/stm32-f429.elf -batch -ex "target extended-remote localhost:4242" -ex "load" -ex "monitor reset halt" -ex "quit"
```

## 🎮 Interactive Mode

```bash
# Start interactive session
arm-none-eabi-gdb Debug/stm32-f429.elf -x .gdbinit

# Then use:
(gdb) connect_stlink
(gdb) break main.c:112
(gdb) continue
(gdb) print counter_1000ms
(gdb) continue
```

## 📱 MCP GDB Server Commands

```bash
# Create session
create_session_stm32-gdb --program stm32-f429/Debug/stm32-f429.elf

# Set breakpoint
set_breakpoint_stm32-gdb --session-id <ID> --file main.c --line 112

# Get variables
get_local_variables_stm32-gdb --session-id <ID>
```

---
**💡 Pro Tip**: Save this file as bookmark for instant access to debug commands!
