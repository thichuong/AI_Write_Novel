use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart};
use crate::ai::instructions::{
    FINALIZE_PROMPT_GENERAL, FINALIZE_PROMPT_IDEATION, FINALIZE_PROMPT_WRITING,
};
use crate::ai::nodes::{run_agent_loop, AgentState, AgentType};
use tauri::State;

pub async fn finalize_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    let mut finalize_prompt = match state.agent_type {
        AgentType::Writing => FINALIZE_PROMPT_WRITING,
        AgentType::Ideation => FINALIZE_PROMPT_IDEATION,
        _ => FINALIZE_PROMPT_GENERAL,
    }
    .to_string();

    // Kiểm tra xem đã có thao tác lưu file vào thư mục chapters/ chưa
    if state.agent_type == AgentType::Writing {
        let has_saved_chapter = state.contents.iter().any(|msg| {
            msg.parts.iter().any(|part| {
                if let GeminiPart::FunctionCall { function_call } = part {
                    if function_call.name == "write_file" {
                        if let Some(path) = function_call.args["path"].as_str() {
                            return path.contains("chapters/");
                        }
                    }
                }
                false
            })
        });

        if !has_saved_chapter {
            finalize_prompt.push_str("\n\n⚠️ CẢNH BÁO: Bạn chưa thực hiện lưu nội dung vào file trong thư mục 'chapters/'. Hãy sử dụng tool 'write_file' để lưu chương truyện, sau đó dùng tool 'list_directory' hoặc 'read_file' để xác nhận chắc chắn rằng file đã tồn tại trên đĩa.");
        }
    }

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: finalize_prompt,
        }],
    });

    run_agent_loop(state, cancel_state, 5, "finalize").await?;
    Ok(())
}
