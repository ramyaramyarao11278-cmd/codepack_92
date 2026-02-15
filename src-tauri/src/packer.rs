use std::fs;
use std::path::Path;

use crate::metadata::extract_metadata;
use crate::types::{ExportFormat, PackResult, ProjectMetadata};

pub fn build_pack_content(
    paths: &[String],
    project_path: &str,
    project_type: &str,
    format: &ExportFormat,
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

    let estimated_tokens = total_bytes as f64 / 4.0;
    let header = build_header(&meta, file_count, estimated_tokens, format);
    let footer = build_footer(format);
    let content = format!("{}{}{}", header, body, footer);

    PackResult {
        content,
        file_count,
        total_bytes,
        estimated_tokens,
    }
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
}
