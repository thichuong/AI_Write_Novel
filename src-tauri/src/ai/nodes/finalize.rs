use super::{run_agent_loop, AgentState};
use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart};
use tauri::State;

pub async fn finalize_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    let finalize_prompt =
        "BƯỚC TỔNG KẾT & CẬP NHẬT BỘ NHỚ: Hãy tổng hợp lại toàn bộ những hành động và thay đổi bạn đã thực hiện trong phiên này. \
        Sau đó, sử dụng công cụ `write_file` để cập nhật file `memory.md` với các thông tin mới nhất về dự án, tiến độ và thay đổi đó. \
        Đảm bảo giữ đúng định dạng Markdown của file memory."
            .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: finalize_prompt,
        }],
    });

    run_agent_loop(state, cancel_state, 5, "finalize").await?;
    Ok(())
}
