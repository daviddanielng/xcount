use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::exit;

#[derive(Serialize, Deserialize)]

pub struct AppConfig {
    pub exe_path: PathBuf,
}

impl AppConfig {
    pub fn new(exe_path: &PathBuf) -> AppConfig {
        AppConfig {
            exe_path: exe_path.clone(),
        }
    }

    pub fn file_path() -> PathBuf {
        let confi_dir = dirs::config_dir();

        match confi_dir {
            None => {
                eprintln!("Could not find config directory");
                exit(1);
            }
            Some(dir) => {
                let p = dir.join("xcount");
                if !p.exists() {
                    std::fs::create_dir_all(&p).unwrap();
                }
                p.join("app_config.json")
            }
        }
    }
    pub fn load() -> AppConfig {
        let confi_dir = AppConfig::file_path();
        if !confi_dir.exists() {
            eprintln!("Could not find config file, run xcount --set-exe-path <path>");
            exit(1);
        }
        let data = std::fs::read_to_string(confi_dir).unwrap();
        let p: AppConfig = serde_json::from_str(&*data).unwrap();

        p
    }
    pub fn save(&self) {
        let data = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(AppConfig::file_path(), data).unwrap();
    }
}

pub fn require_chrome() -> PathBuf {
    let appconfig = AppConfig::load();
    if !is_chrome(&appconfig.exe_path) {
        eprintln!("Could not find chrome executable, run xcount --set-exe-path <path>");
        exit(1);
    }
    return appconfig.exe_path;
}
pub fn is_chrome(path: &PathBuf) -> bool {
    if !path.exists() {
        eprintln!("{}: No such file ", path.display());
        return false;
    }
    if !path.is_file() {
        eprintln!("{}: Not a file", path.display());
        return false;
    }
    true
}
