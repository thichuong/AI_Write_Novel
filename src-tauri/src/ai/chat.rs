use super::api_client::{get_api_key, get_model};
use super::gemini_types::{GeminiContent, GeminiPart};
use super::nodes::{
    analyze::analyze_step, execute::execute_step, summarize::summarize_step, AgentState,
};
use tauri::{AppHandle, Emitter};

use super::instructions::{AGENT_INSTRUCTIONS, WIKI_GRAPH_RULES};

#[tauri::command]
pub async fn ai_chat(
    app_handle: AppHandle,
    root_path: String,
    _current_file: String,
    message: String,
    chat_history: Vec<serde_json::Value>,
) -> Result<(), String> {
    let api_key = get_api_key()?;
    let model = get_model();

    // 1. Khởi tạo State với các giá trị mặc định
    let mut state = AgentState {
        app_handle: app_handle.clone(),
        root_path: root_path.clone(),
        api_key,
        model,
        system_instruction: None,
        contents: Vec::new(),
        loop_count: 0,
    };

    // 2. Setup System Prompt & History
    setup_initial_context(&mut state, message, chat_history);

    // 3. Quy trình Đa bước (State Machine)
    app_handle.emit("ai-agent-step", "analyze").ok();
    analyze_step(&mut state).await?;

    app_handle.emit("ai-agent-step", "execute").ok();
    execute_step(&mut state).await?;

    app_handle.emit("ai-agent-step", "summarize").ok();
    summarize_step(&mut state).await?;

    Ok(())
}

fn setup_initial_context(
    state: &mut AgentState,
    message: String,
    chat_history: Vec<serde_json::Value>,
) {
    let system_instructions = format!(
        "BẠN LÀ AI NOVELIST AGENT - Phiên bản nâng cấp Wiki Graph.\n\n\
        {AGENT_INSTRUCTIONS}\n\n\
        {WIKI_GRAPH_RULES}\n\n\
        Hành động của bạn phải luôn bắt đầu bằng việc PHÂN TÍCH và KHÁM PHÁ trước khi viết."
    );

    state.system_instruction = Some(GeminiContent {
        role: "system".to_string(), // Role cho system_instruction thường là "system" hoặc bỏ trống tùy API, nhưng Gemini internal hay dùng "system"
        parts: vec![GeminiPart::Text {
            text: system_instructions,
        }],
    });

    // Lấy lịch sử chat (4 tin nhắn gần nhất)
    let historical_context: Vec<&serde_json::Value> = chat_history.iter().rev().take(4).collect();
    for msg in historical_context.into_iter().rev() {
        let role = msg["role"].as_str().unwrap_or("user");
        let content = msg["content"].as_str().unwrap_or("");
        let api_role = if role == "assistant" { "model" } else { role };
        state.contents.push(GeminiContent {
            role: api_role.to_string(),
            parts: vec![GeminiPart::Text {
                text: content.to_string(),
            }],
        });
    }

    // Đảm bảo message mới nhất có mặt
    let last_msg_in_history = chat_history.last().and_then(|m| m["content"].as_str());
    if last_msg_in_history != Some(&message) {
        state.contents.push(GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text { text: message }],
        });
    }
}
