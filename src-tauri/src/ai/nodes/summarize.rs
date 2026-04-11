use super::{run_agent_loop, AgentState};
use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart};
use tauri::State;

pub async fn summarize_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    let summarize_prompt =
        "BƯỚC TỔNG HỢP: Hãy tổng hợp lại toàn bộ những hành động và thay đổi bạn đã thực hiện trong phiên này. \
        Đây là bước chuẩn bị để cập nhật vào memory.md ở bước sau."
            .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: summarize_prompt,
        }],
    });

    run_agent_loop(state, cancel_state, 2, "summarize").await?;
    Ok(())
}
