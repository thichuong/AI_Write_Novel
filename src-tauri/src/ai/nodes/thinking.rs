use crate::ai::api_client::stream_gemini_response;
use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart, GeminiRequest, GenerationConfig};
use crate::ai::instructions::{
    THINKING_PROMPT_GENERAL, THINKING_PROMPT_IDEATION, THINKING_PROMPT_WRITING,
};
use crate::ai::nodes::{run_agent_loop, AgentState, AgentType};
use crate::ai::tools;
use serde_json::json;
use tauri::{Emitter, State};

pub async fn thinking_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    if state.agent_type == AgentType::Writing {
        return perform_writing_thinking(state, cancel_state).await;
    }

    if state.agent_type == AgentType::Ideation {
        return perform_ideation_thinking(state, cancel_state).await;
    }

    // General Agent
    let thinking_prompt = THINKING_PROMPT_GENERAL.to_string();
    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: thinking_prompt,
        }],
    });

    // Chạy vòng lặp KHÔNG cho phép dùng Tool
    run_agent_loop(state, cancel_state, 1, "thinking", false).await?;

    // Tự động kiểm tra và lưu nội dung nếu có chapter_content (Dành cho Writing & General)
    process_thinking_feedback(state)?;

    Ok(())
}

async fn perform_ideation_thinking(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: THINKING_PROMPT_IDEATION.to_string(),
        }],
    });

    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "thinking",
                "text": "Đang phát triển ý tưởng sáng tạo (Thinking mode - No Tools)...\n"
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
        "thinking",
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
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_text) {
            let thought = parsed["thought_process"].as_str().unwrap_or("");
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
                        "phase": "thinking",
                        "text": format!("\n💡 **Phác thảo**: {thought}\n\n{ideas}\n")
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

async fn perform_writing_thinking(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
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
                "text": "Đang sáng tác nội dung (Thinking mode - No Tools)...\n"
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
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_text) {
            let chapter_content = parsed["chapter_content"].as_str().unwrap_or("");
            state.last_chapter_content = chapter_content.to_string();
            state.last_saved_file = parsed["suggested_filename"]
                .as_str()
                .unwrap_or("chapters/Unnamed.md")
                .to_string();
            state.last_word_count = chapter_content.split_whitespace().count();

            state
                .app_handle
                .emit(
                    "ai-chat-stream-thought",
                    json!({
                        "phase": "thinking",
                        "text": format!("\n📖 **Bản thảo đã hoàn thành ({})**. Đang chuyển sang bước thực thi...\n", state.last_word_count)
                    }),
                )
                .ok();
        }
    }

    state
        .app_handle
        .emit("ai-chat-stream-done", json!({ "phase": "thinking" }))
        .ok();

    // Tự động kiểm tra và lưu nội dung (Dành cho Writing)
    process_thinking_feedback(state)?;

    Ok(())
}

fn process_thinking_feedback(state: &mut AgentState) -> Result<(), String> {
    let Some(last_msg) = state.contents.last() else {
        return Ok(());
    };

    let full_text = last_msg
        .parts
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
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_text) {
            let chapter_content = parsed["chapter_content"]
                .as_str()
                .or_else(|| parsed["result"].as_str())
                .unwrap_or("");

            if !chapter_content.is_empty() {
                let suggested_filename = parsed["suggested_filename"]
                    .as_str()
                    .unwrap_or("chapters/Unnamed_Chapter.md");

                // Lưu vào State để các bước sau sử dụng
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

                state
                    .app_handle
                    .emit(
                        "ai-chat-stream-thought",
                        json!({
                            "phase": "thinking",
                            "text": format!("\n💾 [System]: Đã tự động lưu nội dung vào `{suggested_filename}` ({}) \n", state.last_word_count)
                        }),
                    )
                    .ok();
            }
        }
    }

    Ok(())
}
