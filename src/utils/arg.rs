use crate::utils::output::OutputKind;
use crate::utils::source::Source;
use crate::utils::validate_dir;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "xcount", about = "An cli tool for extrating twitter/X user followers and following cout", long_about = None,version,about)]
pub struct Args {
    #[arg(short = 'v', long, default_value = "false")]
    pub verbose: bool,
    #[command(flatten, help = "Source of data, either username or file")]
    pub source: Source,
    #[arg(short = 'o', long,value_parser=validate_dir, default_value = ".", help = "Output directory")]
    pub output: PathBuf,
    #[arg(
        short = 'f',
        long,
        help = "Output format: json, csv, excel",
        default_value = "json",
    value_parser= parse_format
    )]
    pub format: OutputKind,
    #[arg(
        short = 'd',
        long,
        help = "Delay between requests in seconds, we don't want to fast track to an ip ban",
        default_value = "1"
    )]
    pub delay: u64,
    #[arg(short = 'e', long, help = "Path to chromium executable")]
    pub set_exe_path: Option<PathBuf>,
}

fn parse_format(s: &str) -> Result<OutputKind, String> {
    if s == "json" {
        return Ok(OutputKind::Json);
    }
    if s == "csv" {
        return Ok(OutputKind::Csv);
    }
    if s == "excel" {
        return Ok(OutputKind::Excel);
    }
    Err(format!("Invalid format: {}", s))
}
