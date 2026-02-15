use std::fs;
use std::path::Path;

use crate::metadata::extract_metadata;
use crate::types::PackResult;

pub fn build_pack_content(
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
    if !meta.runtime.is_empty() {
        header.push_str(&format!("# Runtime: {}\n", meta.runtime.join(", ")));
    }
    if !meta.dependencies.is_empty() {
        header.push_str(&format!("# Dependencies: {}\n", meta.dependencies.join(", ")));
    }
    if !meta.dev_dependencies.is_empty() {
        header.push_str(&format!("# Dev Dependencies: {}\n", meta.dev_dependencies.join(", ")));
    }
    if !meta.requirements.is_empty() {
        header.push_str("# Requirements:\n");
        for req in &meta.requirements {
            header.push_str(&format!("#   {}\n", req));
        }
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

pub fn format_tokens(tokens: f64) -> String {
    if tokens >= 1_000_000.0 {
        format!("{:.1}M", tokens / 1_000_000.0)
    } else if tokens >= 1000.0 {
        format!("{:.1}K", tokens / 1000.0)
    } else {
        format!("{:.0}", tokens)
    }
}
