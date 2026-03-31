use eidra_scan::findings::Finding;
use sha2::{Digest, Sha256};

/// Apply masking to the input string based on findings.
/// Replaces matched text with `[REDACTED:{category}:{hash6}]`.
/// Builds a new string by copying segments between findings, avoiding
/// panics from UTF-8 boundary misalignment or overlapping ranges.
pub fn mask_findings(input: &str, findings: &[Finding]) -> String {
    if findings.is_empty() {
        return input.to_string();
    }

    // Sort findings by offset, deduplicate overlapping ranges
    let mut sorted: Vec<&Finding> = findings.iter().collect();
    sorted.sort_by_key(|f| f.offset);

    // Build result by copying segments between findings
    let input_bytes = input.as_bytes();
    let mut result = String::new();
    let mut pos = 0;

    for finding in &sorted {
        let start = finding.offset;
        let end = (finding.offset + finding.length).min(input_bytes.len());

        // Skip if this finding overlaps with a previous one we already processed
        if start < pos {
            continue;
        }

        // Validate byte boundaries are valid UTF-8 boundaries
        if !input.is_char_boundary(start) || !input.is_char_boundary(end) {
            // Skip this finding rather than panic
            tracing::warn!(
                rule = %finding.rule_name,
                offset = start,
                "skipping finding: offset not on UTF-8 boundary"
            );
            continue;
        }

        // Copy text before this finding
        result.push_str(&input[pos..start]);

        // Insert redaction
        let hash = short_hash(&finding.matched_text);
        result.push_str(&format!("[REDACTED:{}:{}]", finding.category, hash));

        pos = end;
    }

    // Copy remaining text after last finding
    if pos < input.len() {
        result.push_str(&input[pos..]);
    }

    result
}

/// Generate a 6-character hash of the matched text for correlation.
fn short_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..3]) // 6 hex chars
}

/// Mask findings in a JSON body, preserving JSON structure.
/// Falls back to plain text masking if the input is not valid JSON.
pub fn mask_findings_json(input: &str, findings: &[Finding]) -> String {
    if findings.is_empty() {
        return input.to_string();
    }

    // Try JSON-aware masking first
    if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(input) {
        mask_json_value(&mut value, findings);
        return serde_json::to_string(&value).unwrap_or_else(|_| mask_findings(input, findings));
    }

    // Fallback to plain text masking
    mask_findings(input, findings)
}

fn mask_json_value(value: &mut serde_json::Value, findings: &[Finding]) {
    match value {
        serde_json::Value::String(s) => {
            // Check if any findings match within this string
            let string_findings: Vec<&Finding> = findings
                .iter()
                .filter(|f| s.contains(&f.matched_text))
                .collect();
            for finding in string_findings {
                let hash = short_hash(&finding.matched_text);
                let replacement = format!("[REDACTED:{}:{}]", finding.category, hash);
                // Use replacen(1) to only mask the first occurrence per finding,
                // avoiding over-masking when the same text appears multiple times
                *s = s.replacen(&finding.matched_text, &replacement, 1);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                mask_json_value(item, findings);
            }
        }
        serde_json::Value::Object(map) => {
            for (_key, val) in map.iter_mut() {
                mask_json_value(val, findings);
            }
        }
        _ => {}
    }
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidra_scan::findings::{Category, Finding, Severity};

    #[test]
    fn test_mask_single_finding() {
        let input = "my key is AKIAIOSFODNN7EXAMPLE here";
        let finding = Finding::new(
            Category::ApiKey,
            Severity::Critical,
            "aws_access_key",
            "AWS Access Key",
            "AKIAIOSFODNN7EXAMPLE",
            10,
            20,
        );
        let result = mask_findings(input, &[finding]);
        assert!(result.contains("[REDACTED:api_key:"));
        assert!(!result.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_mask_multiple_findings() {
        let input = "key AKIAIOSFODNN7EXAMPLE email test@example.com";
        let findings = vec![
            Finding::new(
                Category::ApiKey,
                Severity::Critical,
                "aws",
                "k",
                "AKIAIOSFODNN7EXAMPLE",
                4,
                20,
            ),
            Finding::new(
                Category::Pii,
                Severity::Medium,
                "email",
                "e",
                "test@example.com",
                31,
                16,
            ),
        ];
        let result = mask_findings(input, &findings);
        assert!(!result.contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(!result.contains("test@example.com"));
        assert!(result.contains("[REDACTED:api_key:"));
        assert!(result.contains("[REDACTED:pii:"));
    }

    #[test]
    fn test_mask_json_preserves_structure() {
        let input =
            r#"{"messages":[{"role":"user","content":"key is AKIAIOSFODNN7EXAMPLE here"}]}"#;
        let finding = Finding::new(
            Category::ApiKey,
            Severity::Critical,
            "aws",
            "k",
            "AKIAIOSFODNN7EXAMPLE",
            42,
            20,
        );
        let result = mask_findings_json(input, &[finding]);
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let content = parsed["messages"][0]["content"].as_str().unwrap();
        assert!(content.contains("[REDACTED:api_key:"));
        assert!(!content.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_mask_json_fallback_plaintext() {
        let input = "not json: AKIAIOSFODNN7EXAMPLE";
        let finding = Finding::new(
            Category::ApiKey,
            Severity::Critical,
            "aws",
            "k",
            "AKIAIOSFODNN7EXAMPLE",
            10,
            20,
        );
        let result = mask_findings_json(input, &[finding]);
        assert!(!result.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_mask_preserves_surrounding_text() {
        let input = "before SECRET after";
        let finding = Finding::new(
            Category::SecretKey,
            Severity::High,
            "secret",
            "s",
            "SECRET",
            7,
            6,
        );
        let result = mask_findings(input, &[finding]);
        assert!(result.starts_with("before "));
        assert!(result.ends_with(" after"));
    }
}
