use super::gemini_types::{GeminiRequest, GeminiStreamResponse};
use futures_util::StreamExt;
use reqwest::Client;
use tauri::{AppHandle, Emitter};

/// Lấy API key từ .env
pub fn get_api_key() -> Result<String, String> {
    dotenvy::dotenv().ok();
    std::env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY chưa được cấu hình trong file .env".to_string())
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
pub async fn list_models() -> Result<Vec<String>, String> {
    Ok(vec![
        "gemma-4-31b-it".to_string(),
        "gemma-4-26b-a4b-it".to_string(),
    ])
}

#[tauri::command]
pub fn save_settings(api_key: String, model: String) -> Result<(), String> {
    use std::fs;
    
    // 1. Lưu vào biến môi trường hiện tại
    std::env::set_var("GEMINI_API_KEY", &api_key);
    std::env::set_var("AI_MODEL", &model);

    // 2. Cập nhật file .env
    let path = ".env";
    let content = if std::path::Path::new(path).exists() {
        fs::read_to_string(path).map_err(|e| format!("Không thể đọc file .env: {e}"))?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut key_found = false;
    let mut model_found = false;

    for line in lines.iter_mut() {
        if line.starts_with("GEMINI_API_KEY=") {
            *line = format!("GEMINI_API_KEY={}", api_key);
            key_found = true;
        } else if line.starts_with("AI_MODEL=") {
            *line = format!("AI_MODEL={}", model);
            model_found = true;
        }
    }

    if !key_found {
        lines.push(format!("GEMINI_API_KEY={}", api_key));
    }
    if !model_found {
        lines.push(format!("AI_MODEL={}", model));
    }

    fs::write(path, lines.join("\n") + "\n").map_err(|e| format!("Không thể ghi file .env: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn save_api_key(api_key: String) -> Result<(), String> {
    save_settings(api_key, get_model())
}

/// Mô hình AI đang dùng
pub fn get_model() -> String {
    std::env::var("AI_MODEL").unwrap_or_else(|_| "gemma-4-31b-it".to_string())
}

/// Gửi request và stream response về frontend, đồng thời trả về toàn bộ các Parts đã nhận được
pub async fn stream_gemini_response(
    app_handle: &AppHandle,
    api_key: &str,
    model: &str,
    request: &GeminiRequest,
    event_name: &str,
    phase: &str,
) -> Result<Vec<super::gemini_types::GeminiPart>, String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent?key={api_key}&alt=sse"
    );

    let client = Client::new();
    let response = client
        .post(&url)
        .json(request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error {status}: {body}"));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut accumulated_parts: Vec<super::gemini_types::GeminiPart> = Vec::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream error: {e}"))?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // SSE format: lines starting with "data: "
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();

            if let Some(json_str) = line.strip_prefix("data: ") {
                if json_str.trim() == "[DONE]" {
                    continue;
                }

                if let Ok(response) = serde_json::from_str::<GeminiStreamResponse>(json_str) {
                    if let Some(candidates) = &response.candidates {
                        for candidate in candidates {
                            if let Some(content) = &candidate.content {
                                if let Some(parts) = &content.parts {
                                    for part in parts {
                                        if let Some(text) = &part.text {
                                            // Stream text to UI with phase context
                                            if part.thought.unwrap_or(false) {
                                                app_handle
                                                    .emit(
                                                        &format!("{event_name}-thought"),
                                                        serde_json::json!({ "text": text, "phase": phase }),
                                                    )
                                                    .ok();
                                            } else {
                                                app_handle
                                                    .emit(
                                                        event_name,
                                                        serde_json::json!({ "text": text, "phase": phase }),
                                                    )
                                                    .ok();
                                            }

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

                                        if let Some(fc) = &part.function_call {
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
                                                    function_call: fc.clone(),
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

    // Phát sự kiện kết thúc
    app_handle
        .emit(
            &format!("{event_name}-done"),
            serde_json::json!({ "phase": phase }),
        )
        .ok();
    Ok(accumulated_parts)
}
