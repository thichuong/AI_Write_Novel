mod ai;
mod fs;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Runs the Tauri application.
///
/// # Panics
///
/// Panics if the Tauri application fails to start or run.
#[allow(clippy::expect_used)]
pub fn run() {
    #[allow(clippy::large_stack_frames)]
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
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
            fs::get_chat_history,
            fs::save_chat_history,
            // AI Agent commands
            ai::chat::ai_chat,
            ai::api_client::check_api_key,
            ai::api_client::save_api_key,
            ai::api_client::save_settings,
            ai::api_client::get_settings,
            ai::api_client::list_models,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
