use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart};
use crate::ai::instructions::{
    COMPLETE_PROMPT_GENERAL, COMPLETE_PROMPT_IDEATION, COMPLETE_PROMPT_WRITING,
};
use crate::ai::nodes::{run_agent_loop, AgentState, AgentType};
use tauri::State;

pub async fn complete_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    let complete_prompt = if state.agent_type == AgentType::Writing {
        format!(
            "{}\n\n[HỆ THỐNG NHẮC NHỞ]: Bạn vừa hoàn thành việc viết khoảng {} từ, lưu vào file '{}'. Đã tự động cập nhật {} thực thể vào Wiki. Hãy dùng thông tin này để báo cáo ngắn gọn, thân thiện cho người dùng.",
            COMPLETE_PROMPT_WRITING,
            state.last_word_count,
            state.last_saved_file,
            state.last_wiki_updates_count
        )
    } else {
        match state.agent_type {
            AgentType::Ideation => COMPLETE_PROMPT_IDEATION,
            _ => COMPLETE_PROMPT_GENERAL,
        }
        .to_string()
    };

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: complete_prompt,
        }],
    });

    // Bước cuối cùng - Cho phép dùng tool nếu AI muốn giải thích thêm gì đó, hoặc không.
    run_agent_loop(state, cancel_state, 1, "complete", true).await?;
    Ok(())
}
