use std::fs;
use std::path::PathBuf;

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
pub fn rename_node(root_path: String, old_path: String, new_name: String) -> Result<(), String> {
    let old_full = PathBuf::from(&root_path).join(&old_path);

    if !old_full.exists() {
        return Err(format!("Không tìm thấy: {}", old_path));
    }

    let new_full = old_full.parent().unwrap().join(&new_name);
    fs::rename(&old_full, &new_full).map_err(|e| e.to_string())
}

/// Xóa file hoặc folder (recursive)
#[tauri::command]
pub fn delete_node(root_path: String, node_path: String) -> Result<(), String> {
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
