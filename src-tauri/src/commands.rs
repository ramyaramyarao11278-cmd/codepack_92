use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::config::{chrono_now, load_app_config, save_app_config};
use crate::metadata::extract_metadata;
use crate::packer::build_pack_content;
use crate::plugins::{
    get_plugin_excluded_dirs, get_plugin_source_extensions, get_plugins_dir, load_plugins,
    PluginDef,
};
use crate::scanner::{build_file_tree, count_files, detect_project_type_with_plugins};
use crate::stats::compute_project_stats;
use crate::types::{PackResult, ProjectConfig, ProjectStats, ScanResult, TokenEstimate};

#[tauri::command]
pub fn scan_directory(path: String) -> Result<ScanResult, String> {
    let root = Path::new(&path);
    if !root.exists() || !root.is_dir() {
        return Err("Path does not exist or is not a directory".to_string());
    }

    let plugins = load_plugins();
    let project_type = detect_project_type_with_plugins(root, &plugins);
    let extra_excludes = get_plugin_excluded_dirs(&plugins);
    let extra_extensions = get_plugin_source_extensions(&plugins);
    let tree = build_file_tree(root, &extra_excludes, &extra_extensions);
    let total_files = count_files(&tree);
    let metadata = extract_metadata(root, &project_type);

    Ok(ScanResult {
        project_type,
        tree,
        total_files,
        metadata,
    })
}

#[tauri::command]
pub fn read_file_content(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
pub fn save_project_config(project_path: String, checked_paths: Vec<String>) -> Result<(), String> {
    let mut config = load_app_config();
    let now = chrono_now();
    let (presets, pinned) = config
        .projects
        .get(&project_path)
        .map(|p| (p.presets.clone(), p.pinned))
        .unwrap_or_default();
    config.projects.insert(
        project_path.clone(),
        ProjectConfig {
            project_path,
            checked_paths,
            excluded_paths: Vec::new(),
            last_opened: now,
            presets,
            pinned,
        },
    );
    save_app_config(&config)
}

#[tauri::command]
pub fn load_project_config(project_path: String) -> Result<Option<ProjectConfig>, String> {
    let config = load_app_config();
    Ok(config.projects.get(&project_path).cloned())
}

#[tauri::command]
pub fn estimate_tokens(paths: Vec<String>) -> Result<TokenEstimate, String> {
    let mut total_bytes: u64 = 0;
    for path in &paths {
        if let Ok(meta) = fs::metadata(path) {
            total_bytes += meta.len();
        }
    }
    Ok(TokenEstimate {
        tokens: total_bytes as f64 / 4.0,
        total_bytes,
    })
}

#[tauri::command]
pub fn pack_files(
    paths: Vec<String>,
    project_path: String,
    project_type: String,
) -> Result<PackResult, String> {
    Ok(build_pack_content(&paths, &project_path, &project_type))
}

#[tauri::command]
pub fn copy_to_clipboard(content: String, app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard()
        .write_text(&content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_to_file(
    paths: Vec<String>,
    project_path: String,
    project_type: String,
    save_path: String,
) -> Result<String, String> {
    let result = build_pack_content(&paths, &project_path, &project_type);
    fs::write(&save_path, &result.content)
        .map_err(|e| format!("Failed to export: {}", e))?;
    Ok(save_path)
}

#[tauri::command]
pub fn open_directory(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    let dir = if p.is_file() {
        p.parent().unwrap_or(p)
    } else {
        p
    };
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(dir.to_string_lossy().to_string())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(dir.to_string_lossy().to_string())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(dir.to_string_lossy().to_string())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_file_size(path: String) -> Result<u64, String> {
    fs::metadata(&path)
        .map(|m| m.len())
        .map_err(|e| format!("Failed to get file size: {}", e))
}

// ─── Preset Commands ───────────────────────────────────────────

#[tauri::command]
pub fn save_preset(
    project_path: String,
    preset_name: String,
    checked_paths: Vec<String>,
) -> Result<(), String> {
    let mut config = load_app_config();
    if let Some(project) = config.projects.get_mut(&project_path) {
        project.presets.insert(preset_name, checked_paths);
    } else {
        let now = chrono_now();
        let mut presets = HashMap::new();
        presets.insert(preset_name, checked_paths.clone());
        config.projects.insert(
            project_path.clone(),
            ProjectConfig {
                project_path,
                checked_paths,
                excluded_paths: Vec::new(),
                last_opened: now,
                presets,
                pinned: false,
            },
        );
    }
    save_app_config(&config)
}

#[tauri::command]
pub fn delete_preset(project_path: String, preset_name: String) -> Result<(), String> {
    let mut config = load_app_config();
    if let Some(project) = config.projects.get_mut(&project_path) {
        project.presets.remove(&preset_name);
    }
    save_app_config(&config)
}

#[tauri::command]
pub fn list_presets(project_path: String) -> Result<HashMap<String, Vec<String>>, String> {
    let config = load_app_config();
    Ok(config
        .projects
        .get(&project_path)
        .map(|p| p.presets.clone())
        .unwrap_or_default())
}

// ─── Plugin Commands ───────────────────────────────────────────

#[tauri::command]
pub fn list_plugins() -> Result<Vec<PluginDef>, String> {
    Ok(load_plugins())
}

#[tauri::command]
pub fn save_plugin(plugin: PluginDef) -> Result<(), String> {
    let dir = get_plugins_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let filename = plugin.name.to_lowercase().replace(' ', "-") + ".json";
    let path = dir.join(filename);
    let json = serde_json::to_string_pretty(&plugin).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_plugin(name: String) -> Result<(), String> {
    let dir = get_plugins_dir();
    let filename = name.to_lowercase().replace(' ', "-") + ".json";
    let path = dir.join(&filename);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ─── Stats Command ─────────────────────────────────────────────

#[tauri::command]
pub fn get_project_stats(paths: Vec<String>) -> Result<ProjectStats, String> {
    Ok(compute_project_stats(&paths))
}
