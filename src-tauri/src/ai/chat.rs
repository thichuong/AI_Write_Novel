#![allow(clippy::too_many_arguments, clippy::format_push_string)]
use super::api_client::{get_api_key, get_model};
use super::cancellation::CancellationState;
use super::nodes::{
    finalize::finalize_step, run_agent_loop, thinking::thinking_step, AgentState, AgentType,
};
use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

use super::instructions::{
    AGENT_INSTRUCTIONS, CHAT_AGENT_INSTRUCTIONS, IDEATION_AGENT_INSTRUCTIONS, NAMING_RULES,
    WIKI_GRAPH_RULES, WRITING_AGENT_INSTRUCTIONS,
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
    selected_knowledge_files: Vec<String>,
    agent_type: String,
) -> AppResult<()> {
    let api_key = get_api_key()?;
    let model = get_model();

    // Safely read selected knowledge files without unwrap/expect
    let mut knowledge_context = String::new();
    for file_path_str in &selected_knowledge_files {
        let file_path = PathBuf::from(file_path_str);
        if file_path.exists() && file_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                let file_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown File");

                // Construct file context block format
                knowledge_context.push_str("\n\n--- KNOWLEDGE FILE: ");
                knowledge_context.push_str(file_name);
                knowledge_context.push_str(" ---\n");
                knowledge_context.push_str(&content);
                knowledge_context.push_str("\n--- END OF FILE ---\n");
            }
        }
    }

    let selected_agent = match agent_type.to_lowercase().as_str() {
        "chat" => AgentType::Chat,
        "ideation" | "ide" => AgentType::Ideation,
        "writing" | "writting" => AgentType::Writing,
        _ => AgentType::General,
    };

    // 1. Initialize State with basic context
    let mut state = AgentState {
        app_handle: app_handle.clone(),
        root_path: PathBuf::from(root_path),
        api_key,
        model,
        agent_type: selected_agent,
        system_instruction: None,
        contents: Vec::new(),
        loop_count: 0,
        last_chapter_content: String::new(),
        last_saved_file: String::new(),
        last_word_count: 0,
        last_wiki_updates_count: 0,
        selected_files_content: knowledge_context,
    };

    // Prepare conversation contents (history + new message)
    prepare_conversation_contents(&mut state, message, chat_history);

    cancel_state.reset();

    // 2. Notify agent selection
    app_handle.emit("ai-agent-step", "coordinating").ok();
    app_handle
        .emit("ai-agent-selected", selected_agent.as_ref())
        .ok();
    app_handle
        .emit(
            "ai-chat-stream-thought",
            serde_json::json!({
                "phase": "coordinating",
                "text": format!("=> Đã chọn Agent chuyên biệt: {}\n", selected_agent.description())
            }),
        )
        .ok();

    // 3. Setup system instructions with automated context injection
    apply_agent_instructions(&mut state, selected_agent);

    // 4. Streamlined Agent Execution Flows
    match selected_agent {
        AgentType::Chat => {
            app_handle.emit("ai-agent-step", "chatting").ok();
            run_agent_loop(&mut state, cancel_state.clone(), 3, "complete", true).await?;
        }
        AgentType::Ideation => {
            // Ide Mode: Runs in 1 optimized step with tools enabled (allows updating Wiki and memory)
            app_handle.emit("ai-agent-step", "thinking").ok();
            run_agent_loop(&mut state, cancel_state.clone(), 5, "complete", true).await?;
        }
        AgentType::Writing | AgentType::General => {
            // Writing Mode: Runs in 2 sequential steps (Writing -> Sync & Finalize)
            // Step 1: Creative Writing
            app_handle.emit("ai-agent-step", "thinking").ok();
            thinking_step(&mut state, cancel_state.clone()).await?;

            if cancel_state.is_cancelled() {
                return Err(AppError::Cancelled("Stopped".to_string()));
            }

            // Step 2: Auto-Sync Wiki & Memory
            app_handle.emit("ai-agent-step", "finalize").ok();
            finalize_step(&mut state, cancel_state.clone()).await?;

            if cancel_state.is_cancelled() {
                return Err(AppError::Cancelled("Stopped".to_string()));
            }

            // Finish
            app_handle.emit("ai-agent-step", "complete").ok();
        }
    }

    Ok(())
}

fn prepare_conversation_contents(
    state: &mut AgentState,
    message: String,
    chat_history: Vec<serde_json::Value>,
) {
    // Keep only the 6 most recent messages
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

    // Ensure the latest message is appended
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

    let mut system_instructions = format!(
        "{base_instruction}\n\n{NAMING_RULES}\n\n{WIKI_GRAPH_RULES}\n\nHÀNH ĐỘNG: Nếu cần thông tin cốt truyện, hãy đọc Wiki hoặc Chương cũ. Nếu cần thông tin thực tế, hãy dùng Google Search."
    );

    // Auto-inject project memory (memory.md) if exists
    let memory_path = state.root_path.join("memory.md");
    if memory_path.exists() && memory_path.is_file() {
        if let Ok(memory_content) = std::fs::read_to_string(&memory_path) {
            let trimmed = memory_content.trim();
            if !trimmed.is_empty() {
                system_instructions.push_str("\n\n--- PROJECT MEMORY (memory.md) ---\n");
                system_instructions.push_str(trimmed);
                system_instructions.push_str("\n--- END OF PROJECT MEMORY ---\n");
            }
        }
    }

    // Proactively auto-inject project directory structure (Chapter list)
    if let Ok(dir_list) = super::tools::tool_list_directory(&state.root_path, ".") {
        system_instructions.push_str("\n\n--- PROJECT DIRECTORY STRUCTURE ---\n");
        system_instructions.push_str(&dir_list);
        system_instructions.push_str("\n--- END OF DIRECTORY STRUCTURE ---\n");
    }

    // Proactively auto-inject the list of all available Wiki Entities
    if let Ok(wiki_list) = super::tools::tool_wiki_list_entities(&state.root_path) {
        system_instructions.push_str("\n\n");
        system_instructions.push_str(&wiki_list);
        system_instructions.push_str("\n--- END OF WIKI ---\n");
    }

    // Append user-selected knowledge context if present
    if !state.selected_files_content.is_empty() {
        system_instructions.push_str(&format!(
            "\n\n--- SELECTED KNOWLEDGE FILES ---\nYou MUST prioritize using the following information from the files selected by the user to answer the query:\n{}",
            state.selected_files_content
        ));
    }

    state.system_instruction = Some(super::gemini_types::GeminiContent {
        role: "system".to_string(),
        parts: vec![super::gemini_types::GeminiPart::Text {
            text: system_instructions,
        }],
    });
}
