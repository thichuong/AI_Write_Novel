use serde_json;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

/// Khởi tạo cấu trúc thư mục truyện tại `root_path`
#[tauri::command]
pub fn initialize_story_folders(root_path: &str) -> Result<(), String> {
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

    // Tạo file memory.md rỗng nếu chưa có
    let memory_file = story_dir.join("memory.md");
    if !memory_file.exists() {
        fs::write(
            &memory_file,
            "# BỘ NHỚ TRUYỆN (MEMORY)\n\n\
             ## Cốt truyện chung\n(Chưa có nội dung)\n\n\
             ## Tóm tắt nội dung đã viết\n(Chưa có nội dung)\n\n\
             ## Tình trạng nhân vật hiện tại\n(Chưa có nội dung)\n",
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Lấy chat history
#[tauri::command]
pub fn get_chat_history(root_path: &str) -> Result<Vec<serde_json::Value>, String> {
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
pub fn save_chat_history(root_path: &str, history: Vec<serde_json::Value>) -> Result<(), String> {
    let chat_file = PathBuf::from(&root_path).join(".chat_history.json");
    let json = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;
    fs::write(&chat_file, json).map_err(|e| e.to_string())
}


