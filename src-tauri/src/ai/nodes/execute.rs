use crate::ai::api_client::stream_gemini_response;
use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart, GeminiRequest, GenerationConfig};
use crate::ai::instructions::{
    EXECUTE_PROMPT_GENERAL, EXECUTE_PROMPT_IDEATION, EXECUTE_PROMPT_WRITING,
};
use crate::ai::nodes::{run_agent_loop, AgentState, AgentType};
use crate::ai::tools;
use serde_json::{json, Value};
use tauri::{Emitter, State};

pub async fn execute_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    if state.agent_type == AgentType::Writing {
        return perform_writing_execution(state, cancel_state).await;
    }

    if state.agent_type == AgentType::Ideation {
        return perform_ideation_execution(state, cancel_state).await;
    }

    // General Agent vẫn giữ vòng lặp linh hoạt cho các tác vụ phức tạp
    let execute_prompt = EXECUTE_PROMPT_GENERAL.to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: execute_prompt,
        }],
    });

    run_agent_loop(state, cancel_state, 10, "execute").await
}

async fn perform_ideation_execution(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: EXECUTE_PROMPT_IDEATION.to_string(),
        }],
    });

    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "execute",
                "text": "Đang xây dựng ý tưởng sáng tạo (JSON Mode)...\n"
            }),
        )
        .ok();

    let request = GeminiRequest {
        contents: state.contents.clone(),
        system_instruction: state.system_instruction.clone(),
        generation_config: Some(GenerationConfig {
            temperature: 0.8,
            max_output_tokens: 4096,
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
        "execute",
    )
    .await?;

    state.contents.push(GeminiContent {
        role: "model".to_string(),
        parts: parts.clone(),
    });

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

    if let Some(json_text) = crate::ai::nodes::extract_json_block(&full_text) {
        let parsed: serde_json::Value = serde_json::from_str(&json_text)
            .map_err(|e| format!("Lỗi parse JSON Ideation: {e}"))?;

        let thought = parsed["thought_process"].as_str().unwrap_or("");
        let recommendation = parsed["recommendation"].as_str().unwrap_or("");
        let ideas = parsed["ideas"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|v| {
                        format!(
                            "### {}\n{}\n",
                            v["title"].as_str().unwrap_or("Ý tưởng"),
                            v["content"].as_str().unwrap_or("")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();

        state
            .app_handle
            .emit(
                "ai-chat-stream-thought",
                json!({
                    "phase": "execute",
                    "text": format!("\n💡 **Phân tích**: {thought}\n\n{ideas}\n🌟 **Khuyến nghị**: {recommendation}\n")
                }),
            )
            .ok();
    }

    state
        .app_handle
        .emit("ai-chat-stream-done", json!({ "phase": "execute" }))
        .ok();

    Ok(())
}


#[allow(clippy::too_many_lines)]
async fn perform_writing_execution(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    // --- LUỒNG RIÊNG CHO WRITING AGENT (JSON MODE) ---
    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: EXECUTE_PROMPT_WRITING.to_string(),
        }],
    });

    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "execute",
                "text": "Đang sáng tác nội dung (JSON Mode)...\n"
            }),
        )
        .ok();

    let request = GeminiRequest {
        contents: state.contents.clone(),
        system_instruction: state.system_instruction.clone(),
        generation_config: Some(GenerationConfig {
            temperature: 0.7,
            max_output_tokens: 8192,
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
        "execute",
    )
    .await?;

    state.contents.push(GeminiContent {
        role: "model".to_string(),
        parts: parts.clone(),
    });

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

    // Xử lý Robust JSON: Tìm khối JSON trong chuỗi
    let json_text = crate::ai::nodes::extract_json_block(&full_text)
        .ok_or_else(|| format!("AI không trả về khối JSON hợp lệ. Nội dung gốc: {full_text}"))?;

    // Hiển thị phần "văn bản thừa" (nếu có) lên Thought UI
    let extra_text = full_text.replacen(&json_text, "", 1);
    if !extra_text.trim().is_empty() {
        state
            .app_handle
            .emit(
                "ai-chat-stream-thought",
                json!({
                    "phase": "execute",
                    "text": format!("\n[AI Thought]: {}\n", extra_text.trim())
                }),
            )
            .ok();
    }

    // Parse JSON
    let parsed_json: Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("Lỗi parse JSON từ AI: {e}. Nội dung gốc: {json_text}"))?;

    let chapter_content = parsed_json["chapter_content"]
        .as_str()
        .ok_or("AI không trả về chapter_content")?;
    let suggested_filename = parsed_json["suggested_filename"]
        .as_str()
        .unwrap_or("chapters/Unsaved_Chapter.md");
    let thought_process = parsed_json["thought_process"]
        .as_str()
        .unwrap_or("Không có suy nghĩ.");

    // Lưu vào State
    state.last_chapter_content = chapter_content.to_string();
    state.last_saved_file = suggested_filename.to_string();
    state.last_word_count = chapter_content.split_whitespace().count();

    // Thực thi lưu file trực tiếp bằng Rust
    tools::tool_write_file(
        &state.app_handle,
        &state.root_path,
        suggested_filename,
        chapter_content,
    )?;

    // Emit thông báo hoàn tất bước cho UI
    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "execute",
                "text": format!("\n✅ Đã lưu chương truyện vào: `{suggested_filename}`\n💡 Phân tích của AI: {thought_process}\n")
            }),
        )
        .ok();

    state
        .app_handle
        .emit("ai-chat-stream-done", json!({ "phase": "execute" }))
        .ok();

    Ok(())
}
