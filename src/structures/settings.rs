use crate::structures::elements::Elements;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use serde::Deserialize;
use std::{env, fs, io};
use log::error;

/// Represents the application's configuration settings.
///
/// This structure holds all the necessary parameters for configuring
/// S3 bucket connections, backup directories, and elements to be processed.
///
/// # Fields
/// - `s3_endpoint` - The endpoint URL for the S3-compatible storage.
/// - `s3_region` - The region of the S3 bucket.
/// - `s3_bucket` - The name of the S3 bucket.
/// - `s3_access` - The access key for the S3 bucket.
/// - `s3_secret` - The secret key for the S3 bucket.
/// - `s3_path_style` - Defines the addressing style for the S3 bucket. Can be either `Path` or `VirtualHost`.
/// - `backup_dir` - The directory path where backups are temporarily stored before uploading.
/// - `elements` - A collection of elements to be processed for backup.
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access: String,
    pub s3_secret: String,
    pub s3_path_style: S3PathStyle,
    pub backup_dir: String,
    pub elements: Vec<Elements>,
}

/// Defines the addressing style for S3 bucket operations.
///
/// # Variants
/// - `Path` - Uses path-style addressing (e.g., `https://s3.amazonaws.com/bucket-name`).
/// - `VirtualHost` - Uses virtual-hosted-style addressing (e.g., `https://bucket-name.s3.amazonaws.com`).
///
/// This enum is serialized and deserialized with `kebab-case` naming conventions
/// (e.g., `"path"` or `"virtual-host"`).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum S3PathStyle {
    Path,
    VirtualHost,
}

impl Settings {
    /// Reads the application's configuration from a JSON file.
    ///
    /// This function attempts to read the `settings.json` file located in the same directory
    /// as the executable, deserialize its content into a `Settings` instance, and return it.
    /// If the file is not found or cannot be parsed, an appropriate error is returned.
    ///
    /// # Returns
    /// - `Ok(Settings)` if the file is successfully read and parsed into a `Settings` instance.
    /// - `Err(io::Error)` if the file cannot be read or if the JSON content is invalid.
    ///
    /// # Errors
    /// - If the file cannot be found or read, an error of kind `io::ErrorKind::NotFound` is returned.
    /// - If the JSON cannot be deserialized, an error of kind `io::ErrorKind::InvalidData` is returned
    ///   with additional error details from the `serde_json` deserialization process.
    ///
    /// # Example
    /// ```rust
    /// let settings = Settings::from_file().expect("Failed to load settings");
    /// ```
    pub fn from_file() -> io::Result<Settings> {
        let exe_path = env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();

        let settings_path = exe_dir.join("settings.json");

        let file_content = fs::read_to_string(settings_path)?;

        let settings: Settings = match serde_json::from_str(&file_content) {
            Ok(data) => data,
            Err(err) => {
                error!("Error parsing JSON file: {}", err);
                return Err(io::Error::new(io::ErrorKind::InvalidData, err));
            }
        };

        Ok(settings)
    }

    /// Creates and initializes an S3 bucket instance.
    ///
    /// This function uses the configuration provided in the `Settings` structure
    /// to create an S3 bucket instance. The bucket is configured according to the
    /// specified `s3_path_style`, which determines how the S3 endpoint is addressed.
    ///
    /// # Returns
    /// - `Some(Bucket)` if the bucket is successfully created and initialized.
    /// - `None` if there is an error during the bucket creation process.
    ///
    /// # Errors
    /// - Logs an error to `stderr` if credentials are invalid, if the bucket cannot be created,
    ///   or if other issues arise during the process.
    ///
    /// # Behavior
    /// - If `s3_path_style` is `S3PathStyle::Path`, the bucket is initialized with path-style addressing
    ///   using the `with_path_style()` method.
    /// - If `s3_path_style` is `S3PathStyle::VirtualHost`, the bucket is initialized without path-style addressing.
    ///
    /// # Example
    /// ```rust
    /// let bucket = settings.get_bucket().expect("Failed to create bucket");
    /// ```
    pub fn get_bucket(&self) -> Option<Bucket> {
        let credentials = Credentials::new(
            Some(&self.s3_access),
            Some(&self.s3_secret),
            None,
            None,
            None,
        )
        .map_err(|err| {
            error!("Error creating credentials: {}", err);
            err
        })
        .ok()?;

        let region = Region::Custom {
            region: self.s3_region.clone(),
            endpoint: self.s3_endpoint.clone(),
        };

        let bucket_result = Bucket::new(self.s3_bucket.as_str(), region, credentials);

        match bucket_result {
            Ok(bucket) => match self.s3_path_style {
                S3PathStyle::VirtualHost => Some(*bucket),
                S3PathStyle::Path => Some(*bucket.with_path_style()),
            },
            Err(err) => {
                error!("Error creating bucket: {}", err);
                None
            }
        }
    }
}
