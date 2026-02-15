use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use tiktoken_rs::CoreBPE;

use crate::config::{chrono_now, load_app_config, save_app_config, load_review_prompts, save_custom_review_prompt, delete_custom_review_prompt};
use crate::metadata::extract_metadata;
use crate::packer::{build_pack_content_with_limit, build_pack_content_extended};

static BPE: LazyLock<CoreBPE> = LazyLock::new(|| {
    tiktoken_rs::cl100k_base().expect("failed to load cl100k_base tokenizer")
});
use crate::plugins::{
    get_plugin_excluded_dirs, get_plugin_source_extensions, get_plugins_dir, load_plugins,
    PluginDef,
};
use crate::scanner::{build_file_tree, count_files, detect_project_type_with_plugins};
use crate::stats::compute_project_stats;
use tauri::Emitter;
use crate::types::{ExportFormat, PackResult, ProjectConfig, ProjectStats, ReviewPrompt, ScanProgress, ScanResult, TokenEstimate};

#[tauri::command]
pub async fn scan_directory_async(
    app: tauri::AppHandle,
    path: String,
    custom_excludes: Option<Vec<String>>,
) -> Result<ScanResult, String> {
    let path_clone = path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let root = Path::new(&path_clone);
        if !root.exists() || !root.is_dir() {
            return Err("Path does not exist or is not a directory".to_string());
        }

        let _ = app.emit("scan-progress", ScanProgress {
            phase: "detecting".to_string(),
            files_found: 0,
            message: "Detecting project type...".to_string(),
        });

        let plugins = load_plugins();
        let project_type = detect_project_type_with_plugins(root, &plugins);
        let mut extra_excludes = get_plugin_excluded_dirs(&plugins);
        if let Some(custom) = custom_excludes {
            extra_excludes.extend(custom);
        }
        let extra_extensions = get_plugin_source_extensions(&plugins);

        let _ = app.emit("scan-progress", ScanProgress {
            phase: "scanning".to_string(),
            files_found: 0,
            message: "Scanning files...".to_string(),
        });

        let tree = build_file_tree(root, &extra_excludes, &extra_extensions);
        let total_files = count_files(&tree);

        let _ = app.emit("scan-progress", ScanProgress {
            phase: "metadata".to_string(),
            files_found: total_files,
            message: format!("Found {} files, extracting metadata...", total_files),
        });

        let metadata = extract_metadata(root, &project_type);

        let _ = app.emit("scan-progress", ScanProgress {
            phase: "done".to_string(),
            files_found: total_files,
            message: format!("Scan complete: {} files", total_files),
        });

        Ok(ScanResult {
            project_type,
            tree,
            total_files,
            metadata,
        })
    })
    .await
    .map_err(|e| format!("Scan task failed: {}", e))?;
    result
}

#[tauri::command]
pub fn scan_directory(path: String, custom_excludes: Option<Vec<String>>) -> Result<ScanResult, String> {
    let root = Path::new(&path);
    if !root.exists() || !root.is_dir() {
        return Err("Path does not exist or is not a directory".to_string());
    }

    let plugins = load_plugins();
    let project_type = detect_project_type_with_plugins(root, &plugins);
    let mut extra_excludes = get_plugin_excluded_dirs(&plugins);
    if let Some(custom) = custom_excludes {
        extra_excludes.extend(custom);
    }
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
    let mut total_tokens: usize = 0;
    let bpe = &*BPE;
    for path in &paths {
        if let Ok(content) = fs::read_to_string(path) {
            total_bytes += content.len() as u64;
            total_tokens += bpe.encode_ordinary(&content).len();
        }
    }
    Ok(TokenEstimate {
        tokens: total_tokens as f64,
        total_bytes,
    })
}

#[tauri::command]
pub fn pack_files(
    paths: Vec<String>,
    project_path: String,
    project_type: String,
    format: Option<ExportFormat>,
    max_file_bytes: Option<u64>,
) -> Result<PackResult, String> {
    let fmt = format.unwrap_or_default();
    Ok(build_pack_content_with_limit(&paths, &project_path, &project_type, &fmt, max_file_bytes))
}

#[tauri::command]
pub fn pack_files_extended(
    paths: Vec<String>,
    project_path: String,
    project_type: String,
    format: Option<ExportFormat>,
    max_file_bytes: Option<u64>,
    include_diff: Option<bool>,
    instruction: Option<String>,
) -> Result<PackResult, String> {
    let fmt = format.unwrap_or_default();
    let diffs = if include_diff.unwrap_or(false) {
        let diff_map = crate::git::get_diffs_for_files(&project_path, &paths);
        if diff_map.is_empty() { None } else { Some(diff_map) }
    } else {
        None
    };
    Ok(build_pack_content_extended(
        &paths, &project_path, &project_type, &fmt, max_file_bytes,
        diffs.as_ref(), instruction.as_deref(),
    ))
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
    format: Option<ExportFormat>,
    max_file_bytes: Option<u64>,
) -> Result<String, String> {
    let fmt = format.unwrap_or_default();
    let result = build_pack_content_with_limit(&paths, &project_path, &project_type, &fmt, max_file_bytes);
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

// ─── Exclude Rules Commands ────────────────────────────────────

#[tauri::command]
pub fn save_exclude_rules(project_path: String, rules: Vec<String>) -> Result<(), String> {
    let mut config = load_app_config();
    if let Some(project) = config.projects.get_mut(&project_path) {
        project.excluded_paths = rules;
    } else {
        let now = chrono_now();
        config.projects.insert(
            project_path.clone(),
            ProjectConfig {
                project_path,
                checked_paths: Vec::new(),
                excluded_paths: rules,
                last_opened: now,
                presets: HashMap::new(),
                pinned: false,
            },
        );
    }
    save_app_config(&config)
}

#[tauri::command]
pub fn load_exclude_rules(project_path: String) -> Result<Vec<String>, String> {
    let config = load_app_config();
    Ok(config
        .projects
        .get(&project_path)
        .map(|p| p.excluded_paths.clone())
        .unwrap_or_default())
}

// ─── Git Command ───────────────────────────────────────────────

#[tauri::command]
pub fn get_git_status_cmd(project_path: String) -> Result<Option<crate::git::GitStatus>, String> {
    Ok(crate::git::get_git_status(&project_path))
}

// ─── Watcher Commands ──────────────────────────────────────────

#[tauri::command]
pub fn start_watching_cmd(app: tauri::AppHandle, project_path: String) -> Result<(), String> {
    crate::watcher::start_watching(&app, &project_path)
}

#[tauri::command]
pub fn stop_watching_cmd(app: tauri::AppHandle) -> Result<(), String> {
    crate::watcher::stop_watching(&app)
}

// ─── Security Commands ─────────────────────────────────────────

#[tauri::command]
pub fn scan_secrets_cmd(path: String) -> Result<Vec<crate::types::SecretMatch>, String> {
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(crate::security::scan_content(&content))
}

#[tauri::command]
pub fn scan_all_secrets_cmd(
    paths: Vec<String>,
    project_path: String,
) -> Result<HashMap<String, Vec<crate::types::SecretMatch>>, String> {
    let root = Path::new(&project_path);
    let mut result = HashMap::new();
    for path in &paths {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let matches = crate::security::scan_content(&content);
        if !matches.is_empty() {
            let relative = Path::new(path)
                .strip_prefix(root)
                .unwrap_or(Path::new(path))
                .to_string_lossy()
                .replace('\\', "/");
            result.insert(relative, matches);
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn mask_file_secrets_cmd(path: String) -> Result<String, String> {
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    let matches = crate::security::scan_content(&content);
    Ok(crate::security::mask_secrets(&content, &matches))
}

// ─── Review Prompt Commands ────────────────────────────────────

#[tauri::command]
pub fn list_review_prompts_cmd() -> Result<Vec<ReviewPrompt>, String> {
    Ok(load_review_prompts())
}

#[tauri::command]
pub fn save_review_prompt_cmd(prompt: ReviewPrompt) -> Result<(), String> {
    if prompt.builtin {
        return Err("Cannot modify builtin prompts".to_string());
    }
    save_custom_review_prompt(&prompt)
}

#[tauri::command]
pub fn delete_review_prompt_cmd(name: String) -> Result<(), String> {
    // Check if it's a builtin
    let builtins = load_review_prompts();
    if builtins.iter().any(|p| p.name == name && p.builtin) {
        return Err("Cannot delete builtin prompts".to_string());
    }
    delete_custom_review_prompt(&name)
}

// ─── Stats Command ─────────────────────────────────────────────

#[tauri::command]
pub fn get_project_stats(paths: Vec<String>) -> Result<ProjectStats, String> {
    Ok(compute_project_stats(&paths))
}
