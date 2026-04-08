use serde_json;
use std::fs;
use std::path::PathBuf;

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

/// Lấy chat history
#[tauri::command]
pub fn get_chat_history(root_path: String) -> Result<Vec<serde_json::Value>, String> {
    let chat_file = PathBuf::from(&root_path).join(".chat_history.json");

    if !chat_file.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&chat_file).map_err(|e| e.to_string())?;
    let history: Vec<serde_json::Value> =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(history)
}

/// Lưu chat history
#[tauri::command]
pub fn save_chat_history(root_path: String, history: Vec<serde_json::Value>) -> Result<(), String> {
    let chat_file = PathBuf::from(&root_path).join(".chat_history.json");
    let json = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;
    fs::write(&chat_file, json).map_err(|e| e.to_string())
}

/// Lấy context (quy tắc, nhân vật, vật phẩm) của truyện bằng cách đọc tất cả file .md
#[tauri::command]
pub fn get_story_context(root_path: String) -> Result<String, String> {
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
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
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
pub fn get_previous_chapters(root_path: String, current_file: String) -> Result<String, String> {
    let chapters_dir = PathBuf::from(&root_path).join("Chương");

    if !chapters_dir.exists() {
        return Ok(String::new());
    }

    let mut chapters: Vec<(String, String)> = Vec::new();
    if let Ok(entries) = fs::read_dir(&chapters_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let content = fs::read_to_string(&path).unwrap_or_default();
                chapters.push((name, content));
            }
        }
    }

    // Sort by name
    chapters.sort_by(|a, b| a.0.cmp(&b.0));

    // Get valid previous chapters before the current file
    let mut prev_chaps = Vec::new();
    for chap in &chapters {
        // Tạo relative path "Chương/<filename>"
        let rel = format!("Chương/{}", chap.0);
        if rel == current_file {
            break;
        }
        if !chap.1.trim().is_empty() {
            prev_chaps.push(chap);
        }
    }

    // Only take the last 10 chapters to avoid overwhelming context
    let start_idx = prev_chaps.len().saturating_sub(10);
    let mut result = String::new();
    for chap in &prev_chaps[start_idx..] {
        result.push_str(&format!("## {}\n{}\n\n", chap.0, chap.1));
    }

    Ok(result)
}
