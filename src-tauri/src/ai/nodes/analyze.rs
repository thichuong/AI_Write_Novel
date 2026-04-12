use crate::ai::api_client::stream_gemini_response;
use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart, GeminiRequest, GenerationConfig};
use crate::ai::instructions::{
    ANALYZE_PROMPT_GENERAL, ANALYZE_PROMPT_IDEATION, ANALYZE_PROMPT_WRITING,
};
use crate::ai::nodes::{AgentState, AgentType};
use crate::ai::tools::{tool_list_directory, tool_read_file, tool_wiki_list_entities};
use serde_json::json;
use tauri::{Emitter, State};

pub async fn analyze_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    // 1. Thu thập tri thức tự động từ backend
    let dir_context =
        tool_list_directory(&state.root_path, ".").unwrap_or_else(|e| format!("Lỗi liệt kê: {e}"));
    let memory_context = tool_read_file(&state.root_path, "memory.md")
        .unwrap_or_else(|_| "Chưa có file memory.md hoặc file trống.".to_string());
    let wiki_context =
        tool_wiki_list_entities(&state.root_path).unwrap_or_else(|e| format!("Lỗi Wiki: {e}"));

    let agent_specific_guidance = match state.agent_type {
        AgentType::Writing => ANALYZE_PROMPT_WRITING,
        AgentType::Ideation => ANALYZE_PROMPT_IDEATION,
        _ => ANALYZE_PROMPT_GENERAL,
    };

    let analyze_prompt = format!(
        "PHÂN TÍCH VÀ NẠP KIẾN THỨC:\n\
        {agent_specific_guidance}\n\n\
        ### CẤU TRÚC THƯ MỤC:\n{dir_context}\n\n\
        ### NỘI DUNG MEMORY.MD:\n{memory_context}\n\n\
        ### DANH SÁCH WIKI:\n{wiki_context}\n"
    );

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: analyze_prompt,
        }],
    });

    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "analyze",
                "text": "Đang phân tích bối cảnh dự án (JSON Mode)...\n"
            }),
        )
        .ok();

    let request = GeminiRequest {
        contents: state.contents.clone(),
        system_instruction: state.system_instruction.clone(),
        generation_config: Some(GenerationConfig {
            temperature: 0.2,
            max_output_tokens: 2048,
            thinking_config: None,
            response_mime_type: Some("application/json".to_string()),
            response_schema: None,
        }),
        tools: None,
        tool_config: None,
    };

    let parts = stream_gemini_response(
        &state.app_handle,
        cancel_state.clone(),
        &state.api_key,
        &state.model,
        &request,
        "ai-chat-stream",
        "analyze",
    )
    .await?;

    state.contents.push(GeminiContent {
        role: "model".to_string(),
        parts: parts.clone(),
    });

    process_analyze_response(state, &parts)?;

    state
        .app_handle
        .emit("ai-chat-stream-done", json!({ "phase": "analyze" }))
        .ok();

    Ok(())
}

fn process_analyze_response(state: &AgentState, parts: &[GeminiPart]) -> Result<(), String> {
    // Trích xuất text từ kết quả
    let full_text = parts
        .iter()
        .filter_map(|p| {
            if let GeminiPart::Text { text } = p {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<String>();

    // Xử lý Robust JSON
    let json_text = crate::ai::nodes::extract_json_block(&full_text).ok_or_else(|| {
        format!("AI không trả về khối JSON Analyze hợp lệ. Nội dung gốc: {full_text}")
    })?;

    let parsed_json: serde_json::Value =
        serde_json::from_str(&json_text).map_err(|e| format!("Lỗi parse JSON Analyze: {e}"))?;

    let thought = parsed_json["thought_process"].as_str().unwrap_or("");
    let status = parsed_json["status_check"].as_str().unwrap_or("");
    let plan = parsed_json["plan"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| format!("• {}", v.as_str().unwrap_or("-")))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();

    // Hiển thị Plan lên UI
    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "analyze",
                "text": format!("\n💡 **Phân tích**: {thought}\n🧐 **Đánh giá**: {status}\n\n📋 **Kế hoạch hành động**:\n{plan}\n")
            }),
        )
        .ok();

    Ok(())
}
