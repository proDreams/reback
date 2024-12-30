use crate::structures::elements::Elements;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use serde::Deserialize;
use std::{fs, io};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access: String,
    pub s3_secret: String,
    pub backup_dir: String,
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

    pub fn get_bucket(&self) -> Option<Bucket> {
        let credentials = Credentials::new(
            Some(&self.s3_access),
            Some(&self.s3_secret),
            None,
            None,
            None,
        )
        .map_err(|err| {
            eprintln!("Error creating credentials: {}", err);
            err
        })
        .ok()?;

        let region = Region::Custom {
            region: self.s3_region.clone(),
            endpoint: self.s3_endpoint.clone(),
        };

        match Bucket::new(self.s3_bucket.as_str(), region, credentials) {
            Ok(bucket) => Some(*bucket.with_path_style()),
            Err(err) => {
                eprintln!("Error creating bucket: {}", err);
                None
            }
        }
    }
}
