use super::{run_agent_loop, AgentState};
use crate::ai::gemini_types::{GeminiContent, GeminiPart};

pub async fn summarize_step(state: &mut AgentState) -> Result<(), String> {
    let summarize_prompt =
        "CUỐI CÙNG: Hãy cập nhật lại `memory.md` để ghi nhận các diễn biến mới nhất.\n\
        Sau đó, hãy tóm tắt những gì bạn đã làm và trả lời người dùng một cách chuyên nghiệp."
            .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: summarize_prompt,
        }],
    });

    run_agent_loop(state, 3, "summarize").await?;
    Ok(())
}
