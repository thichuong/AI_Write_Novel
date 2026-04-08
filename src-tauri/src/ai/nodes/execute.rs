use super::{run_agent_loop, AgentState};
use crate::ai::gemini_types::{GeminiContent, GeminiPart};

pub async fn execute_step(state: &mut AgentState) -> Result<(), String> {
    let execute_prompt = "BÂY GIỜ HÃY THỰC HIỆN KẾ HOẠCH. Sử dụng các tool cần thiết (write_file, delete_file, read_file...). \n\
        Bạn có thể gọi tool liên tục. Khi nào hoàn thành các tác vụ kỹ thuật (cập nhật file, viết chương...), hãy dừng lại và nói 'DONE_EXECUTION'.".to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: execute_prompt,
        }],
    });

    run_agent_loop(state, 10, "execute").await?;
    Ok(())
}
