use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;

use crate::plugins::PluginDef;
use crate::types::FileNode;

// ─── Constants ─────────────────────────────────────────────────

pub const EXCLUDED_DIRS: &[&str] = &[
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

pub const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "vue", "svelte", "py", "kt", "kts", "java", "dart", "go",
    "rb", "php", "swift", "c", "cpp", "h", "hpp", "cs", "m", "mm", "scala", "clj", "ex",
    "exs", "hs", "lua", "r", "jl", "sql", "sh", "bash", "zsh", "fish", "bat", "ps1", "yml",
    "yaml", "toml", "json", "xml", "html", "css", "scss", "sass", "less", "md", "mdx", "txt",
    "cfg", "ini", "conf", "env", "dockerfile", "makefile", "cmake", "gradle", "properties",
    "gitignore", "editorconfig", "eslintrc", "prettierrc", "graphql", "gql", "proto",
    "tf", "hcl", "nix", "astro", "mod", "sum", "lock",
];

// ─── Helpers ───────────────────────────────────────────────────

pub fn is_excluded_dir(name: &str, extra_excludes: &[String]) -> bool {
    EXCLUDED_DIRS.iter().any(|&excluded| name.eq_ignore_ascii_case(excluded))
        || extra_excludes.iter().any(|excluded| name.eq_ignore_ascii_case(excluded))
}

pub fn is_source_file(name: &str, extra_extensions: &[String]) -> bool {
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
            || extra_extensions.iter().any(|se| se.eq_ignore_ascii_case(ext))
    } else {
        false
    }
}

// CodePack: 带插件支持的项目类型识别
pub fn detect_project_type_with_plugins(root: &Path, plugins: &[PluginDef]) -> String {
    use crate::plugins::plugin_matches;
    // 插件优先匹配
    for plugin in plugins {
        if plugin_matches(plugin, root) {
            return plugin.name.clone();
        }
    }
    detect_project_type(root)
}

// CodePack: 增强的项目类型识别，支持 15+ 种项目类型
pub fn detect_project_type(root: &Path) -> String {
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
    for entry in fs::read_dir(root).into_iter().flatten().flatten() {
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

// ─── File Tree (ignore crate powered) ──────────────────────────

pub fn build_file_tree(root: &Path, extra_excludes: &[String], extra_extensions: &[String]) -> FileNode {
    let root_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root.to_string_lossy().to_string());

    let root_path = root.to_string_lossy().to_string();

    let mut root_node = FileNode {
        name: root_name,
        path: root_path.clone(),
        is_dir: true,
        children: Vec::new(),
        checked: true,
    };

    // Build override rules to exclude directories
    let mut override_builder = OverrideBuilder::new(root);
    for dir in EXCLUDED_DIRS {
        let _ = override_builder.add(&format!("!{}/**", dir));
    }
    for dir in extra_excludes {
        let _ = override_builder.add(&format!("!{}/**", dir));
    }
    // Use ignore::WalkBuilder for parallel traversal + .gitignore support
    let mut walk_builder = WalkBuilder::new(root);
    walk_builder
        .hidden(true)       // skip hidden files/dirs (. prefixed)
        .git_ignore(true)   // respect .gitignore
        .git_global(false)
        .git_exclude(true)
        .sort_by_file_name(|a, b| a.cmp(b));

    if let Ok(overrides) = override_builder.build() {
        walk_builder.overrides(overrides);
    }

    let walker = walk_builder.build();

    // Collect all valid entries into a flat list
    let mut dir_children: HashMap<PathBuf, Vec<FileNode>> = HashMap::new();
    let mut seen_dirs: Vec<PathBuf> = Vec::new();

    for result in walker {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path().to_path_buf();
        // Skip the root itself
        if path == root {
            continue;
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let parent_path = path.parent().unwrap_or(root).to_path_buf();

        if entry.file_type().is_some_and(|ft| ft.is_dir()) {
            // Check our custom exclusion list (ignore crate handles .gitignore)
            if is_excluded_dir(&name, extra_excludes) {
                continue;
            }
            seen_dirs.push(path.clone());
            dir_children.entry(path).or_default();
        } else {
            // Only include source files
            if !is_source_file(&name, extra_extensions) {
                continue;
            }
            let file_node = FileNode {
                name,
                path: path.to_string_lossy().to_string(),
                is_dir: false,
                children: Vec::new(),
                checked: true,
            };
            dir_children.entry(parent_path).or_default().push(file_node);
        }
    }

    // Build tree bottom-up: process dirs from deepest to shallowest
    seen_dirs.sort_by_key(|b| std::cmp::Reverse(b.components().count()));

    for dir_path in &seen_dirs {
        let children = dir_children.remove(dir_path).unwrap_or_default();
        if children.is_empty() {
            continue;
        }
        let dir_name = dir_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let dir_node = FileNode {
            name: dir_name,
            path: dir_path.to_string_lossy().to_string(),
            is_dir: true,
            children,
            checked: true,
        };
        let parent = dir_path.parent().unwrap_or(root).to_path_buf();
        dir_children.entry(parent).or_default().push(dir_node);
    }

    // Attach remaining children to root
    if let Some(children) = dir_children.remove(&root.to_path_buf()) {
        root_node.children = children;
    }

    sort_tree(&mut root_node);
    root_node
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

pub fn count_files(node: &FileNode) -> u32 {
    let mut count = 0;
    if !node.is_dir {
        count += 1;
    }
    for child in &node.children {
        count += count_files(child);
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_excluded_dir_builtin() {
        assert!(is_excluded_dir("node_modules", &[]));
        assert!(is_excluded_dir("Node_Modules", &[]));
        assert!(is_excluded_dir(".git", &[]));
        assert!(is_excluded_dir("target", &[]));
        assert!(!is_excluded_dir("src", &[]));
        assert!(!is_excluded_dir("lib", &[]));
    }

    #[test]
    fn test_is_excluded_dir_extra() {
        let extra = vec!["custom_build".to_string()];
        assert!(is_excluded_dir("custom_build", &extra));
        assert!(is_excluded_dir("Custom_Build", &extra));
        assert!(!is_excluded_dir("src", &extra));
    }

    #[test]
    fn test_is_source_file_extensions() {
        assert!(is_source_file("main.rs", &[]));
        assert!(is_source_file("app.vue", &[]));
        assert!(is_source_file("index.ts", &[]));
        assert!(is_source_file("style.css", &[]));
        assert!(is_source_file("config.json", &[]));
        assert!(!is_source_file("image.png", &[]));
        assert!(!is_source_file("video.mp4", &[]));
    }

    #[test]
    fn test_is_source_file_special_names() {
        assert!(is_source_file("Dockerfile", &[]));
        assert!(is_source_file("Makefile", &[]));
        assert!(is_source_file("Gemfile", &[]));
    }

    #[test]
    fn test_is_source_file_extra_extensions() {
        let extra = vec!["xyz".to_string()];
        assert!(is_source_file("data.xyz", &extra));
        assert!(!is_source_file("data.xyz", &[]));
    }

    #[test]
    fn test_detect_project_type_rust() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        assert_eq!(detect_project_type(dir.path()), "Rust");
    }

    #[test]
    fn test_detect_project_type_node() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        assert_eq!(detect_project_type(dir.path()), "Node.js");
    }

    #[test]
    fn test_detect_project_type_python() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "flask").unwrap();
        assert_eq!(detect_project_type(dir.path()), "Python");
    }

    #[test]
    fn test_detect_project_type_go() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("go.mod"), "module example.com/test\ngo 1.21").unwrap();
        assert_eq!(detect_project_type(dir.path()), "Go");
    }

    #[test]
    fn test_detect_project_type_flutter() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pubspec.yaml"), "name: test").unwrap();
        assert_eq!(detect_project_type(dir.path()), "Flutter / Dart");
    }

    #[test]
    fn test_detect_project_type_vite() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("vite.config.ts"), "export default {}").unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        assert_eq!(detect_project_type(dir.path()), "Vite");
    }

    #[test]
    fn test_detect_project_type_unknown() {
        let dir = TempDir::new().unwrap();
        assert_eq!(detect_project_type(dir.path()), "通用");
    }

    #[test]
    fn test_build_file_tree_basic() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("image.png"), "binary").unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub fn hello() {}").unwrap();

        let tree = build_file_tree(dir.path(), &[], &[]);
        assert!(tree.is_dir);
        // Should include main.rs and src/lib.rs but not image.png
        let file_count = count_files(&tree);
        assert_eq!(file_count, 2);
    }

    #[test]
    fn test_build_file_tree_excludes_dirs() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "").unwrap();
        fs::create_dir(dir.path().join("node_modules")).unwrap();
        fs::write(dir.path().join("node_modules/pkg.js"), "").unwrap();

        let tree = build_file_tree(dir.path(), &[], &[]);
        assert_eq!(count_files(&tree), 1);
    }

    #[test]
    fn test_count_files_empty() {
        let node = FileNode {
            name: "root".to_string(),
            path: "/root".to_string(),
            is_dir: true,
            children: Vec::new(),
            checked: true,
        };
        assert_eq!(count_files(&node), 0);
    }
}
