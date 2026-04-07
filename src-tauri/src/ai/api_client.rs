use reqwest::Client;
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use super::gemini_types::{GeminiRequest, GeminiStreamResponse};

/// Lấy API key từ .env
pub fn get_api_key() -> Result<String, String> {
    dotenvy::dotenv().ok();
    std::env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY chưa được cấu hình trong file .env".to_string())
}

/// Mô hình AI đang dùng
pub fn get_model() -> String {
    std::env::var("AI_MODEL").unwrap_or_else(|_| "gemma-4-31b-it".to_string())
}

/// Gửi request và stream response về frontend qua Tauri events
pub async fn stream_gemini_response(
    app_handle: &AppHandle,
    api_key: &str,
    model: &str,
    request: &GeminiRequest,
    event_name: &str,
) -> Result<(), String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}&alt=sse",
        model, api_key
    );

    let client = Client::new();
    let response = client
        .post(&url)
        .json(request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, body));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // SSE format: lines starting with "data: "
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();

            if line.starts_with("data: ") {
                let json_str = &line[6..];
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
                                            if part.thought.unwrap_or(false) {
                                                app_handle.emit(&format!("{}-thought", event_name), text.clone()).ok();
                                            } else {
                                                app_handle.emit(event_name, text.clone()).ok();
                                            }
                                        }
                                        if let Some(fc) = &part.function_call {
                                            // Emit tool call event
                                            app_handle.emit(
                                                &format!("{}-tool", event_name),
                                                serde_json::json!({
                                                    "name": fc.name,
                                                    "args": fc.args,
                                                }),
                                            ).ok();
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
    app_handle.emit(&format!("{}-done", event_name), ()).ok();
    Ok(())
}
