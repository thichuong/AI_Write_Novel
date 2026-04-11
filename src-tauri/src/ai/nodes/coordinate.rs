use crate::ai::api_client::stream_gemini_response;
use crate::ai::gemini_types::{GeminiContent, GeminiPart, GeminiRequest, GenerationConfig, ThinkingConfig};
use crate::ai::nodes::{AgentState, AgentType};
use serde_json::json;
use tauri::Emitter;

pub async fn coordinate_step(
    state: &mut AgentState,
) -> Result<Option<AgentType>, String> {
    state.app_handle.emit("ai-chat-stream-thought", json!({
        "phase": "coordinating",
        "text": "Đang phân tích yêu cầu...\n"
    })).ok();

    let system_prompt = r#"
BẠN LÀ AI COORDINATOR (BỘ ĐIỀU PHỐI THÔNG MINH).
Nhiệm vụ của bạn là phân tích tin nhắn của người dùng và quyết định:
1. TRẢ LỜI TRỰC TIẾP: Nếu là chào hỏi, câu hỏi xã giao đơn giản, hoặc hỏi về cách dùng phần mềm mà không cần tra cứu Wiki/Truyện. 
   - Định dạng: Trả lời như bình thường.
2. ĐIỀU PHỐI AGENT: Nếu yêu cầu liên quan đến sáng tác, tra cứu thông tin dự án, hoặc cần dùng công cụ.
   - Định dạng: BẮT ĐẦU bằng câu xác nhận ngắn gọn (không bắt buộc), sau đó PHẢI kết thúc bằng token: [ROUTE: agent_type]
   - agent_type có thể là: chat, ideation, writing, general.

DÀNH CHO ĐIỀU PHỐI:
- "chat": Tìm kiếm thông tin thực tế, đọc file, xem thư mục, hỏi đáp về nội dung truyện đã có.
- "ideation": Lên ý tưởng, brainstorm, phát triển cốt truyện/nhân vật.
- "writing": Viết mới hoặc chỉnh sửa văn bản truyện.
- "general": Các tác vụ phức tạp hoặc trộn lẫn.

VÍ DỤ:
- User: "Chào bạn" -> AI: "Chào bạn! Tôi có thể giúp gì cho bạn trong việc sáng tác hôm nay?"
- User: "Viết tiếp chương 1" -> AI: "Tôi sẽ phối hợp với Writing Agent để giúp bạn viết tiếp. [ROUTE: writing]"
"#;

    // Tạm thời ghi đè system instruction để điều phối
    let original_instructions = state.system_instruction.clone();
    state.system_instruction = Some(GeminiContent {
        role: "system".to_string(),
        parts: vec![GeminiPart::Text {
            text: system_prompt.to_string(),
        }],
    });

    let generation_config = GenerationConfig {
        temperature: 0.1, // Thấp để chính xác trong điều phối
        max_output_tokens: 500,
        thinking_config: Some(ThinkingConfig {
            thinking_level: "MINIMAL".to_string(),
        }),
    };

    let request = GeminiRequest {
        contents: state.contents.clone(),
        system_instruction: state.system_instruction.clone(),
        generation_config: Some(generation_config),
        tools: None,
        tool_config: None,
    };

    // Gọi AI (stream về UI)
    let parts = stream_gemini_response(
        &state.app_handle,
        &state.api_key,
        &state.model,
        &request,
        "ai-chat-stream",
        "coordinating",
    ).await?;

    // Khôi phục lại system instruction
    state.system_instruction = original_instructions;

    // Phân tích kết quả để tìm token ROUTE
    let mut full_text = String::new();
    for part in &parts {
        if let GeminiPart::Text { text } = part {
            full_text.push_str(text);
        }
    }

    if let Some(pos) = full_text.find("[ROUTE:") {
        let route_part = &full_text[pos..];
        let agent_str = route_part
            .strip_prefix("[ROUTE:")
            .and_then(|s| s.split(']').next())
            .map(|s| s.trim())
            .unwrap_or("general");

        let agent_type = match agent_str {
            "chat" => AgentType::Chat,
            "ideation" => AgentType::Ideation,
            "writing" => AgentType::Writing,
            _ => AgentType::General,
        };

        // Emit lại để UI biết đã chuyển agent
        state.app_handle.emit("ai-agent-selected", agent_type.as_str()).ok();

        state.app_handle.emit("ai-chat-stream-thought", json!({
            "phase": "coordinating",
            "text": format!("=> Đã điều phố tới: {}\n", agent_type.description())
        })).ok();
        
        return Ok(Some(agent_type));
    }

    // Nếu không có ROUTE token, coi như đã trả lời xong trực tiếp
    state.contents.push(GeminiContent {
        role: "model".to_string(),
        parts: parts.clone(),
    });

    Ok(None)
}
