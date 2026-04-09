use super::{run_agent_loop, AgentState};
use crate::ai::gemini_types::{GeminiContent, GeminiPart};

pub async fn memory_step(state: &mut AgentState) -> Result<(), String> {
    let memory_prompt = 
        "BƯỚC CẬP NHẬT BỘ NHỚ: Hãy sử dụng công cụ `write_file` để cập nhật file `memory.md` \
        với các thông tin mới nhất về dự án, tiến độ và thay đổi trong phiên làm việc này. \
        Đảm bảo giữ đúng định dạng Markdown của file memory."
            .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: memory_prompt,
        }],
    });

    run_agent_loop(state, 3, "memory").await?;
    Ok(())
}
