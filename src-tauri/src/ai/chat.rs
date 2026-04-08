use super::api_client::{get_api_key, get_model};
use super::gemini_types::{GeminiContent, GeminiPart};
use super::nodes::{
    analyze::analyze_step, execute::execute_step, summarize::summarize_step, AgentState,
};
use tauri::{AppHandle, Emitter};

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

    // 1. Khởi tạo State
    let mut state = AgentState {
        app_handle: app_handle.clone(),
        root_path: root_path.clone(),
        api_key,
        model,
        contents: Vec::new(),
        goal: String::new(),
        loop_count: 0,
        finished: false,
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
    let system_prompt = "Bạn là AI Novelist Agent. Bạn hoạt động trên cơ chế 'Memory Ground Truth'.\n\n\
        QUY TẮC:\n\
        1. BỘ NHỚ LÀ TRÊN HẾT: Bạn KHÔNG phụ thuộc vào lịch sử chat cũ. Dùng `read_file('memory.md')` và `list_directory('.')` để nắm bắt tình hình.\n\
        2. QUY TRÌNH: Analyze -> Execute (Tools) -> Summarize (Update memory.md).\n\
        3. NGÔN NGỮ: Tiếng Việt.\n\
        4. THỰC THI: Nếu người dùng yêu cầu thay đổi cốt truyện/nhân vật, hãy cập nhật vào cả file chương và memory.md.".to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: system_prompt,
        }],
    });
    state.contents.push(GeminiContent {
        role: "model".to_string(),
        parts: vec![GeminiPart::Text { text: "Tôi đã hiểu kiến trúc Agentic đa bước. Tôi sẽ tuân thủ quy trình Analyze -> Execute -> Summarize.".to_string() }],
    });

    // Lấy link lịch sử ngắn gọn (4 tin nhắn gần nhất)
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
