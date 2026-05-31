use crate::ai::api_client::stream_gemini_response;
use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart, GeminiRequest, GenerationConfig};
use crate::ai::instructions::{
    FINALIZE_PROMPT_GENERAL, FINALIZE_PROMPT_IDEATION, FINALIZE_PROMPT_WRITING,
};
use crate::ai::nodes::{AgentState, AgentType};
use crate::ai::tools;
use crate::error::AppResult;
use serde_json::json;
use tauri::{Emitter, State};

pub async fn finalize_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> AppResult<()> {
    // Luồng Tổng kết (Finalize) - KHÔNG CẬP NHẬT WIKI (Đã làm ở bước Execute)
    let finalize_prompt = match state.agent_type {
        AgentType::Writing => FINALIZE_PROMPT_WRITING,
        AgentType::Ideation => FINALIZE_PROMPT_IDEATION,
        _ => FINALIZE_PROMPT_GENERAL,
    }
    .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: finalize_prompt,
        }],
    });

    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "finalize",
                "text": "Đang tổng kết và ghi nhớ tiến độ dự án (Memory update)...\n"
            }),
        )
        .ok();

    let request = GeminiRequest {
        contents: state.contents.clone(),
        system_instruction: state.system_instruction.clone(),
        generation_config: Some(GenerationConfig {
            temperature: 0.1,
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
        "finalize",
    )
    .await?;

    state.contents.push(GeminiContent {
        role: "model".to_string(),
        parts: parts.clone(),
    });

    process_finalize_response(state, &parts)?;

    state
        .app_handle
        .emit("ai-chat-stream-done", json!({ "phase": "finalize" }))
        .ok();

    Ok(())
}

fn process_finalize_response(state: &AgentState, parts: &[GeminiPart]) -> AppResult<()> {
    let full_text: String = parts
        .iter()
        .filter_map(|p| {
            if let GeminiPart::Text { text } = p {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect();

    let mut project_summary = String::new();
    let mut extracted_successfully = false;

    // Try to extract from JSON first
    if let Some(json_text) = crate::ai::nodes::extract_json_block(&full_text) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_text) {
            // Check potential keys: "project_summary", then "summary", "result", "content"
            let keys = ["project_summary", "summary", "result", "content"];
            for key in &keys {
                if let Some(summary_val) = parsed.get(key) {
                    if let Some(summary_str) = summary_val.as_str() {
                        let trimmed = summary_str.trim();
                        if !trimmed.is_empty() {
                            project_summary = trimmed.to_string();
                            extracted_successfully = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    // Fallback: If JSON extraction failed or field was empty, clean and use the entire full_text
    if !extracted_successfully {
        let trimmed_text = full_text.trim();
        if !trimmed_text.is_empty() {
            // Clean up code block backticks and json markers if any
            let mut cleaned = String::new();
            let mut in_code_block = false;
            for line in trimmed_text.lines() {
                let trimmed_line = line.trim();
                if trimmed_line.starts_with("```") {
                    in_code_block = !in_code_block;
                    continue;
                }
                if !in_code_block {
                    cleaned.push_str(line);
                    cleaned.push('\n');
                }
            }
            let final_fallback = cleaned.trim().to_string();
            if final_fallback.is_empty() {
                // If stripping code block left nothing, use the original trimmed full_text
                project_summary = trimmed_text.to_string();
            } else {
                project_summary = final_fallback;
            }
        }
    }

    // Ensure memory.md is written if we have any content
    if !project_summary.is_empty() {
        tools::tool_write_file(
            &state.app_handle,
            &state.root_path,
            "memory.md",
            &project_summary,
        )?;

        let word_count = project_summary.split_whitespace().count();
        let message_text = if extracted_successfully {
            format!(
                "\n📝 [System]: Đã tự động cập nhật bản tóm tắt dự án mới nhất vào `memory.md` ({word_count} từ)\n"
            )
        } else {
            format!(
                "\n📝 [System]: Đã tự động cập nhật `memory.md` bằng phản hồi trực tiếp ({word_count} từ - Fallback)\n"
            )
        };

        state
            .app_handle
            .emit(
                "ai-chat-stream-thought",
                json!({
                    "phase": "finalize",
                    "text": message_text
                }),
            )
            .ok();
    }

    Ok(())
}
