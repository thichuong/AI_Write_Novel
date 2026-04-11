use super::{run_agent_loop, AgentState};
use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart};
use crate::ai::tools::{tool_list_directory, tool_read_file, tool_wiki_list_entities};
use tauri::State;

pub async fn analyze_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    // 1. Thu thập tri thức tự động từ backend
    let dir_context = tool_list_directory(&state.root_path, ".")
        .unwrap_or_else(|e| format!("Lỗi liệt kê: {e}"));
    let memory_context = tool_read_file(&state.root_path, "memory.md")
        .unwrap_or_else(|_| "Chưa có file memory.md hoặc file trống.".to_string());
    let wiki_context =
        tool_wiki_list_entities(&state.root_path).unwrap_or_else(|e| format!("Lỗi Wiki: {e}"));

    let analyze_prompt = format!(
        "PHÂN TÍCH VÀ NẠP KIẾN THỨC:\n\
        Dưới đây là tri thức về dự án tôi đã nạp sẵn cho bạn:\n\n\
        ### CẤU TRÚC THƯ MỤC:\n{dir_context}\n\n\
        ### NỘI DUNG MEMORY.MD:\n{memory_context}\n\n\
        ### DANH SÁCH WIKI:\n{wiki_context}\n\n\
        Nhiệm vụ của bạn:\n\
        1. Xem xét các thông tin trên và đối chiếu với yêu cầu của người dùng.\n\
        2. Nếu cần đọc chi tiết một file cụ thể hoặc Wiki cụ thể để giải quyết yêu cầu, hãy gọi công cụ tương ứng.\n\
        3. Sau đó, lập kế hoạch chi tiết để thực hiện. Nếu mục tiêu đơn giản, có thể thông báo kế hoạch và bắt đầu thực hiện luôn."
    );

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: analyze_prompt,
        }],
    });

    run_agent_loop(state, cancel_state, 2, "analyze").await?;
    Ok(())
}
