# STM32 GDB Debug PowerShell Script for Windows
# This script provides advanced debugging capabilities for STM32

param(
    [string]$ElfFile = "",
    [int]$Port = 4242,
    [switch]$StartServer,
    [switch]$StopServer,
    [switch]$ListDevices,
    [switch]$Help
)

# Configuration
$StlinkGdbServer = "C:\ST\STM32CubeIDE_1.18.0\STM32CubeIDE\plugins\com.st.stm32cube.ide.mcu.externaltools.stlink-gdb-server.win32_2.2.100.202501151542\tools\bin\ST-LINK_gdbserver.exe"
$CubeProgrammerPath = "C:\Program Files\STMicroelectronics\STM32Cube\STM32CubeProgrammer\bin"
$ArmGdb = "arm-none-eabi-gdb.exe"

function Show-Help {
    Write-Host "STM32 GDB Debug Script for Windows" -ForegroundColor Green
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\debug_stm32_windows.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -ElfFile <path>     Path to the .elf file to debug"
    Write-Host "  -Port <number>      GDB server port (default: 4242)"
    Write-Host "  -StartServer        Start the ST-Link GDB server"
    Write-Host "  -StopServer         Stop the ST-Link GDB server"
    Write-Host "  -ListDevices        List connected ST-Link devices"
    Write-Host "  -Help               Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\debug_stm32_windows.ps1 -ListDevices"
    Write-Host "  .\debug_stm32_windows.ps1 -StartServer"
    Write-Host "  .\debug_stm32_windows.ps1 -ElfFile 'Debug\project.elf'"
}

function Test-StlinkConnection {
    Write-Host "Checking ST-Link connection..." -ForegroundColor Yellow
    try {
        $result = & $StlinkGdbServer -q -cp $CubeProgrammerPath 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "ST-Link devices found:" -ForegroundColor Green
            Write-Host $result
            return $true
        } else {
            Write-Host "No ST-Link devices found or connection failed" -ForegroundColor Red
            Write-Host $result
            return $false
        }
    } catch {
        Write-Host "Error checking ST-Link connection: $_" -ForegroundColor Red
        return $false
    }
}

function Start-StlinkServer {
    Write-Host "Starting ST-Link GDB Server on port $Port..." -ForegroundColor Yellow
    
    if (-not (Test-StlinkConnection)) {
        Write-Host "Cannot start server - no ST-Link device detected" -ForegroundColor Red
        return $false
    }
    
    try {
        $process = Start-Process -FilePath $StlinkGdbServer -ArgumentList "-p", $Port, "-d", "-s", "-cp", $CubeProgrammerPath -PassThru
        Start-Sleep -Seconds 2
        
        if (Test-NetConnection -ComputerName localhost -Port $Port -InformationLevel Quiet) {
            Write-Host "ST-Link GDB Server started successfully on port $Port" -ForegroundColor Green
            Write-Host "Process ID: $($process.Id)"
            return $true
        } else {
            Write-Host "Failed to start ST-Link GDB Server" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "Error starting ST-Link GDB Server: $_" -ForegroundColor Red
        return $false
    }
}

function Stop-StlinkServer {
    Write-Host "Stopping ST-Link GDB Server..." -ForegroundColor Yellow
    try {
        Get-Process -Name "ST-LINK_gdbserver" -ErrorAction SilentlyContinue | Stop-Process -Force
        Write-Host "ST-Link GDB Server stopped" -ForegroundColor Green
    } catch {
        Write-Host "Error stopping ST-Link GDB Server: $_" -ForegroundColor Red
    }
}

function Test-GdbServerRunning {
    return (Test-NetConnection -ComputerName localhost -Port $Port -InformationLevel Quiet)
}

function Start-GdbSession {
    param([string]$ElfPath)
    
    if (-not (Test-Path $ElfPath)) {
        Write-Host "ELF file not found: $ElfPath" -ForegroundColor Red
        return
    }
    
    if (-not (Test-GdbServerRunning)) {
        Write-Host "GDB server is not running on port $Port" -ForegroundColor Red
        Write-Host "Start the server first with: -StartServer" -ForegroundColor Yellow
        return
    }
    
    Write-Host "Starting GDB session with: $ElfPath" -ForegroundColor Green
    
    $gdbCommands = @(
        "target extended-remote localhost:$Port",
        "monitor reset",
        "load",
        "monitor reset",
        "break main",
        "continue"
    )
    
    $gdbArgs = @($ElfPath)
    foreach ($cmd in $gdbCommands) {
        $gdbArgs += "-ex"
        $gdbArgs += $cmd
    }
    
    try {
        & $ArmGdb @gdbArgs
    } catch {
        Write-Host "Error starting GDB: $_" -ForegroundColor Red
    }
}

# Main script logic
if ($Help) {
    Show-Help
    exit
}

if ($ListDevices) {
    Test-StlinkConnection
    exit
}

if ($StartServer) {
    Start-StlinkServer
    exit
}

if ($StopServer) {
    Stop-StlinkServer
    exit
}

if ($ElfFile) {
    Start-GdbSession -ElfPath $ElfFile
} else {
    Show-Help
}
