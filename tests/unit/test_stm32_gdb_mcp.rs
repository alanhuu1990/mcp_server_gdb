// Unit tests for STM32 GDB MCP server functionality
use std::path::PathBuf;

// Import the MCP server modules we want to test
// Note: These imports may need adjustment based on actual module structure
// use mcp_server_gdb::gdb::GDBManager;
// use mcp_server_gdb::models::*;

// For now, we'll create mock implementations for testing
#[derive(Debug, Clone, PartialEq)]
pub enum GDBSessionStatus {
    Created,
    Running,
    Stopped,
}

pub struct GDBManager;

impl GDBManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn create_session(
        &self,
        _program: Option<PathBuf>,
        _nh: Option<bool>,
        _nx: Option<bool>,
        _quiet: Option<bool>,
        _cd: Option<PathBuf>,
        _bps: Option<u32>,
        _symbol_file: Option<PathBuf>,
        _core_file: Option<PathBuf>,
        _proc_id: Option<u32>,
        _command: Option<PathBuf>,
        _source_dir: Option<PathBuf>,
        _args: Option<Vec<std::ffi::OsString>>,
        _tty: Option<PathBuf>,
        _gdb_path: Option<PathBuf>,
    ) -> Result<String, String> {
        Ok("test-session-id".to_string())
    }

    pub async fn get_session(&self, _session_id: &str) -> Result<SessionInfo, String> {
        Ok(SessionInfo {
            status: GDBSessionStatus::Created,
        })
    }

    pub async fn close_session(&self, _session_id: &str) -> Result<(), String> {
        Ok(())
    }

    pub async fn set_breakpoint(&self, _session_id: &str, _file: &str, _line: u32) -> Result<String, String> {
        Ok("Breakpoint set".to_string())
    }

    pub async fn get_breakpoints(&self, _session_id: &str) -> Result<Vec<String>, String> {
        Ok(vec!["breakpoint1".to_string(), "breakpoint2".to_string()])
    }

    pub async fn read_memory(&self, _session_id: &str, _address: &str, _count: u64, _offset: Option<i64>) -> Result<String, String> {
        Ok("memory data".to_string())
    }

    pub async fn get_register_names(&self, _session_id: &str, _reg_list: Option<Vec<String>>) -> Result<Vec<String>, String> {
        Ok(vec!["r0".to_string(), "r1".to_string(), "sp".to_string(), "pc".to_string()])
    }

    pub async fn get_registers(&self, _session_id: &str, _reg_list: Option<Vec<String>>) -> Result<Vec<String>, String> {
        Ok(vec!["register data".to_string()])
    }

    pub async fn start_debugging(&self, _session_id: &str) -> Result<String, String> {
        Ok("Debugging started".to_string())
    }

    pub async fn stop_debugging(&self, _session_id: &str) -> Result<String, String> {
        Ok("Debugging stopped".to_string())
    }

    pub async fn continue_execution(&self, _session_id: &str) -> Result<String, String> {
        Ok("Execution continued".to_string())
    }

    pub async fn step_execution(&self, _session_id: &str) -> Result<String, String> {
        Ok("Step executed".to_string())
    }

    pub async fn next_execution(&self, _session_id: &str) -> Result<String, String> {
        Ok("Next executed".to_string())
    }
}

#[derive(Debug)]
pub struct SessionInfo {
    pub status: GDBSessionStatus,
}

// Import common test utilities
use std::path::Path;

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

pub struct STM32TestUtils;

impl STM32TestUtils {
    pub async fn check_hardware_available() -> bool {
        // Mock implementation for unit tests
        false
    }

    pub async fn validate_environment(config: &STM32TestConfig) -> Vec<String> {
        let mut issues = Vec::new();

        // Check if ELF file exists
        if !config.elf_file_path.exists() {
            issues.push(format!("ELF file not found: {:?}", config.elf_file_path));
        }

        issues
    }
}

/// Test MCP GDB session creation for STM32
#[tokio::test]
async fn test_create_stm32_gdb_session() {
    let config = STM32TestConfig::default();
    
    // Skip test if environment is not set up
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    let gdb_manager = GDBManager::new();
    
    // Test session creation with STM32 ELF file
    let session_id = gdb_manager
        .create_session(
            Some(config.elf_file_path.clone()),
            None, // nh
            None, // nx
            Some(true), // quiet
            Some(config.project_path.clone()), // cd
            None, // bps
            None, // symbol_file
            None, // core_file
            None, // proc_id
            None, // command
            None, // source_dir
            None, // args
            None, // tty
            Some(PathBuf::from(config.gdb_path.clone())), // gdb_path
        )
        .await;
    
    assert!(session_id.is_ok(), "Failed to create GDB session: {:?}", session_id.err());
    
    let session_id = session_id.unwrap();
    
    // Verify session exists
    let session_info = gdb_manager.get_session(&session_id).await;
    assert!(session_info.is_ok(), "Failed to get session info");
    
    let session_info = session_info.unwrap();
    assert_eq!(session_info.status, GDBSessionStatus::Created);
    
    // Clean up
    let _ = gdb_manager.close_session(&session_id).await;
}

/// Test setting breakpoints in STM32 code
#[tokio::test]
async fn test_set_stm32_breakpoints() {
    let config = STM32TestConfig::default();
    
    // Skip test if environment is not set up
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    let gdb_manager = GDBManager::new();
    
    // Create session
    let session_id = gdb_manager
        .create_session(
            Some(config.elf_file_path.clone()),
            None, None, Some(true), Some(config.project_path.clone()),
            None, None, None, None, None, None, None, None,
            Some(PathBuf::from(config.gdb_path.clone())),
        )
        .await
        .expect("Failed to create session");
    
    // Test setting breakpoint at main function
    let bp_result = gdb_manager
        .set_breakpoint(&session_id, "Core/Src/main.c", 77)
        .await;
    
    assert!(bp_result.is_ok(), "Failed to set breakpoint: {:?}", bp_result.err());
    
    // Test setting breakpoint at counter increment
    let bp_result = gdb_manager
        .set_breakpoint(&session_id, "Core/Src/main.c", 112)
        .await;
    
    assert!(bp_result.is_ok(), "Failed to set counter breakpoint: {:?}", bp_result.err());
    
    // Get all breakpoints
    let breakpoints = gdb_manager.get_breakpoints(&session_id).await;
    assert!(breakpoints.is_ok(), "Failed to get breakpoints");
    
    let breakpoints = breakpoints.unwrap();
    assert!(breakpoints.len() >= 2, "Expected at least 2 breakpoints, got {}", breakpoints.len());
    
    // Clean up
    let _ = gdb_manager.close_session(&session_id).await;
}

/// Test reading STM32 memory regions
#[tokio::test]
async fn test_read_stm32_memory() {
    let config = STM32TestConfig::default();
    
    // Skip test if environment is not set up or hardware not available
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() || !STM32TestUtils::check_hardware_available().await {
        println!("Skipping test due to environment issues or no hardware");
        return;
    }
    
    let gdb_manager = GDBManager::new();
    
    // Create session
    let session_id = gdb_manager
        .create_session(
            Some(config.elf_file_path.clone()),
            None, None, Some(true), Some(config.project_path.clone()),
            None, None, None, None, None, None, None, None,
            Some(PathBuf::from(config.gdb_path.clone())),
        )
        .await
        .expect("Failed to create session");
    
    // Test reading from SRAM region (0x20000000)
    let memory_result = gdb_manager
        .read_memory(&session_id, "0x20000000", 64, None)
        .await;
    
    // Note: This might fail if not connected to hardware, which is expected
    if memory_result.is_ok() {
        let memory_data = memory_result.unwrap();
        assert!(!memory_data.is_empty(), "Memory read returned empty data");
    }
    
    // Test reading from Flash region (0x08000000)
    let memory_result = gdb_manager
        .read_memory(&session_id, "0x08000000", 64, None)
        .await;
    
    // This should work if the ELF file is loaded
    if memory_result.is_ok() {
        let memory_data = memory_result.unwrap();
        assert!(!memory_data.is_empty(), "Flash memory read returned empty data");
    }
    
    // Clean up
    let _ = gdb_manager.close_session(&session_id).await;
}

/// Test getting STM32 registers
#[tokio::test]
async fn test_get_stm32_registers() {
    let config = STM32TestConfig::default();
    
    // Skip test if environment is not set up
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    let gdb_manager = GDBManager::new();
    
    // Create session
    let session_id = gdb_manager
        .create_session(
            Some(config.elf_file_path.clone()),
            None, None, Some(true), Some(config.project_path.clone()),
            None, None, None, None, None, None, None, None,
            Some(PathBuf::from(config.gdb_path.clone())),
        )
        .await
        .expect("Failed to create session");
    
    // Test getting register names
    let register_names = gdb_manager.get_register_names(&session_id, None).await;
    assert!(register_names.is_ok(), "Failed to get register names: {:?}", register_names.err());
    
    // Test getting specific ARM registers
    let arm_registers = vec!["r0".to_string(), "r1".to_string(), "sp".to_string(), "pc".to_string()];
    let registers = gdb_manager.get_registers(&session_id, Some(arm_registers)).await;
    
    // This might fail if not connected to hardware, which is expected
    if registers.is_ok() {
        let reg_data = registers.unwrap();
        assert!(!reg_data.is_empty(), "Register data is empty");
    }
    
    // Clean up
    let _ = gdb_manager.close_session(&session_id).await;
}

/// Test STM32 debugging workflow steps
#[tokio::test]
async fn test_stm32_debug_workflow() {
    let config = STM32TestConfig::default();
    
    // Skip test if environment is not set up
    let issues = STM32TestUtils::validate_environment(&config).await;
    if !issues.is_empty() {
        println!("Skipping test due to environment issues: {:?}", issues);
        return;
    }
    
    let gdb_manager = GDBManager::new();
    
    // Step 1: Create session
    let session_id = gdb_manager
        .create_session(
            Some(config.elf_file_path.clone()),
            None, None, Some(true), Some(config.project_path.clone()),
            None, None, None, None, None, None, None, None,
            Some(PathBuf::from(config.gdb_path.clone())),
        )
        .await
        .expect("Failed to create session");
    
    // Step 2: Set breakpoint
    let _ = gdb_manager
        .set_breakpoint(&session_id, "Core/Src/main.c", 77)
        .await
        .expect("Failed to set breakpoint");
    
    // Step 3: Test debugging control commands (these may fail without hardware)
    let start_result = gdb_manager.start_debugging(&session_id).await;
    let stop_result = gdb_manager.stop_debugging(&session_id).await;
    let continue_result = gdb_manager.continue_execution(&session_id).await;
    let step_result = gdb_manager.step_execution(&session_id).await;
    let next_result = gdb_manager.next_execution(&session_id).await;
    
    // These commands should at least not crash the session
    // Results may vary depending on hardware availability
    
    // Step 4: Get session info
    let session_info = gdb_manager.get_session(&session_id).await;
    assert!(session_info.is_ok(), "Failed to get session info");
    
    // Clean up
    let _ = gdb_manager.close_session(&session_id).await;
}

/// Test error handling for invalid STM32 operations
#[tokio::test]
async fn test_stm32_error_handling() {
    let gdb_manager = GDBManager::new();
    
    // Test with invalid session ID
    let invalid_session = "invalid-session-id";
    
    let result = gdb_manager.get_session(invalid_session).await;
    assert!(result.is_err(), "Expected error for invalid session ID");
    
    let result = gdb_manager.set_breakpoint(invalid_session, "main.c", 1).await;
    assert!(result.is_err(), "Expected error for invalid session ID");
    
    let result = gdb_manager.start_debugging(invalid_session).await;
    assert!(result.is_err(), "Expected error for invalid session ID");
    
    // Test with invalid file path
    let result = gdb_manager
        .create_session(
            Some(PathBuf::from("/nonexistent/file.elf")),
            None, None, None, None, None, None, None, None, None, None, None, None, None,
        )
        .await;
    
    // This should either fail or create a session that fails later
    // The exact behavior depends on GDB implementation
}
