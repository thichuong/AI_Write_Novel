use super::{run_agent_loop, AgentState};
use crate::ai::gemini_types::{GeminiContent, GeminiPart};

pub async fn analyze_step(state: &mut AgentState) -> Result<(), String> {
    let analyze_prompt =
        "PHÂN TÍCH VÀ NẠP KIẾN THỨC:\n\
        1. Gọi `list_directory('.')` để nắm cấu trúc.\n\
        2. Gọi `read_file('memory.md')` để hiểu bối cảnh dự án.\n\
        3. Nếu cần thông tin nhân vật/thế giới, gọi `wiki_list_entities()`.\n\
        4. Sau đó, lập kế hoạch chi tiết để giải quyết yêu cầu của người dùng dựa trên tri thức đã nạp."
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
