mod audio;
mod bin_path;
mod commands;
mod gemini;
mod progress;
mod settings;
mod subtitle;
mod whisper;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::process_video,
            commands::get_settings,
            commands::save_settings,
            commands::fetch_models,
            commands::save_log_file
        ])
        .run(tauri::generate_context!())
        .expect("啟動應用程式時發生錯誤");
}
