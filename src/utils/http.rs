use anyhow::Result;
use url::Url;

/// Parse and validate a URL
pub fn parse_url(url_str: &str) -> Result<Url> {
    let url = Url::parse(url_str)?;

    // Validate scheme
    match url.scheme() {
        "http" | "https" => Ok(url),
        scheme => anyhow::bail!("Unsupported URL scheme: {}", scheme),
    }
}

/// Check if a string looks like a GitHub URL
pub fn is_github_url(url_str: &str) -> bool {
    url_str.contains("github.com")
}

/// Extract GitHub repo info from URL
pub fn parse_github_url(url_str: &str) -> Option<(String, String)> {
    let url = Url::parse(url_str).ok()?;

    if url.host_str()? != "github.com" {
        return None;
    }

    let path_segments: Vec<&str> = url.path_segments()?.collect();

    if path_segments.len() >= 2 {
        let owner = path_segments[0].to_string();
        let repo = path_segments[1].trim_end_matches(".git").to_string();
        Some((owner, repo))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        assert!(parse_url("https://example.com").is_ok());
        assert!(parse_url("http://example.com").is_ok());
        assert!(parse_url("ftp://example.com").is_err());
        assert!(parse_url("not a url").is_err());
    }

    #[test]
    fn test_is_github_url() {
        assert!(is_github_url("https://github.com/user/repo"));
        assert!(!is_github_url("https://gitlab.com/user/repo"));
    }

    #[test]
    fn test_parse_github_url() {
        let result = parse_github_url("https://github.com/cisco-ai-defense/mcp-scanner");
        assert_eq!(result, Some(("cisco-ai-defense".to_string(), "mcp-scanner".to_string())));

        let result2 = parse_github_url("https://github.com/user/repo.git");
        assert_eq!(result2, Some(("user".to_string(), "repo".to_string())));

        let result3 = parse_github_url("https://gitlab.com/user/repo");
        assert_eq!(result3, None);
    }
}
