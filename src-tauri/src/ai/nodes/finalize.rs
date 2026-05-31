use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart};
use crate::ai::instructions::WRITING_SYNC_PROMPT;
use crate::ai::nodes::{run_agent_loop, AgentState};
use crate::ai::tools;
use crate::error::AppResult;
use serde_json::json;
use tauri::{Emitter, State};

/// Executes Step 2 of Writing mode: Extracting entities, updating Wiki and Memory with tools enabled
pub async fn finalize_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> AppResult<()> {
    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: WRITING_SYNC_PROMPT.to_string(),
        }],
    });

    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "finalize",
                "text": "Đang phân tích chương mới, tự động trích xuất & đồng bộ hóa Wiki (Tool-enabled)...\n"
            }),
        )
        .ok();

    // Run the agent loop with tool calling enabled to perform wiki upserts and actions
    run_agent_loop(state, cancel_state.clone(), 5, "finalize", true).await?;

    // Post-process the finalize response to safely extract and save memory.md summary
    process_finalize_feedback(state)?;

    if cancel_state.is_cancelled() {
        return Err(crate::error::AppError::Cancelled("Stopped".to_string()));
    }

    // Phase 3 (Final Report): Prompt AI to report completion friendly to the user
    let complete_prompt = format!(
        "HỆ THỐNG NHẮC NHỞ: Bạn đã hoàn thành việc sáng tác chương truyện mới, lưu thành công vào file '{}' (khoảng {} từ), và đã đồng bộ hóa đầy đủ thực thể mới vào Wiki. Hãy viết một báo cáo ngắn gọn, thân thiện bằng Tiếng Việt để thông báo cho tác giả và mời họ tiếp tục sáng tác.",
        state.last_saved_file,
        state.last_word_count
    );
    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: complete_prompt,
        }],
    });

    // Run agent loop with complete phase to stream output to UI's main answer area
    run_agent_loop(state, cancel_state, 1, "complete", false).await?;

    Ok(())
}

fn process_finalize_feedback(state: &mut AgentState) -> AppResult<()> {
    let Some(last_msg) = state.contents.last() else {
        return Ok(());
    };

    let full_text: String = last_msg
        .parts
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

    if let Some(json_text) = crate::ai::nodes::extract_json_block(&full_text) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_text) {
            if let Some(summary_val) = parsed.get("project_summary") {
                if let Some(summary_str) = summary_val.as_str() {
                    let trimmed = summary_str.trim();
                    if !trimmed.is_empty() {
                        project_summary = trimmed.to_string();
                        extracted_successfully = true;
                    }
                }
            }

            if let Some(count) = parsed["wiki_updates_count"].as_u64() {
                state.last_wiki_updates_count = usize::try_from(count).unwrap_or(usize::MAX);
            }

            if let Some(actions) = parsed["actions_taken"].as_array() {
                let action_list = actions
                    .iter()
                    .map(|v| format!("• {}", v.as_str().unwrap_or("-")))
                    .collect::<Vec<_>>()
                    .join("\n");

                state
                    .app_handle
                    .emit(
                        "ai-chat-stream-thought",
                        json!({
                            "phase": "finalize",
                            "text": format!("\n✅ **Kết quả đồng bộ**:\n{action_list}\n")
                        }),
                    )
                    .ok();
            }
        }
    }

    // Fallback if JSON extraction failed or was empty
    if !extracted_successfully {
        let trimmed_text = full_text.trim();
        if !trimmed_text.is_empty() {
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
                project_summary = trimmed_text.to_string();
            } else {
                project_summary = final_fallback;
            }
        }
    }

    // Write to memory.md
    if !project_summary.is_empty() {
        tools::tool_write_file(
            &state.app_handle,
            &state.root_path,
            "memory.md",
            &project_summary,
        )?;

        let word_count = project_summary.split_whitespace().count();
        state
            .app_handle
            .emit(
                "ai-chat-stream-thought",
                json!({
                    "phase": "finalize",
                    "text": format!(
                        "\n💾 [System]: Đã tự động cập nhật bản tóm tắt tiến trình và cây liên kết Wiki mới vào `memory.md` ({} từ)\n",
                        word_count
                    )
                }),
            )
            .ok();
    }

    Ok(())
}
