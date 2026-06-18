use clap::Parser;
use std::sync::OnceLock;

pub mod scraper;
pub mod utils;

use crate::scraper::get_data;
use crate::utils::validate_usernames;
use utils::arg;

static VERBOSE: OnceLock<bool> = OnceLock::new();

#[tokio::main]
async fn main() {
    let args = arg::Args::parse();

    VERBOSE
        .set(args.verbose)
        .expect("Failed to set verbose flag");
    let usernames = args.source.get_usernames();
    validate_usernames(&usernames);
    let results = get_data(args.delay, &usernames).await;

    args.format.export(&results, &args.output);
}
