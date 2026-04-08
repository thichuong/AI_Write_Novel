use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Một node trong cây file explorer (file hoặc folder)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileNode {
    pub name: String,
    pub path: String,      // relative path từ root_path
    pub node_type: String, // "file" | "folder"
    pub children: Vec<Self>,
}

/// Liệt kê file của thư mục hiện tại (không recursive)
#[tauri::command]
pub fn list_nodes(root_path: &str, parent_path: Option<&str>) -> Result<Vec<FileNode>, String> {
    let story_dir = PathBuf::from(&root_path);

    if !story_dir.exists() {
        return Err(format!("Thư mục không tồn tại: {root_path}"));
    }

    let target_dir = parent_path.map_or_else(|| story_dir.clone(), |p| story_dir.join(p));

    if !target_dir.exists() {
        return Err(format!("Thư mục không tồn tại: {}", target_dir.display()));
    }

    let nodes = read_dir_one_level(&target_dir, &story_dir)?;
    Ok(nodes)
}

/// Đọc thư mục (1 level) → array of `FileNode`
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
        let name = path
            .file_name()
            .ok_or_else(|| "Không lấy được tên file".to_string())?
            .to_string_lossy()
            .to_string();

        // Bỏ qua các file ẩn (bắt đầu bằng dấu chấm)
        if name.starts_with('.') {
            continue;
        }

        let rel_path = path
            .strip_prefix(base)
            .map_err(|_| "Lỗi đường dẫn base".to_string())?
            .to_string_lossy()
            .to_string();

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
