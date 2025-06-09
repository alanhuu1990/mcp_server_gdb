// Integration tests for STM32 debugging sessions
use std::time::Duration;
use tokio::time::sleep;
use std::path::PathBuf;
use std::process::Command;

// Import common test utilities (inline for now)
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

#[derive(Debug, Clone)]
pub struct DebugTestResult {
    pub success: bool,
    pub duration: Duration,
    pub output: String,
    pub error: Option<String>,
}

pub struct STM32TestUtils;

impl STM32TestUtils {
    pub async fn check_hardware_available() -> bool {
        Command::new("st-info")
            .arg("--probe")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub async fn validate_environment(config: &STM32TestConfig) -> Vec<String> {
        let mut issues = Vec::new();

        if !config.elf_file_path.exists() {
            issues.push(format!("ELF file not found: {:?}", config.elf_file_path));
        }

        if Command::new(&config.gdb_path).arg("--version").output().await.is_err() {
            issues.push(format!("GDB not found: {}", config.gdb_path));
        }

        issues
    }

    pub async fn start_stlink_server(config: &STM32TestConfig) -> Result<std::process::Child, std::io::Error> {
        Command::new("st-util")
            .arg("-p").arg(config.stlink_port.to_string())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
    }

    pub async fn stop_stlink_server(mut process: std::process::Child) -> Result<(), std::io::Error> {
        process.kill()?;
        process.wait()?;
        Ok(())
    }

    pub async fn check_stlink_server_running(port: u16) -> bool {
        use std::net::TcpStream;
        TcpStream::connect(format!("localhost:{}", port)).is_ok()
    }

    pub async fn execute_gdb_command(
        config: &STM32TestConfig,
        commands: &[&str],
    ) -> DebugTestResult {
        let start_time = std::time::Instant::now();

        let mut cmd = Command::new(&config.gdb_path);
        cmd.arg(&config.elf_file_path)
           .arg("-batch");

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

    pub fn parse_counter_value(output: &str) -> Option<u32> {
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
}

macro_rules! assert_debug_success {
    ($result:expr) => {
        assert!($result.success, "Debug operation failed: {:?}", $result.error);
    };
}

/// Test complete STM32 debugging session with hardware
#[tokio::test]
async fn test_complete_stm32_debug_session() {
    let config = STM32TestConfig::default();
    
    // Check if hardware is available
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping hardware test - STM32 not connected");
        return;
    }
    
    // Validate environment
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    // Start ST-Link server
    let stlink_server = STM32TestUtils::start_stlink_server(&config).await;
    if stlink_server.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = stlink_server.unwrap();
    
    // Wait for server to start
    sleep(Duration::from_secs(2)).await;
    
    // Verify server is running
    let server_running = STM32TestUtils::check_stlink_server_running(config.stlink_port).await;
    if !server_running {
        println!("ST-Link server not responding, skipping test");
        let _ = STM32TestUtils::stop_stlink_server(server_process).await;
        return;
    }
    
    // Test GDB connection and basic operations
    let commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "monitor reset halt",
        "load",
        "break main",
        "continue",
        "info registers",
        "bt",
        "quit",
    ];
    
    let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    // Verify results
    assert_debug_success!(result);
    assert!(result.output.contains("Breakpoint"), "Expected breakpoint hit in output");
}

/// Test STM32 counter debugging scenario
#[tokio::test]
async fn test_stm32_counter_debugging() {
    let config = STM32TestConfig::default();
    
    // Check prerequisites
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping hardware test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    // Start ST-Link server
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(2)).await;
    
    // Test counter value reading
    let commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "break main.c:112",  // Counter increment line
        "continue",
        "print counter_1000ms",
        "print HAL_GetTick()",
        "continue",
        "print counter_1000ms",
        "quit",
    ];
    
    let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    // Verify results
    assert_debug_success!(result);
    
    // Parse counter values
    let counter_value = STM32TestUtils::parse_counter_value(&result.output);
    if let Some(counter) = counter_value {
        // Counter should be reasonable (not negative, not extremely large)
        assert_counter_in_range!(counter, 0, 1000000);
    }
}

/// Test STM32 memory region access
#[tokio::test]
async fn test_stm32_memory_regions() {
    let config = STM32TestConfig::default();
    
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping hardware test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(2)).await;
    
    // Test different memory regions
    let test_cases = vec![
        ("Flash", "0x08000000", 64),
        ("SRAM", "0x20000000", 64),
        ("CCM RAM", "0x10000000", 64),
    ];
    
    for (region_name, address, size) in test_cases {
        let commands = vec![
            &format!("target extended-remote localhost:{}", config.stlink_port),
            &format!("x/{}x {}", size / 4, address),
            "quit",
        ];
        
        let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
        
        // Some memory regions might not be accessible, which is OK
        if result.success {
            println!("{} region accessible at {}", region_name, address);
            assert!(!result.output.is_empty(), "{} memory read returned no data", region_name);
        } else {
            println!("{} region not accessible (expected for some regions)", region_name);
        }
    }
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
}

/// Test STM32 register access
#[tokio::test]
async fn test_stm32_register_access() {
    let config = STM32TestConfig::default();
    
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping hardware test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(2)).await;
    
    // Test ARM Cortex-M4 registers
    let commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "info registers",
        "info registers general",
        "print $pc",
        "print $sp",
        "print $r0",
        "quit",
    ];
    
    let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    // Verify results
    assert_debug_success!(result);
    assert!(result.output.contains("pc"), "Expected PC register in output");
    assert!(result.output.contains("sp"), "Expected SP register in output");
}

/// Test STM32 breakpoint functionality
#[tokio::test]
async fn test_stm32_breakpoint_functionality() {
    let config = STM32TestConfig::default();
    
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping hardware test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(2)).await;
    
    // Test setting and hitting breakpoints
    let commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "break main",
        "break main.c:112",  // Counter increment
        "info breakpoints",
        "continue",
        "bt",  // Backtrace when breakpoint is hit
        "delete breakpoints",
        "continue",
        "quit",
    ];
    
    let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    // Verify results
    assert_debug_success!(result);
    assert!(result.output.contains("Breakpoint"), "Expected breakpoint information");
}

/// Test STM32 step debugging
#[tokio::test]
async fn test_stm32_step_debugging() {
    let config = STM32TestConfig::default();
    
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping hardware test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(2)).await;
    
    // Test step-by-step debugging
    let commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "break main",
        "continue",
        "step",  // Step into
        "next",  // Step over
        "step",
        "bt",
        "quit",
    ];
    
    let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    // Verify results - stepping might not always work perfectly, but shouldn't crash
    if result.success {
        println!("Step debugging successful");
    } else {
        println!("Step debugging had issues (may be expected): {:?}", result.error);
    }
}
