use std::fs;
use std::path::Path;

use crate::types::ProjectMetadata;

pub fn extract_metadata(root: &Path, project_type: &str) -> ProjectMetadata {
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
        runtime: Vec::new(),
        requirements: Vec::new(),
    };

    match project_type {
        "Node.js" | "Next.js" | "Vite" | "Nuxt.js" => extract_package_json(root, &mut meta),
        "Python" => extract_python_meta(root, &mut meta),
        "Rust" => extract_cargo_toml(root, &mut meta),
        "Go" => extract_go_mod(root, &mut meta),
        "Flutter / Dart" => extract_pubspec_yaml(root, &mut meta),
        "Java / Maven" => extract_pom_xml(root, &mut meta),
        "Android / Gradle" | "Gradle" => extract_gradle_meta(root, &mut meta),
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
                if !desc.is_empty() { meta.description = Some(desc.to_string()); }
            }
            if let Some(main) = pkg.get("main").and_then(|v| v.as_str()) {
                meta.entry_point = Some(main.to_string());
            }
            if let Some(engines) = pkg.get("engines").and_then(|v| v.as_object()) {
                for (key, val) in engines {
                    if let Some(v) = val.as_str() {
                        meta.runtime.push(format!("{} {}", key, v));
                    }
                }
            }
            if let Some(deps) = pkg.get("dependencies").and_then(|v| v.as_object()) {
                meta.dependencies = deps.keys().cloned().collect();
                for (key, val) in deps {
                    if let Some(v) = val.as_str() {
                        meta.requirements.push(format!("{}@{}", key, v));
                    }
                }
            }
            if let Some(deps) = pkg.get("devDependencies").and_then(|v| v.as_object()) {
                meta.dev_dependencies = deps.keys().cloned().collect();
            }
            if meta.runtime.is_empty() {
                for rc in &[".nvmrc", ".node-version"] {
                    if let Ok(ver) = fs::read_to_string(root.join(rc)) {
                        let v = ver.trim().to_string();
                        if !v.is_empty() {
                            meta.runtime.push(format!("node {}", v));
                            break;
                        }
                    }
                }
            }
            if let Ok(ts_content) = fs::read_to_string(root.join("tsconfig.json")) {
                if let Ok(ts) = serde_json::from_str::<serde_json::Value>(&ts_content) {
                    if let Some(target) = ts.get("compilerOptions")
                        .and_then(|c| c.get("target"))
                        .and_then(|v| v.as_str()) {
                        meta.runtime.push(format!("ts target: {}", target));
                    }
                }
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
                    if !desc.is_empty() { meta.description = Some(desc.to_string()); }
                }
                if let Some(edition) = pkg.get("edition").and_then(|v| v.as_str()) {
                    meta.runtime.push(format!("rust edition {}", edition));
                }
                if let Some(msrv) = pkg.get("rust-version").and_then(|v| v.as_str()) {
                    meta.runtime.push(format!("rust >={}", msrv));
                }
            }
            if let Some(deps) = doc.get("dependencies").and_then(|v| v.as_table()) {
                meta.dependencies = deps.keys().cloned().collect();
                for (key, val) in deps {
                    let ver_str = if let Some(v) = val.as_str() {
                        v.to_string()
                    } else if let Some(t) = val.as_table() {
                        t.get("version").and_then(|v| v.as_str()).unwrap_or("*").to_string()
                    } else {
                        "*".to_string()
                    };
                    meta.requirements.push(format!("{}@{}", key, ver_str));
                }
            }
            if let Some(deps) = doc.get("dev-dependencies").and_then(|v| v.as_table()) {
                meta.dev_dependencies = deps.keys().cloned().collect();
            }
        }
    }
}

fn extract_python_meta(root: &Path, meta: &mut ProjectMetadata) {
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
                    if !desc.is_empty() { meta.description = Some(desc.to_string()); }
                }
                if let Some(rp) = project.get("requires-python").and_then(|v| v.as_str()) {
                    meta.runtime.push(format!("python {}", rp));
                }
                if let Some(deps) = project.get("dependencies").and_then(|v| v.as_array()) {
                    for dep in deps.iter().filter_map(|v| v.as_str()) {
                        let name_only = dep.split(&['>', '<', '=', '~', '!', ';', '['][..]).next().unwrap_or(dep).trim().to_string();
                        meta.dependencies.push(name_only);
                        meta.requirements.push(dep.trim().to_string());
                    }
                }
            }
        }
    }
    if meta.dependencies.is_empty() {
        if let Ok(content) = fs::read_to_string(root.join("requirements.txt")) {
            for line in content.lines() {
                let l = line.trim();
                if l.is_empty() || l.starts_with('#') || l.starts_with('-') { continue; }
                let name_only = l.split(&['>', '<', '=', '~', '!', ';', '['][..]).next().unwrap_or(l).trim().to_string();
                meta.dependencies.push(name_only);
                meta.requirements.push(l.to_string());
            }
        }
    }
    if meta.runtime.is_empty() {
        if let Ok(ver) = fs::read_to_string(root.join(".python-version")) {
            let v = ver.trim().to_string();
            if !v.is_empty() { meta.runtime.push(format!("python {}", v)); }
        }
    }
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
                let go_ver = trimmed.strip_prefix("go ").unwrap_or("").trim().to_string();
                meta.version = Some(go_ver.clone());
                meta.runtime.push(format!("go {}", go_ver));
            }
        }
        let mut in_require = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "require (" { in_require = true; continue; }
            if trimmed == ")" { in_require = false; continue; }
            if in_require && !trimmed.is_empty() && !trimmed.starts_with("//") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(dep) = parts.first() { meta.dependencies.push(dep.to_string()); }
                if parts.len() >= 2 { meta.requirements.push(format!("{}@{}", parts[0], parts[1])); }
            }
        }
    }
    if root.join("main.go").exists() { meta.entry_point = Some("main.go".to_string()); }
}

fn extract_pubspec_yaml(root: &Path, meta: &mut ProjectMetadata) {
    if let Ok(content) = fs::read_to_string(root.join("pubspec.yaml")) {
        let mut in_deps = false;
        let mut in_dev_deps = false;
        let mut in_environment = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if !line.starts_with(' ') && !line.starts_with('\t') {
                in_deps = false; in_dev_deps = false; in_environment = false;
                if trimmed.starts_with("name:") {
                    meta.name = trimmed.strip_prefix("name:").unwrap_or("").trim().to_string();
                } else if trimmed.starts_with("version:") {
                    meta.version = Some(trimmed.strip_prefix("version:").unwrap_or("").trim().trim_matches('"').trim_matches('\'').to_string());
                } else if trimmed.starts_with("description:") {
                    let desc = trimmed.strip_prefix("description:").unwrap_or("").trim().trim_matches('"').trim_matches('\'').to_string();
                    if !desc.is_empty() { meta.description = Some(desc); }
                } else if trimmed == "dependencies:" { in_deps = true; }
                else if trimmed == "dev_dependencies:" { in_dev_deps = true; }
                else if trimmed == "environment:" { in_environment = true; }
            } else if in_environment && trimmed.contains(':') {
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let val = parts[1].trim().trim_matches('"').trim_matches('\'');
                    if !val.is_empty() { meta.runtime.push(format!("{} {}", key, val)); }
                }
            } else if (in_deps || in_dev_deps) && trimmed.contains(':') {
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                let dep_name = parts[0].trim().to_string();
                let dep_ver = parts.get(1).map(|v| v.trim().trim_matches('"').trim_matches('\'').to_string()).unwrap_or_default();
                if !dep_name.is_empty() && dep_name != "sdk" {
                    if in_deps {
                        meta.dependencies.push(dep_name.clone());
                        if !dep_ver.is_empty() && dep_ver != "^" {
                            meta.requirements.push(format!("{}@{}", dep_name, dep_ver));
                        }
                    } else { meta.dev_dependencies.push(dep_name); }
                }
            }
        }
    }
    if root.join("lib/main.dart").exists() { meta.entry_point = Some("lib/main.dart".to_string()); }
}

fn extract_pom_xml(root: &Path, meta: &mut ProjectMetadata) {
    if let Ok(content) = fs::read_to_string(root.join("pom.xml")) {
        if let Some(aid) = extract_xml_tag(&content, "artifactId") { meta.name = aid; }
        if let Some(ver) = extract_xml_tag(&content, "version") { meta.version = Some(ver); }
        if let Some(desc) = extract_xml_tag(&content, "description") {
            if !desc.is_empty() { meta.description = Some(desc); }
        }
        if let Some(jv) = extract_xml_tag(&content, "java.version") {
            meta.runtime.push(format!("java {}", jv));
        } else if let Some(jv) = extract_xml_tag(&content, "maven.compiler.source") {
            meta.runtime.push(format!("java {}", jv));
        }
        let mut in_deps = false;
        let mut cur_group = String::new();
        let mut cur_artifact = String::new();
        let mut cur_version = String::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("<dependencies>") { in_deps = true; }
            if trimmed.contains("</dependencies>") { in_deps = false; }
            if in_deps {
                if let Some(v) = extract_xml_tag(trimmed, "groupId") { cur_group = v; }
                if let Some(v) = extract_xml_tag(trimmed, "artifactId") { cur_artifact = v; }
                if let Some(v) = extract_xml_tag(trimmed, "version") { cur_version = v; }
                if trimmed.contains("</dependency>") {
                    if !cur_artifact.is_empty() {
                        meta.dependencies.push(cur_artifact.clone());
                        let req = if !cur_version.is_empty() {
                            format!("{}:{}:{}", cur_group, cur_artifact, cur_version)
                        } else {
                            format!("{}:{}", cur_group, cur_artifact)
                        };
                        meta.requirements.push(req);
                    }
                    cur_group.clear(); cur_artifact.clear(); cur_version.clear();
                }
            }
        }
    }
}

fn extract_gradle_meta(root: &Path, meta: &mut ProjectMetadata) {
    for settings_file in &["settings.gradle.kts", "settings.gradle"] {
        if let Ok(content) = fs::read_to_string(root.join(settings_file)) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("rootProject.name") {
                    let name = trimmed.split('=').nth(1).unwrap_or("").trim().trim_matches('"').trim_matches('\'').to_string();
                    if !name.is_empty() { meta.name = name; }
                }
            }
            break;
        }
    }
}

pub fn extract_xml_tag(text: &str, tag: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_xml_tag() {
        assert_eq!(extract_xml_tag("<name>hello</name>", "name"), Some("hello".to_string()));
        assert_eq!(extract_xml_tag("<version> 1.0 </version>", "version"), Some("1.0".to_string()));
        assert_eq!(extract_xml_tag("<foo>bar</foo>", "baz"), None);
        assert_eq!(extract_xml_tag("no tags here", "x"), None);
    }

    #[test]
    fn test_extract_metadata_rust() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Cargo.toml"), r#"
[package]
name = "myapp"
version = "0.2.0"
description = "A test app"
edition = "2021"

[dependencies]
serde = "1"
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
tempfile = "3"
"#).unwrap();

        let meta = extract_metadata(dir.path(), "Rust");
        assert_eq!(meta.name, "myapp");
        assert_eq!(meta.version, Some("0.2.0".to_string()));
        assert_eq!(meta.description, Some("A test app".to_string()));
        assert!(meta.runtime.iter().any(|r| r.contains("edition 2021")));
        assert!(meta.dependencies.contains(&"serde".to_string()));
        assert!(meta.dependencies.contains(&"tokio".to_string()));
        assert!(meta.dev_dependencies.contains(&"tempfile".to_string()));
        assert!(meta.requirements.iter().any(|r| r.contains("serde@1")));
    }

    #[test]
    fn test_extract_metadata_node() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("package.json"), r#"{
  "name": "my-app",
  "version": "1.0.0",
  "description": "Test node app",
  "main": "index.js",
  "engines": { "node": ">=18" },
  "dependencies": { "express": "^4.18.0", "lodash": "^4.17.21" },
  "devDependencies": { "jest": "^29.0.0" }
}"#).unwrap();

        let meta = extract_metadata(dir.path(), "Node.js");
        assert_eq!(meta.name, "my-app");
        assert_eq!(meta.version, Some("1.0.0".to_string()));
        assert_eq!(meta.entry_point, Some("index.js".to_string()));
        assert!(meta.runtime.iter().any(|r| r.contains("node >=18")));
        assert_eq!(meta.dependencies.len(), 2);
        assert!(meta.requirements.iter().any(|r| r == "express@^4.18.0"));
        assert!(meta.dev_dependencies.contains(&"jest".to_string()));
    }

    #[test]
    fn test_extract_metadata_python_pyproject() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pyproject.toml"), r#"
[project]
name = "mylib"
version = "0.1.0"
description = "A Python library"
requires-python = ">=3.9"
dependencies = ["flask>=2.0", "requests"]
"#).unwrap();

        let meta = extract_metadata(dir.path(), "Python");
        assert_eq!(meta.name, "mylib");
        assert_eq!(meta.version, Some("0.1.0".to_string()));
        assert!(meta.runtime.iter().any(|r| r.contains("python >=3.9")));
        assert!(meta.dependencies.contains(&"flask".to_string()));
        assert!(meta.requirements.iter().any(|r| r == "flask>=2.0"));
    }

    #[test]
    fn test_extract_metadata_python_requirements_txt() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "flask>=2.0\nrequests==2.28.0\n# comment\n").unwrap();
        fs::write(dir.path().join("main.py"), "").unwrap();

        let meta = extract_metadata(dir.path(), "Python");
        assert_eq!(meta.dependencies.len(), 2);
        assert!(meta.requirements.iter().any(|r| r == "flask>=2.0"));
        assert_eq!(meta.entry_point, Some("main.py".to_string()));
    }

    #[test]
    fn test_extract_metadata_go() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("go.mod"), "module github.com/user/app\n\ngo 1.21\n\nrequire (\n\tgithub.com/gin-gonic/gin v1.9.1\n)\n").unwrap();
        fs::write(dir.path().join("main.go"), "package main").unwrap();

        let meta = extract_metadata(dir.path(), "Go");
        assert_eq!(meta.name, "github.com/user/app");
        assert!(meta.runtime.iter().any(|r| r.contains("go 1.21")));
        assert!(meta.dependencies.contains(&"github.com/gin-gonic/gin".to_string()));
        assert_eq!(meta.entry_point, Some("main.go".to_string()));
    }

    #[test]
    fn test_extract_metadata_unknown_type() {
        let dir = TempDir::new().unwrap();
        let meta = extract_metadata(dir.path(), "Unknown");
        assert_eq!(meta.project_type, "Unknown");
        assert!(meta.dependencies.is_empty());
        assert!(meta.runtime.is_empty());
    }
}
