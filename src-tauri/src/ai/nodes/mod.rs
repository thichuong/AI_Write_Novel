use crate::ai::api_client::stream_gemini_response;
use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{
    FunctionCallingConfig, FunctionResponseData, GeminiContent, GeminiPart, GeminiRequest,
    GenerationConfig, ThinkingConfig, ToolConfig,
};
use crate::ai::tools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter, State};

pub mod analyze;
pub mod complete;
pub mod coordinate;
pub mod execute;
pub mod finalize;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    Chat,
    Ideation,
    Writing,
    General,
}

impl AgentType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::Ideation => "ideation",
            Self::Writing => "writing",
            Self::General => "general",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Chat => "Chat Agent (Trò chuyện & Tìm kiếm)",
            Self::Ideation => "Ideation Agent (Lên ý tưởng)",
            Self::Writing => "Writing Agent (Sáng tác & Chỉnh sửa)",
            Self::General => "General Agent (Xử lý tác vụ tổng hợp)",
        }
    }
}

/// Trạng thái của Agent trong quá trình xử lý đa bước
pub struct AgentState {
    pub app_handle: AppHandle,
    pub root_path: String,
    pub api_key: String,
    pub model: String,
    pub system_instruction: Option<GeminiContent>,
    pub contents: Vec<GeminiContent>,
    pub loop_count: u32,
}

/// Helper: Chạy vòng lặp gọi AI và xử lý Tool chung cho tất cả các nút
/// Helper: Chạy vòng lặp gọi AI và xử lý Tool chung cho tất cả các nút
pub async fn run_agent_loop(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
    max_local_loops: u32,
    phase: &str,
) -> Result<(), String> {
    let tool_decls = tools::get_tool_declarations();
    let generation_config = GenerationConfig {
        temperature: 0.7,
        max_output_tokens: 8192,
        thinking_config: Some(ThinkingConfig {
            thinking_level: "HIGH".to_string(),
        }),
        response_mime_type: None,
        response_schema: None,
    };

    // ToolConfig để cho phép dùng Built-in tools (Google Search) chung với Function calling
    let tool_config = ToolConfig {
        function_calling_config: Some(FunctionCallingConfig {
            mode: "AUTO".to_string(),
        }),
        include_server_side_tool_invocations: Some(true),
    };

    // Thông báo bắt đầu Phase để UI hiển thị box trống nếu chưa có gì
    if phase != "complete" {
        state
            .app_handle
            .emit(
                "ai-chat-stream-thought",
                json!({
                    "phase": phase,
                    "text": format!("Đang thực hiện bước: {}...\n", phase)
                }),
            )
            .ok();
    }

    let mut local_loops = 0;
    loop {
        local_loops += 1;
        state.loop_count += 1;

        if local_loops > max_local_loops || state.loop_count > 20 {
            break;
        }

        let request = GeminiRequest {
            contents: state.contents.clone(),
            system_instruction: state.system_instruction.clone(),
            generation_config: Some(generation_config.clone()),
            tools: Some(tool_decls.clone()),
            tool_config: Some(tool_config.clone()),
        };

        // Stream kết quả về frontend
        let parts = stream_gemini_response(
            &state.app_handle,
            cancel_state.clone(),
            &state.api_key,
            &state.model,
            &request,
            "ai-chat-stream",
            phase,
        )
        .await?;

        state.contents.push(GeminiContent {
            role: "model".to_string(),
            parts: parts.clone(),
        });

        let (function_calls, has_text_done) = process_model_parts(&parts);

        if function_calls.is_empty() || has_text_done {
            break;
        }

        // Xử lý Tool Calls
        let response_parts = execute_tool_calls(state, cancel_state.clone(), function_calls);

        state.contents.push(GeminiContent {
            role: "function".to_string(),
            parts: response_parts,
        });

        // Thông báo cho UI là đã xử lý xong Tool
        state.app_handle.emit("ai-chat-stream-tool-done", ()).ok();

        // Tối ưu hóa Context (Context Pruning) nếu đang ở các bước sau
        if phase == "finalize" || phase == "complete" {
            prune_history(&mut state.contents);
        }
    }

    // Thông báo toàn bộ phase đã hoàn thành
    state
        .app_handle
        .emit("ai-chat-stream-done", json!({ "phase": phase }))
        .ok();

    Ok(())
}

fn process_model_parts(
    parts: &[GeminiPart],
) -> (Vec<crate::ai::gemini_types::FunctionCallData>, bool) {
    let mut function_calls = Vec::new();
    let mut has_text_done = false;

    for part in parts {
        match part {
            GeminiPart::FunctionCall { function_call } => {
                function_calls.push(function_call.clone());
            }
            GeminiPart::Text { text } => {
                if text.contains("DONE_EXECUTION") || text.contains("Mục tiêu đã hoàn thành")
                {
                    has_text_done = true;
                }
            }
            GeminiPart::FunctionResponse { .. } => {}
        }
    }
    (function_calls, has_text_done)
}

fn execute_tool_calls(
    state: &AgentState,
    cancel_state: State<'_, CancellationState>,
    function_calls: Vec<crate::ai::gemini_types::FunctionCallData>,
) -> Vec<GeminiPart> {
    let mut response_parts = Vec::new();
    for fc in function_calls {
        if cancel_state.is_cancelled() {
            break;
        }
        let tool_result = match fc.name.as_str() {
            "list_directory" => {
                let path = fc.args["path"].as_str().unwrap_or(".");
                tools::tool_list_directory(&state.root_path, path)
            }
            "read_file" => {
                let path = fc.args["path"].as_str().unwrap_or("");
                tools::tool_read_file(&state.root_path, path)
            }
            "write_file" => {
                let path = fc.args["path"].as_str().unwrap_or("");
                let content = fc.args["content"].as_str().unwrap_or("");
                tools::tool_write_file(&state.app_handle, &state.root_path, path, content)
            }
            "delete_file" => {
                let path = fc.args["path"].as_str().unwrap_or("");
                tools::tool_delete_file(&state.app_handle, &state.root_path, path)
            }
            "wiki_list_entities" => tools::tool_wiki_list_entities(&state.root_path),
            "wiki_upsert_entity" => {
                let entity_type = fc.args["entity_type"].as_str().unwrap_or("Others");
                let name = fc.args["name"].as_str().unwrap_or("Unnamed");
                let content = fc.args["content"].as_str().unwrap_or("");
                let tags = fc.args["tags"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .map(|v| v.as_str().unwrap_or_default().to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                tools::tool_wiki_upsert_entity(
                    &state.app_handle,
                    &state.root_path,
                    entity_type,
                    name,
                    content,
                    tags,
                )
            }
            _ => Err(format!("Công cụ không tồn tại: {}", fc.name)),
        };

        let response_json = match tool_result {
            Ok(res) => json!({ "result": res }),
            Err(err) => json!({ "error": err }),
        };

        response_parts.push(GeminiPart::FunctionResponse {
            function_response: FunctionResponseData {
                name: fc.name.clone(),
                response: response_json,
            },
        });
    }
    response_parts
}

/// Tối ưu hóa lịch sử: Loại bỏ các `FunctionResponse` cũ hoặc các nội dung quá lớn
/// để tiết kiệm token cho các bước cuối cùng.
fn prune_history(contents: &mut Vec<GeminiContent>) {
    if contents.len() <= 6 {
        return; // Không dọn dẹp nếu lịch sử còn ngắn
    }

    let mut new_contents = Vec::new();

    // Giữ lại tin nhắn đầu tiên (user prompt đầu)
    if let Some(first_msg) = contents.first().cloned() {
        new_contents.push(first_msg);
    }

    // Giữ lại 5 tin nhắn cuối cùng
    // Trong các tin nhắn được giữ lại, nếu là 'function' (FunctionResponse),
    // ta có thể thay thế nội dung kết quả quá lớn bằng một thông báo ngắn nếu nó không phải là tin nhắn mới nhất.
    for (i, mut msg) in contents.iter().rev().take(5).rev().cloned().enumerate() {
        if msg.role == "function" && i < 4 {
            // Không phải tin nhắn cuối cùng
            for part in &mut msg.parts {
                if let GeminiPart::FunctionResponse { function_response } = part {
                    let res_str = function_response.response.to_string();
                    if res_str.len() > 1000 {
                        function_response.response = json!({
                            "result": "Content truncated to save tokens (information already processed in previous steps)."
                        });
                    }
                }
            }
        }
        new_contents.push(msg);
    }

    *contents = new_contents;
}
