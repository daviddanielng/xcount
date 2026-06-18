use clap::builder::Str;
use std::path::PathBuf;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod arg;
pub mod output;
pub mod source;
pub mod enity;

pub fn validate_dir(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.exists() {
        if !path.is_dir() {
            return Err(format!("Path is not a directory: {}", s));
        }
        Ok(path)
    } else {
        Err(format!("Directory does not exist: {}", s))
    }
}
pub fn validate_file(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.exists() {
        if !path.is_file() {
            return Err(format!("Path is not a file: {}", s));
        }
        Ok(path)
    } else {
        Err(format!("File does not exist: {}", s))
    }
}

pub fn validate_username(username: &str) -> Option<String> {
    if username.is_empty() {
        return Some(format!("`{}` Username cannot be empty", username));
    }

    if username.len() > 15 {
        return Some(format!(
            "`{}` Username cannot be longer than 15 characters",
            username
        ));
    }

    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Some(format!(
            "`{}` Username can only contain letters, numbers, and underscores",
            username
        ));
    }

    None
}
pub fn validate_usernames(usernames: &Vec<String>) {
    for username in usernames {
        if let Some(err) = validate_username(username) {
            println!("Invalid username: {}", err);
            exit(1);
        }
    }
}


fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mins = (secs / 60) % 60;
    let hours = (secs / 3600) % 24;
    let days = secs / 86400;

    // days since epoch to yyyy-mm-dd
    let mut year = 1970u32;
    let mut remaining = days as u32;

    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let months = [31, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u32;
    for days_in_month in months {
        if remaining < days_in_month {
            break;
        }
        remaining -= days_in_month;
        month += 1;
    }

    let day = remaining + 1;

    format!("{year}-{month:02}-{day:02}-{hours:02}:{mins:02}")
}

fn is_leap(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
pub fn parse_count(s: &str) -> u64 {
    let s = s.trim().to_lowercase();
    let s = s.replace(',', "");

    if let Some(n) = s.strip_suffix('k') {
        let n: f64 = n.trim().parse().unwrap_or(0.0);
        return (n * 1_000.0) as u64;
    }
    if let Some(n) = s.strip_suffix('m') {
        let n: f64 = n.trim().parse().unwrap_or(0.0);
        return (n * 1_000_000.0) as u64;
    }
    if let Some(n) = s.strip_suffix('b') {
        let n: f64 = n.trim().parse().unwrap_or(0.0);
        return (n * 1_000_000_000.0) as u64;
    }

    s.parse().unwrap_or(0)
}