use clap::Parser;
use std::process::exit;
use std::sync::OnceLock;

pub mod appconfig;
pub mod browser;
pub mod utils;

use crate::appconfig::is_chrome;
use crate::browser::get_data;
use crate::utils::validate_usernames;
use utils::arg;

static VERBOSE: OnceLock<bool> = OnceLock::new();

#[tokio::main]
async fn main() {
    let args = arg::Args::parse();
    match args.set_exe_path {
        Some(path) => {
            if !is_chrome(&path) {
                exit(1);
            }
            appconfig::AppConfig::new(&path).save();
            eprintln!("Using chrome executable: {}", path.display());
            exit(0);
        }
        None => {} // auto-detect
    };
    VERBOSE
        .set(args.verbose)
        .expect("Failed to set verbose flag");
    let config = appconfig::AppConfig::load();
    let usernames = args.source.get_usernames();
    validate_usernames(&usernames);
    let results = get_data(&config.exe_path, args.delay, usernames).await;
    args.format.export(&results, &args.output);
}
