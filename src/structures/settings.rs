use serde::Deserialize;
use std::{fs, io};
use crate::structures::elements::Elements;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access: String,
    pub s3_secret: String,
    pub temp_dir: String,
    pub elements: Vec<Elements>,
}


impl Settings {
    pub fn from_file() -> io::Result<Settings> {
        let file_content = fs::read_to_string("settings.json")?;

        let settings: Settings = match serde_json::from_str(&file_content) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Error parsing JSON file: {}", err);
                return Err(io::Error::new(io::ErrorKind::InvalidData, err));
            }
        };

        Ok(settings)
    }
}
