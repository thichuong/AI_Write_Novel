use crate::ai::api_client::stream_gemini_response;
use crate::ai::gemini_types::{
    FunctionResponseData, GeminiContent, GeminiPart, GeminiRequest, GenerationConfig,
    ThinkingConfig,
};
use crate::ai::tools;
use serde_json::json;
use tauri::{AppHandle, Emitter};

pub mod analyze;
pub mod execute;
pub mod summarize;

/// Trạng thái của Agent trong quá trình xử lý đa bước
pub struct AgentState {
    pub app_handle: AppHandle,
    pub root_path: String,
    pub api_key: String,
    pub model: String,
    pub system_instruction: Option<GeminiContent>,
    pub contents: Vec<GeminiContent>,
    pub goal: String,
    pub loop_count: u32,
    pub finished: bool,
}

/// Helper: Chạy vòng lặp gọi AI và xử lý Tool chung cho tất cả các nút
pub async fn run_agent_loop(
    state: &mut AgentState,
    max_local_loops: u32,
    _phase: &str,
) -> Result<(), String> {
    let tool_decls = tools::get_tool_declarations();
    let generation_config = GenerationConfig {
        temperature: 0.7,
        max_output_tokens: 8192,
        thinking_config: Some(ThinkingConfig {
            thinking_level: "HIGH".to_string(),
        }),
    };

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
        };

        // Stream kết quả về frontend (luôn dùng tên chung ai-chat-stream)
        let streaming_event = "ai-chat-stream";
        let parts = stream_gemini_response(
            &state.app_handle,
            &state.api_key,
            &state.model,
            &request,
            streaming_event,
        )
        .await?;

        state.contents.push(GeminiContent {
            role: "model".to_string(),
            parts: parts.clone(),
        });

        let mut function_calls = Vec::new();
        let mut has_text_done = false;

        for part in &parts {
            match part {
                GeminiPart::FunctionCall { function_call } => {
                    function_calls.push(function_call.clone())
                }
                GeminiPart::Text { text } => {
                    if text.contains("DONE_EXECUTION") || text.contains("Mục tiêu đã hoàn thành")
                    {
                        has_text_done = true;
                    }
                }
                _ => {}
            }
        }

        if function_calls.is_empty() || has_text_done {
            break;
        }

        // Xử lý Tool Calls
        let mut response_parts = Vec::new();
        for fc in function_calls {
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

        state.contents.push(GeminiContent {
            role: "function".to_string(),
            parts: response_parts,
        });

        // Thông báo cho UI là đã xử lý xong Tool, đang chờ AI phản hồi tiếp
        state.app_handle.emit("ai-chat-stream-tool-done", ()).ok();
    }

    Ok(())
}
