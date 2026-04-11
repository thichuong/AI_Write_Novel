use crate::ai::api_client::stream_gemini_response;
use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{
    GeminiContent, GeminiPart, GeminiRequest, GenerationConfig, ThinkingConfig,
};
use crate::ai::nodes::{AgentState, AgentType};
use serde_json::json;
use tauri::{Emitter, State};

const COORDINATOR_SYSTEM_PROMPT: &str = r#"
BẠN LÀ AI COORDINATOR (BỘ ĐIỀU PHỐI THÔNG MINH).
Nhiệm vụ của bạn là phân tích tin nhắn của người dùng và quyết định luồng xử lý.

PHẢI trả về JSON theo định dạng:
{
  "explanation": "Giải thích ngắn gọn lý do chọn agent",
  "agent": "chat" | "ideation" | "writing" | "general"
}

DÀNH CHO ĐIỀU PHỐI:
- "chat": Chào hỏi, câu hỏi xã giao, tìm kiếm thông tin thực tế, đọc file, xem thư mục, hỏi đáp về nội dung truyện.
- "ideation": Lên ý tưởng, brainstorm, phát triển cốt truyện/nhân vật.
- "writing": Viết mới hoặc chỉnh sửa văn bản truyện.
- "general": Các tác vụ phức tạp hoặc trộn lẫn.
"#;

pub async fn coordinate_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<Option<AgentType>, String> {
    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "coordinating",
                "text": "Đang phân tích yêu cầu...\n"
            }),
        )
        .ok();

    // Tạm thời ghi đè system instruction để điều phối
    let original_instructions = state.system_instruction.clone();
    state.system_instruction = Some(GeminiContent {
        role: "system".to_string(),
        parts: vec![GeminiPart::Text {
            text: COORDINATOR_SYSTEM_PROMPT.to_string(),
        }],
    });

    let generation_config = GenerationConfig {
        temperature: 0.1,
        max_output_tokens: 500,
        thinking_config: Some(ThinkingConfig {
            thinking_level: "MINIMAL".to_string(),
        }),
        response_mime_type: Some("application/json".to_string()),
        response_schema: Some(get_coordinate_schema()),
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
        cancel_state.clone(),
        &state.api_key,
        &state.model,
        &request,
        "ai-chat-stream",
        "coordinating",
    )
    .await?;

    // Khôi phục lại system instruction
    state.system_instruction = original_instructions;

    // Phân tích kết quả JSON
    Ok(get_full_text(&parts).and_then(|json_text| handle_coordinate_response(state, &json_text)))
}

fn get_coordinate_schema() -> crate::ai::gemini_types::Schema {
    let mut p = std::collections::HashMap::new();
    p.insert(
        "explanation".to_string(),
        crate::ai::gemini_types::Schema {
            field_type: "string".to_string(),
            description: Some("Lời giải thích hoặc nội dung trả lời trực tiếp".to_string()),
            properties: None,
            items: None,
            required: None,
        },
    );
    p.insert(
        "agent".to_string(),
        crate::ai::gemini_types::Schema {
            field_type: "string".to_string(),
            description: Some(
                "Loại agent: 'chat', 'ideation', 'writing', 'general'. Null nếu trả lời trực tiếp."
                    .to_string(),
            ),
            properties: None,
            items: None,
            required: None,
        },
    );

    crate::ai::gemini_types::Schema {
        field_type: "object".to_string(),
        properties: Some(p),
        items: None,
        required: Some(vec!["explanation".to_string(), "agent".to_string()]),
        description: None,
    }
}

fn handle_coordinate_response(
    state: &mut AgentState,
    json_text: &str,
) -> Option<AgentType> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_text) {
        let explanation = v["explanation"].as_str().unwrap_or("");
        let agent_str = v["agent"].as_str();

        // Nếu agent là null (literal hoặc chuỗi "null") hoặc trống, trả lời trực tiếp
        if agent_str.is_none() || agent_str == Some("null") || agent_str == Some("") {
            state.contents.push(GeminiContent {
                role: "model".to_string(),
                parts: vec![GeminiPart::Text {
                    text: explanation.to_string(),
                }],
            });

            state
                .app_handle
                .emit("ai-chat-stream-done", json!({ "phase": "coordinating" }))
                .ok();

            return None;
        }

        if let Some(agent_name) = agent_str {
            let agent_type = match agent_name {
                "chat" => AgentType::Chat,
                "ideation" => AgentType::Ideation,
                "writing" => AgentType::Writing,
                _ => AgentType::General,
            };

            // Emit lại để UI biết đã chuyển agent
            state
                .app_handle
                .emit("ai-agent-selected", agent_type.as_str())
                .ok();

            state
                .app_handle
                .emit(
                    "ai-chat-stream-thought",
                    json!({
                        "phase": "coordinating",
                        "text": format!("=> {}\n=> Đã điều phối tới: {}\n", explanation, agent_type.description())
                    }),
                )
                .ok();

            return Some(agent_type);
        }
    }
    None
}


fn get_full_text(parts: &[GeminiPart]) -> Option<String> {
    let mut full_text = String::new();
    for part in parts {
        if let GeminiPart::Text { text } = part {
            full_text.push_str(text);
        }
    }
    if full_text.is_empty() {
        None
    } else {
        Some(full_text)
    }
}
