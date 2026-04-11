use super::api_client::{get_api_key, get_model};
use super::cancellation::CancellationState;
use super::nodes::{
    analyze::analyze_step, complete::complete_step, coordinate::coordinate_step,
    execute::execute_step, memory::memory_step, run_agent_loop, summarize::summarize_step,
    AgentState, AgentType,
};
use tauri::{AppHandle, Emitter, State};

use super::instructions::{
    AGENT_INSTRUCTIONS, CHAT_AGENT_INSTRUCTIONS, IDEATION_AGENT_INSTRUCTIONS, WIKI_GRAPH_RULES,
    WRITING_AGENT_INSTRUCTIONS,
};

#[tauri::command]
pub fn stop_ai_chat(cancel_state: State<'_, CancellationState>) {
    cancel_state.cancel();
}

#[tauri::command]
pub async fn ai_chat(
    app_handle: AppHandle,
    cancel_state: State<'_, CancellationState>,
    root_path: String,
    _current_file: String,
    message: String,
    chat_history: Vec<serde_json::Value>,
) -> Result<(), String> {
    let api_key = get_api_key()?;
    let model = get_model();

    // 1. Khởi tạo State với ngữ cảnh cơ bản
    let mut state = AgentState {
        app_handle: app_handle.clone(),
        root_path: root_path.clone(),
        api_key,
        model,
        system_instruction: None,
        contents: Vec::new(),
        loop_count: 0,
    };

    // Chuẩn bị nội dung hội thoại (lịch sử + tin nhắn mới)
    prepare_conversation_contents(&mut state, message, chat_history);

    cancel_state.reset();

    // 2. Điều phối thông minh (Coordinator)
    // Node này có thể tự trả lời nếu câu hỏi đơn giản
    app_handle.emit("ai-agent-step", "coordinating").ok();
    let agent_type = match coordinate_step(&mut state, cancel_state.clone()).await {
        Ok(Some(at)) => at,
        Ok(None) => {
            // Đã trả lời trực tiếp, thông báo hoàn tất toàn hệ thống
            app_handle
                .emit(
                    "ai-chat-stream-done",
                    serde_json::json!({ "phase": "complete" }),
                )
                .ok();
            return Ok(());
        }
        Err(e) => {
            if cancel_state.is_cancelled() {
                return Err("Agent stopped by user".to_string());
            }
            app_handle.emit("ai-chat-stream-thought", serde_json::json!({
                "phase": "coordinating",
                "text": format!("⚠️ Cảnh báo: Lỗi điều phối ({e}). Chuyển sang General Agent.\n")
            })).ok();
            AgentType::General
        }
    };

    // 3. Setup Instruction chuyên biệt cho Agent đã chọn
    apply_agent_instructions(&mut state, agent_type);

    // 4. Luồng xử lý theo Agent
    match agent_type {
        AgentType::Chat => {
            app_handle.emit("ai-agent-step", "chatting").ok();

            // Chat Agent giờ đây cũng có thể dùng Tool (Search, Read File)
            run_agent_loop(&mut state, cancel_state.clone(), 3, "complete").await?;
        }
        AgentType::Ideation => {
            app_handle.emit("ai-agent-step", "analyze").ok();
            analyze_step(&mut state, cancel_state.clone()).await?;

            if cancel_state.is_cancelled() {
                return Err("Stopped".to_string());
            }
            app_handle.emit("ai-agent-step", "ideate").ok();
            execute_step(&mut state, cancel_state.clone()).await?;

            if cancel_state.is_cancelled() {
                return Err("Stopped".to_string());
            }
            app_handle.emit("ai-agent-step", "summarize").ok();
            summarize_step(&mut state, cancel_state.clone()).await?;

            if cancel_state.is_cancelled() {
                return Err("Stopped".to_string());
            }
            app_handle.emit("ai-agent-step", "complete").ok();
            complete_step(&mut state, cancel_state.clone()).await?;
        }
        AgentType::Writing | AgentType::General => {
            app_handle.emit("ai-agent-step", "analyze").ok();
            analyze_step(&mut state, cancel_state.clone()).await?;

            if cancel_state.is_cancelled() {
                return Err("Stopped".to_string());
            }
            app_handle.emit("ai-agent-step", "execute").ok();
            execute_step(&mut state, cancel_state.clone()).await?;

            if cancel_state.is_cancelled() {
                return Err("Stopped".to_string());
            }
            app_handle.emit("ai-agent-step", "summarize").ok();
            summarize_step(&mut state, cancel_state.clone()).await?;

            if cancel_state.is_cancelled() {
                return Err("Stopped".to_string());
            }
            app_handle.emit("ai-agent-step", "memory").ok();
            memory_step(&mut state, cancel_state.clone()).await?;

            if cancel_state.is_cancelled() {
                return Err("Stopped".to_string());
            }
            app_handle.emit("ai-agent-step", "complete").ok();
            complete_step(&mut state, cancel_state.clone()).await?;
        }
    }

    Ok(())
}

fn prepare_conversation_contents(
    state: &mut AgentState,
    message: String,
    chat_history: Vec<serde_json::Value>,
) {
    // Lấy lịch sử chat (6 tin nhắn gần nhất)
    let historical_context: Vec<&serde_json::Value> = chat_history.iter().rev().take(6).collect();
    for msg in historical_context.into_iter().rev() {
        let role = msg["role"].as_str().unwrap_or("user");
        let content = msg["content"].as_str().unwrap_or("");
        let api_role = if role == "assistant" { "model" } else { role };
        state.contents.push(super::gemini_types::GeminiContent {
            role: api_role.to_string(),
            parts: vec![super::gemini_types::GeminiPart::Text {
                text: content.to_string(),
            }],
        });
    }

    // Đảm bảo message mới nhất có mặt
    let last_msg_in_history = chat_history.last().and_then(|m| m["content"].as_str());
    if last_msg_in_history != Some(&message) {
        state.contents.push(super::gemini_types::GeminiContent {
            role: "user".to_string(),
            parts: vec![super::gemini_types::GeminiPart::Text { text: message }],
        });
    }
}

fn apply_agent_instructions(state: &mut AgentState, agent_type: AgentType) {
    let base_instruction = match agent_type {
        AgentType::Chat => CHAT_AGENT_INSTRUCTIONS,
        AgentType::Ideation => IDEATION_AGENT_INSTRUCTIONS,
        AgentType::Writing => WRITING_AGENT_INSTRUCTIONS,
        AgentType::General => AGENT_INSTRUCTIONS,
    };

    let system_instructions = format!(
        "{base_instruction}\n\n{WIKI_GRAPH_RULES}\n\nHÀNH ĐỘNG: Nếu cần thông tin cốt truyện, hãy đọc Wiki hoặc Chương cũ. Nếu cần thông tin thực tế, hãy dùng Google Search."
    );

    state.system_instruction = Some(super::gemini_types::GeminiContent {
        role: "system".to_string(),
        parts: vec![super::gemini_types::GeminiPart::Text {
            text: system_instructions,
        }],
    });
}
