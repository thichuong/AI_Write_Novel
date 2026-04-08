use super::api_client::{get_api_key, get_model, stream_gemini_response};
use super::gemini_types::{
    GeminiContent, GeminiPart, GeminiRequest, GenerationConfig, ThinkingConfig,
};
use crate::fs;
use tauri::AppHandle;

/// Chat với AI (streaming) — dùng cho hỗ trợ sáng tác
#[tauri::command]
pub async fn ai_chat(
    app_handle: AppHandle,
    root_path: String,
    current_file: String,
    message: String,
    chat_history: Vec<serde_json::Value>,
) -> Result<(), String> {
    let api_key = get_api_key()?;
    let model = get_model();

    // Lấy context từ file system
    let kb_context = fs::get_story_context(root_path.clone())?;
    let prev_chapters = fs::get_previous_chapters(root_path.clone(), current_file)?;

    let mut full_context = kb_context;
    if !prev_chapters.is_empty() {
        full_context.push_str(&format!(
            "\n# TÓM TẮT CÁC CHƯƠNG TRƯỚC\n{}\n",
            prev_chapters
        ));
    }

    let system_prompt = format!(
        "Bạn là một trợ lý sáng tác chuyên nghiệp. Hãy sử dụng KIẾN THỨC VỀ TRUYỆN dưới đây để trả lời.\n\n{}\n",
        full_context
    );

    // Build contents
    let mut contents = vec![
        GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text {
                text: system_prompt,
            }],
        },
        GeminiContent {
            role: "model".to_string(),
            parts: vec![GeminiPart::Text {
                text: "Tôi đã nắm được toàn bộ kiến thức về truyện. Hãy hỏi tôi bất cứ điều gì!"
                    .to_string(),
            }],
        },
    ];

    // Thêm chat history
    for msg in &chat_history {
        let role = msg["role"].as_str().unwrap_or("user");
        let content = msg["content"].as_str().unwrap_or("");
        // API dùng "model" thay vì "assistant"
        let api_role = if role == "assistant" { "model" } else { role };
        contents.push(GeminiContent {
            role: api_role.to_string(),
            parts: vec![GeminiPart::Text {
                text: content.to_string(),
            }],
        });
    }

    // Thêm message mới
    contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text { text: message }],
    });

    let request = GeminiRequest {
        contents,
        generation_config: Some(GenerationConfig {
            temperature: 0.8,
            max_output_tokens: 8192,
            thinking_config: Some(ThinkingConfig {
                thinking_level: "HIGH".to_string(),
            }),
        }),
        tools: None, // Chat mode không dùng tools
    };

    // Stream response
    stream_gemini_response(&app_handle, &api_key, &model, &request, "ai-chat-stream").await
}
