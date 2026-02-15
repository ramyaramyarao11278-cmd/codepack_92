use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{AppConfig, ReviewPrompt};

pub fn get_config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("codepack_config.json")
}

pub fn load_app_config() -> AppConfig {
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

pub fn save_app_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn chrono_now() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

// ─── Review Prompts ──────────────────────────────────────────

fn get_review_prompts_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("codepack_review_prompts.json")
}

fn builtin_prompts() -> Vec<ReviewPrompt> {
    vec![
        ReviewPrompt {
            name: "Security Expert".to_string(),
            icon: "\u{1f512}".to_string(), // 🔒
            instruction: "You are a senior security expert. Focus your review on:\n- Authentication and authorization vulnerabilities\n- Injection risks (SQL, XSS, command injection)\n- Hardcoded secrets or credentials\n- Insecure data handling and exposure\n- Input validation and sanitization\n- Dependency vulnerabilities\nProvide specific, actionable recommendations with severity levels.".to_string(),
            builtin: true,
        },
        ReviewPrompt {
            name: "Performance Optimizer".to_string(),
            icon: "\u{26a1}".to_string(), // ⚡
            instruction: "You are a performance optimization specialist. Focus your review on:\n- Algorithm complexity and bottlenecks\n- Memory leaks and excessive allocations\n- N+1 query problems and database optimization\n- Unnecessary re-renders or computations\n- Caching opportunities\n- Async/concurrent processing improvements\nProvide benchmarkable suggestions with expected impact.".to_string(),
            builtin: true,
        },
        ReviewPrompt {
            name: "Clean Code".to_string(),
            icon: "\u{1f9f9}".to_string(), // 🧹
            instruction: "You are a clean code advocate. Focus your review on:\n- SOLID principles violations\n- Code smells and anti-patterns\n- Naming conventions and readability\n- DRY principle (Don't Repeat Yourself)\n- Function/method length and complexity\n- Error handling patterns\n- Test coverage gaps\nSuggest refactoring with concrete before/after examples.".to_string(),
            builtin: true,
        },
    ]
}

pub fn load_review_prompts() -> Vec<ReviewPrompt> {
    let mut prompts = builtin_prompts();
    let path = get_review_prompts_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(custom) = serde_json::from_str::<Vec<ReviewPrompt>>(&data) {
                prompts.extend(custom);
            }
        }
    }
    prompts
}

pub fn save_custom_review_prompt(prompt: &ReviewPrompt) -> Result<(), String> {
    let path = get_review_prompts_path();
    let mut custom: Vec<ReviewPrompt> = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|d| serde_json::from_str(&d).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    // Update existing or add new
    if let Some(existing) = custom.iter_mut().find(|p| p.name == prompt.name) {
        existing.icon = prompt.icon.clone();
        existing.instruction = prompt.instruction.clone();
    } else {
        custom.push(prompt.clone());
    }
    let json = serde_json::to_string_pretty(&custom).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_custom_review_prompt(name: &str) -> Result<(), String> {
    let path = get_review_prompts_path();
    if !path.exists() {
        return Ok(());
    }
    let mut custom: Vec<ReviewPrompt> = fs::read_to_string(&path)
        .ok()
        .and_then(|d| serde_json::from_str(&d).ok())
        .unwrap_or_default();
    custom.retain(|p| p.name != name);
    let json = serde_json::to_string_pretty(&custom).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}
