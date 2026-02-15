use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub checked: bool,
    #[serde(default)]
    pub indeterminate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub project_type: String,
    pub tree: FileNode,
    pub total_files: u32,
    pub metadata: ProjectMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project_path: String,
    pub checked_paths: Vec<String>,
    pub excluded_paths: Vec<String>,
    pub last_opened: String,
    #[serde(default)]
    pub presets: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub pinned: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub projects: HashMap<String, ProjectConfig>,
}

// CodePack: 导出格式
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ExportFormat {
    #[default]
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "markdown")]
    Markdown,
    #[serde(rename = "xml")]
    Xml,
}

// CodePack: pack_files 返回结构，包含统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackResult {
    pub content: String,
    pub file_count: u32,
    pub total_bytes: u64,
    pub estimated_tokens: f64,
    #[serde(default)]
    pub skipped_files: Vec<SkippedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedFile {
    pub path: String,
    pub reason: String,
    pub size_bytes: u64,
}

// CodePack: estimate_tokens 返回结构，附带文件大小
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEstimate {
    pub tokens: f64,
    pub total_bytes: u64,
}

// CodePack: 项目元数据，用于导出时附加丰富上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub project_type: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub dev_dependencies: Vec<String>,
    pub entry_point: Option<String>,
    #[serde(default)]
    pub runtime: Vec<String>,
    #[serde(default)]
    pub requirements: Vec<String>,
}

// CodePack: 扫描进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub phase: String,
    pub files_found: u32,
    pub message: String,
}

// CodePack: 项目统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangStat {
    pub language: String,
    pub extension: String,
    pub file_count: u32,
    pub line_count: u64,
    pub byte_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub total_files: u32,
    pub total_lines: u64,
    pub total_bytes: u64,
    pub languages: Vec<LangStat>,
}
