use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart};
use crate::ai::instructions::{
    EXECUTE_PROMPT_GENERAL, EXECUTE_PROMPT_IDEATION, EXECUTE_PROMPT_WRITING,
};
use crate::ai::nodes::{run_agent_loop, AgentState, AgentType};
use serde_json::json;
use tauri::{Emitter, State};

pub async fn execute_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    // Luồng Thực thi (Execute) - ĐƯỢC DÙNG TOOL
    let execute_prompt = match state.agent_type {
        AgentType::Writing => EXECUTE_PROMPT_WRITING,
        AgentType::Ideation => EXECUTE_PROMPT_IDEATION,
        _ => EXECUTE_PROMPT_GENERAL,
    }
    .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: execute_prompt,
        }],
    });

    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "execute",
                "text": "Đang thực thi các tác vụ hệ thống (Cập nhật Wiki & Đồng bộ)...\n"
            }),
        )
        .ok();

    // Chạy vòng lặp CHO PHÉP dùng Tool
    run_agent_loop(state, cancel_state, 5, "execute", true).await?;

    // Sau khi vòng lặp hoàn tất, chúng ta có thể thực hiện thêm các bước đồng bộ bổ sung từ JSON model nếu cần
    process_execute_feedback(state);

    Ok(())
}

fn process_execute_feedback(state: &mut AgentState) {
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
            let actions = parsed["actions_taken"].as_array();
            if let Some(actions) = actions {
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
                            "phase": "execute",
                            "text": format!("\n✅ **Kết quả thực thi**:\n{action_list}\n")
                        }),
                    )
                    .ok();
            }

            if let Some(count) = parsed["wiki_updates_count"].as_u64() {
                state.last_wiki_updates_count = usize::try_from(count).unwrap_or(usize::MAX);
            }
        }
    }
}
