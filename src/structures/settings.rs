use std::io;
use serde::Deserialize;
use crate::utils;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_bucket: String,
    elements: Vec<Elements>,
}

#[derive(Debug, Deserialize)]
struct Elements {
    s3_folder: String,
    dest_type: String,
    backup_retention_days: u64,
    s3_backup_retention_days: u64,
}

impl Settings {
    pub fn new() -> io::Result<Settings> {
        let settings: Settings = match utils::json_utils::read_json() {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Error reading JSON file: {}", err);
                return Err(err);
            }
        };

        Ok(settings)
    }
}
