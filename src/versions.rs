use semver::Version;
use tokio::task;

fn loose_parse(v: &str) -> Option<Version> {
    let v = v.trim().trim_start_matches('v'); // Also handle "v1.2.3"

    // Separate version numbers from metadata
    let main_part = v.split(|c| c == '-' || c == '+').next()?;
    let metadata = &v[main_part.len()..];

    // Pad version parts
    let mut parts: Vec<&str> = main_part.split('.').collect();
    while parts.len() < 3 {
        parts.push("0");
    }

    let mut normalized = parts.join(".");
    normalized.push_str(metadata);

    Version::parse(&normalized).ok()
}

pub async fn get_vulnerable_versions(
    all_versions: Vec<String>,
    version_range: String,
) -> Vec<String> {
    // Offload to blocking thread to keep the async reactor free for network I/O
    task::spawn_blocking(move || {
        let conditions: Vec<(&str, Version)> = version_range
            .split(',')
            .filter_map(|part| {
                let part = part.trim();
                let (op, val) = if let Some(s) = part.strip_prefix(">=") {
                    (">=", s)
                } else if let Some(s) = part.strip_prefix("<=") {
                    ("<=", s)
                } else if let Some(s) = part.strip_prefix('>') {
                    (">", s)
                } else if let Some(s) = part.strip_prefix('<') {
                    ("<", s)
                } else if let Some(s) = part.strip_prefix('=') {
                    ("=", s)
                } else {
                    ("=", part)
                };

                loose_parse(val).map(|v| (op, v))
            })
            .collect();

        all_versions
            .into_iter()
            .filter(|v_str| {
                if let Some(version) = loose_parse(v_str) {
                    conditions.iter().all(|(op, target)| match *op {
                        ">=" => version >= *target,
                        "<=" => version <= *target,
                        ">" => version > *target,
                        "<" => version < *target,
                        "=" => version == *target,
                        _ => false,
                    })
                } else {
                    false
                }
            })
            .collect()
    })
    .await
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vulnerable_range_less_than() {
        let all_versions = vec![
            "1.0.0".to_string(),
            "2.0.0".to_string(),
            "5.0.1".to_string(),
        ];
        let range = "< 5.0.1".to_string();
        let result = get_vulnerable_versions(all_versions, range).await;
        assert_eq!(result, vec!["1.0.0", "2.0.0"]);
    }

    #[tokio::test]
    async fn test_compound_range() {
        let all_versions = vec![
            "0.9.0".to_string(),
            "0.10.2".to_string(),
            "0.10.5".to_string(),
            "0.11.0".to_string(),
        ];
        let range = ">= 0.10.0, < 0.10.5".to_string();
        let result = get_vulnerable_versions(all_versions, range).await;
        assert_eq!(result, vec!["0.10.2"]);
    }

    #[tokio::test]
    async fn test_pre_release_versions() {
        let all_versions = vec![
            "1.0.0-beta.25".to_string(),
            "1.0.0".to_string(),
            "2.0.0".to_string(),
        ];
        let range = "< 1.0.0".to_string();
        let result = get_vulnerable_versions(all_versions, range).await;
        assert_eq!(result, vec!["1.0.0-beta.25"]);
    }
}
