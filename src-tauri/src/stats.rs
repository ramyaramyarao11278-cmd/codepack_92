use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::types::{LangStat, ProjectStats};

pub fn ext_to_language(ext: &str) -> &str {
    match ext.to_lowercase().as_str() {
        "rs" => "Rust",
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" => "JavaScript",
        "vue" => "Vue",
        "svelte" => "Svelte",
        "py" => "Python",
        "kt" | "kts" => "Kotlin",
        "java" => "Java",
        "dart" => "Dart",
        "go" => "Go",
        "rb" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "c" => "C",
        "cpp" | "cc" | "cxx" => "C++",
        "h" | "hpp" => "C/C++ Header",
        "cs" => "C#",
        "scala" => "Scala",
        "html" => "HTML",
        "css" => "CSS",
        "scss" | "sass" | "less" => "CSS (preprocessor)",
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "xml" => "XML",
        "md" | "mdx" => "Markdown",
        "sql" => "SQL",
        "sh" | "bash" | "zsh" | "fish" => "Shell",
        "bat" | "ps1" => "PowerShell/Batch",
        "graphql" | "gql" => "GraphQL",
        "proto" => "Protobuf",
        "tf" | "hcl" => "Terraform/HCL",
        "lua" => "Lua",
        "r" => "R",
        "jl" => "Julia",
        _ => ext,
    }
}

pub fn compute_project_stats(paths: &[String]) -> ProjectStats {
    let mut lang_map: HashMap<String, (String, u32, u64, u64)> = HashMap::new();
    let mut total_files: u32 = 0;
    let mut total_lines: u64 = 0;
    let mut total_bytes: u64 = 0;

    for path in paths {
        if let Ok(content) = fs::read_to_string(path) {
            let bytes = content.len() as u64;
            let lines = content.lines().count() as u64;
            total_files += 1;
            total_lines += lines;
            total_bytes += bytes;

            let ext = Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("other")
                .to_lowercase();
            let lang = ext_to_language(&ext).to_string();

            let entry = lang_map.entry(lang.clone()).or_insert((ext.clone(), 0, 0, 0));
            entry.1 += 1;
            entry.2 += lines;
            entry.3 += bytes;
        }
    }

    let mut languages: Vec<LangStat> = lang_map
        .into_iter()
        .map(|(lang, (ext, fc, lc, bc))| LangStat {
            language: lang,
            extension: ext,
            file_count: fc,
            line_count: lc,
            byte_count: bc,
        })
        .collect();
    languages.sort_by(|a, b| b.line_count.cmp(&a.line_count));

    ProjectStats {
        total_files,
        total_lines,
        total_bytes,
        languages,
    }
}
