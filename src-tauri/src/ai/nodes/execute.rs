use crate::ai::cancellation::CancellationState;
use crate::ai::gemini_types::{GeminiContent, GeminiPart};
use crate::ai::instructions::{
    EXECUTE_PROMPT_GENERAL, EXECUTE_PROMPT_IDEATION, EXECUTE_PROMPT_WRITING,
};
use crate::ai::nodes::{run_agent_loop, AgentState, AgentType};
use tauri::State;

pub async fn execute_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    let execute_prompt = match state.agent_type {
        AgentType::Writing => EXECUTE_PROMPT_WRITING,
        AgentType::Ideation => EXECUTE_PROMPT_IDEATION,
        _ => EXECUTE_PROMPT_GENERAL,
    }
    .to_string();

    state.contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: execute_prompt,
        }],
    });

    run_agent_loop(state, cancel_state, 10, "execute").await?;
    Ok(())
}
