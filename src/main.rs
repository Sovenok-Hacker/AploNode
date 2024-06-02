mod errors;
mod log;
mod manager;
mod models;
mod node;
use std::fs;
use tracing::{debug, error, info, warn};

use clap::Parser;

/// Aplo coin node
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the file with config
    #[arg(short, long, default_value = "config.conf")]
    config_file: String,
}

fn load_config(config_path: &str) -> models::config::Config {
    let file_content = fs::read_to_string(config_path).unwrap();

    serde_json::from_str::<models::config::Config>(&file_content).unwrap()
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config = load_config(&args.config_file);

    log::init_logger(&config.log_config);

    debug!(
        "Logger initialization completed \n {:?} \n {:?}",
        &args, &config
    );
}
