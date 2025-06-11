@echo off
REM STM32 ST-Link GDB Server Startup Script for Windows
REM This script starts the ST-Link GDB server with proper configuration

echo Starting ST-Link GDB Server for STM32...
echo.

REM Set paths
set STLINK_GDBSERVER="C:\ST\STM32CubeIDE_1.18.0\STM32CubeIDE\plugins\com.st.stm32cube.ide.mcu.externaltools.stlink-gdb-server.win32_2.2.100.202501151542\tools\bin\ST-LINK_gdbserver.exe"
set CUBEPROG_PATH="C:\Program Files\STMicroelectronics\STM32Cube\STM32CubeProgrammer\bin"

REM Check if ST-Link device is connected
echo Checking for connected ST-Link devices...
%STLINK_GDBSERVER% -q -cp %CUBEPROG_PATH%
if %ERRORLEVEL% neq 0 (
    echo ERROR: No ST-Link device found or connection failed
    echo Please check:
    echo 1. STM32 board is connected via USB
    echo 2. ST-Link drivers are installed
    echo 3. Device is powered on
    pause
    exit /b 1
)

echo.
echo ST-Link device detected successfully!
echo Starting GDB server on port 4242...
echo.
echo Press Ctrl+C to stop the server
echo.

REM Start the GDB server
%STLINK_GDBSERVER% -p 4242 -d -s -cp %CUBEPROG_PATH%

echo.
echo GDB Server stopped.
pause
