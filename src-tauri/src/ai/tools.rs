use crate::ai::gemini_types::{FunctionDecl, Schema, ToolDeclaration};
use crate::fs;
use serde_json::json;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

/// Danh sách các file trong thư mục
pub fn tool_list_directory(root_path: &str, path: &str) -> Result<String, String> {
    let nodes = fs::list_nodes(root_path, Some(path))?;
    let mut result = String::new();
    for node in nodes {
        let icon = if node.node_type == "folder" {
            "📁"
        } else {
            "📄"
        };
        result.push_str(&format!("{} {} ({})\n", icon, node.name, node.path));
    }
    if result.is_empty() {
        Ok("Thư mục trống.".to_string())
    } else {
        Ok(result)
    }
}

/// Đọc nội dung một file
pub fn tool_read_file(root_path: &str, path: &str) -> Result<String, String> {
    fs::read_file(root_path, path)
}

/// Ghi nội dung vào file
pub fn tool_write_file(
    app_handle: &AppHandle,
    root_path: &str,
    path: &str,
    content: &str,
) -> Result<String, String> {
    fs::write_file(root_path, path, content)?;
    // Emit event để frontend đồng bộ file explorer
    let _ = app_handle.emit(
        "file-system-changed",
        json!({ "path": path, "action": "write" }),
    );
    Ok(format!("Đã ghi thành công vào file: {}", path))
}

/// Xóa một file hoặc thư mục
pub fn tool_delete_file(
    app_handle: &AppHandle,
    root_path: &str,
    path: &str,
) -> Result<String, String> {
    fs::delete_node(root_path, path)?;
    // Emit event để frontend đồng bộ file explorer
    let _ = app_handle.emit(
        "file-system-changed",
        json!({ "path": path, "action": "delete" }),
    );
    Ok(format!("Đã xóa thành công: {}", path))
}

/// Trả về khai báo các công cụ cho Gemini
pub fn get_tool_declarations() -> Vec<ToolDeclaration> {
    vec![
        ToolDeclaration {
            function_declarations: vec![
                FunctionDecl {
                    name: "list_directory".to_string(),
                    description: "Liệt kê danh sách file và thư mục trong một đường dẫn cụ thể (tương đối so với root).".to_string(),
                    parameters: Schema {
                        field_type: "object".to_string(),
                        properties: Some({
                            let mut p = HashMap::new();
                            p.insert("path".to_string(), Schema {
                                field_type: "string".to_string(),
                                description: Some("Đường dẫn thư mục muốn xem (ví dụ: '.', 'Chương', 'Nhân vật'). Dùng '.' cho thư mục gốc.".to_string()),
                                properties: None,
                                required: None,
                            });
                            p
                        }),
                        required: Some(vec!["path".to_string()]),
                        description: None,
                    },
                },
                FunctionDecl {
                    name: "read_file".to_string(),
                    description: "Đọc nội dung chi tiết của một file .md.".to_string(),
                    parameters: Schema {
                        field_type: "object".to_string(),
                        properties: Some({
                            let mut p = HashMap::new();
                            p.insert("path".to_string(), Schema {
                                field_type: "string".to_string(),
                                description: Some("Đường dẫn đến file cần đọc (ví dụ: 'Chương/Chuong1.md').".to_string()),
                                properties: None,
                                required: None,
                            });
                            p
                        }),
                        required: Some(vec!["path".to_string()]),
                        description: None,
                    },
                },
                FunctionDecl {
                    name: "write_file".to_string(),
                    description: "Tạo mới hoặc ghi đè nội dung vào một file. Chuyên dùng để lưu chương truyện hoặc cập nhật thông tin nhân vật, bối cảnh.".to_string(),
                    parameters: Schema {
                        field_type: "object".to_string(),
                        properties: Some({
                            let mut p = HashMap::new();
                            p.insert("path".to_string(), Schema {
                                field_type: "string".to_string(),
                                description: Some("Đường dẫn file (ví dụ: 'Chương/Chuong2.md').".to_string()),
                                properties: None,
                                required: None,
                            });
                            p.insert("content".to_string(), Schema {
                                field_type: "string".to_string(),
                                description: Some("Toàn bộ nội dung văn bản muốn ghi vào file.".to_string()),
                                properties: None,
                                required: None,
                            });
                            p
                        }),
                        required: Some(vec!["path".to_string(), "content".to_string()]),
                        description: None,
                    },
                },
                FunctionDecl {
                    name: "delete_file".to_string(),
                    description: "Xóa một file hoặc thư mục không còn cần thiết.".to_string(),
                    parameters: Schema {
                        field_type: "object".to_string(),
                        properties: Some({
                            let mut p = HashMap::new();
                            p.insert("path".to_string(), Schema {
                                field_type: "string".to_string(),
                                description: Some("Đường dẫn file hoặc thư mục cần xóa.".to_string()),
                                properties: None,
                                required: None,
                            });
                            p
                        }),
                        required: Some(vec!["path".to_string()]),
                        description: None,
                    },
                },
            ],
        }
    ]
}
