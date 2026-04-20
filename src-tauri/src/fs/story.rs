use crate::ai::memory::DEFAULT_MEMORY_CONTENT;
use crate::error::AppResult;
use std::fs;
use std::path::PathBuf;

/// Khởi tạo cấu trúc thư mục truyện tại `root_path`
#[tauri::command]
pub fn initialize_story_folders(root_path: String) -> AppResult<()> {
    let story_dir = PathBuf::from(&root_path);

    if !story_dir.exists() {
        fs::create_dir_all(&story_dir)?;
    }

    // Tạo các thư mục con mặc định (Tiếng Anh + Hệ thống Wiki)
    let default_folders = [
        "chapters",
        "wiki",
        "wiki/Characters",
        "wiki/World",
        "wiki/Lore",
        "wiki/Plot",
    ];
    for folder in &default_folders {
        fs::create_dir_all(story_dir.join(folder))?;
    }

    // Tạo chat history rỗng nếu chưa có
    let chat_file = story_dir.join("chat_history.json");
    if !chat_file.exists() {
        fs::write(&chat_file, "[]")?;
    }

    // Tạo file memory.md từ Template nếu chưa có
    let memory_file = story_dir.join("memory.md");
    if !memory_file.exists() {
        fs::write(&memory_file, DEFAULT_MEMORY_CONTENT)?;
    }

    Ok(())
}

/// Lấy chat history
#[tauri::command]
pub fn get_chat_history(root_path: String) -> AppResult<Vec<serde_json::Value>> {
    let chat_file = PathBuf::from(&root_path).join("chat_history.json");

    if !chat_file.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&chat_file)?;
    let history: Vec<serde_json::Value> = serde_json::from_str(&content)?;
    Ok(history)
}

/// Lưu chat history
#[tauri::command]
pub fn save_chat_history(root_path: String, history: Vec<serde_json::Value>) -> AppResult<()> {
    let chat_file = PathBuf::from(&root_path).join("chat_history.json");
    let json = serde_json::to_string_pretty(&history)?;
    fs::write(&chat_file, json)?;
    Ok(())
}
