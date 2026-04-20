pub use super::cancellation::CancellationState;
use super::gemini_types::{GeminiRequest, GeminiStreamResponse};
use crate::error::{AppError, AppResult};
use futures_util::StreamExt;
use reqwest::Client;
use std::fs;
use tauri::{AppHandle, Emitter, State};

/// Lấy API key từ .env
pub fn get_api_key() -> AppResult<String> {
    dotenvy::dotenv().ok();
    std::env::var("GEMINI_API_KEY")
        .map_err(|_| AppError::Env("GEMINI_API_KEY chưa được cấu hình trong file .env".to_string()))
}

#[tauri::command]
pub fn check_api_key() -> bool {
    get_api_key().is_ok()
}

#[tauri::command]
pub fn get_settings() -> serde_json::Value {
    serde_json::json!({
        "api_key": std::env::var("GEMINI_API_KEY").unwrap_or_default(),
        "model": get_model(),
    })
}

#[tauri::command]
pub async fn list_models() -> AppResult<Vec<String>> {
    Ok(vec![
        "gemma-4-31b-it".to_string(),
        "gemma-4-26b-a4b-it".to_string(),
    ])
}

#[tauri::command]
pub fn save_settings(api_key: String, model: String) -> AppResult<()> {
    // 1. Lưu vào biến môi trường hiện tại
    std::env::set_var("GEMINI_API_KEY", &api_key);
    std::env::set_var("AI_MODEL", &model);

    // 2. Cập nhật file .env
    let path = ".env";
    let content = if std::path::Path::new(path).exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let mut key_found = false;
    let mut model_found = false;

    for line in &mut lines {
        if line.starts_with("GEMINI_API_KEY=") {
            *line = format!("GEMINI_API_KEY={api_key}");
            key_found = true;
        } else if line.starts_with("AI_MODEL=") {
            *line = format!("AI_MODEL={model}");
            model_found = true;
        }
    }

    if !key_found {
        lines.push(format!("GEMINI_API_KEY={api_key}"));
    }
    if !model_found {
        lines.push(format!("AI_MODEL={model}"));
    }

    let mut final_content = lines.join("\n");
    if !final_content.is_empty() {
        final_content.push('\n');
    }
    fs::write(path, final_content)?;

    Ok(())
}

#[tauri::command]
pub fn save_api_key(api_key: String) -> AppResult<()> {
    save_settings(api_key, get_model())
}

/// Mô hình AI đang dùng
pub fn get_model() -> String {
    std::env::var("AI_MODEL").unwrap_or_else(|_| "gemma-4-31b-it".to_string())
}

/// Gửi request và stream response về frontend, đồng thời trả về toàn bộ các Parts đã nhận được
pub async fn stream_gemini_response(
    app_handle: &AppHandle,
    cancel_state: State<'_, CancellationState>,
    api_key: &str,
    model: &str,
    request: &GeminiRequest,
    event_name: &str,
    phase: &str,
) -> AppResult<Vec<super::gemini_types::GeminiPart>> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent?key={api_key}&alt=sse"
    );

    let client = Client::new();
    let response = client
        .post(&url)
        .json(request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Api { status, body });
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut accumulated_parts: Vec<super::gemini_types::GeminiPart> = Vec::new();

    while let Some(chunk_result) = stream.next().await {
        if cancel_state.is_cancelled() {
            return Err(AppError::Cancelled("Agent stopped by user".to_string()));
        }
        let chunk = chunk_result?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // SSE format: lines starting with "data: "
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer.drain(..=pos);

            if let Some(json_str) = line.strip_prefix("data: ") {
                let json_str = json_str.trim();
                if json_str == "[DONE]" {
                    continue;
                }

                if let Ok(response) = serde_json::from_str::<GeminiStreamResponse>(json_str) {
                    if let Some(candidates) = response.candidates {
                        for candidate in candidates {
                            if let Some(content) = candidate.content {
                                if let Some(parts) = content.parts {
                                    for part in parts {
                                        if let Some(text) = &part.text {
                                            // Stream text to UI with phase context
                                            let target_event = if part.thought.unwrap_or(false) {
                                                format!("{event_name}-thought")
                                            } else {
                                                event_name.to_string()
                                            };

                                            app_handle
                                                .emit(
                                                    &target_event,
                                                    serde_json::json!({ "text": text, "phase": phase }),
                                                )
                                                .ok();

                                            // Accumulate text parts
                                            if let Some(super::gemini_types::GeminiPart::Text {
                                                text: last_text,
                                            }) = accumulated_parts.last_mut()
                                            {
                                                last_text.push_str(text);
                                            } else {
                                                accumulated_parts.push(
                                                    super::gemini_types::GeminiPart::Text {
                                                        text: text.clone(),
                                                    },
                                                );
                                            }
                                        }

                                        if let Some(fc) = part.function_call {
                                            // Emit tool call event so UI knows what's happening
                                            app_handle
                                                .emit(
                                                    &format!("{event_name}-tool"),
                                                    serde_json::json!({
                                                        "name": fc.name,
                                                        "args": fc.args,
                                                        "phase": phase,
                                                    }),
                                                )
                                                .ok();

                                            // Accumulate function call
                                            accumulated_parts.push(
                                                super::gemini_types::GeminiPart::FunctionCall {
                                                    function_call: fc,
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(accumulated_parts)
}
