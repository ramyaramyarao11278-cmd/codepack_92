pub mod types;
pub mod config;
pub mod plugins;
pub mod scanner;
pub mod metadata;
pub mod stats;
pub mod packer;
pub mod git;
pub mod commands;

use commands::*;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            read_file_content,
            save_project_config,
            load_project_config,
            estimate_tokens,
            pack_files,
            copy_to_clipboard,
            export_to_file,
            open_directory,
            get_file_size,
            save_preset,
            delete_preset,
            list_presets,
            list_plugins,
            save_plugin,
            delete_plugin,
            get_project_stats,
            get_git_status_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
