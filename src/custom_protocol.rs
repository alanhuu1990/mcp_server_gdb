use std::path::PathBuf;

use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::tools::{self, PositiveInt, SignedInt};

/// Custom protocol request structure
#[derive(Debug, Deserialize)]
pub struct ToolRequest {
    pub params: Option<Value>,
}

/// Custom protocol response structure
#[derive(Debug, Serialize)]
pub struct ToolResponse {
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
}

impl ToolResponse {
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Create the custom protocol router
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/tools/list", get(list_tools))
        .route("/api/tools/:tool_name", post(call_tool))
        // Specific tool routes for better organization
        .route("/api/tools/create_session", post(create_session_handler))
        .route("/api/tools/get_session", post(get_session_handler))
        .route("/api/tools/get_all_sessions", post(get_all_sessions_handler))
        .route("/api/tools/close_session", post(close_session_handler))
        .route("/api/tools/start_debugging", post(start_debugging_handler))
        .route("/api/tools/stop_debugging", post(stop_debugging_handler))
        .route("/api/tools/get_breakpoints", post(get_breakpoints_handler))
        .route("/api/tools/set_breakpoint", post(set_breakpoint_handler))
        .route("/api/tools/delete_breakpoint", post(delete_breakpoint_handler))
        .route("/api/tools/get_stack_frames", post(get_stack_frames_handler))
        .route("/api/tools/get_local_variables", post(get_local_variables_handler))
        .route("/api/tools/continue_execution", post(continue_execution_handler))
        .route("/api/tools/step_execution", post(step_execution_handler))
        .route("/api/tools/next_execution", post(next_execution_handler))
        .route("/api/tools/get_registers", post(get_registers_handler))
        .route("/api/tools/get_register_names", post(get_register_names_handler))
        .route("/api/tools/read_memory", post(read_memory_handler))
}

/// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "mcp-server-gdb-custom-protocol",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// List all available tools
async fn list_tools() -> Json<Value> {
    let tools = vec![
        "create_session", "get_session", "get_all_sessions", "close_session",
        "start_debugging", "stop_debugging", "get_breakpoints", "set_breakpoint", 
        "delete_breakpoint", "get_stack_frames", "get_local_variables",
        "continue_execution", "step_execution", "next_execution",
        "get_registers", "get_register_names", "read_memory"
    ];
    
    Json(json!({
        "tools": tools,
        "count": tools.len(),
        "protocol": "custom-sse-bypass"
    }))
}

/// Generic tool call handler (fallback)
async fn call_tool(
    Path(tool_name): Path<String>,
    Json(request): Json<ToolRequest>,
) -> Result<Json<ToolResponse>, StatusCode> {
    info!("Custom protocol tool call: {} with params: {:?}", tool_name, request.params);
    
    match tool_name.as_str() {
        "create_session" => handle_create_session(request.params).await,
        "get_session" => handle_get_session(request.params).await,
        "get_all_sessions" => handle_get_all_sessions(request.params).await,
        "close_session" => handle_close_session(request.params).await,
        "start_debugging" => handle_start_debugging(request.params).await,
        "stop_debugging" => handle_stop_debugging(request.params).await,
        "get_breakpoints" => handle_get_breakpoints(request.params).await,
        "set_breakpoint" => handle_set_breakpoint(request.params).await,
        "delete_breakpoint" => handle_delete_breakpoint(request.params).await,
        "get_stack_frames" => handle_get_stack_frames(request.params).await,
        "get_local_variables" => handle_get_local_variables(request.params).await,
        "continue_execution" => handle_continue_execution(request.params).await,
        "step_execution" => handle_step_execution(request.params).await,
        "next_execution" => handle_next_execution(request.params).await,
        "get_registers" => handle_get_registers(request.params).await,
        "get_register_names" => handle_get_register_names(request.params).await,
        "read_memory" => handle_read_memory(request.params).await,
        _ => {
            error!("Unknown tool: {}", tool_name);
            Ok(Json(ToolResponse::error(format!("Unknown tool: {}", tool_name))))
        }
    }
}

// Individual tool handlers for better organization and specific routing
async fn create_session_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_create_session(request.params).await
}

async fn get_session_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_get_session(request.params).await
}

async fn get_all_sessions_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_get_all_sessions(request.params).await
}

async fn close_session_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_close_session(request.params).await
}

async fn start_debugging_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_start_debugging(request.params).await
}

async fn stop_debugging_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_stop_debugging(request.params).await
}

async fn get_breakpoints_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_get_breakpoints(request.params).await
}

async fn set_breakpoint_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_set_breakpoint(request.params).await
}

async fn delete_breakpoint_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_delete_breakpoint(request.params).await
}

async fn get_stack_frames_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_get_stack_frames(request.params).await
}

async fn get_local_variables_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_get_local_variables(request.params).await
}

async fn continue_execution_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_continue_execution(request.params).await
}

async fn step_execution_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_step_execution(request.params).await
}

async fn next_execution_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_next_execution(request.params).await
}

async fn get_registers_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_get_registers(request.params).await
}

async fn get_register_names_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_get_register_names(request.params).await
}

async fn read_memory_handler(Json(request): Json<ToolRequest>) -> Result<Json<ToolResponse>, StatusCode> {
    handle_read_memory(request.params).await
}

// Helper function to extract parameter from JSON
fn extract_param<T: for<'de> Deserialize<'de>>(params: &Option<Value>, key: &str) -> Result<Option<T>, String> {
    match params {
        Some(Value::Object(map)) => {
            if let Some(value) = map.get(key) {
                serde_json::from_value(value.clone())
                    .map(Some)
                    .map_err(|e| format!("Failed to parse parameter '{}': {}", key, e))
            } else {
                Ok(None)
            }
        }
        Some(_) => Err("Parameters must be a JSON object".to_string()),
        None => Ok(None),
    }
}

// Helper function to extract required parameter from JSON
fn extract_required_param<T: for<'de> Deserialize<'de>>(params: &Option<Value>, key: &str) -> Result<T, String> {
    extract_param(params, key)?
        .ok_or_else(|| format!("Missing required parameter: {}", key))
}

// Tool handler implementations
async fn handle_create_session(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling create_session with params: {:?}", params);

    let program: Option<PathBuf> = extract_param(&params, "program").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    let nh: Option<bool> = extract_param(&params, "nh").map_err(|_| StatusCode::BAD_REQUEST)?;
    let nx: Option<bool> = extract_param(&params, "nx").map_err(|_| StatusCode::BAD_REQUEST)?;
    let quiet: Option<bool> = extract_param(&params, "quiet").map_err(|_| StatusCode::BAD_REQUEST)?;
    let cd: Option<PathBuf> = extract_param(&params, "cd").map_err(|_| StatusCode::BAD_REQUEST)?;
    let bps: Option<u32> = extract_param(&params, "bps").map_err(|_| StatusCode::BAD_REQUEST)?;
    let symbol_file: Option<PathBuf> = extract_param(&params, "symbol_file").map_err(|_| StatusCode::BAD_REQUEST)?;
    let core_file: Option<PathBuf> = extract_param(&params, "core_file").map_err(|_| StatusCode::BAD_REQUEST)?;
    let proc_id: Option<u32> = extract_param(&params, "proc_id").map_err(|_| StatusCode::BAD_REQUEST)?;
    let command: Option<PathBuf> = extract_param(&params, "command").map_err(|_| StatusCode::BAD_REQUEST)?;
    let source_dir: Option<PathBuf> = extract_param(&params, "source_dir").map_err(|_| StatusCode::BAD_REQUEST)?;
    let args: Option<Vec<String>> = extract_param(&params, "args").map_err(|_| StatusCode::BAD_REQUEST)?;
    let tty: Option<PathBuf> = extract_param(&params, "tty").map_err(|_| StatusCode::BAD_REQUEST)?;
    let gdb_path: Option<PathBuf> = extract_param(&params, "gdb_path").map_err(|_| StatusCode::BAD_REQUEST)?;

    match tools::create_session_tool(
        program, nh, nx, quiet, cd, bps.map(PositiveInt), symbol_file, core_file,
        proc_id.map(PositiveInt), command, source_dir, args, tty, gdb_path
    ).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Session created successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("create_session error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_get_session(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling get_session with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match tools::get_session_tool(session_id).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Session retrieved successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("get_session error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_get_all_sessions(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling get_all_sessions with params: {:?}", params);

    match tools::get_all_sessions_tool().await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Sessions retrieved successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("get_all_sessions error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_close_session(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling close_session with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match tools::close_session_tool(session_id).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Session closed successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("close_session error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_start_debugging(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling start_debugging with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match tools::start_debugging_tool(session_id).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Debugging started successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("start_debugging error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_stop_debugging(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling stop_debugging with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match tools::stop_debugging_tool(session_id).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Debugging stopped successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("stop_debugging error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_get_breakpoints(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling get_breakpoints with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match tools::get_breakpoints_tool(session_id).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Breakpoints retrieved successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("get_breakpoints error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_set_breakpoint(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling set_breakpoint with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    let file: String = extract_required_param(&params, "file").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    let line: u32 = extract_required_param(&params, "line").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match tools::set_breakpoint_tool(session_id, file, PositiveInt(line)).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Breakpoint set successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("set_breakpoint error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_delete_breakpoint(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling delete_breakpoint with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    let breakpoints: Vec<String> = extract_required_param(&params, "breakpoints").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match tools::delete_breakpoint_tool(session_id, breakpoints).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Breakpoints deleted successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("delete_breakpoint error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_get_stack_frames(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling get_stack_frames with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match tools::get_stack_frames_tool(session_id).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Stack frames retrieved successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("get_stack_frames error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_get_local_variables(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling get_local_variables with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    let frame_id: Option<u32> = extract_param(&params, "frame_id").map_err(|_| StatusCode::BAD_REQUEST)?;

    match tools::get_local_variables_tool(session_id, frame_id.map(PositiveInt)).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Local variables retrieved successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("get_local_variables error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_continue_execution(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling continue_execution with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match tools::continue_execution_tool(session_id).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Execution continued successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("continue_execution error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_step_execution(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling step_execution with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match tools::step_execution_tool(session_id).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Step execution successful".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("step_execution error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_next_execution(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling next_execution with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match tools::next_execution_tool(session_id).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Next execution successful".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("next_execution error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_get_registers(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling get_registers with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    let reg_list: Option<Vec<String>> = extract_param(&params, "reg_list").map_err(|_| StatusCode::BAD_REQUEST)?;

    match tools::get_registers_tool(session_id, reg_list).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Registers retrieved successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("get_registers error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_get_register_names(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling get_register_names with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    let reg_list: Option<Vec<String>> = extract_param(&params, "reg_list").map_err(|_| StatusCode::BAD_REQUEST)?;

    match tools::get_register_names_tool(session_id, reg_list).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Register names retrieved successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("get_register_names error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}

async fn handle_read_memory(params: Option<Value>) -> Result<Json<ToolResponse>, StatusCode> {
    debug!("Handling read_memory with params: {:?}", params);

    let session_id: String = extract_required_param(&params, "session_id").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    let address: String = extract_required_param(&params, "address").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    let count: u32 = extract_required_param(&params, "count").map_err(|e| {
        error!("Parameter error: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    let offset: Option<i32> = extract_param(&params, "offset").map_err(|_| StatusCode::BAD_REQUEST)?;

    match tools::read_memory_tool(session_id, address, PositiveInt(count), offset.map(SignedInt)).await {
        Ok(response) => {
            let content = match response {
                mcp_core::types::ToolResponseContent::Text { text } => text,
                _ => "Memory read successfully".to_string(),
            };
            Ok(Json(ToolResponse::success(json!({ "message": content }))))
        }
        Err(e) => {
            error!("read_memory error: {}", e);
            Ok(Json(ToolResponse::error(e.to_string())))
        }
    }
}
