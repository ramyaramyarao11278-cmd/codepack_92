use regex::Regex;
use std::sync::OnceLock;

use crate::types::{SecretMatch, SecretType};

// ─── Precompiled Rules ─────────────────────────────────────────

struct Rule {
    regex: Regex,
    secret_type: SecretType,
    description: &'static str,
}

fn rules() -> &'static Vec<Rule> {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| {
        vec![
            // 1. AWS Access Key ID (AKIA...)
            Rule {
                regex: Regex::new(
                    r"(A3T[A-Z0-9]|AKIA|AGPA|AIDA|AROA|AIPA|ANPA|ANVA|ASIA)[A-Z0-9]{16}",
                )
                .unwrap(),
                secret_type: SecretType::ApiKey,
                description: "AWS Access Key ID",
            },
            // 2. SSH/RSA Private Key Header
            Rule {
                regex: Regex::new(r"-----BEGIN [A-Z]+ PRIVATE KEY-----").unwrap(),
                secret_type: SecretType::PrivateKey,
                description: "Private Key Header",
            },
            // 3. OpenAI API Key (sk-...)
            Rule {
                regex: Regex::new(r"sk-[a-zA-Z0-9]{32,}").unwrap(),
                secret_type: SecretType::ApiKey,
                description: "OpenAI API Key",
            },
            // 4. GitHub Personal Access Token (ghp_...)
            Rule {
                regex: Regex::new(r"ghp_[a-zA-Z0-9]{36}").unwrap(),
                secret_type: SecretType::ApiKey,
                description: "GitHub PAT",
            },
            // 5. Google API Key (AIza...)
            Rule {
                regex: Regex::new(r"AIza[0-9A-Za-z\-_]{35}").unwrap(),
                secret_type: SecretType::ApiKey,
                description: "Google API Key",
            },
            // 6. Hardcoded password/secret pattern
            Rule {
                regex: Regex::new(
                    r#"(?i)(password|passwd|pwd|secret|api_key|apikey|access_token)\s*[:=]\s*["'](?P<secret>[^"']{6,})["']"#,
                )
                .unwrap(),
                secret_type: SecretType::Password,
                description: "Potential Hardcoded Secret",
            },
        ]
    })
}

// ─── Scan ──────────────────────────────────────────────────────

pub fn scan_content(content: &str) -> Vec<SecretMatch> {
    let mut matches = Vec::new();
    let rules = rules();

    for (line_idx, line) in content.lines().enumerate() {
        // Skip very long lines (e.g. minified JS) to prevent regex backtracking
        if line.len() > 1000 {
            continue;
        }

        for rule in rules {
            if let Some(mat) = rule.regex.find(line) {
                matches.push(SecretMatch {
                    line_number: line_idx + 1,
                    match_content: mat.as_str().to_string(),
                    secret_type: rule.secret_type.clone(),
                    description: rule.description.to_string(),
                    start_index: mat.start(),
                    end_index: mat.end(),
                });
            }
        }
    }
    matches
}

// ─── Mask ──────────────────────────────────────────────────────

pub fn mask_secrets(content: &str, matches: &[SecretMatch]) -> String {
    let mut result = content.to_string();
    // Replace each unique match string (longer matches first to avoid partial replacements)
    let mut unique: Vec<&str> = matches.iter().map(|m| m.match_content.as_str()).collect();
    unique.sort_by_key(|b| std::cmp::Reverse(b.len()));
    unique.dedup();

    for secret in unique {
        let prefix: String = secret.chars().take(3).collect();
        let mask = format!("{}******", prefix);
        result = result.replace(secret, &mask);
    }
    result
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_aws_key() {
        let content = "aws_key = AKIAIOSFODNN7EXAMPLE";
        let matches = scan_content(content);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].description, "AWS Access Key ID");
        assert_eq!(matches[0].line_number, 1);
        assert!(matches[0].match_content.starts_with("AKIA"));
    }

    #[test]
    fn test_detect_private_key() {
        let content = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA0Z3VS5JJcds3xfn/ygWyF\n-----END RSA PRIVATE KEY-----";
        let matches = scan_content(content);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].description, "Private Key Header");
        assert_eq!(matches[0].line_number, 1);
    }

    #[test]
    fn test_detect_openai_key() {
        let content = r#"OPENAI_KEY = "sk-abcdefghijklmnopqrstuvwxyz123456789012""#;
        let matches = scan_content(content);
        assert!(matches.iter().any(|m| m.description == "OpenAI API Key"));
    }

    #[test]
    fn test_detect_hardcoded_password() {
        let content = r#"db_password = "SuperSecret123!""#;
        let matches = scan_content(content);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].description, "Potential Hardcoded Secret");
    }

    #[test]
    fn test_detect_github_pat() {
        let content = "token = ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij";
        let matches = scan_content(content);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].description, "GitHub PAT");
    }

    #[test]
    fn test_no_false_positive_on_normal_code() {
        let content = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let matches = scan_content(content);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_skip_long_lines() {
        // Line > 1000 chars should be skipped
        let long_line = format!("password = \"{}\"", "a".repeat(1100));
        let matches = scan_content(&long_line);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_mask_secrets() {
        let content = "key = AKIAIOSFODNN7EXAMPLE and password = \"secret123\"";
        let matches = scan_content(content);
        let masked = mask_secrets(content, &matches);
        assert!(!masked.contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(masked.contains("AKI******"));
    }
}
