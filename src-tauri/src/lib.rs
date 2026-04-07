mod fs_manager;
mod ai_agent;

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
            fs_manager::initialize_story_folders,
            fs_manager::list_nodes,
            fs_manager::read_file,
            fs_manager::write_file,
            fs_manager::create_node,
            fs_manager::rename_node,
            fs_manager::delete_node,
            fs_manager::get_chat_history,
            fs_manager::save_chat_history,
            fs_manager::get_story_context,
            fs_manager::get_previous_chapters,
            // AI Agent commands
            ai_agent::ai_chat,
            ai_agent::ai_write,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
