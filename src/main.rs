use clap::Parser;
use futures::stream::{self, StreamExt};
use log::debug;
use reqwest::Client;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::time::Duration;
use versionhub::get_package_versions_with_client;
use versionhub::structs::Package;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A list of packages to fetch (`ecosystem/package` or `ecosystem/package@version_range`).
    #[arg(short, long, alias = "pkg", alias = "pkgs", required = true, num_args = 1..)]
    packages: Vec<Package>,

    #[arg(short, long, default_value_t = 2)]
    concurrency: usize,

    /// Optional path to write JSON output
    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long, alias = "log", default_value = "warn")]
    log_level: String,
}

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
                get_package_versions_with_client(&client, pkg).await
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
