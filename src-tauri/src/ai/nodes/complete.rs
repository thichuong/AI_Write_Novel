use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart};
use crate::ai::instructions::{
    COMPLETE_PROMPT_GENERAL, COMPLETE_PROMPT_IDEATION, COMPLETE_PROMPT_WRITING,
};
use crate::ai::nodes::{run_agent_loop, AgentState, AgentType};
use tauri::State;

pub async fn complete_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    let complete_prompt = match state.agent_type {
        AgentType::Writing => COMPLETE_PROMPT_WRITING,
        AgentType::Ideation => COMPLETE_PROMPT_IDEATION,
        _ => COMPLETE_PROMPT_GENERAL,
    }
    .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: complete_prompt,
        }],
    });

    run_agent_loop(state, cancel_state, 1, "complete").await?;
    Ok(())
}
