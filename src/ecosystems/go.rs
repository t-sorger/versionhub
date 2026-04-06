use crate::structs::PackageVersions;
use log::debug;
use reqwest::Client;
use std::error::Error;
use std::string::ToString;

pub async fn get_versions(
    client: &Client,
    name: String,
) -> Result<PackageVersions, Box<dyn Error>> {
    let ecosystem: String = "go".to_string();
    let url = format!("https://proxy.golang.org/{}/@v/list", name);
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch versions: HTTP {}", response.status()).into());
    }

    let text = response.text().await?;

    let versions: Vec<String> = text
        .lines()
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();

    debug!("Found {} versions for {}", versions.len(), name);

    Ok(PackageVersions {
        ecosystem,
        name,
        versions,
    })
}

#[tokio::test]
async fn test_get_versions_runc() {
    let client = reqwest::Client::new();
    let name = "github.com/opencontainers/runc".to_string();

    let result_struct = get_versions(&client, name)
        .await
        .expect("Failed to fetch versions");

    let result = result_struct.versions;

    let expected_to_be_included = vec![
        "v1.0.0-rc6",
        "v1.3.0",
        "v1.0.2",
        "v1.0.0-rc8",
        "v1.2.3",
        "v1.0.0-rc90",
        "v1.0.0-rc4",
        "v1.2.6",
        "v1.1.0-rc.1",
        "v1.0.0-rc7",
        "v1.2.7",
        "v1.0.0-rc93",
        "v1.0.0",
        "v1.2.8",
        "v0.0.1",
        "v1.3.3",
        "v1.1.6",
        "v1.4.0-rc.2",
        "v1.2.5",
        "v1.1.1",
        "v1.0.0-rc95",
        "v1.3.1",
        "v1.3.4",
        "v0.0.5",
        "v1.4.1",
        "v1.0.3",
        "v1.1.3",
        "v1.2.0-rc.3",
        "v1.0.0-rc94",
        "v1.1.2",
        "v1.1.11",
        "v1.3.0-rc.1",
        "v1.0.0-rc5",
        "v1.1.12",
        "v0.0.3",
        "v1.0.0-rc10",
        "v1.0.0-rc91",
        "v0.0.4",
        "v1.0.0-rc1",
        "v1.2.4",
        "v1.3.2",
        "v1.4.0-rc.1",
        "v1.2.1",
        "v1.3.0-rc.2",
        "v1.1.5",
        "v1.0.0-rc92",
        "v0.1.0",
        "v1.4.0",
        "v0.0.2",
        "v1.1.4",
        "v1.2.0",
        "v1.2.9",
        "v0.1.1",
        "v1.1.10",
        "v1.2.2",
        "v0.0.8",
        "v0.0.9",
        "v1.1.9",
        "v1.0.0-rc9",
        "v1.1.8",
        "v1.3.5",
        "v1.5.0-rc.1",
        "v1.2.0-rc.1",
        "v1.1.13",
        "v1.1.14",
        "v1.0.0-rc3",
        "v1.0.1",
        "v1.4.0-rc.3",
        "v1.1.0",
        "v1.0.0-rc2",
        "v1.1.7",
        "v0.0.6",
        "v0.0.7",
        "v1.2.0-rc.2",
        "v1.1.15",
    ];

    for item in expected_to_be_included {
        assert!(
            result.contains(&item.to_string()),
            "Required version {} was not found in the result list",
            item
        );
    }
}
