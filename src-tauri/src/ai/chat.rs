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
    let system_instructions = "BẠN LÀ AI NOVELIST AGENT - Chuyên gia hỗ trợ viết tiểu thuyết chuyên nghiệp.\n\n\
        PHƯƠNG CHÂM HOẠT ĐỘNG: 'Memory Ground Truth' (Bộ nhớ là sự thật duy nhất).\n\n\
        CÔNG CỤ CỦA BẠN (TOOLS):\n\
        1. `list_directory(path: string)`: Liệt kê các file trong thư mục. Luôn dùng '.' để xem thư mục gốc.\n\
        2. `read_file(path: string)`: Đọc nội dung file. Luôn đọc 'memory.md' đầu tiên để hiểu ngữ cảnh.\n\
        3. `write_file(path: string, content: string)`: Tạo hoặc ghi đè nội dung file. Dùng để viết chương mới hoặc cập nhật cốt truyện.\n\
        4. `delete_file(path: string)`: Xóa file không cần thiết.\n\n\
        QUY TẮC CỐT LÕI:\n\
        - Luôn bắt đầu bằng việc thăm dò: Gọi `list_directory` và `read_file('memory.md')` nếu bạn chưa biết tình hình dự án.\n\
        - Luôn cập nhật memory: Khi bạn thay đổi nội dung truyện (viết thêm, sửa nhân vật), bạn PHẢI cập nhật tóm tắt vào 'memory.md'.\n\
        - Ngôn ngữ: Tiếng Việt (trừ các từ chuyên ngành kỹ thuật).\n\n\
        VÍ DỤ GỌI TOOL (FEW-SHOT):\n\
        * Người dùng: 'Viết tiếp chương 2.'\n\
        * Agent Phân tích: Cần biết nội dung chương 1 và memory.\n\
        * Agent Thực thi:\n\
          - Gọi `read_file('memory.md')`\n\
          - Gọi `read_file('chapter_1.md')`\n\
          - Sau khi nhận kết quả, gọi `write_file('chapter_2.md', '...')` và `write_file('memory.md', '...')` (đã cập nhật tóm tắt chương 2).\n\n\
        BẮT ĐẦU: Hãy lắng nghe yêu cầu của người dùng và thực hiện theo quy trình Analyze -> Execute -> Summarize.".to_string();

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
