use crate::utils::validate_file;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

#[derive(clap::Args, Debug)]
#[group(required = false, multiple = false)]
pub struct Source {
    #[arg(short = 'u', long, help = "Twitter usernames, comma separated")]
    pub username: Option<String>,

    #[arg(short = 'i', long,value_parser = validate_file, help = "List of username in file new line spereted")]
    pub input: Option<PathBuf>,
}
impl Source {
    pub fn get_usernames(&self) -> Vec<String> {
        if let Some(username) = &self.username {
            let mut usernames = Vec::new();
            for username in username.split(',') {
                usernames.push(username.to_string());
            }
            if usernames.is_empty() {
                println!("No usernames provided");
                exit(1);
            }
            usernames
        } else if let Some(input) = &self.input {
            let contents = fs::read_to_string(input).unwrap();
            if contents.is_empty() {
                println!("file is empty");
                exit(1);
            }
            let usernames: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
            return usernames;
        } else {
            println!("no input");
            exit(1);
        }
    }
}
