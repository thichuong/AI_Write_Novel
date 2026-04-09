use crate::ai::gemini_types::{FunctionDecl, Schema, ToolDeclaration};
use serde_json::json;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

/// Danh sách các file trong thư mục
pub fn tool_list_directory(root_path: &str, path: &str) -> Result<String, String> {
    let target_dir = PathBuf::from(root_path).join(path);
    if !target_dir.exists() {
        return Err(format!("Thư mục không tồn tại: {path}"));
    }

    let mut result = String::new();
    let entries = fs::read_dir(&target_dir).map_err(|e| e.to_string())?;

    let mut files_and_folders = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        let name = p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if name.starts_with('.') {
            continue;
        }

        let rel_path = p
            .strip_prefix(root_path)
            .map_or_else(|_| name.clone(), |dp| dp.to_string_lossy().to_string());

        let is_dir = p.is_dir();
        files_and_folders.push((name, rel_path, is_dir));
    }

    // Sort: folders first, then alphabetically
    files_and_folders.sort_by(|a, b| match (a.2, b.2) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.0.cmp(&b.0),
    });

    for (name, rel_path, is_dir) in files_and_folders {
        let icon = if is_dir { "📁" } else { "📄" };
        let _ = writeln!(result, "{icon} {name} ({rel_path})");
    }

    if result.is_empty() {
        Ok("Thư mục trống.".to_string())
    } else {
        Ok(result)
    }
}

/// Đọc nội dung một file
pub fn tool_read_file(root_path: &str, path: &str) -> Result<String, String> {
    let full_path = PathBuf::from(root_path).join(path);
    fs::read_to_string(full_path).map_err(|e| e.to_string())
}

/// Ghi nội dung vào file
pub fn tool_write_file(
    app_handle: &AppHandle,
    root_path: &str,
    path: &str,
    content: &str,
) -> Result<String, String> {
    let full_path = PathBuf::from(root_path).join(path);

    // Đảm bảo thư mục cha tồn tại
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::write(&full_path, content).map_err(|e| e.to_string())?;

    // Emit event để frontend đồng bộ file explorer
    let _ = app_handle.emit(
        "file-system-changed",
        json!({ "path": path, "action": "write" }),
    );
    Ok(format!("Đã ghi thành công vào file: {path}"))
}

/// Xóa một file hoặc thư mục
pub fn tool_delete_file(
    app_handle: &AppHandle,
    root_path: &str,
    path: &str,
) -> Result<String, String> {
    let full_path = PathBuf::from(root_path).join(path);
    if !full_path.exists() {
        return Ok(format!("Không tìm thấy file để xóa: {path}"));
    }

    if full_path.is_dir() {
        fs::remove_dir_all(full_path).map_err(|e| e.to_string())?;
    } else {
        fs::remove_file(full_path).map_err(|e| e.to_string())?;
    }

    // Emit event để frontend đồng bộ file explorer
    let _ = app_handle.emit(
        "file-system-changed",
        json!({ "path": path, "action": "delete" }),
    );
    Ok(format!("Đã xóa thành công: {path}"))
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
