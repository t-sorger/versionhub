use crate::structs::PackageVersions;
use log::debug;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;

pub async fn get_versions(
    client: &Client,
    name: String,
) -> Result<PackageVersions, Box<dyn Error>> {
    let url = format!("https://registry.npmjs.org/{}", name);
    debug!("Fetching NPM metadata from: {}", url);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!(
            "NPM package not found: {} (Status {})",
            name,
            response.status()
        )
        .into());
    }

    let text = response.text().await?;

    // Offload JSON parsing to a blocking thread
    // Parsing a huge JSON string into a Value map is CPU-intensive.
    let versions = tokio::task::spawn_blocking(move || {
        let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

        let mut versions_list = Vec::new();

        // Access the "versions" object and collect the keys (version strings)
        if let Some(versions_map) = v.get("versions").and_then(|v| v.as_object()) {
            versions_list = versions_map.keys().cloned().collect();
        }

        Ok::<Vec<String>, String>(versions_list)
    })
    .await??;

    debug!("Found {} versions for {}", versions.len(), name);

    Ok(PackageVersions {
        ecosystem: "npm".to_string(),
        name,
        versions,
    })
}

#[tokio::test]
async fn test_get_versions_npm() {
    let client = reqwest::Client::new();
    let name = "lodash.template".to_string();

    let result_struct = get_versions(&client, name)
        .await
        .expect("Failed to fetch NPM versions from registry");

    let result = result_struct.versions;

    let expected_to_be_included = vec![
        "2.0.0", "2.1.0", "2.2.0", "2.2.1", "2.3.0", "2.4.0", "2.4.1", "3.0.0", "3.0.1", "3.1.0",
        "3.2.0", "3.3.0", "3.3.1", "3.3.2", "3.4.0", "3.5.0", "3.5.1", "3.6.0", "3.6.1", "3.6.2",
        "4.0.0", "4.0.1", "4.0.2", "4.1.0", "4.1.1", "4.18.0", "4.18.1", "4.2.0", "4.2.1", "4.2.2",
        "4.2.3", "4.2.4", "4.2.5", "4.3.0", "4.4.0", "4.5.0",
    ];

    for item in expected_to_be_included {
        assert!(
            result.contains(&item.to_string()),
            "NPM version {} was missing from registry response",
            item
        );
    }

    debug!("Validated {} versions for NPM ", result.len());
}
