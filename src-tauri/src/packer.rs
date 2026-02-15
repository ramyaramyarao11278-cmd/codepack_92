use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use tiktoken_rs::CoreBPE;

use crate::metadata::extract_metadata;
use crate::types::{ExportFormat, PackResult, ProjectMetadata, SkippedFile};

const DEFAULT_MAX_FILE_BYTES: u64 = 1_048_576; // 1 MB
const MAX_FILE_COUNT: usize = 5_000;

static BPE: LazyLock<CoreBPE> = LazyLock::new(|| {
    tiktoken_rs::cl100k_base().expect("failed to load cl100k_base tokenizer")
});

pub fn build_pack_content(
    paths: &[String],
    project_path: &str,
    project_type: &str,
    format: &ExportFormat,
) -> PackResult {
    build_pack_content_with_limit(paths, project_path, project_type, format, None)
}

pub fn build_pack_content_with_limit(
    paths: &[String],
    project_path: &str,
    project_type: &str,
    format: &ExportFormat,
    max_file_bytes: Option<u64>,
) -> PackResult {
    let root = Path::new(project_path);
    let meta = extract_metadata(root, project_type);
    let limit = max_file_bytes.unwrap_or(DEFAULT_MAX_FILE_BYTES);

    let mut body = String::new();
    let mut file_count: u32 = 0;
    let mut total_bytes: u64 = 0;
    let mut skipped_files: Vec<SkippedFile> = Vec::new();

    for path in paths {
        let file_path = Path::new(path);
        let relative = file_path
            .strip_prefix(root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .replace('\\', "/");

        // Check file size before reading
        let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if file_size > limit {
            skipped_files.push(SkippedFile {
                path: relative.clone(),
                reason: format!("exceeds {}KB limit ({}KB)", limit / 1024, file_size / 1024),
                size_bytes: file_size,
            });
            // Insert a placeholder in the output
            match format {
                ExportFormat::Plain => {
                    let comment = comment_delimiter(&relative);
                    body.push_str(&format!(
                        "{} ===== {} [SKIPPED: {}KB > {}KB limit] =====\n\n",
                        comment, relative, file_size / 1024, limit / 1024
                    ));
                }
                ExportFormat::Markdown => {
                    body.push_str(&format!(
                        "## {} *(skipped: {}KB > {}KB limit)*\n\n",
                        relative, file_size / 1024, limit / 1024
                    ));
                }
                ExportFormat::Xml => {
                    body.push_str(&format!(
                        "<file path=\"{}\" skipped=\"true\" size_kb=\"{}\" />\n\n",
                        xml_escape(&relative), file_size / 1024
                    ));
                }
            }
            continue;
        }

        // Binary file detection: skip non-UTF-8 files
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                skipped_files.push(SkippedFile {
                    path: relative.clone(),
                    reason: "binary or unreadable file".to_string(),
                    size_bytes: file_size,
                });
                continue;
            }
        };

        // Enforce max file count
        if file_count as usize >= MAX_FILE_COUNT {
            skipped_files.push(SkippedFile {
                path: relative.clone(),
                reason: format!("exceeds {} file limit", MAX_FILE_COUNT),
                size_bytes: file_size,
            });
            continue;
        }

        {
            total_bytes += content.len() as u64;
            file_count += 1;

            match format {
                ExportFormat::Plain => {
                    let comment = comment_delimiter(&relative);
                    body.push_str(&format!("{} ===== {} =====\n", comment, relative));
                    body.push_str(&content);
                    body.push_str("\n\n");
                }
                ExportFormat::Markdown => {
                    let ext = Path::new(&relative)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    body.push_str(&format!("## {}\n\n```{}\n", relative, ext));
                    body.push_str(&content);
                    if !content.ends_with('\n') {
                        body.push('\n');
                    }
                    body.push_str("```\n\n");
                }
                ExportFormat::Xml => {
                    let escaped_path = xml_escape(&relative);
                    body.push_str(&format!("<file path=\"{}\">\n<![CDATA[\n", escaped_path));
                    body.push_str(&content);
                    if !content.ends_with('\n') {
                        body.push('\n');
                    }
                    body.push_str("]]>\n</file>\n\n");
                }
            }
        }
    }

    let estimated_tokens = BPE.encode_ordinary(&body).len() as f64;

    // Collect relative paths for tree overview
    let relative_paths: Vec<String> = paths
        .iter()
        .filter_map(|p| {
            Path::new(p)
                .strip_prefix(root)
                .ok()
                .map(|r| r.to_string_lossy().replace('\\', "/"))
        })
        .collect();

    let header = build_header(&meta, file_count, estimated_tokens, format);
    let tree_overview = build_tree_overview(&relative_paths, format);
    let footer = build_footer(format);
    let content = format!("{}{}{}{}", header, tree_overview, body, footer);

    PackResult {
        content,
        file_count,
        total_bytes,
        estimated_tokens,
        skipped_files,
    }
}

/// Extended pack with optional git diff and instruction sections
pub fn build_pack_content_extended(
    paths: &[String],
    project_path: &str,
    project_type: &str,
    format: &ExportFormat,
    max_file_bytes: Option<u64>,
    diffs: Option<&std::collections::HashMap<String, String>>,
    instruction: Option<&str>,
) -> PackResult {
    let mut result = build_pack_content_with_limit(paths, project_path, project_type, format, max_file_bytes);

    let mut extra = String::new();

    // Append git diffs section
    if let Some(diff_map) = diffs {
        if !diff_map.is_empty() {
            match format {
                ExportFormat::Plain => {
                    extra.push_str("# ===== Git Diff (Working Changes) =====\n\n");
                    for (path, diff) in diff_map {
                        extra.push_str(&format!("# --- {} ---\n", path));
                        extra.push_str(diff);
                        if !diff.ends_with('\n') { extra.push('\n'); }
                        extra.push('\n');
                    }
                }
                ExportFormat::Markdown => {
                    extra.push_str("## Git Diff (Working Changes)\n\n");
                    for (path, diff) in diff_map {
                        extra.push_str(&format!("### {}\n\n```diff\n", path));
                        extra.push_str(diff);
                        if !diff.ends_with('\n') { extra.push('\n'); }
                        extra.push_str("```\n\n");
                    }
                }
                ExportFormat::Xml => {
                    extra.push_str("<diffs>\n");
                    for (path, diff) in diff_map {
                        extra.push_str(&format!("<diff path=\"{}\">\n<![CDATA[\n", xml_escape(path)));
                        extra.push_str(diff);
                        if !diff.ends_with('\n') { extra.push('\n'); }
                        extra.push_str("]]>\n</diff>\n");
                    }
                    extra.push_str("</diffs>\n\n");
                }
            }
        }
    }

    // Append instruction section
    if let Some(instr) = instruction {
        if !instr.is_empty() {
            match format {
                ExportFormat::Plain => {
                    extra.push_str("# ===== Review Instructions =====\n");
                    extra.push_str(instr);
                    if !instr.ends_with('\n') { extra.push('\n'); }
                    extra.push('\n');
                }
                ExportFormat::Markdown => {
                    extra.push_str("## Review Instructions\n\n");
                    extra.push_str(instr);
                    if !instr.ends_with('\n') { extra.push('\n'); }
                    extra.push('\n');
                }
                ExportFormat::Xml => {
                    extra.push_str("<instruction>\n<![CDATA[\n");
                    extra.push_str(instr);
                    if !instr.ends_with('\n') { extra.push('\n'); }
                    extra.push_str("]]>\n</instruction>\n\n");
                }
            }
        }
    }

    if !extra.is_empty() {
        result.content.push_str(&extra);
        result.estimated_tokens = BPE.encode_ordinary(&result.content).len() as f64;
    }

    result
}

fn build_header(
    meta: &ProjectMetadata,
    file_count: u32,
    estimated_tokens: f64,
    format: &ExportFormat,
) -> String {
    match format {
        ExportFormat::Plain => build_plain_header(meta, file_count, estimated_tokens),
        ExportFormat::Markdown => build_markdown_header(meta, file_count, estimated_tokens),
        ExportFormat::Xml => build_xml_header(meta, file_count, estimated_tokens),
    }
}

fn build_plain_header(meta: &ProjectMetadata, file_count: u32, estimated_tokens: f64) -> String {
    let mut h = String::new();
    h.push_str(&format!("# Project: {}\n", meta.name));
    h.push_str(&format!("# Type: {}\n", meta.project_type));
    if let Some(ref ver) = meta.version {
        h.push_str(&format!("# Version: {}\n", ver));
    }
    if let Some(ref desc) = meta.description {
        h.push_str(&format!("# Description: {}\n", desc));
    }
    if let Some(ref entry) = meta.entry_point {
        h.push_str(&format!("# Entry Point: {}\n", entry));
    }
    if !meta.runtime.is_empty() {
        h.push_str(&format!("# Runtime: {}\n", meta.runtime.join(", ")));
    }
    if !meta.dependencies.is_empty() {
        h.push_str(&format!("# Dependencies: {}\n", meta.dependencies.join(", ")));
    }
    if !meta.dev_dependencies.is_empty() {
        h.push_str(&format!("# Dev Dependencies: {}\n", meta.dev_dependencies.join(", ")));
    }
    if !meta.requirements.is_empty() {
        h.push_str("# Requirements:\n");
        for req in &meta.requirements {
            h.push_str(&format!("#   {}\n", req));
        }
    }
    h.push_str(&format!("# Files: {}\n", file_count));
    h.push_str(&format!("# Estimated Tokens: {}\n", format_tokens(estimated_tokens)));
    h.push_str("============================================================\n\n");
    h
}

fn build_markdown_header(meta: &ProjectMetadata, file_count: u32, estimated_tokens: f64) -> String {
    let mut h = String::new();
    h.push_str(&format!("# {}\n\n", meta.name));
    h.push_str(&format!("- **Type:** {}\n", meta.project_type));
    if let Some(ref ver) = meta.version {
        h.push_str(&format!("- **Version:** {}\n", ver));
    }
    if let Some(ref desc) = meta.description {
        h.push_str(&format!("- **Description:** {}\n", desc));
    }
    if let Some(ref entry) = meta.entry_point {
        h.push_str(&format!("- **Entry Point:** `{}`\n", entry));
    }
    if !meta.runtime.is_empty() {
        h.push_str(&format!("- **Runtime:** {}\n", meta.runtime.join(", ")));
    }
    if !meta.dependencies.is_empty() {
        h.push_str(&format!("- **Dependencies ({}):** {}\n", meta.dependencies.len(), meta.dependencies.join(", ")));
    }
    if !meta.dev_dependencies.is_empty() {
        h.push_str(&format!("- **Dev Dependencies ({}):** {}\n", meta.dev_dependencies.len(), meta.dev_dependencies.join(", ")));
    }
    if !meta.requirements.is_empty() {
        h.push_str("- **Requirements:**\n");
        for req in &meta.requirements {
            h.push_str(&format!("  - `{}`\n", req));
        }
    }
    h.push_str(&format!("- **Files:** {}\n", file_count));
    h.push_str(&format!("- **Estimated Tokens:** {}\n", format_tokens(estimated_tokens)));
    h.push_str("\n---\n\n");
    h
}

fn build_xml_header(meta: &ProjectMetadata, file_count: u32, estimated_tokens: f64) -> String {
    let mut h = String::new();
    h.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    h.push_str("<codepack>\n");
    h.push_str("<metadata>\n");
    h.push_str(&format!("  <name>{}</name>\n", xml_escape(&meta.name)));
    h.push_str(&format!("  <type>{}</type>\n", xml_escape(&meta.project_type)));
    if let Some(ref ver) = meta.version {
        h.push_str(&format!("  <version>{}</version>\n", xml_escape(ver)));
    }
    if let Some(ref desc) = meta.description {
        h.push_str(&format!("  <description>{}</description>\n", xml_escape(desc)));
    }
    if let Some(ref entry) = meta.entry_point {
        h.push_str(&format!("  <entry_point>{}</entry_point>\n", xml_escape(entry)));
    }
    if !meta.runtime.is_empty() {
        h.push_str("  <runtime>\n");
        for r in &meta.runtime {
            h.push_str(&format!("    <env>{}</env>\n", xml_escape(r)));
        }
        h.push_str("  </runtime>\n");
    }
    if !meta.dependencies.is_empty() {
        h.push_str("  <dependencies>\n");
        for dep in &meta.dependencies {
            h.push_str(&format!("    <dep>{}</dep>\n", xml_escape(dep)));
        }
        h.push_str("  </dependencies>\n");
    }
    h.push_str(&format!("  <file_count>{}</file_count>\n", file_count));
    h.push_str(&format!("  <estimated_tokens>{}</estimated_tokens>\n", format_tokens(estimated_tokens)));
    h.push_str("</metadata>\n<files>\n\n");
    h
}

// ─── File Tree Overview ────────────────────────────────────────

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
}

fn build_tree_overview(relative_paths: &[String], format: &ExportFormat) -> String {
    if relative_paths.is_empty() {
        return String::new();
    }

    // Build a nested tree from flat paths
    let mut root = TreeNode::default();
    for path in relative_paths {
        let mut current = &mut root;
        for part in path.split('/') {
            current = current.children.entry(part.to_string()).or_default();
        }
    }

    let mut lines: Vec<String> = Vec::new();
    render_tree_node(&root, "", true, &mut lines);

    match format {
        ExportFormat::Plain => {
            let mut out = String::from("# File Tree:\n");
            for line in &lines {
                out.push_str(&format!("#   {}\n", line));
            }
            out.push_str("#\n\n");
            out
        }
        ExportFormat::Markdown => {
            let mut out = String::from("## File Tree\n\n```\n");
            for line in &lines {
                out.push_str(line);
                out.push('\n');
            }
            out.push_str("```\n\n");
            out
        }
        ExportFormat::Xml => {
            let mut out = String::from("<file_tree>\n<![CDATA[\n");
            for line in &lines {
                out.push_str(line);
                out.push('\n');
            }
            out.push_str("]]>\n</file_tree>\n\n");
            out
        }
    }
}

fn render_tree_node(node: &TreeNode, prefix: &str, is_root: bool, lines: &mut Vec<String>) {
    let entries: Vec<_> = node.children.iter().collect();
    let count = entries.len();
    for (i, (name, child)) in entries.iter().enumerate() {
        let is_last = i == count - 1;
        if is_root {
            // Top-level entries have no connector
            let has_children = !child.children.is_empty();
            if has_children {
                lines.push(format!("{}/", name));
                render_tree_node(child, "  ", false, lines);
            } else {
                lines.push(name.to_string());
            }
        } else {
            let connector = if is_last { "└── " } else { "├── " };
            let has_children = !child.children.is_empty();
            if has_children {
                lines.push(format!("{}{}{}/", prefix, connector, name));
                let child_prefix = if is_last {
                    format!("{}    ", prefix)
                } else {
                    format!("{}│   ", prefix)
                };
                render_tree_node(child, &child_prefix, false, lines);
            } else {
                lines.push(format!("{}{}{}", prefix, connector, name));
            }
        }
    }
}

fn build_footer(format: &ExportFormat) -> String {
    match format {
        ExportFormat::Xml => "</files>\n</codepack>\n".to_string(),
        _ => String::new(),
    }
}

fn comment_delimiter(relative_path: &str) -> &'static str {
    let ext = Path::new(relative_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    // Return the ext as a static reference by matching
    match ext.as_str() {
        "html" | "xml" | "svg" | "vue" | "svelte" => "<!--",
        "css" | "scss" | "sass" | "less" => "/*",
        "py" | "rb" | "sh" | "bash" | "zsh" | "fish" | "yaml" | "yml" | "toml" | "ini"
        | "cfg" | "conf" | "r" | "jl" | "pl" => "#",
        "sql" | "lua" | "hs" => "--",
        "bat" => "REM",
        _ => "//",
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn format_tokens(tokens: f64) -> String {
    if tokens >= 1_000_000.0 {
        format!("{:.1}M", tokens / 1_000_000.0)
    } else if tokens >= 1000.0 {
        format!("{:.1}K", tokens / 1000.0)
    } else {
        format!("{:.0}", tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {\n    println!(\"hello\");\n}\n").unwrap();
        fs::write(dir.path().join("style.css"), "body { color: red; }\n").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();
        dir
    }

    #[test]
    fn test_comment_delimiter() {
        assert_eq!(comment_delimiter("main.rs"), "//");
        assert_eq!(comment_delimiter("app.py"), "#");
        assert_eq!(comment_delimiter("style.css"), "/*");
        assert_eq!(comment_delimiter("index.html"), "<!--");
        assert_eq!(comment_delimiter("query.sql"), "--");
        assert_eq!(comment_delimiter("run.bat"), "REM");
        assert_eq!(comment_delimiter("config.yaml"), "#");
        assert_eq!(comment_delimiter("unknown"), "//");
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("a<b>c&d\"e"), "a&lt;b&gt;c&amp;d&quot;e");
        assert_eq!(xml_escape("normal"), "normal");
    }

    #[test]
    fn test_plain_format() {
        let dir = setup_test_project();
        let paths = vec![
            dir.path().join("main.rs").to_string_lossy().to_string(),
            dir.path().join("style.css").to_string_lossy().to_string(),
        ];
        let result = build_pack_content(&paths, &dir.path().to_string_lossy(), "Rust", &ExportFormat::Plain);
        assert_eq!(result.file_count, 2);
        assert!(result.content.contains("// ===== main.rs ====="));
        assert!(result.content.contains("/* ===== style.css ====="));
        assert!(result.content.contains("# Project:"));
    }

    #[test]
    fn test_markdown_format() {
        let dir = setup_test_project();
        let paths = vec![dir.path().join("main.rs").to_string_lossy().to_string()];
        let result = build_pack_content(&paths, &dir.path().to_string_lossy(), "Rust", &ExportFormat::Markdown);
        assert!(result.content.contains("## main.rs"));
        assert!(result.content.contains("```rs"));
        assert!(result.content.contains("- **Type:** Rust"));
    }

    #[test]
    fn test_xml_format() {
        let dir = setup_test_project();
        let paths = vec![dir.path().join("main.rs").to_string_lossy().to_string()];
        let result = build_pack_content(&paths, &dir.path().to_string_lossy(), "Rust", &ExportFormat::Xml);
        assert!(result.content.contains("<?xml version="));
        assert!(result.content.contains("<file path=\"main.rs\">"));
        assert!(result.content.contains("<![CDATA["));
        assert!(result.content.contains("</codepack>"));
    }

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(500.0), "500");
        assert_eq!(format_tokens(1500.0), "1.5K");
        assert_eq!(format_tokens(1_500_000.0), "1.5M");
    }

    #[test]
    fn test_tree_overview_plain() {
        let paths = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "Cargo.toml".to_string(),
        ];
        let overview = build_tree_overview(&paths, &ExportFormat::Plain);
        assert!(overview.contains("# File Tree:"));
        assert!(overview.contains("src/"));
        assert!(overview.contains("main.rs"));
        assert!(overview.contains("lib.rs"));
        assert!(overview.contains("Cargo.toml"));
    }

    #[test]
    fn test_tree_overview_markdown() {
        let paths = vec![
            "src/main.rs".to_string(),
            "README.md".to_string(),
        ];
        let overview = build_tree_overview(&paths, &ExportFormat::Markdown);
        assert!(overview.contains("## File Tree"));
        assert!(overview.contains("```"));
        assert!(overview.contains("src/"));
        assert!(overview.contains("main.rs"));
    }

    #[test]
    fn test_tree_overview_xml() {
        let paths = vec!["main.rs".to_string()];
        let overview = build_tree_overview(&paths, &ExportFormat::Xml);
        assert!(overview.contains("<file_tree>"));
        assert!(overview.contains("main.rs"));
        assert!(overview.contains("</file_tree>"));
    }

    #[test]
    fn test_tree_overview_empty() {
        let paths: Vec<String> = vec![];
        let overview = build_tree_overview(&paths, &ExportFormat::Plain);
        assert!(overview.is_empty());
    }

    #[test]
    fn test_large_file_skipped() {
        let dir = TempDir::new().unwrap();
        // Create a small file and a "large" file (>50 bytes with a 50-byte limit)
        fs::write(dir.path().join("small.rs"), "fn main() {}").unwrap();
        let large_content = "x".repeat(200);
        fs::write(dir.path().join("big.rs"), &large_content).unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();

        let paths = vec![
            dir.path().join("small.rs").to_string_lossy().to_string(),
            dir.path().join("big.rs").to_string_lossy().to_string(),
        ];
        let result = build_pack_content_with_limit(
            &paths, &dir.path().to_string_lossy(), "Rust", &ExportFormat::Plain, Some(100),
        );
        // small.rs should be included, big.rs should be skipped
        assert_eq!(result.file_count, 1);
        assert_eq!(result.skipped_files.len(), 1);
        assert!(result.skipped_files[0].path.contains("big.rs"));
        assert!(result.content.contains("SKIPPED"));
        assert!(result.content.contains("small.rs"));
    }

    #[test]
    fn test_no_skip_when_limit_high() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();

        let paths = vec![dir.path().join("main.rs").to_string_lossy().to_string()];
        let result = build_pack_content_with_limit(
            &paths, &dir.path().to_string_lossy(), "Rust", &ExportFormat::Plain, Some(10_000_000),
        );
        assert_eq!(result.file_count, 1);
        assert!(result.skipped_files.is_empty());
    }

    #[test]
    fn test_binary_file_skipped() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        // Write invalid UTF-8 bytes
        fs::write(dir.path().join("image.rs"), &[0xFF, 0xFE, 0x00, 0x01, 0x80, 0x90]).unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n").unwrap();

        let paths = vec![
            dir.path().join("main.rs").to_string_lossy().to_string(),
            dir.path().join("image.rs").to_string_lossy().to_string(),
        ];
        let result = build_pack_content_with_limit(
            &paths, &dir.path().to_string_lossy(), "Rust", &ExportFormat::Plain, Some(10_000_000),
        );
        assert_eq!(result.file_count, 1);
        assert_eq!(result.skipped_files.len(), 1);
        assert!(result.skipped_files[0].reason.contains("binary"));
    }

    #[test]
    fn test_export_contains_tree() {
        let dir = setup_test_project();
        let paths = vec![
            dir.path().join("main.rs").to_string_lossy().to_string(),
            dir.path().join("style.css").to_string_lossy().to_string(),
        ];
        let result = build_pack_content(&paths, &dir.path().to_string_lossy(), "Rust", &ExportFormat::Markdown);
        // Should contain both the file tree overview and the actual file content
        assert!(result.content.contains("## File Tree"));
        assert!(result.content.contains("## main.rs"));
    }
}
