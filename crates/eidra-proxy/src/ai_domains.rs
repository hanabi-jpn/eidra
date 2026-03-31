const AI_PROVIDER_DOMAINS: &[&str] = &[
    "api.openai.com",
    "api.anthropic.com",
    "generativelanguage.googleapis.com",
    "api.cohere.ai",
    "api.cohere.com",
    "api.mistral.ai",
    "api.groq.com",
    "api.together.xyz",
    "api.fireworks.ai",
    "api.perplexity.ai",
    "api.deepseek.com",
];

pub fn is_ai_provider(host: &str) -> bool {
    let host_lower = host.to_lowercase();
    // Strip port if present
    let hostname = host_lower.split(':').next().unwrap_or(&host_lower);
    AI_PROVIDER_DOMAINS.contains(&hostname)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_provider_match() {
        assert!(is_ai_provider("api.openai.com"));
        assert!(is_ai_provider("api.anthropic.com"));
        assert!(is_ai_provider("api.openai.com:443"));
    }

    #[test]
    fn test_non_ai_provider() {
        assert!(!is_ai_provider("example.com"));
        assert!(!is_ai_provider("google.com"));
    }
}
