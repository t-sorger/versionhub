use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Package {
    pub ecosystem: String,
    pub name: String,
    pub version_range: Option<String>,
}

impl FromStr for Package {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Separate the version first (everything after the LAST '@')
        // We use rsplit_once in case the package name itself contains an @
        let (full_name, version) = match s.rsplit_once('@') {
            Some((name, ver)) => (name, Some(ver.to_string())),
            None => (s, None),
        };

        // Split at the FIRST '/' to separate ecosystem from package
        let (ecosystem, package) = full_name
            .split_once('/')
            .ok_or_else(|| format!("Invalid format: '{}'. Expected 'ecosystem/package'", s))?;

        Ok(Package {
            ecosystem: ecosystem.to_string(),
            name: package.to_string(),
            version_range: version, // Matches your struct field name
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PackageVersions {
    pub ecosystem: String,
    pub name: String,
    pub versions: Vec<String>,
}
