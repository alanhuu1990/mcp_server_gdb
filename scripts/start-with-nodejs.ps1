# MCP GDB Server with Node.js Real-Time Debugging (Windows PowerShell)
# This script starts both the Rust MCP server and Node.js bridge

param(
    [int]$RustServerPort = 8080,
    [int]$NodejsPort = 3000,
    [int]$WebSocketPort = 3001
)

# Configuration
$ErrorActionPreference = "Stop"

# Directories
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$NodejsDir = Join-Path $ProjectRoot "nodejs"
$RustBinary = Join-Path $ProjectRoot "target\release\mcp-server-gdb.exe"
$LogsDir = Join-Path $ProjectRoot "logs"

Write-Host "=== MCP GDB Server with Node.js Real-Time Debugging ===" -ForegroundColor Blue
Write-Host "Project Root: $ProjectRoot" -ForegroundColor Blue
Write-Host "Node.js Directory: $NodejsDir" -ForegroundColor Blue
Write-Host ""

# Function to check if port is available
function Test-Port {
    param([int]$Port)
    
    try {
        $connection = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue
        if ($connection) {
            Write-Host "Error: Port $Port is already in use" -ForegroundColor Red
            return $false
        }
        return $true
    }
    catch {
        return $true
    }
}

# Function to cleanup processes on exit
function Stop-Servers {
    Write-Host "`nShutting down servers..." -ForegroundColor Yellow
    
    if ($global:RustProcess -and !$global:RustProcess.HasExited) {
        Write-Host "Stopping Rust MCP server (PID: $($global:RustProcess.Id))" -ForegroundColor Yellow
        $global:RustProcess.Kill()
    }
    
    if ($global:NodejsProcess -and !$global:NodejsProcess.HasExited) {
        Write-Host "Stopping Node.js bridge (PID: $($global:NodejsProcess.Id))" -ForegroundColor Yellow
        $global:NodejsProcess.Kill()
    }
    
    Write-Host "Cleanup complete" -ForegroundColor Green
}

# Set up cleanup on Ctrl+C
$null = Register-EngineEvent PowerShell.Exiting -Action { Stop-Servers }

try {
    # Check prerequisites
    Write-Host "Checking prerequisites..." -ForegroundColor Blue

    # Check if Rust binary exists
    if (!(Test-Path $RustBinary)) {
        Write-Host "Error: Rust binary not found at $RustBinary" -ForegroundColor Red
        Write-Host "Please build the project first: cargo build --release" -ForegroundColor Yellow
        exit 1
    }

    # Check if Node.js is installed
    try {
        $nodeVersion = node --version
        Write-Host "Node.js version: $nodeVersion" -ForegroundColor Green
    }
    catch {
        Write-Host "Error: Node.js is not installed" -ForegroundColor Red
        Write-Host "Please install Node.js (version 16 or higher)" -ForegroundColor Yellow
        exit 1
    }

    # Check if npm is installed
    try {
        $npmVersion = npm --version
        Write-Host "npm version: $npmVersion" -ForegroundColor Green
    }
    catch {
        Write-Host "Error: npm is not installed" -ForegroundColor Red
        exit 1
    }

    # Check Node.js directory
    if (!(Test-Path $NodejsDir)) {
        Write-Host "Error: Node.js directory not found at $NodejsDir" -ForegroundColor Red
        exit 1
    }

    # Check if package.json exists
    if (!(Test-Path (Join-Path $NodejsDir "package.json"))) {
        Write-Host "Error: package.json not found in $NodejsDir" -ForegroundColor Red
        exit 1
    }

    Write-Host "Prerequisites check passed" -ForegroundColor Green

    # Check ports availability
    Write-Host "Checking port availability..." -ForegroundColor Blue
    if (!(Test-Port $RustServerPort)) { exit 1 }
    if (!(Test-Port $NodejsPort)) { exit 1 }
    if (!(Test-Port $WebSocketPort)) { exit 1 }
    Write-Host "All ports are available" -ForegroundColor Green

    # Create logs directory
    if (!(Test-Path $LogsDir)) {
        New-Item -ItemType Directory -Path $LogsDir | Out-Null
    }

    # Install Node.js dependencies if needed
    Write-Host "Checking Node.js dependencies..." -ForegroundColor Blue
    Set-Location $NodejsDir

    if (!(Test-Path "node_modules") -or !(Test-Path "package-lock.json")) {
        Write-Host "Installing Node.js dependencies..." -ForegroundColor Yellow
        npm install
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Error: Failed to install Node.js dependencies" -ForegroundColor Red
            exit 1
        }
        Write-Host "Node.js dependencies installed" -ForegroundColor Green
    }
    else {
        Write-Host "Node.js dependencies already installed" -ForegroundColor Green
    }

    # Start Rust MCP server
    Write-Host "Starting Rust MCP server on port $RustServerPort..." -ForegroundColor Blue
    Set-Location $ProjectRoot

    # Set environment variables
    $env:SERVER_IP = "127.0.0.1"
    $env:SERVER_PORT = $RustServerPort.ToString()
    $env:GDB_COMMAND_TIMEOUT = "30"
    $env:RUST_LOG = "info"

    $rustLogFile = Join-Path $LogsDir "rust-server.log"
    $rustErrorFile = Join-Path $LogsDir "rust-server-error.log"
    $global:RustProcess = Start-Process -FilePath $RustBinary -ArgumentList "--log-level", "info", "sse" -RedirectStandardOutput $rustLogFile -RedirectStandardError $rustErrorFile -PassThru

    # Wait a moment for the server to start
    Start-Sleep -Seconds 3

    # Check if Rust server is running
    if ($global:RustProcess.HasExited) {
        Write-Host "Error: Failed to start Rust MCP server" -ForegroundColor Red
        Write-Host "Check $rustLogFile for details" -ForegroundColor Yellow
        Get-Content $rustLogFile | Write-Host
        exit 1
    }

    Write-Host "Rust MCP server started (PID: $($global:RustProcess.Id))" -ForegroundColor Green

    # Start Node.js bridge
    Write-Host "Starting Node.js bridge on port $NodejsPort..." -ForegroundColor Blue
    Set-Location $NodejsDir

    $env:PORT = $NodejsPort.ToString()
    $env:HOST = "127.0.0.1"

    $nodejsLogFile = Join-Path $LogsDir "nodejs-bridge.log"
    $nodejsErrorFile = Join-Path $LogsDir "nodejs-bridge-error.log"
    $global:NodejsProcess = Start-Process -FilePath "cmd" -ArgumentList "/c", "npm", "start" -RedirectStandardOutput $nodejsLogFile -RedirectStandardError $nodejsErrorFile -PassThru

    # Wait a moment for the Node.js server to start
    Start-Sleep -Seconds 5

    # Check if Node.js server is running
    if ($global:NodejsProcess.HasExited) {
        Write-Host "Error: Failed to start Node.js bridge" -ForegroundColor Red
        Write-Host "Check $nodejsLogFile for details" -ForegroundColor Yellow
        Get-Content $nodejsLogFile | Write-Host
        Stop-Servers
        exit 1
    }

    Write-Host "Node.js bridge started (PID: $($global:NodejsProcess.Id))" -ForegroundColor Green

    # Display status
    Write-Host ""
    Write-Host "=== Servers Started Successfully ===" -ForegroundColor Green
    Write-Host "Rust MCP Server:    http://127.0.0.1:$RustServerPort" -ForegroundColor Green
    Write-Host "Node.js Dashboard:  http://127.0.0.1:$NodejsPort" -ForegroundColor Green
    Write-Host "WebSocket Server:   ws://127.0.0.1:$WebSocketPort" -ForegroundColor Green
    Write-Host ""
    Write-Host "Dashboard URL: http://127.0.0.1:$NodejsPort" -ForegroundColor Blue
    Write-Host ""
    Write-Host "Press Ctrl+C to stop all servers" -ForegroundColor Yellow

    # Wait for processes
    while (!$global:RustProcess.HasExited -and !$global:NodejsProcess.HasExited) {
        Start-Sleep -Seconds 1
    }

    if ($global:RustProcess.HasExited) {
        Write-Host "Rust server exited unexpectedly" -ForegroundColor Red
        Get-Content $rustLogFile | Write-Host
    }

    if ($global:NodejsProcess.HasExited) {
        Write-Host "Node.js server exited unexpectedly" -ForegroundColor Red
        Get-Content $nodejsLogFile | Write-Host
    }
}
finally {
    Stop-Servers
}
