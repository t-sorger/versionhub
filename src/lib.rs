pub mod ecosystems {
    pub mod go;
    pub mod maven;
    pub mod npm;
    pub mod rust;
}

pub mod structs;
pub mod versions;

use reqwest::Client;
use std::error::Error;
use std::time::Duration;
use structs::{Package, PackageVersions};
use versions::get_vulnerable_versions;

pub type VersionHubResult<T> = Result<T, Box<dyn Error>>;

/// Fetches package versions for a single package spec:
/// `ecosystem/package@version_range`
///
/// `@version_range` is optional.
pub async fn get_package_versions(package_spec: &str) -> VersionHubResult<PackageVersions> {
    let package = package_spec.parse::<Package>()?;
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    get_package_versions_with_client(&client, package).await
}

/// Same as `get_package_versions`, but reuses a caller-provided HTTP client.
pub async fn get_package_versions_with_client(
    client: &Client,
    package: Package,
) -> VersionHubResult<PackageVersions> {
    let mut pkg_versions = match package.ecosystem.to_lowercase().as_str() {
        "go" => ecosystems::go::get_versions(client, package.name).await,
        "maven" => ecosystems::maven::get_versions(client, package.name).await,
        "npm" => ecosystems::npm::get_versions(client, package.name).await,
        "rust" => ecosystems::rust::get_versions(client, package.name).await,
        _ => Err(format!("Unsupported ecosystem: {}.", package.ecosystem).into()),
    }?;

    if let Some(range) = package.version_range {
        let filtered = get_vulnerable_versions(pkg_versions.versions, range).await;
        pkg_versions.versions = filtered;
    }

    Ok(pkg_versions)
}
