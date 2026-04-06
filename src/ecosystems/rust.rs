use crate::structs::PackageVersions;
use log::debug;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;

pub async fn get_versions(
    client: &Client,
    name: String,
) -> Result<PackageVersions, Box<dyn Error>> {
    let pairs: Vec<String> = name
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .take(2)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect();
    let dir = pairs.join("/");

    let url = format!("https://index.crates.io/{}/{}", dir, name);
    debug!("Fetching Rust metadata from: {}", url);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Crate not found: {} (Status {})", name, response.status()).into());
    }

    let text = response.text().await?;

    // Offload JSON Lines parsing to blocking thread
    let versions = tokio::task::spawn_blocking(move || {
        let mut versions_list = Vec::new();

        // Crates.io returns one JSON object per line
        for line in text.lines().filter(|l| !l.is_empty()) {
            if let Ok(json) = serde_json::from_str::<Value>(line) {
                if let Some(v) = json.get("vers").and_then(|v| v.as_str()) {
                    versions_list.push(v.to_string());
                }
            }
        }

        versions_list
    })
    .await?;

    debug!("Found {} versions for {}", versions.len(), name);

    Ok(PackageVersions {
        ecosystem: "rust".to_string(),
        name,
        versions,
    })
}

#[tokio::test]
async fn test_get_versions_rust() {
    let client = reqwest::Client::new();
    let name = "sparse-merkle-tree".to_string();

    let result_struct = get_versions(&client, name)
        .await
        .expect("Failed to fetch Rust crate versions from crates.io");

    let result = result_struct.versions;

    let expected_to_be_included = vec![
        "0.1.0-alpha1",
        "0.1.0-alpha2",
        "0.1.0-alpha3",
        "0.1.0",
        "0.1.1",
        "0.1.2",
        "0.1.3",
        "0.2.0",
        "0.3.0",
        "0.3.1-pre",
        "0.4.0-rc1",
        "0.5.0-rc1",
        "0.5.0-rc2",
        "0.5.2-rc1",
        "0.5.2",
        "0.5.3",
        "0.5.4",
        "0.6.0",
        "0.6.1",
    ];

    for item in expected_to_be_included {
        assert!(
            result.contains(&item.to_string()),
            "Crate version {} was not found in the crates.io index response",
            item
        );
    }

    debug!("Successfully validated {} Rust versions", result.len());
}
