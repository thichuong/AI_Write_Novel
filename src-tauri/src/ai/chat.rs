use super::api_client::{get_api_key, get_model};
use super::nodes::{
    analyze::analyze_step, complete::complete_step, execute::execute_step, memory::memory_step,
    summarize::summarize_step, run_agent_loop, AgentState,
};
use super::router::{classify_intent, AgentType};
use tauri::{AppHandle, Emitter};

use super::instructions::{
    AGENT_INSTRUCTIONS, CHAT_AGENT_INSTRUCTIONS, IDEATION_AGENT_INSTRUCTIONS,
    WIKI_GRAPH_RULES, WRITING_AGENT_INSTRUCTIONS,
};

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

    // 1. Phân loại ý định (Router)
    app_handle.emit("ai-agent-step", "routing").ok();
    let agent_type = classify_intent(&api_key, &model, &message).await.unwrap_or(AgentType::General);
    
    // Thông báo cho UI biết đang dùng Agent nào
    app_handle.emit("ai-agent-selected", agent_type.as_str()).ok();

    // 2. Khởi tạo State
    let mut state = AgentState {
        app_handle: app_handle.clone(),
        root_path: root_path.clone(),
        api_key,
        model,
        system_instruction: None,
        contents: Vec::new(),
        loop_count: 0,
    };

    // 3. Setup Context dựa trên AgentType
    setup_initial_context(&mut state, message, chat_history, agent_type);

    // 4. Luồng xử lý theo Agent
    match agent_type {
        AgentType::Chat => {
            app_handle.emit("ai-agent-step", "chatting").ok();
            // Chat Agent giờ đây cũng có thể dùng Tool (Search, Read File)
            run_agent_loop(&mut state, 3, "chat").await?;
        }
        AgentType::Ideation => {
            // Quy trình rút gọn cho Ideation: Analyze -> Execute (Brainstorm) -> Summarize
            app_handle.emit("ai-agent-step", "analyze").ok();
            analyze_step(&mut state).await?;

            app_handle.emit("ai-agent-step", "ideate").ok();
            execute_step(&mut state).await?;

            app_handle.emit("ai-agent-step", "summarize").ok();
            summarize_step(&mut state).await?;
        }
        AgentType::Writing | AgentType::General => {
            // Quy trình Đa bước đầy đủ (State Machine)
            app_handle.emit("ai-agent-step", "analyze").ok();
            analyze_step(&mut state).await?;

            app_handle.emit("ai-agent-step", "execute").ok();
            execute_step(&mut state).await?;

            app_handle.emit("ai-agent-step", "summarize").ok();
            summarize_step(&mut state).await?;

            app_handle.emit("ai-agent-step", "memory").ok();
            memory_step(&mut state).await?;

            app_handle.emit("ai-agent-step", "complete").ok();
            complete_step(&mut state).await?;
        }
    }

    Ok(())
}

fn setup_initial_context(
    state: &mut AgentState,
    message: String,
    chat_history: Vec<serde_json::Value>,
    agent_type: AgentType,
) {
    let base_instruction = match agent_type {
        AgentType::Chat => CHAT_AGENT_INSTRUCTIONS,
        AgentType::Ideation => IDEATION_AGENT_INSTRUCTIONS,
        AgentType::Writing => WRITING_AGENT_INSTRUCTIONS,
        AgentType::General => AGENT_INSTRUCTIONS,
    };

    let system_instructions = format!(
        "{}\n\n{}\n\nHÀNH ĐỘNG: Nếu cần thông tin cốt truyện, hãy đọc Wiki hoặc Chương cũ. Nếu cần thông tin thực tế, hãy dùng Google Search.",
        base_instruction, WIKI_GRAPH_RULES
    );

    state.system_instruction = Some(super::gemini_types::GeminiContent {
        role: "system".to_string(),
        parts: vec![super::gemini_types::GeminiPart::Text {
            text: system_instructions,
        }],
    });

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
