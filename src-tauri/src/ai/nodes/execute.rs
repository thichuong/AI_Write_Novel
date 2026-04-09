use super::{run_agent_loop, AgentState};
use crate::ai::gemini_types::{GeminiContent, GeminiPart};

pub async fn execute_step(state: &mut AgentState) -> Result<(), String> {
    let execute_prompt =
        "THỰC HIỆN KẾ HOẠCH: Hãy sử dụng các công cụ cần thiết để hoàn thành mục tiêu.\n\
        - Bạn có thể gọi nhiều công cụ liên tục.\n\
        - Luôn cập nhật nhân vật/cốt truyện mới vào CẢ file chương và 'memory.md'.\n\
        - Khi đã hoàn tất các thay đổi file, hãy kết thúc bằng chuỗi 'DONE_EXECUTION'."
            .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: execute_prompt,
        }],
    });

    run_agent_loop(state, 10, "execute").await?;
    Ok(())
}
