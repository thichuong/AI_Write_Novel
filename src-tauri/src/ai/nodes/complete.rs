use super::{run_agent_loop, AgentState};
use crate::ai::gemini_types::{GeminiContent, GeminiPart};

pub async fn complete_step(state: &mut AgentState) -> Result<(), String> {
    let complete_prompt = 
        "HOÀN TẤT: Hãy thông báo cho người dùng rằng tất cả các bước (Phân tích, Thực thi, Tổng hợp, Cập nhật Memory) đã xong. \
        Tóm tắt ngắn gọn kết quả cuối cùng và hỏi người dùng xem họ muốn thực hiện bước tiếp theo là gì."
            .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: complete_prompt,
        }],
    });

    run_agent_loop(state, 1, "complete").await?;
    Ok(())
}
