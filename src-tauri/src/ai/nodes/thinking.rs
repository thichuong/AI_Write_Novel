use crate::ai::api_client::stream_gemini_response;
use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart, GeminiRequest, GenerationConfig};
use crate::ai::instructions::THINKING_PROMPT_WRITING;
use crate::ai::nodes::AgentState;
use crate::ai::tools;
use crate::error::AppResult;
use serde_json::json;
use tauri::{Emitter, State};

/// Executes Step 1 of Writing mode: Creative writing without tools
pub async fn thinking_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> AppResult<()> {
    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: THINKING_PROMPT_WRITING.to_string(),
        }],
    });

    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "thinking",
                "text": "Đang sáng tác nội dung chương mới (Tập trung viết - Không gọi Tools)...\n"
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
        "thinking",
    )
    .await?;

    state.contents.push(GeminiContent {
        role: "model".to_string(),
        parts: parts.clone(),
    });

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

    if let Some(json_text) = crate::ai::nodes::extract_json_block(&full_text) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_text) {
            let chapter_content = parsed["chapter_content"].as_str().unwrap_or("");
            state.last_chapter_content = chapter_content.to_string();
            state.last_saved_file = parsed["suggested_filename"]
                .as_str()
                .unwrap_or("chapters/Unnamed.md")
                .to_string();
            state.last_word_count = chapter_content.split_whitespace().count();

            // Save the chapter content immediately using Rust
            tools::tool_write_file(
                &state.app_handle,
                &state.root_path,
                &state.last_saved_file,
                chapter_content,
            )?;

            state
                .app_handle
                .emit(
                    "ai-chat-stream-thought",
                    json!({
                        "phase": "thinking",
                        "text": format!(
                            "\n📖 **Bản thảo đã hoàn thành ({})**. Đã lưu vào `{}`\n",
                            state.last_word_count, state.last_saved_file
                        )
                    }),
                )
                .ok();
        }
    }

    state
        .app_handle
        .emit("ai-chat-stream-done", json!({ "phase": "thinking" }))
        .ok();

    Ok(())
}
