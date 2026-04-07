use tauri::AppHandle;
use super::gemini_types::{ToolDeclaration, FunctionDecl};
use crate::fs;

/// Tạo danh sách tools mà Agent có thể gọi
pub fn get_agent_tools() -> Vec<ToolDeclaration> {
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
pub fn execute_tool(
    _app_handle: &AppHandle,
    root_path: &str,
    tool_name: &str,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    match tool_name {
        "read_file" => {
            let file_path = args["file_path"].as_str().unwrap_or("").to_string();
            let content = fs::read_file(
                root_path.to_string(),
                file_path,
            )?;
            Ok(serde_json::json!({"content": content}))
        }
        "write_file" => {
            let file_path = args["file_path"].as_str().unwrap_or("").to_string();
            let content = args["content"].as_str().unwrap_or("").to_string();
            fs::write_file(
                root_path.to_string(),
                file_path.clone(),
                content,
            )?;
            Ok(serde_json::json!({"status": "success", "file": file_path}))
        }
        "list_files" => {
            let directory = args["directory"].as_str().map(|s| s.to_string());
            let nodes = fs::list_nodes(
                root_path.to_string(),
                directory,
            )?;
            Ok(serde_json::to_value(&nodes).unwrap_or(serde_json::json!([])))
        }
        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}
