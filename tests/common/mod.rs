// Common test utilities for STM32 GDB debugging tests
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test configuration for STM32 debugging
#[derive(Debug, Clone)]
pub struct STM32TestConfig {
    pub workspace_path: PathBuf,
    pub project_path: PathBuf,
    pub elf_file_path: PathBuf,
    pub gdb_path: String,
    pub stlink_port: u16,
    pub timeout_seconds: u64,
}

impl Default for STM32TestConfig {
    fn default() -> Self {
        let workspace = PathBuf::from("tests/stm32-f0-disco");
        let project = workspace.join("stm32-f429");
        
        Self {
            workspace_path: workspace.clone(),
            project_path: project.clone(),
            elf_file_path: project.join("Debug/stm32-f429.elf"),
            gdb_path: "arm-none-eabi-gdb".to_string(),
            stlink_port: 4242,
            timeout_seconds: 30,
        }
    }
}

/// Test result for debugging operations
#[derive(Debug, Clone)]
pub struct DebugTestResult {
    pub success: bool,
    pub duration: Duration,
    pub output: String,
    pub error: Option<String>,
}

/// STM32 test utilities
pub struct STM32TestUtils;

impl STM32TestUtils {
    /// Check if STM32 hardware is connected and accessible
    pub async fn check_hardware_available() -> bool {
        let output = Command::new("st-info")
            .arg("--probe")
            .output()
            .await;
            
        match output {
            Ok(result) => result.status.success(),
            Err(_) => false,
        }
    }
    
    /// Check if ST-Link GDB server is running
    pub async fn check_stlink_server_running(port: u16) -> bool {
        use std::net::TcpStream;
        TcpStream::connect(format!("localhost:{}", port)).is_ok()
    }
    
    /// Start ST-Link GDB server for testing
    pub async fn start_stlink_server(config: &STM32TestConfig) -> Result<std::process::Child, std::io::Error> {
        let mut cmd = Command::new("st-util");
        cmd.arg("-p").arg(config.stlink_port.to_string())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
           
        cmd.spawn()
    }
    
    /// Stop ST-Link GDB server
    pub async fn stop_stlink_server(mut process: std::process::Child) -> Result<(), std::io::Error> {
        process.kill()?;
        process.wait()?;
        Ok(())
    }
    
    /// Execute GDB command and return result
    pub async fn execute_gdb_command(
        config: &STM32TestConfig,
        commands: &[&str],
    ) -> DebugTestResult {
        let start_time = Instant::now();
        
        let mut cmd = Command::new(&config.gdb_path);
        cmd.arg(&config.elf_file_path)
           .arg("-batch");
           
        // Add each command as an argument
        for command in commands {
            cmd.arg("-ex").arg(command);
        }
        
        let output = cmd.output().await;
        let duration = start_time.elapsed();
        
        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout).to_string();
                let stderr = String::from_utf8_lossy(&result.stderr).to_string();
                
                DebugTestResult {
                    success: result.status.success(),
                    duration,
                    output: stdout,
                    error: if stderr.is_empty() { None } else { Some(stderr) },
                }
            }
            Err(e) => DebugTestResult {
                success: false,
                duration,
                output: String::new(),
                error: Some(e.to_string()),
            },
        }
    }
    
    /// Wait for condition with timeout
    pub async fn wait_for_condition<F, Fut>(
        condition: F,
        timeout: Duration,
        check_interval: Duration,
    ) -> bool
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = Instant::now();
        
        while start.elapsed() < timeout {
            if condition().await {
                return true;
            }
            sleep(check_interval).await;
        }
        
        false
    }
    
    /// Parse counter value from GDB output
    pub fn parse_counter_value(output: &str) -> Option<u32> {
        // Look for patterns like "$1 = 42" or "counter_1000ms = 42"
        for line in output.lines() {
            if line.contains("counter_1000ms") || line.contains("$") {
                if let Some(equals_pos) = line.find('=') {
                    let value_part = &line[equals_pos + 1..].trim();
                    if let Ok(value) = value_part.parse::<u32>() {
                        return Some(value);
                    }
                }
            }
        }
        None
    }
    
    /// Validate STM32 debugging environment
    pub async fn validate_environment(config: &STM32TestConfig) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check if ELF file exists
        if !config.elf_file_path.exists() {
            issues.push(format!("ELF file not found: {:?}", config.elf_file_path));
        }
        
        // Check if GDB is available
        if Command::new(&config.gdb_path).arg("--version").output().await.is_err() {
            issues.push(format!("GDB not found: {}", config.gdb_path));
        }
        
        // Check if st-util is available
        if Command::new("st-util").arg("--version").output().await.is_err() {
            issues.push("st-util not found (ST-Link utilities not installed)".to_string());
        }
        
        issues
    }
}

/// Test macros for common assertions
#[macro_export]
macro_rules! assert_debug_success {
    ($result:expr) => {
        assert!($result.success, "Debug operation failed: {:?}", $result.error);
    };
}

#[macro_export]
macro_rules! assert_counter_in_range {
    ($counter:expr, $min:expr, $max:expr) => {
        assert!(
            $counter >= $min && $counter <= $max,
            "Counter {} not in expected range [{}, {}]",
            $counter, $min, $max
        );
    };
}

#[macro_export]
macro_rules! assert_timing_within_tolerance {
    ($actual:expr, $expected:expr, $tolerance_ms:expr) => {
        let diff = if $actual > $expected { $actual - $expected } else { $expected - $actual };
        assert!(
            diff <= $tolerance_ms,
            "Timing difference {} ms exceeds tolerance {} ms",
            diff, $tolerance_ms
        );
    };
}
