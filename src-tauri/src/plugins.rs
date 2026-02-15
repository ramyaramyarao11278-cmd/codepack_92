use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDef {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub detect_files: Vec<String>,
    #[serde(default)]
    pub detect_dirs: Vec<String>,
    #[serde(default)]
    pub exclude_dirs: Vec<String>,
    #[serde(default)]
    pub source_extensions: Vec<String>,
}

pub fn get_plugins_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("codepack")
        .join("plugins")
}

pub fn load_plugins() -> Vec<PluginDef> {
    let dir = get_plugins_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut plugins = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(plugin) = serde_json::from_str::<PluginDef>(&content) {
                        plugins.push(plugin);
                    }
                }
            }
        }
    }
    plugins
}

pub fn plugin_matches(plugin: &PluginDef, root: &Path) -> bool {
    let files_match = plugin.detect_files.is_empty()
        || plugin.detect_files.iter().all(|f| root.join(f).exists());
    let dirs_match = plugin.detect_dirs.is_empty()
        || plugin.detect_dirs.iter().all(|d| root.join(d).is_dir());
    // At least one detect rule must be non-empty
    (!plugin.detect_files.is_empty() || !plugin.detect_dirs.is_empty())
        && files_match
        && dirs_match
}

// CodePack: 收集所有插件的额外排除目录
pub fn get_plugin_excluded_dirs(plugins: &[PluginDef]) -> Vec<String> {
    plugins
        .iter()
        .flat_map(|p| p.exclude_dirs.iter().cloned())
        .collect()
}

// CodePack: 收集所有插件的额外源码扩展名
pub fn get_plugin_source_extensions(plugins: &[PluginDef]) -> Vec<String> {
    plugins
        .iter()
        .flat_map(|p| p.source_extensions.iter().cloned())
        .collect()
}
