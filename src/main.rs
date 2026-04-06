use clap::Parser;
use futures::stream::{self, StreamExt};
use log::debug;
use reqwest::Client;
use std::fs::File;
use std::io::BufWriter;
use std::time::Duration;

use versionhub::ecosystems::go::get_versions as get_versions_go;
use versionhub::ecosystems::maven::get_versions as get_versions_maven;
use versionhub::ecosystems::npm::get_versions as get_versions_npm;
use versionhub::ecosystems::rust::get_versions as get_versions_rust;
use versionhub::structs::{Args, PackageVersions};
use versionhub::versions::get_vulnerable_versions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&args.log_level))
        .init();

    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

    let results = stream::iter(args.packages)
        .map(|pkg| {
            let client = client.clone();
            async move {
                debug!(
                    "Fetching: Ecosystem: {}, Name: {}, Range: {:?}.",
                    pkg.ecosystem, pkg.name, pkg.version_range
                );

                // Fetch all versions from the ecosystem
                let mut pkg_versions = match pkg.ecosystem.to_lowercase().as_str() {
                    "go" => get_versions_go(&client, pkg.name).await,
                    "maven" => get_versions_maven(&client, pkg.name).await,
                    "npm" => get_versions_npm(&client, pkg.name).await,
                    "rust" => get_versions_rust(&client, pkg.name).await,
                    _ => Err(format!("Unsupported ecosystem: {}.", pkg.ecosystem).into()),
                }?;

                // If a version range is provided, filter the results
                if let Some(range) = pkg.version_range {
                    debug!(
                        "Filtering {} versions for {} based on range: {}.",
                        pkg_versions.versions.len(),
                        pkg_versions.name,
                        range
                    );

                    let filtered = get_vulnerable_versions(pkg_versions.versions, range).await;
                    pkg_versions.versions = filtered;
                }

                Ok::<PackageVersions, Box<dyn std::error::Error>>(pkg_versions)
            }
        })
        .buffer_unordered(args.concurrency)
        .collect::<Vec<_>>()
        .await;

    let mut successful_results = Vec::new();

    for res in results {
        match res {
            Ok(versions) => {
                if args.output.is_none() {
                    if let Ok(json) = serde_json::to_string(&versions) {
                        println!("{}", json);
                    }
                }
                successful_results.push(versions);
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    if let Some(path) = args.output {
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &successful_results)?;
        println!("Results saved to {:?}.", path);
    }

    Ok(())
}
