use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart};
use crate::ai::instructions::{
    ANALYZE_PROMPT_GENERAL, ANALYZE_PROMPT_IDEATION, ANALYZE_PROMPT_WRITING,
};
use crate::ai::nodes::{run_agent_loop, AgentState, AgentType};
use crate::ai::tools::{tool_list_directory, tool_read_file, tool_wiki_list_entities};
use serde_json::json;
use tauri::{Emitter, State};

pub async fn analyze_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    // 1. Thu thập tri thức tự động từ backend (Trước khi AI chạy để giảm số lượt gọi tool)
    let dir_context =
        tool_list_directory(&state.root_path, ".").unwrap_or_else(|e| format!("Lỗi liệt kê: {e}"));
    let memory_context = tool_read_file(&state.root_path, "memory.md")
        .unwrap_or_else(|_| "Chưa có file memory.md hoặc file trống.".to_string());
    let wiki_context =
        tool_wiki_list_entities(&state.root_path).unwrap_or_else(|e| format!("Lỗi Wiki: {e}"));

    let agent_specific_guidance = match state.agent_type {
        AgentType::Writing => ANALYZE_PROMPT_WRITING,
        AgentType::Ideation => ANALYZE_PROMPT_IDEATION,
        _ => ANALYZE_PROMPT_GENERAL,
    };

    let analyze_prompt = format!(
        "YÊU CẦU PHÂN TÍCH VÀ NẠP KIẾN THỨC (KHÔNG VIẾT TRUYỆN):\n\
        {agent_specific_guidance}\n\n\
        ### CẤU TRÚC THƯ MỤC:\n{dir_context}\n\n\
        ### NỘI DUNG MEMORY.MD:\n{memory_context}\n\n\
        ### DANH SÁCH WIKI:\n{wiki_context}\n\n\
        HÀNH ĐỘNG: Nếu cần tìm hiểu thêm thông tin để phục vụ lập kế hoạch, hãy dùng `google_search` hoặc `read_file`."
    );

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: analyze_prompt,
        }],
    });

    state
        .app_handle
        .emit(
            "ai-chat-stream-thought",
            json!({
                "phase": "analyze",
                "text": "Đang phân tích bối cảnh và nghiên cứu dữ liệu (Tool-enabled)...\n"
            }),
        )
        .ok();

    // Dùng run_agent_loop để hỗ trợ Tool calling (Search, Read File) trong bước Analyze
    run_agent_loop(state, cancel_state, 3, "analyze", true).await?;

    process_analyze_feedback(state);

    Ok(())
}

fn process_analyze_feedback(state: &AgentState) {
    let Some(last_msg) = state.contents.last() else {
        return;
    };

    let full_text = last_msg
        .parts
        .iter()
        .filter_map(|p| {
            if let GeminiPart::Text { text } = p {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<String>();

    if let Some(json_text) = crate::ai::nodes::extract_json_block(&full_text) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_text) {
            let thought = parsed["thought_process"].as_str().unwrap_or("");
            let status = parsed["status_check"].as_str().unwrap_or("");
            let plan = parsed["plan"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|v| format!("• {}", v.as_str().unwrap_or("-")))
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();

            state
                .app_handle
                .emit(
                    "ai-chat-stream-thought",
                    json!({
                        "phase": "analyze",
                        "text": format!("\n💡 **Phân tích**: {thought}\n🧐 **Đánh giá**: {status}\n\n📋 **Kế hoạch hành động**:\n{plan}\n")
                    }),
                )
                .ok();
        }
    }
}
