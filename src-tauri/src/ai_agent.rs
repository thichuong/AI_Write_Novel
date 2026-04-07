use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use reqwest::Client;
use futures_util::StreamExt;

use crate::fs_manager;

/// Cấu trúc request gửi lên Gemini API
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDeclaration>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: FunctionCallData,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: FunctionResponseData,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FunctionCallData {
    name: String,
    args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FunctionResponseData {
    name: String,
    response: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
    #[serde(rename = "thinkingConfig", skip_serializing_if = "Option::is_none")]
    thinking_config: Option<ThinkingConfig>,
}

#[derive(Debug, Serialize)]
struct ThinkingConfig {
    #[serde(rename = "thinkingLevel")]
    thinking_level: String,
}

#[derive(Debug, Serialize)]
struct ToolDeclaration {
    #[serde(rename = "functionDeclarations")]
    function_declarations: Vec<FunctionDecl>,
}

#[derive(Debug, Serialize)]
struct FunctionDecl {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

/// Response từ Gemini streaming API
#[derive(Debug, Deserialize)]
struct GeminiStreamResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Option<CandidateContent>,
}

#[derive(Debug, Deserialize)]
struct CandidateContent {
    parts: Option<Vec<CandidatePart>>,
}

#[derive(Debug, Deserialize)]
struct CandidatePart {
    text: Option<String>,
    #[serde(rename = "functionCall")]
    function_call: Option<FunctionCallData>,
}

/// Lấy API key từ .env
fn get_api_key() -> Result<String, String> {
    dotenvy::dotenv().ok();
    std::env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY chưa được cấu hình trong file .env".to_string())
}

/// Mô hình AI đang dùng
fn get_model() -> String {
    std::env::var("AI_MODEL").unwrap_or_else(|_| "gemma-4-31b-it".to_string())
}

/// Tạo danh sách tools mà Agent có thể gọi
fn get_agent_tools() -> Vec<ToolDeclaration> {
    vec![ToolDeclaration {
        function_declarations: vec![
            FunctionDecl {
                name: "read_file".to_string(),
                description: "Đọc nội dung một file trong truyện".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Đường dẫn file relative, ví dụ: Chương/Chương 1.md"
                        }
                    },
                    "required": ["file_path"]
                }),
            },
            FunctionDecl {
                name: "write_file".to_string(),
                description: "Ghi nội dung vào một file trong truyện".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Đường dẫn file relative, ví dụ: Chương/Chương 1.md"
                        },
                        "content": {
                            "type": "string",
                            "description": "Nội dung mới để ghi vào file"
                        }
                    },
                    "required": ["file_path", "content"]
                }),
            },
            FunctionDecl {
                name: "list_files".to_string(),
                description: "Liệt kê tất cả files và folders trong một thư mục".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "directory": {
                            "type": "string",
                            "description": "Đường dẫn thư mục relative, ví dụ: Chương"
                        }
                    },
                    "required": ["directory"]
                }),
            },
        ],
    }]
}

/// Thực thi tool call từ AI
fn execute_tool(
    _app_handle: &AppHandle,
    root_path: &str,
    tool_name: &str,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    match tool_name {
        "read_file" => {
            let file_path = args["file_path"].as_str().unwrap_or("").to_string();
            let content = fs_manager::read_file(
                root_path.to_string(),
                file_path,
            )?;
            Ok(serde_json::json!({"content": content}))
        }
        "write_file" => {
            let file_path = args["file_path"].as_str().unwrap_or("").to_string();
            let content = args["content"].as_str().unwrap_or("").to_string();
            fs_manager::write_file(
                root_path.to_string(),
                file_path.clone(),
                content,
            )?;
            Ok(serde_json::json!({"status": "success", "file": file_path}))
        }
        "list_files" => {
            let directory = args["directory"].as_str().map(|s| s.to_string());
            let nodes = fs_manager::list_nodes(
                root_path.to_string(),
                directory,
            )?;
            Ok(serde_json::to_value(&nodes).unwrap_or(serde_json::json!([])))
        }
        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}

/// Chat với AI (streaming) — dùng cho hỗ trợ sáng tác
#[tauri::command]
pub async fn ai_chat(
    app_handle: AppHandle,
    root_path: String,
    message: String,
    chat_history: Vec<serde_json::Value>,
) -> Result<(), String> {
    let api_key = get_api_key()?;
    let model = get_model();

    // Lấy context từ file system
    let kb_context = fs_manager::get_story_context(root_path.clone())?;

    let system_prompt = format!(
        "Bạn là một trợ lý sáng tác chuyên nghiệp. Hãy sử dụng KIẾN THỨC VỀ TRUYỆN dưới đây để trả lời.\n\n{}\n",
        kb_context
    );

    // Build contents
    let mut contents = vec![GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: system_prompt,
        }],
    },
    GeminiContent {
        role: "model".to_string(),
        parts: vec![GeminiPart::Text {
            text: "Tôi đã nắm được toàn bộ kiến thức về truyện. Hãy hỏi tôi bất cứ điều gì!".to_string(),
        }],
    }];

    // Thêm chat history
    for msg in &chat_history {
        let role = msg["role"].as_str().unwrap_or("user");
        let content = msg["content"].as_str().unwrap_or("");
        // API dùng "model" thay vì "assistant"
        let api_role = if role == "assistant" { "model" } else { role };
        contents.push(GeminiContent {
            role: api_role.to_string(),
            parts: vec![GeminiPart::Text {
                text: content.to_string(),
            }],
        });
    }

    // Thêm message mới
    contents.push(GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart::Text {
            text: message,
        }],
    });

    let request = GeminiRequest {
        contents,
        generation_config: Some(GenerationConfig {
            temperature: 0.8,
            max_output_tokens: 8192,
            thinking_config: Some(ThinkingConfig {
                thinking_level: "HIGH".to_string(),
            }),
        }),
        tools: None, // Chat mode không dùng tools
    };

    // Stream response
    stream_gemini_response(&app_handle, &api_key, &model, &request, "ai-chat-stream").await
}

/// AI viết truyện (streaming) — có function calling
#[tauri::command]
pub async fn ai_write(
    app_handle: AppHandle,
    root_path: String,
    current_file: String,
    instruction: String,
    current_content: String,
    selection: Option<String>,
) -> Result<(), String> {
    let api_key = get_api_key()?;
    let model = get_model();

    // Lấy context
    let kb_context = fs_manager::get_story_context(root_path.clone())?;
    let prev_chapters = fs_manager::get_previous_chapters(
        root_path.clone(),
        current_file.clone(),
    )?;

    let mut full_context = kb_context;
    if !prev_chapters.is_empty() {
        full_context.push_str(&format!("\n# TÓM TẮT CÁC CHƯƠNG TRƯỚC\n{}\n", prev_chapters));
    }
    full_context.push_str(&format!("\n# NỘI DUNG HIỆN TẠI ({})\n{}\n", current_file, current_content));

    let system_prompt = format!(
        "Bạn là nhà văn chuyên nghiệp. Hãy viết tiếp hoặc sửa đổi dựa trên các kiến thức và chỉ dẫn sau.\n\
         Tuyệt đối bám sát các Quy tắc, Nhân vật và thông tin bối cảnh đã cung cấp.\n\n\
         {}\n",
        full_context
    );

    let user_prompt = if let Some(sel) = &selection {
        format!(
            "Phần văn bản được chọn: \"{}\"\n\nChỉ dẫn: {}\n\nChỉ trả về nội dung mới, không giải thích.",
            sel, instruction
        )
    } else {
        format!(
            "Chỉ dẫn viết tiếp: {}\n\nChỉ trả về nội dung mới, không giải thích.",
            instruction
        )
    };

    let contents = vec![
        GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text {
                text: system_prompt,
            }],
        },
        GeminiContent {
            role: "model".to_string(),
            parts: vec![GeminiPart::Text {
                text: "Tôi đã nắm được toàn bộ ngữ cảnh. Sẵn sàng viết.".to_string(),
            }],
        },
        GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text {
                text: user_prompt,
            }],
        },
    ];

    let request = GeminiRequest {
        contents,
        generation_config: Some(GenerationConfig {
            temperature: 0.9,
            max_output_tokens: 16384,
            thinking_config: Some(ThinkingConfig {
                thinking_level: "HIGH".to_string(),
            }),
        }),
        tools: None, // Writing mode — trả text trực tiếp để user preview
    };

    stream_gemini_response(&app_handle, &api_key, &model, &request, "ai-write-stream").await
}

/// Gửi request và stream response về frontend qua Tauri events
async fn stream_gemini_response(
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
                                            app_handle.emit(event_name, text.clone()).ok();
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
