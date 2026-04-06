use crate::structs::PackageVersions;
use log::debug;
use reqwest::Client;
use roxmltree::Document;
use std::error::Error;

pub async fn get_versions(
    client: &Client,
    name: String,
) -> Result<PackageVersions, Box<dyn Error>> {
    let url = format!(
        "https://repo1.maven.org/maven2/{}/maven-metadata.xml",
        name.replace('.', "/").replace(':', "/")
    );
    debug!("Fetching Maven metadata from: {}", url);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!(
            "Maven package not found: {} (Status {})",
            name,
            response.status()
        )
        .into());
    }

    let xml_data = response.text().await?;

    // Offload XML parsing to a blocking thread
    // This prevents heavy parsing from stuttering the async runtime
    let versions = tokio::task::spawn_blocking(move || {
        let doc = Document::parse(&xml_data).map_err(|e| e.to_string())?;

        let v_list: Vec<String> = doc
            .descendants()
            .filter(|n| n.has_tag_name("version"))
            .filter_map(|n| n.text())
            .map(String::from)
            .collect();

        Ok::<Vec<String>, String>(v_list)
    })
    .await??; // Double ?? because spawn_blocking returns a Result, and our closure returns a Result

    debug!("Found {} versions for {}", versions.len(), name);

    Ok(PackageVersions {
        ecosystem: "maven".to_string(),
        name,
        versions,
    })
}

#[tokio::test]
async fn test_get_versions_pax_logging_log4j2() {
    let client = reqwest::Client::new();
    let name = "org.ops4j.pax.logging:pax-logging-log4j2".to_string();

    let result_struct = get_versions(&client, name)
        .await
        .expect("Failed to fetch Maven versions");

    let result = result_struct.versions;

    let expected_to_be_included = vec![
        "1.8.0", "1.8.1", "1.8.2", "1.8.3", "1.8.4", "1.8.5", "1.8.6", "1.8.7", "1.9.0", "1.9.1",
        "1.9.2", "1.10.0", "1.10.1", "1.10.2", "1.10.3", "1.10.4", "1.10.5", "1.10.6", "1.10.7",
        "1.10.8", "1.10.9", "1.10.10", "1.11.0", "1.11.1", "1.11.2", "1.11.3", "1.11.4", "1.11.5",
        "1.11.6", "1.11.7", "1.11.8", "1.11.9", "1.11.10", "1.11.11", "1.11.12", "1.11.13",
        "1.11.14", "1.11.15", "1.11.16", "1.11.17", "1.12.0", "1.12.1", "1.12.2", "1.12.3",
        "1.12.4", "1.12.5", "1.12.6", "1.12.7", "1.12.8", "1.12.9", "1.12.10", "1.12.11",
        "1.12.12", "1.12.13", "1.12.14", "1.12.15", "2.0.0", "2.0.1", "2.0.2", "2.0.3", "2.0.4",
        "2.0.5", "2.0.6", "2.0.7", "2.0.8", "2.0.9", "2.0.10", "2.0.11", "2.0.12", "2.0.13",
        "2.0.14", "2.0.15", "2.0.16", "2.0.17", "2.0.18", "2.0.19", "2.1.0", "2.1.1", "2.1.2",
        "2.1.3", "2.1.4", "2.2.0", "2.2.1", "2.2.2", "2.2.3", "2.2.4", "2.2.5", "2.2.6", "2.2.7",
        "2.2.8", "2.2.9", "2.2.10", "2.2.11", "2.3.0", "2.3.1", "2.3.2",
    ];

    for item in expected_to_be_included {
        assert!(
            result.contains(&item.to_string()),
            "Maven version {} was missing from registry response",
            item
        );
    }
}
