pub mod types;
pub mod config;
pub mod plugins;
pub mod scanner;
pub mod metadata;
pub mod stats;
pub mod packer;
pub mod git;
pub mod security;
pub mod watcher;
pub mod commands;

use commands::*;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .manage(watcher::WatcherState::new())
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            scan_directory_async,
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
            save_exclude_rules,
            load_exclude_rules,
            get_git_status_cmd,
            start_watching_cmd,
            stop_watching_cmd,
            pack_files_extended,
            scan_secrets_cmd,
            scan_all_secrets_cmd,
            mask_file_secrets_cmd,
            list_review_prompts_cmd,
            save_review_prompt_cmd,
            delete_review_prompt_cmd,
            load_api_config_cmd,
            save_api_config_cmd,
            start_ai_review,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
