use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ─── Data Types ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub checked: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub projects: HashMap<String, ProjectConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }
}

// CodePack: pack_files 返回结构，包含统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackResult {
    pub content: String,
    pub file_count: u32,
    pub total_bytes: u64,
    pub estimated_tokens: f64,
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
}

// ─── Constants ─────────────────────────────────────────────────

const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    "build",
    "dist",
    ".gradle",
    ".idea",
    ".vscode",
    "__pycache__",
    ".git",
    ".svn",
    ".hg",
    "target",
    ".next",
    ".nuxt",
    ".output",
    "venv",
    ".venv",
    "env",
    ".env",
    ".dart_tool",
    ".pub-cache",
    "Pods",
    "DerivedData",
    ".cache",
    "coverage",
    ".turbo",
    "out",
    ".DS_Store",
    "bin",
    "obj",
    ".tox",
    "vendor",
    ".bundle",
    ".swiftpm",
];

const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "vue", "svelte", "py", "kt", "kts", "java", "dart", "go",
    "rb", "php", "swift", "c", "cpp", "h", "hpp", "cs", "m", "mm", "scala", "clj", "ex",
    "exs", "hs", "lua", "r", "jl", "sql", "sh", "bash", "zsh", "fish", "bat", "ps1", "yml",
    "yaml", "toml", "json", "xml", "html", "css", "scss", "sass", "less", "md", "mdx", "txt",
    "cfg", "ini", "conf", "env", "dockerfile", "makefile", "cmake", "gradle", "properties",
    "gitignore", "editorconfig", "eslintrc", "prettierrc", "graphql", "gql", "proto",
    "tf", "hcl", "nix", "astro", "mod", "sum", "lock",
];

// ─── Helpers ───────────────────────────────────────────────────

fn get_config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("codepack_config.json")
}

fn load_app_config() -> AppConfig {
    let path = get_config_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&data) {
                return config;
            }
        }
    }
    AppConfig::default()
}

fn save_app_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn is_excluded_dir(name: &str) -> bool {
    EXCLUDED_DIRS.iter().any(|&excluded| {
        name.eq_ignore_ascii_case(excluded)
    })
}

fn is_source_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    if matches!(
        lower.as_str(),
        "dockerfile" | "makefile" | "cmakelists.txt" | "rakefile" | "gemfile" | "procfile"
            | "justfile" | "taskfile" | "vagrantfile"
    ) {
        return true;
    }
    if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
        SOURCE_EXTENSIONS.iter().any(|&se| se.eq_ignore_ascii_case(ext))
    } else {
        false
    }
}

// CodePack: 增强的项目类型识别，支持 15+ 种项目类型
fn detect_project_type(root: &Path) -> String {
    // 1. Android / Gradle (most specific first)
    if root.join("build.gradle.kts").exists() || root.join("build.gradle").exists() {
        if root.join("app").is_dir() || root.join("AndroidManifest.xml").exists() {
            return "Android / Gradle".to_string();
        }
        return "Gradle".to_string();
    }
    // 2. Flutter / Dart
    if root.join("pubspec.yaml").exists() {
        return "Flutter / Dart".to_string();
    }
    // 3. Rust
    if root.join("Cargo.toml").exists() {
        return "Rust".to_string();
    }
    // 4. Go
    if root.join("go.mod").exists() {
        return "Go".to_string();
    }
    // 5. Java / Maven
    if root.join("pom.xml").exists() {
        return "Java / Maven".to_string();
    }
    // 6. Swift
    if root.join("Package.swift").exists() {
        return "Swift".to_string();
    }
    // 7. C++ / CMake
    if root.join("CMakeLists.txt").exists() {
        return "C++ / CMake".to_string();
    }
    // 8. C (Makefile + .c/.h files)
    if root.join("Makefile").exists() || root.join("makefile").exists() {
        let has_c_files = fs::read_dir(root).into_iter().flatten().any(|entry| {
            if let Ok(e) = entry {
                let name = e.file_name().to_string_lossy().to_string();
                name.ends_with(".c") || name.ends_with(".h")
            } else {
                false
            }
        });
        if has_c_files {
            return "C".to_string();
        }
    }
    // 9. Ruby
    if root.join("Gemfile").exists() {
        return "Ruby".to_string();
    }
    // 10. Docker
    if root.join("docker-compose.yml").exists() || root.join("docker-compose.yaml").exists() {
        return "Docker".to_string();
    }
    // 11-13. JS frameworks (check config files)
    for entry in fs::read_dir(root).into_iter().flatten() {
        if let Ok(entry) = entry {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("next.config") {
                return "Next.js".to_string();
            }
            if name.starts_with("nuxt.config") {
                return "Nuxt.js".to_string();
            }
            if name.starts_with("vite.config") {
                return "Vite".to_string();
            }
        }
    }
    // 14. Python
    if root.join("pyproject.toml").exists()
        || root.join("requirements.txt").exists()
        || root.join("setup.py").exists()
    {
        return "Python".to_string();
    }
    // 15. Node.js (generic)
    if root.join("package.json").exists() {
        return "Node.js".to_string();
    }
    "通用".to_string()
}

// CodePack: 从项目配置文件中提取元数据
fn extract_metadata(root: &Path, project_type: &str) -> ProjectMetadata {
    let project_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());

    let mut meta = ProjectMetadata {
        name: project_name,
        project_type: project_type.to_string(),
        version: None,
        description: None,
        dependencies: Vec::new(),
        dev_dependencies: Vec::new(),
        entry_point: None,
    };

    match project_type {
        "Node.js" | "Next.js" | "Vite" | "Nuxt.js" => {
            extract_package_json(root, &mut meta);
        }
        "Python" => {
            extract_python_meta(root, &mut meta);
        }
        "Rust" => {
            extract_cargo_toml(root, &mut meta);
        }
        "Go" => {
            extract_go_mod(root, &mut meta);
        }
        "Flutter / Dart" => {
            extract_pubspec_yaml(root, &mut meta);
        }
        "Java / Maven" => {
            extract_pom_xml(root, &mut meta);
        }
        "Android / Gradle" | "Gradle" => {
            // Gradle 项目也可能有 settings.gradle 或 build.gradle
            extract_gradle_meta(root, &mut meta);
        }
        _ => {}
    }

    meta
}

fn extract_package_json(root: &Path, meta: &mut ProjectMetadata) {
    if let Ok(content) = fs::read_to_string(root.join("package.json")) {
        if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = pkg.get("name").and_then(|v| v.as_str()) {
                meta.name = name.to_string();
            }
            if let Some(ver) = pkg.get("version").and_then(|v| v.as_str()) {
                meta.version = Some(ver.to_string());
            }
            if let Some(desc) = pkg.get("description").and_then(|v| v.as_str()) {
                if !desc.is_empty() {
                    meta.description = Some(desc.to_string());
                }
            }
            if let Some(main) = pkg.get("main").and_then(|v| v.as_str()) {
                meta.entry_point = Some(main.to_string());
            }
            if let Some(deps) = pkg.get("dependencies").and_then(|v| v.as_object()) {
                meta.dependencies = deps.keys().cloned().collect();
            }
            if let Some(deps) = pkg.get("devDependencies").and_then(|v| v.as_object()) {
                meta.dev_dependencies = deps.keys().cloned().collect();
            }
        }
    }
}

fn extract_cargo_toml(root: &Path, meta: &mut ProjectMetadata) {
    if let Ok(content) = fs::read_to_string(root.join("Cargo.toml")) {
        if let Ok(doc) = content.parse::<toml::Table>() {
            if let Some(pkg) = doc.get("package").and_then(|v| v.as_table()) {
                if let Some(name) = pkg.get("name").and_then(|v| v.as_str()) {
                    meta.name = name.to_string();
                }
                if let Some(ver) = pkg.get("version").and_then(|v| v.as_str()) {
                    meta.version = Some(ver.to_string());
                }
                if let Some(desc) = pkg.get("description").and_then(|v| v.as_str()) {
                    if !desc.is_empty() {
                        meta.description = Some(desc.to_string());
                    }
                }
            }
            if let Some(deps) = doc.get("dependencies").and_then(|v| v.as_table()) {
                meta.dependencies = deps.keys().cloned().collect();
            }
            if let Some(deps) = doc.get("dev-dependencies").and_then(|v| v.as_table()) {
                meta.dev_dependencies = deps.keys().cloned().collect();
            }
        }
    }
}

fn extract_python_meta(root: &Path, meta: &mut ProjectMetadata) {
    // 尝试 pyproject.toml
    if let Ok(content) = fs::read_to_string(root.join("pyproject.toml")) {
        if let Ok(doc) = content.parse::<toml::Table>() {
            if let Some(project) = doc.get("project").and_then(|v| v.as_table()) {
                if let Some(name) = project.get("name").and_then(|v| v.as_str()) {
                    meta.name = name.to_string();
                }
                if let Some(ver) = project.get("version").and_then(|v| v.as_str()) {
                    meta.version = Some(ver.to_string());
                }
                if let Some(desc) = project.get("description").and_then(|v| v.as_str()) {
                    if !desc.is_empty() {
                        meta.description = Some(desc.to_string());
                    }
                }
                if let Some(deps) = project.get("dependencies").and_then(|v| v.as_array()) {
                    meta.dependencies = deps
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.split(&['>', '<', '=', '~', '!', ';'][..]).next().unwrap_or(s).trim().to_string())
                        .collect();
                }
            }
        }
    }
    // fallback: requirements.txt
    if meta.dependencies.is_empty() {
        if let Ok(content) = fs::read_to_string(root.join("requirements.txt")) {
            meta.dependencies = content
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .map(|l| l.split(&['>', '<', '=', '~', '!', ';'][..]).next().unwrap_or(l).trim().to_string())
                .collect();
        }
    }
    // 入口文件检测
    for entry in &["main.py", "app.py", "manage.py", "run.py"] {
        if root.join(entry).exists() {
            meta.entry_point = Some(entry.to_string());
            break;
        }
    }
}

fn extract_go_mod(root: &Path, meta: &mut ProjectMetadata) {
    if let Ok(content) = fs::read_to_string(root.join("go.mod")) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("module ") {
                meta.name = trimmed.strip_prefix("module ").unwrap_or("").trim().to_string();
            }
            if trimmed.starts_with("go ") {
                meta.version = Some(trimmed.strip_prefix("go ").unwrap_or("").trim().to_string());
            }
        }
        // 提取 require 块中的依赖
        let mut in_require = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "require (" {
                in_require = true;
                continue;
            }
            if trimmed == ")" {
                in_require = false;
                continue;
            }
            if in_require && !trimmed.is_empty() && !trimmed.starts_with("//") {
                if let Some(dep) = trimmed.split_whitespace().next() {
                    meta.dependencies.push(dep.to_string());
                }
            }
        }
    }
    if root.join("main.go").exists() {
        meta.entry_point = Some("main.go".to_string());
    }
}

fn extract_pubspec_yaml(root: &Path, meta: &mut ProjectMetadata) {
    // 简单行解析 pubspec.yaml（避免引入 yaml 库）
    if let Ok(content) = fs::read_to_string(root.join("pubspec.yaml")) {
        let mut in_deps = false;
        let mut in_dev_deps = false;
        for line in content.lines() {
            let trimmed = line.trim();
            // 顶层 key
            if !line.starts_with(' ') && !line.starts_with('\t') {
                in_deps = false;
                in_dev_deps = false;
                if trimmed.starts_with("name:") {
                    meta.name = trimmed.strip_prefix("name:").unwrap_or("").trim().to_string();
                } else if trimmed.starts_with("version:") {
                    meta.version = Some(trimmed.strip_prefix("version:").unwrap_or("").trim().trim_matches('"').trim_matches('\'').to_string());
                } else if trimmed.starts_with("description:") {
                    let desc = trimmed.strip_prefix("description:").unwrap_or("").trim().trim_matches('"').trim_matches('\'').to_string();
                    if !desc.is_empty() {
                        meta.description = Some(desc);
                    }
                } else if trimmed == "dependencies:" {
                    in_deps = true;
                } else if trimmed == "dev_dependencies:" {
                    in_dev_deps = true;
                }
            } else if (in_deps || in_dev_deps) && trimmed.contains(':') {
                let dep_name = trimmed.split(':').next().unwrap_or("").trim().to_string();
                if !dep_name.is_empty() && dep_name != "sdk" {
                    if in_deps {
                        meta.dependencies.push(dep_name);
                    } else {
                        meta.dev_dependencies.push(dep_name);
                    }
                }
            }
        }
    }
    if root.join("lib/main.dart").exists() {
        meta.entry_point = Some("lib/main.dart".to_string());
    }
}

fn extract_pom_xml(root: &Path, meta: &mut ProjectMetadata) {
    // 简单文本解析 pom.xml（避免引入 XML 库）
    if let Ok(content) = fs::read_to_string(root.join("pom.xml")) {
        // 提取顶层 artifactId 和 version
        if let Some(aid) = extract_xml_tag(&content, "artifactId") {
            meta.name = aid;
        }
        if let Some(ver) = extract_xml_tag(&content, "version") {
            meta.version = Some(ver);
        }
        if let Some(desc) = extract_xml_tag(&content, "description") {
            if !desc.is_empty() {
                meta.description = Some(desc);
            }
        }
        // 提取 dependencies 中的 artifactId
        let mut in_deps = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("<dependencies>") {
                in_deps = true;
            }
            if trimmed.contains("</dependencies>") {
                in_deps = false;
            }
            if in_deps {
                if let Some(dep) = extract_xml_tag(trimmed, "artifactId") {
                    meta.dependencies.push(dep);
                }
            }
        }
    }
}

fn extract_gradle_meta(root: &Path, meta: &mut ProjectMetadata) {
    // 从 settings.gradle 或 settings.gradle.kts 提取项目名
    for settings_file in &["settings.gradle.kts", "settings.gradle"] {
        if let Ok(content) = fs::read_to_string(root.join(settings_file)) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("rootProject.name") {
                    let name = trimmed
                        .split('=')
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                    if !name.is_empty() {
                        meta.name = name;
                    }
                }
            }
            break;
        }
    }
}

fn extract_xml_tag(text: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    if let Some(start) = text.find(&open) {
        let after = start + open.len();
        if let Some(end) = text[after..].find(&close) {
            return Some(text[after..after + end].trim().to_string());
        }
    }
    None
}

fn build_file_tree(root: &Path) -> FileNode {
    let root_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root.to_string_lossy().to_string());

    let mut root_node = FileNode {
        name: root_name,
        path: root.to_string_lossy().to_string(),
        is_dir: true,
        children: Vec::new(),
        checked: true,
    };

    build_tree_recursive(root, &mut root_node);
    sort_tree(&mut root_node);
    root_node
}

fn build_tree_recursive(dir: &Path, parent: &mut FileNode) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    let mut entries_vec: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries_vec.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for entry in entries_vec {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if is_excluded_dir(&name) {
                continue;
            }
            let mut dir_node = FileNode {
                name,
                path: path.to_string_lossy().to_string(),
                is_dir: true,
                children: Vec::new(),
                checked: true,
            };
            build_tree_recursive(&path, &mut dir_node);
            if !dir_node.children.is_empty() {
                parent.children.push(dir_node);
            }
        } else if is_source_file(&name) {
            parent.children.push(FileNode {
                name,
                path: path.to_string_lossy().to_string(),
                is_dir: false,
                children: Vec::new(),
                checked: true,
            });
        }
    }
}

fn sort_tree(node: &mut FileNode) {
    node.children.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    for child in &mut node.children {
        if child.is_dir {
            sort_tree(child);
        }
    }
}

fn count_files(node: &FileNode) -> u32 {
    let mut count = 0;
    if !node.is_dir {
        count += 1;
    }
    for child in &node.children {
        count += count_files(child);
    }
    count
}

// CodePack: 构建带项目摘要头部的导出内容（增强版，包含元数据）
fn build_pack_content(
    paths: &[String],
    project_path: &str,
    project_type: &str,
) -> PackResult {
    let root = Path::new(project_path);
    let meta = extract_metadata(root, project_type);

    let mut body = String::new();
    let mut file_count: u32 = 0;
    let mut total_bytes: u64 = 0;

    for path in paths {
        let file_path = Path::new(path);
        let relative = file_path
            .strip_prefix(root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .replace('\\', "/");

        if let Ok(content) = fs::read_to_string(path) {
            total_bytes += content.len() as u64;
            file_count += 1;
            body.push_str(&format!("// ===== {} =====\n", relative));
            body.push_str(&content);
            body.push_str("\n\n");
        }
    }

    let estimated_tokens = total_bytes as f64 / 4.0;

    // CodePack: 构建增强的项目摘要头部
    let mut header = String::new();
    header.push_str(&format!("# Project: {}\n", meta.name));
    header.push_str(&format!("# Type: {}\n", meta.project_type));
    if let Some(ref ver) = meta.version {
        header.push_str(&format!("# Version: {}\n", ver));
    }
    if let Some(ref desc) = meta.description {
        header.push_str(&format!("# Description: {}\n", desc));
    }
    if let Some(ref entry) = meta.entry_point {
        header.push_str(&format!("# Entry Point: {}\n", entry));
    }
    if !meta.dependencies.is_empty() {
        header.push_str(&format!("# Dependencies: {}\n", meta.dependencies.join(", ")));
    }
    if !meta.dev_dependencies.is_empty() {
        header.push_str(&format!("# Dev Dependencies: {}\n", meta.dev_dependencies.join(", ")));
    }
    header.push_str(&format!("# Files: {}\n", file_count));
    header.push_str(&format!("# Estimated Tokens: {}\n", format_tokens(estimated_tokens)));
    header.push_str("============================================================\n\n");

    let content = format!("{}{}", header, body);

    PackResult {
        content,
        file_count,
        total_bytes,
        estimated_tokens,
    }
}

fn format_tokens(tokens: f64) -> String {
    if tokens >= 1_000_000.0 {
        format!("{:.1}M", tokens / 1_000_000.0)
    } else if tokens >= 1000.0 {
        format!("{:.1}K", tokens / 1000.0)
    } else {
        format!("{:.0}", tokens)
    }
}

// ─── Tauri Commands ────────────────────────────────────────────

#[tauri::command]
fn scan_directory(path: String) -> Result<ScanResult, String> {
    let root = Path::new(&path);
    if !root.exists() || !root.is_dir() {
        return Err("Path does not exist or is not a directory".to_string());
    }

    let project_type = detect_project_type(root);
    let tree = build_file_tree(root);
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
fn read_file_content(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
fn save_project_config(project_path: String, checked_paths: Vec<String>) -> Result<(), String> {
    let mut config = load_app_config();
    let now = chrono_now();
    // CodePack: 保留已有的 presets 和 pinned 状态
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
fn load_project_config(project_path: String) -> Result<Option<ProjectConfig>, String> {
    let config = load_app_config();
    Ok(config.projects.get(&project_path).cloned())
}

// CodePack: 返回 tokens 和文件大小
#[tauri::command]
fn estimate_tokens(paths: Vec<String>) -> Result<TokenEstimate, String> {
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

// CodePack: pack_files 现在返回 PackResult，包含摘要头部
#[tauri::command]
fn pack_files(
    paths: Vec<String>,
    project_path: String,
    project_type: String,
) -> Result<PackResult, String> {
    Ok(build_pack_content(&paths, &project_path, &project_type))
}

#[tauri::command]
fn copy_to_clipboard(content: String, app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard()
        .write_text(&content)
        .map_err(|e| e.to_string())
}

// CodePack: export_to_file 接受 save_path 参数（前端通过 save dialog 获取）
#[tauri::command]
fn export_to_file(
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

// CodePack: 打开文件所在目录
#[tauri::command]
fn open_directory(path: String) -> Result<(), String> {
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

// CodePack: 获取文件大小
#[tauri::command]
fn get_file_size(path: String) -> Result<u64, String> {
    fs::metadata(&path)
        .map(|m| m.len())
        .map_err(|e| format!("Failed to get file size: {}", e))
}

// CodePack: Preset 管理命令
#[tauri::command]
fn save_preset(
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
fn delete_preset(project_path: String, preset_name: String) -> Result<(), String> {
    let mut config = load_app_config();
    if let Some(project) = config.projects.get_mut(&project_path) {
        project.presets.remove(&preset_name);
    }
    save_app_config(&config)
}

#[tauri::command]
fn list_presets(project_path: String) -> Result<HashMap<String, Vec<String>>, String> {
    let config = load_app_config();
    Ok(config
        .projects
        .get(&project_path)
        .map(|p| p.presets.clone())
        .unwrap_or_default())
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

// ─── App Entry ─────────────────────────────────────────────────

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
