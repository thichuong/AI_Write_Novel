use super::{run_agent_loop, AgentState};
use crate::ai::gemini_types::{GeminiContent, GeminiPart};

pub async fn analyze_step(state: &mut AgentState) -> Result<(), String> {
    let analyze_prompt =
        "HÃY PHÂN TÍCH yêu cầu trên và xác định MỤC TIÊU (Goal) cụ thể bạn cần làm.\n\
        Đầu tiên, hãy gọi `read_file('memory.md')` và `list_directory('.')` để lấy ngữ cảnh.\n\
        Sau đó, hãy trả lời bằng một đoạn text ngắn gọn về 'Kế hoạch thực hiện' của bạn."
            .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: analyze_prompt,
        }],
    });

    run_agent_loop(state, 2, "analyze").await?;
    Ok(())
}
