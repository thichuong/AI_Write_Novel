use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use tauri::Manager;

/// Một node trong cây file explorer (file hoặc folder)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileNode {
    pub name: String,
    pub path: String,           // relative path từ root_path
    pub node_type: String,      // "file" | "folder"
    pub children: Vec<FileNode>,
}

/// Khởi tạo cấu trúc thư mục truyện tại root_path
#[tauri::command]
pub fn initialize_story_folders(root_path: String) -> Result<(), String> {
    let story_dir = PathBuf::from(&root_path);

    if !story_dir.exists() {
        fs::create_dir_all(&story_dir).map_err(|e| e.to_string())?;
    }

    // Tạo các thư mục con mặc định
    let default_folders = ["Chương", "Nhân vật", "Cốt truyện", "Bối cảnh", "Ghi chú"];
    for folder in &default_folders {
        fs::create_dir_all(story_dir.join(folder)).map_err(|e| e.to_string())?;
    }

    // Tạo file chương đầu tiên nếu chưa có gì trong Chương/
    let chapter_dir = story_dir.join("Chương");
    if chapter_dir.exists() {
        let entries = fs::read_dir(&chapter_dir).map_err(|e| e.to_string())?;
        if entries.count() == 0 {
            fs::write(
                chapter_dir.join("Chương 1.md"),
                "# Chương 1\n\nBắt đầu viết tại đây...",
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Tạo chat history rỗng nếu chưa có
    let chat_file = story_dir.join(".chat_history.json");
    if !chat_file.exists() {
        fs::write(&chat_file, "[]").map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Liệt kê file của thư mục hiện tại (không recursive)
#[tauri::command]
pub fn list_nodes(root_path: String, parent_path: Option<String>) -> Result<Vec<FileNode>, String> {
    let story_dir = PathBuf::from(&root_path);

    if !story_dir.exists() {
        return Err(format!("Thư mục không tồn tại: {}", root_path));
    }

    let target_dir = match parent_path {
        Some(p) => story_dir.join(p),
        None => story_dir.clone(),
    };

    if !target_dir.exists() {
        return Err(format!("Thư mục không tồn tại: {:?}", target_dir));
    }

    let nodes = read_dir_one_level(&target_dir, &story_dir)?;
    Ok(nodes)
}

/// Đọc thư mục (1 level) → array of FileNode
fn read_dir_one_level(dir: &Path, base: &Path) -> Result<Vec<FileNode>, String> {
    let mut nodes = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();

    // Sort: folders first, then files, both alphabetically
    entries.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    for entry in entries {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        // Bỏ qua các file ẩn (bắt đầu bằng dấu chấm)
        if name.starts_with('.') {
            continue;
        }

        let rel_path = path.strip_prefix(base).unwrap().to_string_lossy().to_string();

        if path.is_dir() {
            nodes.push(FileNode {
                name,
                path: rel_path,
                node_type: "folder".to_string(),
                children: vec![],
            });
        } else {
            nodes.push(FileNode {
                name,
                path: rel_path,
                node_type: "file".to_string(),
                children: vec![],
            });
        }
    }

    Ok(nodes)
}

/// Đọc nội dung file
#[tauri::command]
pub fn read_file(root_path: String, file_path: String) -> Result<String, String> {
    let full_path = PathBuf::from(&root_path).join(&file_path);

    if !full_path.exists() {
        return Err(format!("File không tồn tại: {}", file_path));
    }

    fs::read_to_string(&full_path).map_err(|e| e.to_string())
}

/// Ghi nội dung file
#[tauri::command]
pub fn write_file(root_path: String, file_path: String, content: String) -> Result<(), String> {
    let full_path = PathBuf::from(&root_path).join(&file_path);

    // Đảm bảo thư mục cha tồn tại
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::write(&full_path, content).map_err(|e| e.to_string())
}

/// Tạo file hoặc folder mới
#[tauri::command]
pub fn create_node(
    root_path: String,
    parent_path: String,
    name: String,
    node_type: String,
) -> Result<(), String> {
    let parent = PathBuf::from(&root_path).join(&parent_path);

    if !parent.exists() {
        return Err(format!("Thư mục cha không tồn tại: {}", parent_path));
    }

    if node_type == "folder" {
        fs::create_dir_all(parent.join(&name)).map_err(|e| e.to_string())?;
    } else {
        // File mặc định là .md
        let file_name = if name.ends_with(".md") {
            name
        } else {
            format!("{}.md", name)
        };
        fs::write(parent.join(&file_name), "").map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Đổi tên file hoặc folder
#[tauri::command]
pub fn rename_node(
    root_path: String,
    old_path: String,
    new_name: String,
) -> Result<(), String> {
    let old_full = PathBuf::from(&root_path).join(&old_path);

    if !old_full.exists() {
        return Err(format!("Không tìm thấy: {}", old_path));
    }

    let new_full = old_full.parent().unwrap().join(&new_name);
    fs::rename(&old_full, &new_full).map_err(|e| e.to_string())
}

/// Xóa file hoặc folder (recursive)
#[tauri::command]
pub fn delete_node(
    root_path: String,
    node_path: String,
) -> Result<(), String> {
    let full_path = PathBuf::from(&root_path).join(&node_path);

    if !full_path.exists() {
        return Ok(());
    }

    if full_path.is_dir() {
        fs::remove_dir_all(&full_path).map_err(|e| e.to_string())?;
    } else {
        fs::remove_file(&full_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Lấy chat history
#[tauri::command]
pub fn get_chat_history(
    root_path: String,
) -> Result<Vec<serde_json::Value>, String> {
    let chat_file = PathBuf::from(&root_path).join(".chat_history.json");

    if !chat_file.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&chat_file).map_err(|e| e.to_string())?;
    let history: Vec<serde_json::Value> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(history)
}

/// Lưu chat history
#[tauri::command]
pub fn save_chat_history(
    root_path: String,
    history: Vec<serde_json::Value>,
) -> Result<(), String> {
    let chat_file = PathBuf::from(&root_path).join(".chat_history.json");
    let json = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;
    fs::write(&chat_file, json).map_err(|e| e.to_string())
}

/// Lấy context (quy tắc, nhân vật, vật phẩm) của truyện bằng cách đọc tất cả file .md
#[tauri::command]
pub fn get_story_context(
    root_path: String,
) -> Result<String, String> {
    let story_dir = PathBuf::from(&root_path);

    let mut context = String::from("# KIẾN THỨC VỀ TRUYỆN\n\n");

    let sections = [
        ("Quy tắc", "QUY TẮC TRUYỆN"),
        ("Nhân vật", "NHÂN VẬT"),
        ("Vật phẩm", "VẬT PHẨM & BỐI CẢNH"),
        ("Cốt truyện", "CỐT TRUYỆN"),
    ];

    for (folder, heading) in &sections {
        let folder_path = story_dir.join(folder);
        if folder_path.exists() {
            let mut section_content = String::new();
            if let Ok(entries) = fs::read_dir(&folder_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                        let name = path.file_stem().unwrap().to_string_lossy().to_string();
                        let content = fs::read_to_string(&path).unwrap_or_default();
                        if !content.trim().is_empty() {
                            section_content.push_str(&format!("### {}\n{}\n\n", name, content));
                        }
                    }
                }
            }
            if !section_content.is_empty() {
                context.push_str(&format!("## {}\n{}\n", heading, section_content));
            }
        }
    }

    Ok(context)
}

/// Lấy nội dung các chương trước chương hiện tại
#[tauri::command]
pub fn get_previous_chapters(
    root_path: String,
    current_file: String,
) -> Result<String, String> {
    let chapters_dir = PathBuf::from(&root_path).join("Chương");

    if !chapters_dir.exists() {
        return Ok(String::new());
    }

    let mut chapters: Vec<(String, String)> = Vec::new();
    if let Ok(entries) = fs::read_dir(&chapters_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let content = fs::read_to_string(&path).unwrap_or_default();
                chapters.push((name, content));
            }
        }
    }

    // Sort by name
    chapters.sort_by(|a, b| a.0.cmp(&b.0));

    // Get everything before the current file
    let mut result = String::new();
    for (name, content) in &chapters {
        // Tạo relative path "Chương/<filename>"
        let rel = format!("Chương/{}", name);
        if rel == current_file {
            break;
        }
        if !content.trim().is_empty() {
            result.push_str(&format!("## {}\n{}\n\n", name, content));
        }
    }

    Ok(result)
}
