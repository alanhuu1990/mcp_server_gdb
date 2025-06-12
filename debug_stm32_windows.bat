@echo off
REM STM32 GDB Debug Script for Windows
REM This script connects to the ST-Link GDB server and starts debugging

echo STM32 GDB Debug Session
echo.

REM Check if GDB server is running
echo Checking if ST-Link GDB server is running on port 4242...
netstat -an | findstr ":4242" >nul
if %ERRORLEVEL% neq 0 (
    echo ERROR: ST-Link GDB server is not running on port 4242
    echo Please start the server first using: start_stlink_server_windows.bat
    pause
    exit /b 1
)

echo GDB server detected on port 4242
echo.

REM Set the ELF file path (modify this to match your project)
set ELF_FILE=Debug\stm32-project.elf
if not exist "%ELF_FILE%" (
    echo WARNING: ELF file not found: %ELF_FILE%
    echo Please build your project first or update the ELF_FILE path in this script
    echo.
    set /p ELF_FILE="Enter path to your .elf file: "
)

echo Starting GDB session with: %ELF_FILE%
echo.

REM Start ARM GDB with connection to ST-Link server
arm-none-eabi-gdb.exe "%ELF_FILE%" ^
    -ex "target extended-remote localhost:4242" ^
    -ex "monitor reset" ^
    -ex "load" ^
    -ex "monitor reset" ^
    -ex "break main" ^
    -ex "continue"

echo.
echo GDB session ended.
pause
