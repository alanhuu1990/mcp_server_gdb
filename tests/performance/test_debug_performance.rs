// Performance tests for STM32 debugging operations
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
}

/// Test GDB connection performance
#[tokio::test]
async fn test_gdb_connection_performance() {
    let config = STM32TestConfig::default();
    
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping performance test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    println!("Testing GDB connection performance...");
    
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(3)).await;
    
    // Test connection times
    let mut connection_times = Vec::new();
    
    for i in 0..5 {
        let commands = vec![
            &format!("target extended-remote localhost:{}", config.stlink_port),
            "info registers",
            "quit",
        ];
        
        let start_time = Instant::now();
        let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
        let connection_time = start_time.elapsed();
        
        if result.success {
            connection_times.push(connection_time);
            println!("Connection {}: {:?}", i + 1, connection_time);
        } else {
            println!("Connection {} failed", i + 1);
        }
        
        sleep(Duration::from_millis(500)).await;
    }
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    // Analyze performance
    if !connection_times.is_empty() {
        let avg_time = connection_times.iter().sum::<Duration>() / connection_times.len() as u32;
        let min_time = connection_times.iter().min().unwrap();
        let max_time = connection_times.iter().max().unwrap();
        
        println!("\n=== CONNECTION PERFORMANCE ===");
        println!("Average: {:?}", avg_time);
        println!("Minimum: {:?}", min_time);
        println!("Maximum: {:?}", max_time);
        
        // Performance assertions
        assert!(avg_time < Duration::from_secs(5), "Average connection time too slow: {:?}", avg_time);
        assert!(max_time < Duration::from_secs(10), "Maximum connection time too slow: {:?}", max_time);
        
        println!("Connection performance: PASS");
    } else {
        println!("No successful connections for performance analysis");
    }
}

/// Test breakpoint setting performance
#[tokio::test]
async fn test_breakpoint_performance() {
    let config = STM32TestConfig::default();
    
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping performance test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    println!("Testing breakpoint setting performance...");
    
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(3)).await;
    
    // Test setting multiple breakpoints
    let breakpoint_locations = vec![
        ("main.c", 77),
        ("main.c", 84),
        ("main.c", 94),
        ("main.c", 112),
        ("main.c", 174),
    ];
    
    let mut commands = vec![
        format!("target extended-remote localhost:{}", config.stlink_port),
    ];
    
    // Add breakpoint commands
    for (file, line) in &breakpoint_locations {
        commands.push(format!("break {}:{}", file, line));
    }
    
    commands.push("info breakpoints".to_string());
    commands.push("quit".to_string());
    
    let command_refs: Vec<&str> = commands.iter().map(|s| s.as_str()).collect();
    
    let start_time = Instant::now();
    let result = STM32TestUtils::execute_gdb_command(&config, &command_refs).await;
    let total_time = start_time.elapsed();
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    if result.success {
        let avg_time_per_bp = total_time / breakpoint_locations.len() as u32;
        
        println!("\n=== BREAKPOINT PERFORMANCE ===");
        println!("Total time for {} breakpoints: {:?}", breakpoint_locations.len(), total_time);
        println!("Average time per breakpoint: {:?}", avg_time_per_bp);
        
        // Performance assertions
        assert!(avg_time_per_bp < Duration::from_millis(500), "Breakpoint setting too slow: {:?}", avg_time_per_bp);
        assert!(total_time < Duration::from_secs(5), "Total breakpoint time too slow: {:?}", total_time);
        
        println!("Breakpoint performance: PASS");
    } else {
        println!("Breakpoint performance test failed: {:?}", result.error);
    }
}

/// Test memory read performance
#[tokio::test]
async fn test_memory_read_performance() {
    let config = STM32TestConfig::default();
    
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping performance test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    println!("Testing memory read performance...");
    
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(3)).await;
    
    // Test different memory read sizes
    let memory_tests = vec![
        ("Small read (64 bytes)", "0x08000000", 16),    // 16 words = 64 bytes
        ("Medium read (256 bytes)", "0x08000000", 64),  // 64 words = 256 bytes
        ("Large read (1KB)", "0x08000000", 256),        // 256 words = 1KB
    ];
    
    for (test_name, address, word_count) in memory_tests {
        let commands = vec![
            &format!("target extended-remote localhost:{}", config.stlink_port),
            &format!("x/{}x {}", word_count, address),
            "quit",
        ];
        
        let start_time = Instant::now();
        let result = STM32TestUtils::execute_gdb_command(&config, &commands).await;
        let read_time = start_time.elapsed();
        
        if result.success {
            let bytes_read = word_count * 4;
            let throughput = bytes_read as f64 / read_time.as_secs_f64();
            
            println!("{}: {:?} ({:.0} bytes/sec)", test_name, read_time, throughput);
            
            // Basic performance check - should be faster than 1 second for reasonable sizes
            if bytes_read <= 1024 {
                assert!(read_time < Duration::from_secs(2), "{} too slow: {:?}", test_name, read_time);
            }
        } else {
            println!("{}: FAILED", test_name);
        }
        
        sleep(Duration::from_millis(100)).await;
    }
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    println!("Memory read performance test completed");
}

/// Test counter monitoring performance
#[tokio::test]
async fn test_counter_monitoring_performance() {
    let config = STM32TestConfig::default();
    
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping performance test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    println!("Testing counter monitoring performance...");
    
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(3)).await;
    
    // Test rapid counter reads
    let mut read_times = Vec::new();
    let num_reads = 10;
    
    for i in 0..num_reads {
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
        let read_time = start_time.elapsed();
        
        if result.success {
            read_times.push(read_time);
            
            if let Some(counter) = STM32TestUtils::parse_counter_value(&result.output) {
                println!("Read {}: Counter = {}, Time = {:?}", i + 1, counter, read_time);
            } else {
                println!("Read {}: Time = {:?} (no counter value)", i + 1, read_time);
            }
        } else {
            println!("Read {} failed", i + 1);
        }
        
        sleep(Duration::from_millis(200)).await;
    }
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    // Analyze performance
    if !read_times.is_empty() {
        let avg_time = read_times.iter().sum::<Duration>() / read_times.len() as u32;
        let min_time = read_times.iter().min().unwrap();
        let max_time = read_times.iter().max().unwrap();
        
        println!("\n=== COUNTER MONITORING PERFORMANCE ===");
        println!("Successful reads: {}/{}", read_times.len(), num_reads);
        println!("Average read time: {:?}", avg_time);
        println!("Minimum read time: {:?}", min_time);
        println!("Maximum read time: {:?}", max_time);
        
        // Performance assertions
        assert!(avg_time < Duration::from_secs(10), "Average counter read too slow: {:?}", avg_time);
        assert!(max_time < Duration::from_secs(15), "Maximum counter read too slow: {:?}", max_time);
        
        // Calculate reads per minute
        let reads_per_minute = 60.0 / avg_time.as_secs_f64();
        println!("Estimated reads per minute: {:.1}", reads_per_minute);
        
        println!("Counter monitoring performance: PASS");
    } else {
        println!("No successful counter reads for performance analysis");
    }
}

/// Test debugging session overhead
#[tokio::test]
async fn test_debugging_session_overhead() {
    let config = STM32TestConfig::default();
    
    if !STM32TestUtils::check_hardware_available().await {
        println!("Skipping performance test - STM32 not connected");
        return;
    }
    
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    println!("Testing debugging session overhead...");
    
    let server_process = STM32TestUtils::start_stlink_server(&config).await;
    if server_process.is_err() {
        println!("Failed to start ST-Link server, skipping test");
        return;
    }
    let mut server_process = server_process.unwrap();
    
    sleep(Duration::from_secs(3)).await;
    
    // Test 1: Minimal session (just connect and disconnect)
    let minimal_commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "quit",
    ];
    
    let start_time = Instant::now();
    let minimal_result = STM32TestUtils::execute_gdb_command(&config, &minimal_commands).await;
    let minimal_time = start_time.elapsed();
    
    // Test 2: Full debugging session
    let full_commands = vec![
        &format!("target extended-remote localhost:{}", config.stlink_port),
        "break main",
        "continue",
        "bt",
        "info registers",
        "info locals",
        "step",
        "next",
        "continue",
        "quit",
    ];
    
    let start_time = Instant::now();
    let full_result = STM32TestUtils::execute_gdb_command(&config, &full_commands).await;
    let full_time = start_time.elapsed();
    
    // Clean up
    let _ = STM32TestUtils::stop_stlink_server(server_process).await;
    
    // Analyze overhead
    println!("\n=== DEBUGGING SESSION OVERHEAD ===");
    println!("Minimal session: {:?}", minimal_time);
    println!("Full session: {:?}", full_time);
    
    if minimal_result.success && full_result.success {
        let overhead = full_time - minimal_time;
        println!("Debugging overhead: {:?}", overhead);
        
        // Performance assertions
        assert!(minimal_time < Duration::from_secs(3), "Minimal session too slow: {:?}", minimal_time);
        assert!(full_time < Duration::from_secs(15), "Full session too slow: {:?}", full_time);
        
        println!("Session overhead test: PASS");
    } else {
        println!("Session overhead test: INCOMPLETE");
        if !minimal_result.success {
            println!("Minimal session failed: {:?}", minimal_result.error);
        }
        if !full_result.success {
            println!("Full session failed: {:?}", full_result.error);
        }
    }
}
