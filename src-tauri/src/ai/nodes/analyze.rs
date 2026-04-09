use super::{run_agent_loop, AgentState};
use crate::ai::gemini_types::{GeminiContent, GeminiPart};

pub async fn analyze_step(state: &mut AgentState) -> Result<(), String> {
    let analyze_prompt =
        "PHÂN TÍCH YÊU CẦU: Hãy xác định mục tiêu cụ thể bạn cần đạt được.\n\
        1. Gọi `list_directory('.')` và `read_file('memory.md')` nếu cần ngữ cảnh.\n\
        2. Sau đó, viết một đoạn ngắn trình bày: 'Mục tiêu' và 'Kế hoạch các bước' bạn sẽ thực hiện."
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
