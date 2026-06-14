use crate::appconfig::AppConfig;
use crate::utils::timestamp;
use clap::Parser;
use rust_xlsxwriter::{Workbook, XlsxError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]

pub enum OutputKind {
    Json,
    Csv,
    Excel,
}
#[derive(Serialize, Deserialize)]

#[derive(Debug)]
pub struct CrawlResult {
    pub username: String,
    pub followers: u64,
    pub following: u64,
}
impl CrawlResult {
    pub fn new(username: String, followers: u64, following: u64) -> CrawlResult {
        CrawlResult {
            username,
            followers,
            following,
        }
    }
}

impl OutputKind {
    pub fn export(&self, result: &Vec<CrawlResult>, dir: &PathBuf) {
        let mut file = dir.join(format!("output-{}", timestamp()));
        match self {
            OutputKind::Json => {
                file.add_extension("json");
                OutputKind::save_to_json(result, &file)
                    .expect("An error occurred saving json")
            },
            OutputKind::Csv => {
                file.add_extension("csv");
                OutputKind::save_to_csv(result, &file)
                    .expect("An error occurred saving csv");
            }
            OutputKind::Excel => {
                file.add_extension("xlsx");
                OutputKind::save_to_excel(result, &file)
                    .expect("An error occurred saving excel");
            }
        }
    }
    pub fn saved(path: &PathBuf) {
        eprintln!("Filed saved to {}", path.to_string_lossy());
    }
    pub fn save_to_json(result: &Vec<CrawlResult>, path: &PathBuf) -> Result<(), String> {
        let json = serde_json::to_string_pretty(result);
        if json.is_err() {
            return Err(json.unwrap_err().to_string());
        }
        let json = json.unwrap();

        std::fs::write(&path, json).expect("TODO: panic message");
        OutputKind::saved(&path);
        Ok(())
    }
    pub fn save_to_csv(result: &Vec<CrawlResult>, path: &PathBuf) -> Result<(), csv::Error> {
        let mut writer = csv::Writer::from_path(&path)?;
        for record in result {
            writer.serialize(record)?;
        }
        writer.flush()?;
        OutputKind::saved(&path);
        Ok(())
    }
    pub fn save_to_excel(result: &Vec<CrawlResult>, path: &PathBuf) -> Result<(), XlsxError> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        // headers
        worksheet.write_row(0, 0, ["username", "followers", "following"])?;

        // rows
        for (i, record) in result.iter().enumerate() {
            let row = (i + 1) as u32;
            worksheet.write(row, 0, &record.username)?;
            worksheet.write(row, 1, record.followers)?;
            worksheet.write(row, 2, record.following)?;
        }
        workbook.save(&path)?;
        OutputKind::saved(&path);
        Ok(())
    }
}
