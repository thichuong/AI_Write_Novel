mod fs;
mod ai;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            app.handle().plugin(tauri_plugin_dialog::init())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // File System commands
            fs::initialize_story_folders,
            fs::list_nodes,
            fs::read_file,
            fs::write_file,
            fs::create_node,
            fs::rename_node,
            fs::delete_node,
            fs::get_chat_history,
            fs::save_chat_history,
            fs::get_story_context,
            fs::get_previous_chapters,
            // AI Agent commands
            ai::commands::ai_chat,
            ai::commands::ai_write,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
