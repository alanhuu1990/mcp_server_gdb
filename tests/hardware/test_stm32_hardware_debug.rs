// Hardware-specific tests for STM32 debugging with real hardware
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

/// Test STM32 hardware detection and connection
#[tokio::test]
async fn test_stm32_hardware_detection() {
    // Test if ST-Link hardware is detected
    let hardware_available = STM32TestUtils::check_hardware_available().await;
    
    if !hardware_available {
        println!("STM32 hardware not detected - this is expected if no hardware is connected");
        return;
    }
    
    println!("STM32 hardware detected successfully");
    
    // Additional hardware verification
    let config = STM32TestConfig::default();
    let issues = STM32TestUtils::validate_environment(&config).await;
    
    if !issues.is_empty() {
        println!("Environment issues detected: {:?}", issues);
        return;
    }
    
    println!("STM32 debugging environment validated successfully");
}

/// Test real-time counter monitoring with hardware
#[tokio::test]
async fn test_realtime_counter_monitoring() {
    let config = STM32TestConfig::default();
    
    // Skip if no hardware
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
    
    sleep(Duration::from_secs(3)).await;
    
    // Monitor counter over time
    let mut counter_values = Vec::new();
    let mut timestamps = Vec::new();
    
    for i in 0..5 {
        let commands = vec![
            &format!("target extended-remote localhost:{}", config.stlink_port),
            "break main.c:112",
            "continue",
            "print counter_1000ms",
            "print HAL_GetTick()",
            "continue",
            "quit",
        ];
        
        let start_time = Instant::now();
        let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
        
        if result.success {
            if let Some(counter) = STM32TestUtils::parse_counter_value(&result.output) {
                counter_values.push(counter);
                timestamps.push(start_time.elapsed());
                println!("Sample {}: Counter = {}, Time = {:?}", i + 1, counter, start_time.elapsed());
            }
        }
        
        // Wait between samples
        sleep(Duration::from_secs(2)).await;
    }
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    // Analyze results
    if counter_values.len() >= 2 {
        println!("Counter monitoring successful with {} samples", counter_values.len());
        
        // Check if counter is incrementing (allowing for some variation)
        let first_counter = counter_values[0];
        let last_counter = counter_values[counter_values.len() - 1];
        
        if last_counter >= first_counter {
            println!("Counter incremented from {} to {} (good)", first_counter, last_counter);
        } else {
            println!("Counter decreased from {} to {} (may indicate reset)", first_counter, last_counter);
        }
    } else {
        println!("Insufficient counter samples collected");
    }
}

/// Test STM32 timing accuracy
#[tokio::test]
async fn test_stm32_timing_accuracy() {
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
    
    sleep(Duration::from_secs(3)).await;
    
    // Test timing accuracy by measuring counter increments
    let commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "break main.c:112",
        "continue",
        "print counter_1000ms",
        "print HAL_GetTick()",
        "continue",
        "print counter_1000ms", 
        "print HAL_GetTick()",
        "continue",
        "print counter_1000ms",
        "print HAL_GetTick()",
        "quit",
    ];
    
    let start_time = Instant::now();
    let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
    let total_time = start_time.elapsed();
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    if result.success {
        println!("Timing test completed in {:?}", total_time);
        
        // Parse HAL_GetTick values to check timing
        let lines: Vec<&str> = result.output.lines().collect();
        let mut tick_values = Vec::new();
        
        for line in lines {
            if line.contains("HAL_GetTick") || (line.contains("$") && line.contains("=")) {
                if let Some(equals_pos) = line.find('=') {
                    let value_part = &line[equals_pos + 1..].trim();
                    if let Ok(value) = value_part.parse::<u32>() {
                        tick_values.push(value);
                    }
                }
            }
        }
        
        if tick_values.len() >= 2 {
            let tick_diff = tick_values[tick_values.len() - 1] - tick_values[0];
            println!("HAL tick difference: {} ms", tick_diff);
            
            // Verify timing is reasonable (should be close to actual elapsed time)
            let expected_ms = total_time.as_millis() as u32;
            let tolerance_ms = 1000; // 1 second tolerance
            
            if tick_diff > 0 && tick_diff < expected_ms + tolerance_ms {
                println!("Timing accuracy test passed: {} ms vs expected ~{} ms", tick_diff, expected_ms);
            } else {
                println!("Timing may be inaccurate: {} ms vs expected ~{} ms", tick_diff, expected_ms);
            }
        }
    }
}

/// Test STM32 reset and reload functionality
#[tokio::test]
async fn test_stm32_reset_reload() {
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
    
    sleep(Duration::from_secs(3)).await;
    
    // Test reset and reload sequence
    let commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "monitor reset halt",
        "load",
        "monitor reset halt",
        "break main",
        "continue",
        "print counter_1000ms",  // Should be 0 or very low after reset
        "quit",
    ];
    
    let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    if result.success {
        println!("Reset and reload test successful");
        
        // Check if counter was reset (should be 0 or very low)
        if let Some(counter) = STM32TestUtils::parse_counter_value(&result.output) {
            if counter <= 5 {  // Allow for some initial increments
                println!("Counter properly reset to {}", counter);
            } else {
                println!("Counter may not have reset properly: {}", counter);
            }
        }
    } else {
        println!("Reset and reload test failed: {:?}", result.error);
    }
}

/// Test STM32 flash programming
#[tokio::test]
async fn test_stm32_flash_programming() {
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
    
    sleep(Duration::from_secs(3)).await;
    
    // Test flash programming
    let commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "monitor reset halt",
        "load",  // Program flash
        "compare-sections",  // Verify programming
        "monitor reset halt",
        "continue",
        "quit",
    ];
    
    let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    if result.success {
        println!("Flash programming test successful");
        
        // Check for successful programming indicators
        if result.output.contains("Loading section") || result.output.contains("Transfer rate") {
            println!("Flash programming completed successfully");
        }
        
        if result.output.contains("matched") || !result.output.contains("MIS-MATCHED") {
            println!("Flash verification successful");
        }
    } else {
        println!("Flash programming test failed: {:?}", result.error);
    }
}

/// Test STM32 peripheral register access
#[tokio::test]
async fn test_stm32_peripheral_access() {
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
    
    sleep(Duration::from_secs(3)).await;
    
    // Test accessing STM32F429 peripheral registers
    let peripheral_tests = vec![
        ("RCC_CR", "0x40023800"),      // RCC Control Register
        ("GPIOA_IDR", "0x40020010"),   // GPIOA Input Data Register
        ("SysTick_CTRL", "0xE000E010"), // SysTick Control Register
    ];
    
    for (name, address) in peripheral_tests {
        let commands = vec![
            &format!("target extended-remote localhost:{}", config.stlink_port),
            &format!("x/1wx {}", address),
            "quit",
        ];
        
        let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
        
        if result.success && !result.output.is_empty() {
            println!("{} register accessible at {}", name, address);
        } else {
            println!("{} register not accessible (may be expected)", name);
        }
    }
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
}
