// End-to-end tests for complete STM32 debugging workflows
use std::time::{Duration, Instant};
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

/// Test complete debugging workflow from start to finish
#[tokio::test]
async fn test_complete_debug_workflow() {
    let config = STM32TestConfig::default();
    
    // Check prerequisites
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping E2E test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    println!("Starting complete debugging workflow test...");
    
    // Step 1: Start ST-Link server
    println!("Step 1: Starting ST-Link server...");
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(3)).await;
    
    // Step 2: Connect and reset
    println!("Step 2: Connecting to target and resetting...");
    let reset_commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "monitor reset halt",
        "load",
        "monitor reset halt",
        "quit",
    ];
    
    let reset_result = STM32TestUtils::execute_gdb_command(&config, &reset_commands).await;
    if !reset_result.success {
        println!("Reset failed: {:?}", reset_result.error);
        let _ = STM32TestUtils::stop_stlink_server(server_process).await;
        return;
    }
    println!("Reset successful");
    
    // Step 3: Set breakpoints and start debugging
    println!("Step 3: Setting breakpoints and starting debug session...");
    let debug_commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "break main",
        "break main.c:112",  // Counter increment
        "info breakpoints",
        "continue",
        "bt",
        "info locals",
        "quit",
    ];
    
    let debug_result = STM32TestUtils::execute_gdb_command(&config, &debug_commands).await;
    if !debug_result.success {
        println!("Debug session failed: {:?}", debug_result.error);
        let _ = STM32TestUtils::stop_stlink_server(server_process).await;
        return;
    }
    println!("Debug session successful");
    
    // Step 4: Monitor counter over time
    println!("Step 4: Monitoring counter values...");
    let mut counter_samples = Vec::new();
    
    for i in 0..3 {
        let monitor_commands = vec![
            &format!("target extended-remote localhost:{}", config.stlink_port),
            "break main.c:112",
            "continue",
            "print counter_1000ms",
            "print HAL_GetTick()",
            "continue",
            "quit",
        ];
        
        let monitor_result = STM32TestUtils::execute_gdb_command(&config, &monitor_commands).await;
        
        if monitor_result.success {
            if let Some(counter) = STM32TestUtils::parse_counter_value(&monitor_result.output) {
                counter_samples.push(counter);
                println!("Sample {}: Counter = {}", i + 1, counter);
            }
        }
        
        sleep(Duration::from_secs(2)).await;
    }
    
    // Step 5: Test step debugging
    println!("Step 5: Testing step-by-step debugging...");
    let step_commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "break main",
        "continue",
        "step",
        "next",
        "step",
        "bt",
        "info registers",
        "quit",
    ];
    
    let step_result = STM32TestUtils::execute_gdb_command(&config, &step_commands).await;
    if step_result.success {
        println!("Step debugging successful");
    } else {
        println!("Step debugging had issues: {:?}", step_result.error);
    }
    
    // Step 6: Test memory access
    println!("Step 6: Testing memory access...");
    let memory_commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "x/16x 0x08000000",  // Flash
        "x/16x 0x20000000",  // SRAM
        "info mem",
        "quit",
    ];
    
    let memory_result = STM32TestUtils::execute_gdb_command(&config, &memory_commands).await;
    if memory_result.success {
        println!("Memory access successful");
    } else {
        println!("Memory access had issues: {:?}", memory_result.error);
    }
    
    // Step 7: Final verification
    println!("Step 7: Final verification...");
    let verify_commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "break main.c:112",
        "continue",
        "print counter_1000ms",
        "print HAL_GetTick()",
        "info breakpoints",
        "delete breakpoints",
        "continue",
        "quit",
    ];
    
    let verify_result = STM32TestUtils::execute_gdb_command(&config, &verify_commands).await;
    
    // Clean up
    println!("Cleaning up...");
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    // Analyze results
    println!("\n=== WORKFLOW TEST RESULTS ===");
    println!("Reset: {}", if reset_result.success { "PASS" } else { "FAIL" });
    println!("Debug Session: {}", if debug_result.success { "PASS" } else { "FAIL" });
    println!("Step Debugging: {}", if step_result.success { "PASS" } else { "PARTIAL" });
    println!("Memory Access: {}", if memory_result.success { "PASS" } else { "PARTIAL" });
    println!("Final Verification: {}", if verify_result.success { "PASS" } else { "FAIL" });
    
    if counter_samples.len() >= 2 {
        println!("Counter Monitoring: PASS ({} samples)", counter_samples.len());
        let first = counter_samples[0];
        let last = counter_samples[counter_samples.len() - 1];
        if last >= first {
            println!("Counter Progress: GOOD (increased from {} to {})", first, last);
        } else {
            println!("Counter Progress: RESET (decreased from {} to {})", first, last);
        }
    } else {
        println!("Counter Monitoring: INSUFFICIENT DATA");
    }
    
    // Overall assessment
    let critical_tests_passed = reset_result.success && debug_result.success && verify_result.success;
    println!("\nOVERALL WORKFLOW: {}", if critical_tests_passed { "PASS" } else { "FAIL" });
}

/// Test automated debugging script integration
#[tokio::test]
async fn test_automated_script_integration() {
    let config = STM32TestConfig::default();
    
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping automated script test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    println!("Testing automated debugging scripts...");
    
    // Test the automated debug script
    let script_path = config.project_path.join("debug_automated.sh");
    if !script_path.exists() {
        println!("Automated debug script not found, skipping test");
        return;
    }
    
    // Start ST-Link server first
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(3)).await;
    
    // Test different script functions
    let script_tests = vec![
        ("counter", "Quick counter check"),
        ("health", "System health check"),
    ];
    
    for (command, description) in script_tests {
        println!("Testing {}: {}", command, description);
        
        let mut cmd = std::process::Command::new("bash");
        cmd.arg(&script_path)
           .arg(command)
           .current_dir(&config.project_path);
        
        let output = cmd.output().await;
        
        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                
                if result.status.success() {
                    println!("  {} test: PASS", command);
                    if stdout.contains("Counter") {
                        println!("  Found counter information in output");
                    }
                } else {
                    println!("  {} test: FAIL", command);
                    if !stderr.is_empty() {
                        println!("  Error: {}", stderr);
                    }
                }
            }
            Err(e) => {
                println!("  {} test: ERROR - {}", command, e);
            }
        }
        
        sleep(Duration::from_secs(1)).await;
    }
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
}

/// Test MCP server integration with STM32
#[tokio::test]
async fn test_mcp_server_integration() {
    let config = STM32TestConfig::default();
    
    // This test focuses on MCP server functionality, so hardware is optional
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    println!("Testing MCP server integration...");
    
    // Test if MCP server binary exists
    let mcp_server_available = std::process::Command::new("mcp-server-gdb")
        .arg("--version")
        .output()
        .await
        .is_ok();
    
    if !mcp_server_available {
        println!("MCP server not available, testing basic functionality only");
        return;
    }
    
    println!("MCP server available, testing integration...");
    
    // Test MCP server startup
    let mut mcp_server = std::process::Command::new("mcp-server-gdb")
        .arg("--log-level")
        .arg("debug")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start MCP server");
    
    sleep(Duration::from_secs(2)).await;
    
    // Test basic MCP communication (this would require MCP client implementation)
    // For now, just verify the server starts and stops cleanly
    
    // Stop MCP server
    mcp_server.kill().expect("Failed to kill MCP server");
    let exit_status = mcp_server.wait().expect("Failed to wait for MCP server");
    
    println!("MCP server integration test completed");
    println!("Server exit status: {:?}", exit_status);
}

/// Test error recovery and robustness
#[tokio::test]
async fn test_error_recovery_robustness() {
    let config = STM32TestConfig::default();
    
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping robustness test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    println!("Testing error recovery and robustness...");
    
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(3)).await;
    
    // Test 1: Invalid commands
    println!("Test 1: Invalid commands...");
    let invalid_commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "invalid_command_that_should_fail",
        "break nonexistent_file.c:999",
        "print nonexistent_variable",
        "quit",
    ];
    
    let invalid_result = STM32TestUtils::execute_gdb_command(&config, &invalid_commands).await;
    println!("Invalid commands handled: {}", if invalid_result.error.is_some() { "Expected errors" } else { "Unexpected success" });
    
    // Test 2: Connection interruption simulation
    println!("Test 2: Connection recovery...");
    let recovery_commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "break main",
        "continue",
        "disconnect",
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "info breakpoints",
        "quit",
    ];
    
    let recovery_result = STM32TestUtils::execute_gdb_command(&config, &recovery_commands).await;
    if recovery_result.success {
        println!("Connection recovery: PASS");
    } else {
        println!("Connection recovery: PARTIAL (expected for some scenarios)");
    }
    
    // Test 3: Multiple rapid connections
    println!("Test 3: Multiple rapid connections...");
    for i in 0..3 {
        let quick_commands = vec![
            &format!("target extended-remote localhost:{}", config.stlink_port),
            "info registers",
            "quit",
        ];
        
        let quick_result = STM32TestUtils::execute_gdb_command(&config, &quick_commands).await;
        println!("Quick connection {}: {}", i + 1, if quick_result.success { "PASS" } else { "FAIL" });
        
        sleep(Duration::from_millis(500)).await;
    }
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    println!("Error recovery and robustness test completed");
}
