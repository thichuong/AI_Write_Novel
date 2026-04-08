use super::api_client::{get_api_key, get_model, stream_gemini_response};
use super::gemini_types::{
    FunctionResponseData, GeminiContent, GeminiPart, GeminiRequest, GenerationConfig,
    ThinkingConfig,
};
use super::tools;
use tauri::AppHandle;

/// Chat với AI (streaming) — dùng kiến trúc Agentic chủ động với cơ chế Memory Ground Truth
#[tauri::command]
pub async fn ai_chat(
    app_handle: AppHandle,
    root_path: String,
    _current_file: String,
    message: String,
    chat_history: Vec<serde_json::Value>,
) -> Result<(), String> {
    let api_key = get_api_key()?;
    let model = get_model();

    let system_prompt = "Bạn là AI Novelist Agent. Bạn hoạt động trên cơ chế 'Memory Ground Truth'.\n\n\
        QUY TẮC QUAN TRỌNG NHẤT:\n\
        1. BỘ NHỚ LÀ TRÊN HẾT: Bạn KHÔNG phụ thuộc vào lịch sử chat cũ. Mỗi khi bắt đầu một yêu cầu mới, bạn PHẢI dùng `read_file('memory.md')` và `list_directory('.')` để nắm bắt tình hình dự án. Đây là bước BẮT BUỘC.\n\
        2. QUY TRÌNH LÀM VIỆC:\n\
           - Bước 1: Gọi tool để đọc `memory.md`, danh sách file, và các chương/nhân vật liên quan.\n\
           - Bước 2: Thực hiện yêu cầu của người dùng (viết truyện, sửa đổi, tóm tắt...).\n\
           - Bước 3: Trước khi kết thúc và trả lời người dùng, bạn PHẢI dùng `write_file` để cập nhật lại `memory.md` với các thay đổi mới nhất (diễn biến, trạng thái nhân vật).\n\
        3. CÔNG CỤ: Bạn có toàn quyền dùng Tools. Nếu chưa rõ thông tin, hãy dùng `read_file` để tìm hiểu thay vì đoán.\n\
        4. NGÔN NGỮ: Trả lời bằng tiếng Việt, ngắn gọn, súc tích sau khi đã hoàn thành các tác vụ tool.\n\
        5. NHẤT QUÁN: Đảm bảo mọi thay đổi đều được ghi nhận vào hệ thống file để bảo tồn ngữ cảnh cho lần chat sau.".to_string();

    // Khởi tạo danh sách tin nhắn
    let mut contents = vec![
        GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text {
                text: system_prompt,
            }],
        },
        GeminiContent {
            role: "model".to_string(),
            parts: vec![GeminiPart::Text {
                text: "Tôi đã hiểu. Tôi sẽ luôn bắt đầu bằng việc đọc memory.md và kết thúc bằng việc cập nhật bộ nhớ này để đảm bảo tính nhất quán cho dự án của bạn.".to_string(),
            }],
        },
    ];

    // Chỉ lấy 2 tin nhắn gần nhất từ history để giữ ngữ cảnh sạch (chỉ tập trung vào turn hiện tại)
    // Người dùng muốn history chỉ có tác dụng trong 1 lần chat.
    let historical_context: Vec<&serde_json::Value> = chat_history.iter().rev().take(2).collect();
    for msg in historical_context.into_iter().rev() {
        let role = msg["role"].as_str().unwrap_or("user");
        let content = msg["content"].as_str().unwrap_or("");
        let api_role = if role == "assistant" { "model" } else { role };
        contents.push(GeminiContent {
            role: api_role.to_string(),
            parts: vec![GeminiPart::Text {
                text: content.to_string(),
            }],
        });
    }

    // Thêm message mới của user (nếu chưa có trong take(2) - thực tế addChatMessage đã đẩy vào history)
    // Để an toàn, nếu history rỗng hoặc tin nhắn cuối không khớp, ta thêm thủ công.
    let last_msg_in_history = chat_history.last().and_then(|m| m["content"].as_str());
    if last_msg_in_history != Some(&message) {
        contents.push(GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text { text: message }],
        });
    }

    let tool_decls = tools::get_tool_declarations();
    let generation_config = GenerationConfig {
        temperature: 0.7,
        max_output_tokens: 8192,
        thinking_config: Some(ThinkingConfig {
            thinking_level: "HIGH".to_string(),
        }),
    };

    let mut loop_count = 0;
    const MAX_LOOPS: u32 = 15;

    loop {
        loop_count += 1;
        if loop_count > MAX_LOOPS {
            return Err("Agent đã vượt quá giới hạn vòng lặp (Max Loops).".to_string());
        }

        let request = GeminiRequest {
            contents: contents.clone(),
            generation_config: Some(GenerationConfig {
                temperature: generation_config.temperature,
                max_output_tokens: generation_config.max_output_tokens,
                thinking_config: generation_config.thinking_config.as_ref().map(|tc| {
                    ThinkingConfig {
                        thinking_level: tc.thinking_level.clone(),
                    }
                }),
            }),
            tools: Some(tool_decls.clone()),
        };

        let parts =
            stream_gemini_response(&app_handle, &api_key, &model, &request, "ai-chat-stream")
                .await?;

        contents.push(GeminiContent {
            role: "model".to_string(),
            parts: parts.clone(),
        });

        let mut function_calls = Vec::new();
        for part in &parts {
            if let GeminiPart::FunctionCall { function_call } = part {
                function_calls.push(function_call.clone());
            }
        }

        if function_calls.is_empty() {
            // Kiểm tra xem đã gọi tool write memory.md chưa?
            // Nếu model trả về Text mà chưa thấy gọi write_file('memory.md') trong history của turn này,
            // ta có thể coi là nó chưa hoàn thành quy tắc.
            // Nhưng để đơn giản và tránh loop vô tận, ta tin tưởng vào Prompt.
            break;
        }

        let mut response_parts = Vec::new();
        for fc in function_calls {
            let tool_result = match fc.name.as_str() {
                "list_directory" => {
                    let path = fc.args["path"].as_str().unwrap_or(".");
                    tools::tool_list_directory(&root_path, path)
                }
                "read_file" => {
                    let path = fc.args["path"].as_str().unwrap_or("");
                    tools::tool_read_file(&root_path, path)
                }
                "write_file" => {
                    let path = fc.args["path"].as_str().unwrap_or("");
                    let content = fc.args["content"].as_str().unwrap_or("");
                    tools::tool_write_file(&app_handle, &root_path, path, content)
                }
                "delete_file" => {
                    let path = fc.args["path"].as_str().unwrap_or("");
                    tools::tool_delete_file(&app_handle, &root_path, path)
                }
                _ => Err(format!("Công cụ không tồn tại: {}", fc.name)),
            };

            let response_json = match tool_result {
                Ok(res) => serde_json::json!({ "result": res }),
                Err(err) => serde_json::json!({ "error": err }),
            };

            response_parts.push(GeminiPart::FunctionResponse {
                function_response: FunctionResponseData {
                    name: fc.name.clone(),
                    response: response_json,
                },
            });
        }

        contents.push(GeminiContent {
            role: "function".to_string(),
            parts: response_parts,
        });
    }

    Ok(())
}
