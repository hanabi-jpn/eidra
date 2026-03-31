use crate::classifier::Classifier;
use crate::findings::{Category, Finding, Severity};
use regex::Regex;

struct Rule {
    name: &'static str,
    pattern: Regex,
    category: Category,
    severity: Severity,
    description: &'static str,
}

pub struct TextClassifier {
    rules: Vec<Rule>,
}

impl TextClassifier {
    pub fn new() -> Self {
        let rules = vec![
            // 1. AWS Access Key
            Rule {
                name: "aws_access_key",
                pattern: Regex::new(r"AKIA[0-9A-Z]{16}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Critical,
                description: "AWS Access Key ID",
            },
            // 2. AWS Secret Key
            Rule {
                name: "aws_secret_key",
                pattern: Regex::new(r"(?i)(?:aws_secret_access_key|aws_secret)\s*[:=]\s*[A-Za-z0-9/+=]{40}").expect("valid regex"),
                category: Category::SecretKey,
                severity: Severity::Critical,
                description: "AWS Secret Access Key",
            },
            // 3. GitHub Token
            Rule {
                name: "github_token",
                pattern: Regex::new(r"gh[posru]_[A-Za-z0-9_]{36,}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "GitHub Personal Access Token",
            },
            // 4. GitLab Token
            Rule {
                name: "gitlab_token",
                pattern: Regex::new(r"glpat-[A-Za-z0-9\-_]{20,}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "GitLab Personal Access Token",
            },
            // 5. Slack Token
            Rule {
                name: "slack_token",
                pattern: Regex::new(r"xox[baprs]-[A-Za-z0-9\-]+").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Slack API Token",
            },
            // 6. Stripe Key
            Rule {
                name: "stripe_key",
                pattern: Regex::new(r"[sr]k_live_[A-Za-z0-9]{20,}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Critical,
                description: "Stripe Live API Key",
            },
            // 7. Google API Key
            Rule {
                name: "google_api_key",
                pattern: Regex::new(r"AIza[A-Za-z0-9\-_]{35}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Google API Key",
            },
            // 8. JWT
            Rule {
                name: "jwt",
                pattern: Regex::new(r"eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_.+/=]*").expect("valid regex"),
                category: Category::Token,
                severity: Severity::Medium,
                description: "JSON Web Token",
            },
            // 9. Private Key Block
            Rule {
                name: "private_key",
                pattern: Regex::new(r"-----BEGIN[\s\w]*PRIVATE KEY-----").expect("valid regex"),
                category: Category::PrivateKey,
                severity: Severity::Critical,
                description: "Private Key Block",
            },
            // 10. Email Address
            Rule {
                name: "email_address",
                pattern: Regex::new(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Medium,
                description: "Email Address",
            },
            // 11. Phone (International)
            Rule {
                name: "phone_international",
                pattern: Regex::new(r"\+[1-9]\d{6,14}").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Medium,
                description: "International Phone Number",
            },
            // 12. Credit Card (Visa/MC/Amex)
            Rule {
                name: "credit_card",
                pattern: Regex::new(r"\b(?:4\d{15}|5[1-5]\d{14}|3[47]\d{13})\b").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Critical,
                description: "Credit Card Number",
            },
            // 13. US SSN
            Rule {
                name: "us_ssn",
                pattern: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Critical,
                description: "US Social Security Number",
            },
            // 14. IPv4 Address (excluding common non-sensitive IPs)
            Rule {
                name: "ipv4_address",
                pattern: Regex::new(r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b").expect("valid regex"),
                category: Category::InternalInfra,
                severity: Severity::Low,
                description: "IPv4 Address",
            },
            // 15. DB Connection String
            Rule {
                name: "db_connection_string",
                pattern: Regex::new(r#"(?i)(?:postgres|mysql|mongodb|redis)://[^\s'""]+"#).expect("valid regex"),
                category: Category::Credential,
                severity: Severity::High,
                description: "Database Connection String",
            },
            // 16. Env Variable Assignment (secrets)
            Rule {
                name: "env_secret_assignment",
                pattern: Regex::new(r#"(?i)(?:api[_\-]?key|secret|password|token|auth[_\-]?token)\s*=\s*['"]?[A-Za-z0-9/+=]{8,}['"]?"#).expect("valid regex"),
                category: Category::Credential,
                severity: Severity::High,
                description: "Environment Variable Secret Assignment",
            },
            // 17. Internal Hostname
            Rule {
                name: "internal_hostname",
                pattern: Regex::new(r"\b\w+\.(?:internal|local|corp|private)\.\w+\b").expect("valid regex"),
                category: Category::InternalInfra,
                severity: Severity::Low,
                description: "Internal Hostname",
            },
            // 18. Password Assignment
            Rule {
                name: "password_assignment",
                pattern: Regex::new(r#"(?i)password\s*[:=]\s*['"][^'"]{4,}['"]"#).expect("valid regex"),
                category: Category::Credential,
                severity: Severity::High,
                description: "Password Assignment",
            },
            // 19. High Entropy Base64 (40+ chars)
            Rule {
                name: "high_entropy_base64",
                pattern: Regex::new(r"[A-Za-z0-9+/=]{40,}").expect("valid regex"),
                category: Category::HighEntropy,
                severity: Severity::Medium,
                description: "High Entropy Base64 String",
            },
            // 20. Sensitive File Path
            Rule {
                name: "sensitive_file_path",
                pattern: Regex::new(r"(?:/\.ssh/|/\.aws/|/\.env\b|/\.gnupg/)").expect("valid regex"),
                category: Category::SensitivePath,
                severity: Severity::Medium,
                description: "Sensitive File Path",
            },
            // 21. Azure Storage Key
            Rule {
                name: "azure_storage_key",
                pattern: Regex::new(r"AccountKey=[A-Za-z0-9+/=]{88}").expect("valid regex"),
                category: Category::SecretKey,
                severity: Severity::Critical,
                description: "Azure Storage Account Key",
            },
            // 22. Heroku API Key
            Rule {
                name: "heroku_api_key",
                pattern: Regex::new(r"[hH][eE][rR][oO][kK][uU].*[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Heroku API Key",
            },
            // 23. Twilio Account SID
            Rule {
                name: "twilio_account_sid",
                pattern: Regex::new(r"AC[a-z0-9]{32}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Twilio Account SID",
            },
            // 24. Twilio Auth Token
            Rule {
                name: "twilio_auth_token",
                pattern: Regex::new(r"(?i)twilio.*[0-9a-fA-F]{32}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Twilio Auth Token",
            },
            // 25. Mailgun API Key
            Rule {
                name: "mailgun_api_key",
                pattern: Regex::new(r"key-[0-9a-zA-Z]{32}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Mailgun API Key",
            },
            // 26. SendGrid API Key
            Rule {
                name: "sendgrid_api_key",
                pattern: Regex::new(r"SG\.[A-Za-z0-9\-_]{22,}\.[A-Za-z0-9\-_]{43,}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Critical,
                description: "SendGrid API Key",
            },
            // 27. Telegram Bot Token
            Rule {
                name: "telegram_bot_token",
                pattern: Regex::new(r"[0-9]{8,10}:[A-Za-z0-9_-]{35}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Telegram Bot Token",
            },
            // 28. Discord Webhook
            Rule {
                name: "discord_webhook",
                pattern: Regex::new(r"https://discord(?:app)?\.com/api/webhooks/[0-9]+/[A-Za-z0-9_-]+").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Discord Webhook URL",
            },
            // 29. Discord Bot Token
            Rule {
                name: "discord_bot_token",
                pattern: Regex::new(r"[MN][A-Za-z0-9]{23,}\.[A-Za-z0-9_-]{6}\.[A-Za-z0-9_-]{27,}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Discord Bot Token",
            },
            // 30. npm Token
            Rule {
                name: "npm_token",
                pattern: Regex::new(r"npm_[A-Za-z0-9]{36}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "npm Access Token",
            },
            // 31. PyPI Token
            Rule {
                name: "pypi_token",
                pattern: Regex::new(r"pypi-[A-Za-z0-9]{150,}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "PyPI API Token",
            },
            // 32. Docker Registry Password
            Rule {
                name: "docker_registry_password",
                pattern: Regex::new(r#"(?i)docker.*password\s*[:=]\s*['"]?[^\s'"]+""#).expect("valid regex"),
                category: Category::Credential,
                severity: Severity::High,
                description: "Docker Registry Password",
            },
            // 33. Generic API Key Assignment
            Rule {
                name: "generic_api_key",
                pattern: Regex::new(r#"(?i)(?:api_key|apikey|api-key)\s*[:=]\s*['"]?[A-Za-z0-9_\-]{20,}['"]?"#).expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Medium,
                description: "Generic API Key Assignment",
            },
            // 34. My Number (マイナンバー 12桁)
            Rule {
                name: "my_number",
                pattern: Regex::new(r"(?i)(?:マイナンバー|my[\s_-]*number|個人番号)[:\s]*\d{4}\s?\d{4}\s?\d{4}\b").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Critical,
                description: "My Number (マイナンバー 12-digit)",
            },
            // 35. Japanese Phone Number
            Rule {
                name: "japanese_phone",
                pattern: Regex::new(r"0[789]0-?\d{4}-?\d{4}").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Medium,
                description: "Japanese Mobile Phone Number",
            },
            // 36. Physical Address Heuristic
            Rule {
                name: "physical_address",
                pattern: Regex::new(r"\d{1,5}\s\w+\s(?:Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Drive|Dr|Court|Ct|Lane|Ln)").expect("valid regex"),
                category: Category::Pii,
                severity: Severity::Medium,
                description: "Physical Address (US format heuristic)",
            },
            // 37. Kubernetes Secret
            Rule {
                name: "kubernetes_secret",
                pattern: Regex::new(r"(?i)kind:\s*Secret").expect("valid regex"),
                category: Category::InternalInfra,
                severity: Severity::High,
                description: "Kubernetes Secret Manifest",
            },
            // 38. Terraform State Secret
            Rule {
                name: "terraform_state_secret",
                pattern: Regex::new(r#"(?i)"type":\s*"aws_"#).expect("valid regex"),
                category: Category::InternalInfra,
                severity: Severity::Medium,
                description: "Terraform State File (AWS resource)",
            },
            // 39. Authorization Bearer Header
            Rule {
                name: "authorization_bearer",
                pattern: Regex::new(r"(?i)authorization:\s*bearer\s+[A-Za-z0-9\-._~+/]+=*").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Authorization Bearer Header",
            },
            // 40. Redis URL
            Rule {
                name: "redis_url",
                pattern: Regex::new(r#"redis://[^\s'"]+"#).expect("valid regex"),
                category: Category::Credential,
                severity: Severity::High,
                description: "Redis Connection URL",
            },
            // 41. Postmark Server Token
            Rule {
                name: "postmark_server_token",
                pattern: Regex::new(r"(?i)postmark.*[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Postmark Server Token",
            },
            // 42. Databricks Token
            Rule {
                name: "databricks_token",
                pattern: Regex::new(r"dapi[a-z0-9]{32}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::High,
                description: "Databricks Access Token",
            },
            // 43. OpenAI API Key
            Rule {
                name: "openai_api_key",
                pattern: Regex::new(r"sk-[A-Za-z0-9]{20,}T3BlbkFJ[A-Za-z0-9]{20,}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Critical,
                description: "OpenAI API Key",
            },
            // 44. Anthropic API Key
            Rule {
                name: "anthropic_api_key",
                pattern: Regex::new(r"sk-ant-[A-Za-z0-9\-_]{80,}").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::Critical,
                description: "Anthropic API Key",
            },
            // 45. Supabase Key
            Rule {
                name: "supabase_key",
                pattern: Regex::new(r"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Supabase API Key (JWT)",
            },
            // 46. Firebase Config
            Rule {
                name: "firebase_config",
                pattern: Regex::new(r"(?i)firebase.*apiKey.*AIza").expect("valid regex"),
                category: Category::ApiKey,
                severity: Severity::High,
                description: "Firebase Configuration with API Key",
            },
            // 47. HashiCorp Vault Token
            Rule {
                name: "hashicorp_vault_token",
                pattern: Regex::new(r"hvs\.[A-Za-z0-9]{24,}").expect("valid regex"),
                category: Category::Token,
                severity: Severity::Critical,
                description: "HashiCorp Vault Token",
            },
        ];

        Self { rules }
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for TextClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Classifier for TextClassifier {
    fn classify(&self, input: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        for rule in &self.rules {
            for mat in rule.pattern.find_iter(input) {
                // Rule 14: skip common non-sensitive IPs
                if rule.name == "ipv4_address" {
                    let ip = mat.as_str();
                    if ip == "127.0.0.1" || ip == "0.0.0.0" || ip.starts_with("255.") {
                        continue;
                    }
                }
                // Rule 19: check Shannon entropy for high-entropy strings
                if rule.name == "high_entropy_base64" {
                    let entropy = shannon_entropy(mat.as_str());
                    if entropy < 4.5 {
                        continue;
                    }
                }
                findings.push(Finding::new(
                    rule.category.clone(),
                    rule.severity.clone(),
                    rule.name,
                    rule.description,
                    mat.as_str(),
                    mat.start(),
                    mat.len(),
                ));
            }
        }
        findings
    }

    fn name(&self) -> &str {
        "text_classifier"
    }
}

fn shannon_entropy(s: &str) -> f64 {
    let mut freq = [0u32; 256];
    let len = s.len() as f64;
    for &b in s.as_bytes() {
        freq[b as usize] += 1;
    }
    freq.iter()
        .filter(|&&count| count > 0)
        .map(|&count| {
            let p = count as f64 / len;
            -p * p.log2()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(input: &str) -> Vec<Finding> {
        let classifier = TextClassifier::new();
        classifier.classify(input)
    }

    fn has_rule(findings: &[Finding], rule_name: &str) -> bool {
        findings.iter().any(|f| f.rule_name == rule_name)
    }

    // 1. AWS Access Key
    #[test]
    fn test_aws_access_key_match() {
        let findings = classify("key is AKIAIOSFODNN7EXAMPLE");
        assert!(has_rule(&findings, "aws_access_key"));
    }
    #[test]
    fn test_aws_access_key_no_match() {
        let findings = classify("key is NOTAKEY1234567890AB");
        assert!(!has_rule(&findings, "aws_access_key"));
    }

    // 2. AWS Secret Key
    #[test]
    fn test_aws_secret_key_match() {
        let findings = classify("aws_secret_access_key=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY");
        assert!(has_rule(&findings, "aws_secret_key"));
    }
    #[test]
    fn test_aws_secret_key_no_match() {
        let findings = classify("aws_region=us-east-1");
        assert!(!has_rule(&findings, "aws_secret_key"));
    }

    // 3. GitHub Token
    #[test]
    fn test_github_token_match() {
        let findings = classify("token: ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn");
        assert!(has_rule(&findings, "github_token"));
    }
    #[test]
    fn test_github_token_no_match() {
        let findings = classify("token: ghx_short");
        assert!(!has_rule(&findings, "github_token"));
    }

    // 4. GitLab Token
    #[test]
    fn test_gitlab_token_match() {
        let findings = classify("token: glpat-ABCDEFGHIJKLMNOPQRST");
        assert!(has_rule(&findings, "gitlab_token"));
    }
    #[test]
    fn test_gitlab_token_no_match() {
        let findings = classify("token: glpat-short");
        assert!(!has_rule(&findings, "gitlab_token"));
    }

    // 5. Slack Token
    #[test]
    fn test_slack_token_match() {
        let findings = classify("slack: xoxb-123456789-abcdef");
        assert!(has_rule(&findings, "slack_token"));
    }
    #[test]
    fn test_slack_token_no_match() {
        let findings = classify("slack: xoxz-nothing");
        assert!(!has_rule(&findings, "slack_token"));
    }

    // 6. Stripe Key
    #[test]
    fn test_stripe_key_match() {
        let findings = classify("key: sk_live_ABCDEFghijklmnopqrst");
        assert!(has_rule(&findings, "stripe_key"));
    }
    #[test]
    fn test_stripe_key_no_match() {
        let findings = classify("key: sk_test_ABCDEFghijklmnopqrst");
        assert!(!has_rule(&findings, "stripe_key"));
    }

    // 7. Google API Key
    #[test]
    fn test_google_api_key_match() {
        let findings = classify(concat!("key: AIza", "SyA1234567890abcdefghijklmnopqrstuv"));
        assert!(has_rule(&findings, "google_api_key"));
    }
    #[test]
    fn test_google_api_key_no_match() {
        let findings = classify("key: notakey");
        assert!(!has_rule(&findings, "google_api_key"));
    }

    // 8. JWT
    #[test]
    fn test_jwt_match() {
        let findings =
            classify("token: eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signature");
        assert!(has_rule(&findings, "jwt"));
    }
    #[test]
    fn test_jwt_no_match() {
        let findings = classify("token: notajwt.notajwt.sig");
        assert!(!has_rule(&findings, "jwt"));
    }

    // 9. Private Key
    #[test]
    fn test_private_key_match() {
        let findings = classify("-----BEGIN RSA PRIVATE KEY-----");
        assert!(has_rule(&findings, "private_key"));
    }
    #[test]
    fn test_private_key_no_match() {
        let findings = classify("-----BEGIN PUBLIC KEY-----");
        assert!(!has_rule(&findings, "private_key"));
    }

    // 10. Email
    #[test]
    fn test_email_match() {
        let findings = classify("contact: user@example.com");
        assert!(has_rule(&findings, "email_address"));
    }
    #[test]
    fn test_email_no_match() {
        let findings = classify("contact: not-an-email");
        assert!(!has_rule(&findings, "email_address"));
    }

    // 11. Phone
    #[test]
    fn test_phone_match() {
        let findings = classify("call: +15551234567");
        assert!(has_rule(&findings, "phone_international"));
    }
    #[test]
    fn test_phone_no_match() {
        let findings = classify("call: 555-1234");
        assert!(!has_rule(&findings, "phone_international"));
    }

    // 12. Credit Card
    #[test]
    fn test_credit_card_match() {
        let findings = classify("card: 4111111111111111");
        assert!(has_rule(&findings, "credit_card"));
    }
    #[test]
    fn test_credit_card_no_match() {
        let findings = classify("card: 1234567890123456");
        assert!(!has_rule(&findings, "credit_card"));
    }

    // 13. US SSN
    #[test]
    fn test_ssn_match() {
        let findings = classify("ssn: 123-45-6789");
        assert!(has_rule(&findings, "us_ssn"));
    }
    #[test]
    fn test_ssn_no_match() {
        let findings = classify("ssn: 123456789");
        assert!(!has_rule(&findings, "us_ssn"));
    }

    // 14. IPv4
    #[test]
    fn test_ipv4_match() {
        let findings = classify("host: 192.168.1.1");
        assert!(has_rule(&findings, "ipv4_address"));
    }
    #[test]
    fn test_ipv4_skip_localhost() {
        let findings = classify("host: 127.0.0.1");
        assert!(!has_rule(&findings, "ipv4_address"));
    }

    // 15. DB Connection String
    #[test]
    fn test_db_conn_match() {
        let findings = classify("url: postgres://user:pass@host/db");
        assert!(has_rule(&findings, "db_connection_string"));
    }
    #[test]
    fn test_db_conn_no_match() {
        let findings = classify("url: https://example.com");
        assert!(!has_rule(&findings, "db_connection_string"));
    }

    // 16. Env Secret Assignment
    #[test]
    fn test_env_secret_match() {
        let findings = classify("API_KEY=sk1234567890abcdef");
        assert!(has_rule(&findings, "env_secret_assignment"));
    }
    #[test]
    fn test_env_secret_no_match() {
        let findings = classify("NAME=John");
        assert!(!has_rule(&findings, "env_secret_assignment"));
    }

    // 17. Internal Hostname
    #[test]
    fn test_internal_hostname_match() {
        let findings = classify("host: db.internal.acme");
        assert!(has_rule(&findings, "internal_hostname"));
    }
    #[test]
    fn test_internal_hostname_no_match() {
        let findings = classify("host: example.com");
        assert!(!has_rule(&findings, "internal_hostname"));
    }

    // 18. Password Assignment
    #[test]
    fn test_password_match() {
        let findings = classify(r#"password="SuperSecret123""#);
        assert!(has_rule(&findings, "password_assignment"));
    }
    #[test]
    fn test_password_no_match() {
        let findings = classify("password policy enforced");
        assert!(!has_rule(&findings, "password_assignment"));
    }

    // 19. High Entropy Base64
    #[test]
    fn test_high_entropy_match() {
        // Random-looking base64 string
        let findings = classify("secret: K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols=AAAA");
        assert!(has_rule(&findings, "high_entropy_base64"));
    }
    #[test]
    fn test_high_entropy_no_match() {
        // Repetitive — low entropy
        let findings = classify("data: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        assert!(!has_rule(&findings, "high_entropy_base64"));
    }

    // 20. Sensitive File Path
    #[test]
    fn test_sensitive_path_match() {
        let findings = classify("file: /home/user/.ssh/id_rsa");
        assert!(has_rule(&findings, "sensitive_file_path"));
    }
    #[test]
    fn test_sensitive_path_no_match() {
        let findings = classify("file: /home/user/documents/report.pdf");
        assert!(!has_rule(&findings, "sensitive_file_path"));
    }

    // 21. Azure Storage Key
    #[test]
    fn test_azure_storage_key_match() {
        let key = format!("AccountKey={}", "A".repeat(86) + "==");
        let findings = classify(&key);
        assert!(has_rule(&findings, "azure_storage_key"));
    }
    #[test]
    fn test_azure_storage_key_no_match() {
        let findings = classify("AccountKey=shortkey");
        assert!(!has_rule(&findings, "azure_storage_key"));
    }

    // 22. Heroku API Key
    #[test]
    fn test_heroku_api_key_match() {
        let findings = classify("HEROKU_API_KEY=12345678-1234-1234-1234-123456789abc");
        assert!(has_rule(&findings, "heroku_api_key"));
    }
    #[test]
    fn test_heroku_api_key_no_match() {
        let findings = classify("HEROKU_REGION=us");
        assert!(!has_rule(&findings, "heroku_api_key"));
    }

    // 23. Twilio Account SID
    #[test]
    fn test_twilio_sid_match() {
        let sid = format!("AC{}", "a".repeat(32));
        let findings = classify(&sid);
        assert!(has_rule(&findings, "twilio_account_sid"));
    }
    #[test]
    fn test_twilio_sid_no_match() {
        let findings = classify("ACshort");
        assert!(!has_rule(&findings, "twilio_account_sid"));
    }

    // 24. Twilio Auth Token
    #[test]
    fn test_twilio_auth_token_match() {
        let token = format!("twilio_auth_token={}", "a1b2c3d4".repeat(4));
        let findings = classify(&token);
        assert!(has_rule(&findings, "twilio_auth_token"));
    }
    #[test]
    fn test_twilio_auth_token_no_match() {
        let findings = classify("twilio_region=us1");
        assert!(!has_rule(&findings, "twilio_auth_token"));
    }

    // 25. Mailgun API Key
    #[test]
    fn test_mailgun_api_key_match() {
        let key = format!("key-{}", "a1b2c3d4e5f6g7h8".repeat(2));
        let findings = classify(&key);
        assert!(has_rule(&findings, "mailgun_api_key"));
    }
    #[test]
    fn test_mailgun_api_key_no_match() {
        let findings = classify("key-short");
        assert!(!has_rule(&findings, "mailgun_api_key"));
    }

    // 26. SendGrid API Key
    #[test]
    fn test_sendgrid_api_key_match() {
        let key = format!("SG.{}.{}", "A".repeat(22), "B".repeat(43));
        let findings = classify(&key);
        assert!(has_rule(&findings, "sendgrid_api_key"));
    }
    #[test]
    fn test_sendgrid_api_key_no_match() {
        let findings = classify("SG.short.short");
        assert!(!has_rule(&findings, "sendgrid_api_key"));
    }

    // 27. Telegram Bot Token
    #[test]
    fn test_telegram_bot_token_match() {
        let token = format!("123456789:{}", "A".repeat(35));
        let findings = classify(&token);
        assert!(has_rule(&findings, "telegram_bot_token"));
    }
    #[test]
    fn test_telegram_bot_token_no_match() {
        let findings = classify("123:short");
        assert!(!has_rule(&findings, "telegram_bot_token"));
    }

    // 28. Discord Webhook
    #[test]
    fn test_discord_webhook_match() {
        let findings = classify("https://discord.com/api/webhooks/123456789/ABCdef_token-here");
        assert!(has_rule(&findings, "discord_webhook"));
    }
    #[test]
    fn test_discord_webhook_no_match() {
        let findings = classify("https://discord.com/channels/123");
        assert!(!has_rule(&findings, "discord_webhook"));
    }

    // 29. Discord Bot Token
    #[test]
    fn test_discord_bot_token_match() {
        let token = format!("M{}.abcdef.{}", "A".repeat(23), "B".repeat(27));
        let findings = classify(&token);
        assert!(has_rule(&findings, "discord_bot_token"));
    }
    #[test]
    fn test_discord_bot_token_no_match() {
        let findings = classify("Xshort.ab.cd");
        assert!(!has_rule(&findings, "discord_bot_token"));
    }

    // 30. npm Token
    #[test]
    fn test_npm_token_match() {
        let token = format!("npm_{}", "A".repeat(36));
        let findings = classify(&token);
        assert!(has_rule(&findings, "npm_token"));
    }
    #[test]
    fn test_npm_token_no_match() {
        let findings = classify("npm_short");
        assert!(!has_rule(&findings, "npm_token"));
    }

    // 31. PyPI Token
    #[test]
    fn test_pypi_token_match() {
        let token = format!("pypi-{}", "A".repeat(150));
        let findings = classify(&token);
        assert!(has_rule(&findings, "pypi_token"));
    }
    #[test]
    fn test_pypi_token_no_match() {
        let findings = classify("pypi-short");
        assert!(!has_rule(&findings, "pypi_token"));
    }

    // 32. Docker Registry Password
    #[test]
    fn test_docker_password_match() {
        let findings = classify(r#"docker_password="mysecretpass""#);
        assert!(has_rule(&findings, "docker_registry_password"));
    }
    #[test]
    fn test_docker_password_no_match() {
        let findings = classify("docker pull nginx");
        assert!(!has_rule(&findings, "docker_registry_password"));
    }

    // 33. Generic API Key Assignment
    #[test]
    fn test_generic_api_key_match() {
        let findings = classify("api_key=ABCDEFGHIJKLMNOPQRSTUVWX");
        assert!(has_rule(&findings, "generic_api_key"));
    }
    #[test]
    fn test_generic_api_key_no_match() {
        let findings = classify("api_key=short");
        assert!(!has_rule(&findings, "generic_api_key"));
    }

    // 34. My Number (マイナンバー)
    #[test]
    fn test_my_number_match() {
        let findings = classify("マイナンバー: 1234 5678 9012");
        assert!(has_rule(&findings, "my_number"));
    }
    #[test]
    fn test_my_number_no_match() {
        // Plain 12-digit numbers without context should NOT match (false positive prevention)
        let findings = classify("order 1234 5678 9012");
        assert!(!has_rule(&findings, "my_number"));
    }

    // 35. Japanese Phone Number
    #[test]
    fn test_japanese_phone_match() {
        let findings = classify("tel: 090-1234-5678");
        assert!(has_rule(&findings, "japanese_phone"));
    }
    #[test]
    fn test_japanese_phone_no_match() {
        let findings = classify("tel: 03-1234-5678");
        assert!(!has_rule(&findings, "japanese_phone"));
    }

    // 36. Physical Address
    #[test]
    fn test_physical_address_match() {
        let findings = classify("address: 123 Main Street");
        assert!(has_rule(&findings, "physical_address"));
    }
    #[test]
    fn test_physical_address_no_match() {
        let findings = classify("address: Tokyo, Japan");
        assert!(!has_rule(&findings, "physical_address"));
    }

    // 37. Kubernetes Secret
    #[test]
    fn test_kubernetes_secret_match() {
        let findings = classify("kind: Secret");
        assert!(has_rule(&findings, "kubernetes_secret"));
    }
    #[test]
    fn test_kubernetes_secret_no_match() {
        let findings = classify("kind: ConfigMap");
        assert!(!has_rule(&findings, "kubernetes_secret"));
    }

    // 38. Terraform State Secret
    #[test]
    fn test_terraform_state_match() {
        let findings = classify(r#""type": "aws_iam_role""#);
        assert!(has_rule(&findings, "terraform_state_secret"));
    }
    #[test]
    fn test_terraform_state_no_match() {
        let findings = classify(r#""type": "google_compute""#);
        assert!(!has_rule(&findings, "terraform_state_secret"));
    }

    // 39. Authorization Bearer
    #[test]
    fn test_auth_bearer_match() {
        let findings = classify("Authorization: Bearer eyJhbGcitoken123.test=");
        assert!(has_rule(&findings, "authorization_bearer"));
    }
    #[test]
    fn test_auth_bearer_no_match() {
        let findings = classify("Authorization: Basic dXNlcjpwYXNz");
        assert!(!has_rule(&findings, "authorization_bearer"));
    }

    // 40. Redis URL
    #[test]
    fn test_redis_url_match() {
        let findings = classify("url: redis://user:pass@localhost:6379/0");
        assert!(has_rule(&findings, "redis_url"));
    }
    #[test]
    fn test_redis_url_no_match() {
        let findings = classify("url: https://redis.io");
        assert!(!has_rule(&findings, "redis_url"));
    }

    // 41. Postmark Server Token
    #[test]
    fn test_postmark_token_match() {
        let findings = classify("postmark_token=12345678-1234-1234-1234-123456789abc");
        assert!(has_rule(&findings, "postmark_server_token"));
    }
    #[test]
    fn test_postmark_token_no_match() {
        let findings = classify("postmark_region=us");
        assert!(!has_rule(&findings, "postmark_server_token"));
    }

    // 42. Databricks Token
    #[test]
    fn test_databricks_token_match() {
        let token = format!("dapi{}", "a".repeat(32));
        let findings = classify(&token);
        assert!(has_rule(&findings, "databricks_token"));
    }
    #[test]
    fn test_databricks_token_no_match() {
        let findings = classify("dapishort");
        assert!(!has_rule(&findings, "databricks_token"));
    }

    // 43. OpenAI API Key
    #[test]
    fn test_openai_api_key_match() {
        let key = format!("sk-{}T3BlbkFJ{}", "A".repeat(20), "B".repeat(20));
        let findings = classify(&key);
        assert!(has_rule(&findings, "openai_api_key"));
    }
    #[test]
    fn test_openai_api_key_no_match() {
        let findings = classify("sk-shortkey");
        assert!(!has_rule(&findings, "openai_api_key"));
    }

    // 44. Anthropic API Key
    #[test]
    fn test_anthropic_api_key_match() {
        let key = format!("sk-ant-{}", "A".repeat(80));
        let findings = classify(&key);
        assert!(has_rule(&findings, "anthropic_api_key"));
    }
    #[test]
    fn test_anthropic_api_key_no_match() {
        let findings = classify("sk-ant-short");
        assert!(!has_rule(&findings, "anthropic_api_key"));
    }

    // 45. Supabase Key
    #[test]
    fn test_supabase_key_match() {
        let findings = classify(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSJ9.signature_here",
        );
        assert!(has_rule(&findings, "supabase_key"));
    }
    #[test]
    fn test_supabase_key_no_match() {
        let findings = classify("eyJhbGciOiJSUzI1NiJ9.payload.sig");
        assert!(!has_rule(&findings, "supabase_key"));
    }

    // 46. Firebase Config
    #[test]
    fn test_firebase_config_match() {
        let findings = classify(r#"firebase config apiKey: "AIzaSyABCDEF""#);
        assert!(has_rule(&findings, "firebase_config"));
    }
    #[test]
    fn test_firebase_config_no_match() {
        let findings = classify("firebase deploy --project myapp");
        assert!(!has_rule(&findings, "firebase_config"));
    }

    // 47. HashiCorp Vault Token
    #[test]
    fn test_vault_token_match() {
        let token = format!("hvs.{}", "A".repeat(24));
        let findings = classify(&token);
        assert!(has_rule(&findings, "hashicorp_vault_token"));
    }
    #[test]
    fn test_vault_token_no_match() {
        let findings = classify("hvs.short");
        assert!(!has_rule(&findings, "hashicorp_vault_token"));
    }
}
