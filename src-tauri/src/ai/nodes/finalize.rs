use crate::ai::api_client::stream_gemini_response;
use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart, GeminiRequest, GenerationConfig};
use crate::ai::instructions::{
    FINALIZE_PROMPT_GENERAL, FINALIZE_PROMPT_IDEATION, FINALIZE_PROMPT_WRITING,
};
use crate::ai::nodes::{run_agent_loop, AgentState, AgentType};
use crate::ai::tools;
use serde_json::{json, Value};
use tauri::{Emitter, State};

pub async fn finalize_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    if state.agent_type != AgentType::Writing {
        let finalize_prompt = match state.agent_type {
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

        // Vẫn sử dụng run_agent_loop cho non-writing để cho phép gọi tool nếu cần, 
        // nhưng sau đó chúng ta sẽ cố gắng parse JSON từ kết quả cuối cùng.
        run_agent_loop(state, cancel_state, 5, "finalize").await?;
        perform_final_data_sync(state);
        return Ok(());
    }

    perform_wiki_extraction(state, cancel_state).await
}

fn perform_final_data_sync(state: &AgentState) {
    let Some(last_msg) = state.contents.last() else {
        return;
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
            let summary = parsed["summary"].as_str().unwrap_or("");
            let memory_updates = parsed["memory_updates"].as_str().unwrap_or("");

            // Cập nhật Memory
            if !memory_updates.is_empty() {
                let current_memory = tools::tool_read_file(&state.root_path, "memory.md").unwrap_or_default();
                let new_memory = format!("{current_memory}\n\n### Cập nhật mới:\n{memory_updates}");
                tools::tool_write_file(&state.app_handle, &state.root_path, "memory.md", &new_memory).ok();
            }

            // Hiển thị summary
            state.app_handle.emit("ai-chat-stream-thought", json!({
                "phase": "finalize",
                "text": format!("\n✅ Tóm tắt: {summary}\n📝 Đã cập nhật memory.md\n")
            })).ok();
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn perform_wiki_extraction(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    // --- LUỒNG RIÊNG CHO WRITING AGENT (Unified Finalize JSON) ---
    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "finalize",
                "text": "Đang tổng kết và hậu kiểm chương truyện (JSON Mode)...\n"
            }),
        )
        .ok();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: FINALIZE_PROMPT_WRITING.to_string(),
        }],
    });

    let request = GeminiRequest {
        contents: state.contents.clone(),
        system_instruction: state.system_instruction.clone(),
        generation_config: Some(GenerationConfig {
            temperature: 0.1,
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
        "finalize",
    )
    .await?;

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
    let json_text = crate::ai::nodes::extract_json_block(&full_text)
        .ok_or_else(|| format!("AI không trả về khối JSON Finalize hợp lệ. Nội dung gốc: {full_text}"))?;

    let parsed_json_value: Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("Lỗi parse Finalize JSON: {e}"))?;

    // 1. Xử lý Wiki Updates
    let entities = parsed_json_value["wiki_updates"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let mut update_count = 0;
    for entity in entities {
        let name = entity["name"].as_str().unwrap_or("Unnamed");
        let entity_type = entity["type"].as_str().unwrap_or("Lore");
        let description = entity["description"].as_str().unwrap_or("");
        let tags = entity["tags"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|v| v.as_str().unwrap_or_default().to_string())
                    .collect()
            })
            .unwrap_or_default();

        if let Ok(_res) = tools::tool_wiki_upsert_entity(
            &state.app_handle,
            &state.root_path,
            entity_type,
            name,
            description,
            tags,
        ) {
            update_count += 1;
            state
                .app_handle
                .emit(
                    "ai-chat-stream-thought",
                    json!({
                        "phase": "finalize",
                        "text": format!("• Cập nhật Wiki: {name} ({entity_type})\n")
                    }),
                )
                .ok();
        }
    }
    state.last_wiki_updates_count = update_count;

    // 2. Xử lý Memory Updates
    let memory_updates = parsed_json_value["memory_updates"].as_str().unwrap_or("");
    if !memory_updates.is_empty() {
        let current_memory = tools::tool_read_file(&state.root_path, "memory.md").unwrap_or_default();
        let new_memory = format!("{current_memory}\n\n### Cập nhật ({}) - {}:\n{memory_updates}", state.agent_type.as_str(), state.last_saved_file);
        tools::tool_write_file(&state.app_handle, &state.root_path, "memory.md", &new_memory).ok();
    }

    // 3. Xử lý Summary
    let summary = parsed_json_value["summary"].as_str().unwrap_or("Đã hoàn thành.");
    
    state.contents.push(GeminiContent {
        role: "model".to_string(),
        parts: vec![GeminiPart::Text {
            text: format!("TỔNG KẾT: {summary}\nSố thực thể Wiki đã cập nhật: {update_count}"),
        }],
    });

    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "finalize",
                "text": format!("\n✅ Tóm tắt: {summary}\n📝 Đã cập nhật memory.md\n")
            }),
        )
        .ok();

    state
        .app_handle
        .emit("ai-chat-stream-done", json!({ "phase": "finalize" }))
        .ok();

    Ok(())
}
