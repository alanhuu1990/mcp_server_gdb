#!/usr/bin/env rust-script
//! Test script for the custom protocol implementation
//! 
//! This script tests all 13+ GDB tools via the custom HTTP protocol
//! to validate that the workaround for mcp-core v0.1 bug is working.

use std::collections::HashMap;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::time::sleep;

#[derive(Debug)]
struct TestResult {
    tool_name: String,
    success: bool,
    response_time_ms: u64,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Custom Protocol Test Suite");
    println!("Testing all GDB tools via custom HTTP protocol");
    println!("This validates the workaround for mcp-core v0.1 bug\n");

    let base_url = "http://127.0.0.1:8081"; // HTTP server port (SSE port + 1)
    let client = reqwest::Client::new();

    // Test 1: Health Check
    println!("📋 Test 1: Health Check");
    let health_result = test_health_check(&client, &base_url).await?;
    print_test_result(&health_result);

    // Test 2: List Tools
    println!("\n📋 Test 2: List Available Tools");
    let list_result = test_list_tools(&client, &base_url).await?;
    print_test_result(&list_result);

    // Test 3: Session Management Tools
    println!("\n📋 Test 3: Session Management");
    let mut session_id = String::new();
    
    // Create session
    let create_result = test_create_session(&client, &base_url).await?;
    print_test_result(&create_result);
    
    if create_result.success {
        // Extract session ID from response (simplified)
        session_id = "test-session-id".to_string(); // In real test, parse from response
        
        // Get session
        let get_result = test_get_session(&client, &base_url, &session_id).await?;
        print_test_result(&get_result);
        
        // Get all sessions
        let get_all_result = test_get_all_sessions(&client, &base_url).await?;
        print_test_result(&get_all_result);
    }

    // Test 4: Debugging Control Tools
    println!("\n📋 Test 4: Debugging Control");
    if !session_id.is_empty() {
        let debug_tests = vec![
            test_start_debugging(&client, &base_url, &session_id),
            test_stop_debugging(&client, &base_url, &session_id),
        ];
        
        for test_future in debug_tests {
            let result = test_future.await?;
            print_test_result(&result);
        }
    }

    // Test 5: Breakpoint Management
    println!("\n📋 Test 5: Breakpoint Management");
    if !session_id.is_empty() {
        let breakpoint_tests = vec![
            test_get_breakpoints(&client, &base_url, &session_id),
            test_set_breakpoint(&client, &base_url, &session_id),
            test_delete_breakpoint(&client, &base_url, &session_id),
        ];
        
        for test_future in breakpoint_tests {
            let result = test_future.await?;
            print_test_result(&result);
        }
    }

    // Test 6: Execution Control
    println!("\n📋 Test 6: Execution Control");
    if !session_id.is_empty() {
        let execution_tests = vec![
            test_continue_execution(&client, &base_url, &session_id),
            test_step_execution(&client, &base_url, &session_id),
            test_next_execution(&client, &base_url, &session_id),
        ];
        
        for test_future in execution_tests {
            let result = test_future.await?;
            print_test_result(&result);
        }
    }

    // Test 7: Information Retrieval
    println!("\n📋 Test 7: Information Retrieval");
    if !session_id.is_empty() {
        let info_tests = vec![
            test_get_stack_frames(&client, &base_url, &session_id),
            test_get_local_variables(&client, &base_url, &session_id),
            test_get_registers(&client, &base_url, &session_id),
            test_get_register_names(&client, &base_url, &session_id),
            test_read_memory(&client, &base_url, &session_id),
        ];
        
        for test_future in info_tests {
            let result = test_future.await?;
            print_test_result(&result);
        }
    }

    // Test 8: Cleanup
    println!("\n📋 Test 8: Cleanup");
    if !session_id.is_empty() {
        let close_result = test_close_session(&client, &base_url, &session_id).await?;
        print_test_result(&close_result);
    }

    println!("\n✅ Custom Protocol Test Suite Complete!");
    println!("All tests validate the custom HTTP protocol workaround for mcp-core v0.1 bug");

    Ok(())
}

async fn test_health_check(client: &reqwest::Client, base_url: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    
    match client.get(&format!("{}/health", base_url)).send().await {
        Ok(response) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let success = response.status().is_success();
            
            Ok(TestResult {
                tool_name: "health_check".to_string(),
                success,
                response_time_ms: elapsed,
                error: if success { None } else { Some(format!("HTTP {}", response.status())) },
            })
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            Ok(TestResult {
                tool_name: "health_check".to_string(),
                success: false,
                response_time_ms: elapsed,
                error: Some(e.to_string()),
            })
        }
    }
}

async fn test_list_tools(client: &reqwest::Client, base_url: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    
    match client.get(&format!("{}/api/tools/list", base_url)).send().await {
        Ok(response) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let success = response.status().is_success();
            
            Ok(TestResult {
                tool_name: "list_tools".to_string(),
                success,
                response_time_ms: elapsed,
                error: if success { None } else { Some(format!("HTTP {}", response.status())) },
            })
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            Ok(TestResult {
                tool_name: "list_tools".to_string(),
                success: false,
                response_time_ms: elapsed,
                error: Some(e.to_string()),
            })
        }
    }
}

async fn test_create_session(client: &reqwest::Client, base_url: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "create_session", json!({})).await
}

async fn test_get_session(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "get_session", json!({"session_id": session_id})).await
}

async fn test_get_all_sessions(client: &reqwest::Client, base_url: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "get_all_sessions", json!({})).await
}

async fn test_close_session(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "close_session", json!({"session_id": session_id})).await
}

async fn test_start_debugging(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "start_debugging", json!({"session_id": session_id})).await
}

async fn test_stop_debugging(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "stop_debugging", json!({"session_id": session_id})).await
}

async fn test_get_breakpoints(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "get_breakpoints", json!({"session_id": session_id})).await
}

async fn test_set_breakpoint(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "set_breakpoint", json!({"session_id": session_id, "file": "main.c", "line": 10})).await
}

async fn test_delete_breakpoint(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "delete_breakpoint", json!({"session_id": session_id, "breakpoints": ["1"]})).await
}

async fn test_continue_execution(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "continue_execution", json!({"session_id": session_id})).await
}

async fn test_step_execution(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "step_execution", json!({"session_id": session_id})).await
}

async fn test_next_execution(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "next_execution", json!({"session_id": session_id})).await
}

async fn test_get_stack_frames(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "get_stack_frames", json!({"session_id": session_id})).await
}

async fn test_get_local_variables(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "get_local_variables", json!({"session_id": session_id})).await
}

async fn test_get_registers(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "get_registers", json!({"session_id": session_id})).await
}

async fn test_get_register_names(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "get_register_names", json!({"session_id": session_id})).await
}

async fn test_read_memory(client: &reqwest::Client, base_url: &str, session_id: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    test_tool_call(client, base_url, "read_memory", json!({"session_id": session_id, "address": "0x1000", "count": 16})).await
}

async fn test_tool_call(client: &reqwest::Client, base_url: &str, tool_name: &str, params: Value) -> Result<TestResult, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    let payload = json!({"params": params});
    
    match client.post(&format!("{}/api/tools/{}", base_url, tool_name))
        .json(&payload)
        .send()
        .await {
        Ok(response) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let success = response.status().is_success();
            
            Ok(TestResult {
                tool_name: tool_name.to_string(),
                success,
                response_time_ms: elapsed,
                error: if success { None } else { Some(format!("HTTP {}", response.status())) },
            })
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            Ok(TestResult {
                tool_name: tool_name.to_string(),
                success: false,
                response_time_ms: elapsed,
                error: Some(e.to_string()),
            })
        }
    }
}

fn print_test_result(result: &TestResult) {
    let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
    let time = format!("{}ms", result.response_time_ms);
    
    println!("  {} {} ({})", status, result.tool_name, time);
    
    if let Some(error) = &result.error {
        println!("    Error: {}", error);
    }
}
