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
    let finalize_prompt = match state.agent_type {
        AgentType::Writing => FINALIZE_PROMPT_WRITING,
        AgentType::Ideation => FINALIZE_PROMPT_IDEATION,
        _ => FINALIZE_PROMPT_GENERAL,
    }
    .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: finalize_prompt,
        }],
    });

    run_agent_loop(state, cancel_state, 5, "finalize").await?;
    Ok(())
}
