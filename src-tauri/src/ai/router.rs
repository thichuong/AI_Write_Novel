use serde::{Deserialize, Serialize};
use crate::ai::gemini_types::{GeminiContent, GeminiPart, GeminiRequest};
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum AgentType {
    Chat,
    Ideation,
    Writing,
    General,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentType::Chat => "chat",
            AgentType::Ideation => "ideation",
            AgentType::Writing => "writing",
            AgentType::General => "general",
        }
    }
}

pub async fn classify_intent(
    api_key: &str,
    model: &str,
    message: &str,
) -> Result<AgentType, String> {
    let system_prompt = r#"
BẠN LÀ BỘ PHÂN LOẠI Ý ĐỊNH (INTENT CLASSIFIER).
Nhiệm vụ của bạn là phân tích tin nhắn của người dùng và trả về một trong các nhãn sau dưới dạng JSON:

- "chat": Cho các câu hỏi xã giao, chào hỏi, hỏi về phần mềm, hoặc nội dung không liên quan trực tiếp đến sáng tác.
- "ideation": Khi người dùng yêu cầu lên ý tưởng, brainstorm cốt truyện, nhân vật, bối cảnh, hoặc hỏi "nên làm gì tiếp theo".
- "writing": Khi người dùng yêu cầu viết một đoạn văn, viết chương truyện, hoặc chỉnh sửa văn phong của một đoạn cụ thể.
- "general": Khi yêu cầu phức tạp, bao gồm nhiều nhiệm vụ hoặc không rõ ràng, cần sự phân tích sâu.

CHỈ TRẢ VỀ JSON: {"intent": "nhãn"}
"#;

    let request = GeminiRequest {
        contents: vec![GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text {
                text: message.to_string(),
            }],
        }],
        system_instruction: Some(GeminiContent {
            role: "system".to_string(),
            parts: vec![GeminiPart::Text {
                text: system_prompt.to_string(),
            }],
        }),
        generation_config: None,
        tools: None,
        tool_config: None,
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let client = Client::new();
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Lỗi router: {}", e))?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Lỗi parse JSON router: {}", e))?;

    let text = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or("Không nhận được kết quả từ router")?;

    // Trích xuất JSON từ text (trong trường hợp AI bao quanh bằng ```json)
    let clean_text = if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            &text[start..=end]
        } else {
            text
        }
    } else {
        text
    };

    let result: serde_json::Value = serde_json::from_str(clean_text)
        .map_err(|e| format!("Lỗi parse kết quả router: {} | Text: {}", e, text))?;

    match result["intent"].as_str() {
        Some("chat") => Ok(AgentType::Chat),
        Some("ideation") => Ok(AgentType::Ideation),
        Some("writing") => Ok(AgentType::Writing),
        Some("general") => Ok(AgentType::General),
        _ => Ok(AgentType::General),
    }
}
