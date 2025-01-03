use chrono::{DateTime, Duration, Local};
use log::{info, warn};
use s3::bucket::Bucket;
use std::error::Error;
use std::path::Path;
use tokio::fs::File;
use tokio::io::BufReader;

/// Uploads a file to an S3 bucket asynchronously.
///
/// This function uploads the specified file to the given S3 bucket at the path determined by the
/// `s3_folder` and the file's name. It uses asynchronous I/O to open and read the file from the
/// provided local `path`, ensuring efficient resource usage without blocking operations.
/// The file is then streamed to the specified S3 folder.
///
/// # Arguments
/// - `bucket` - The S3 bucket where the file will be uploaded.
/// - `path` - The local path to the file that will be uploaded.
/// - `s3_folder` - The folder in the S3 bucket where the file will be stored.
///
/// # Returns
/// - `Ok(())` if the file is uploaded successfully.
/// - `Err(Box<dyn Error>)` if any error occurs, such as failing to open the file, extract its name, or upload it to S3.
///
/// # Errors
/// This function will return an error if:
/// - The file cannot be opened asynchronously from the provided path.
/// - The file name cannot be extracted from the path.
/// - The upload to S3 fails.
///
/// # Example
/// ```rust
/// let bucket: Bucket = /* Obtain the S3 bucket instance */;
/// let path: Path = /* Local path to the file */;
/// let s3_folder = "backup_folder".to_string();
/// upload_file_to_s3(&bucket, &path, &s3_folder).await?;
/// ```
pub async fn upload_file_to_s3(
    bucket: &Bucket,
    path: &Path,
    s3_folder: &String,
) -> Result<(), Box<dyn Error>> {
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("Failed to extract file name from {}", path.display()))?;
    let file_name = file_name.to_string_lossy();

    let s3_path = format!("/{}/{}", s3_folder, file_name);

    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);

    bucket
        .put_object_stream(&mut reader, s3_path.clone())
        .await
        .map_err(|e| format!("Failed to upload file to S3: {}", e))?;

    info!("File uploaded successfully to {}", s3_path);
    Ok(())
}

/// Checks for outdated backups in an S3 bucket and deletes them if they exceed the retention period.
///
/// This function lists the files in the specified S3 folder and checks their timestamps based on the
/// `last_modified` property provided by the S3 API. If a file's modification time indicates it is older
/// than the specified retention period, the file is deleted from the bucket.
///
/// # Arguments
/// - `bucket` - The S3 bucket where the backups are stored.
/// - `folder` - The S3 folder containing the backup files.
/// - `retention` - The retention period in days. Files older than this period will be deleted.
///
/// # Returns
/// - `Ok(())` if the outdated backups were successfully deleted.
/// - `Err(Box<dyn Error>)` if an error occurs during the process, such as issues with listing objects
///   or deleting files.
///
/// # Errors
/// This function will return an error if:
/// - Listing the objects in the S3 bucket fails.
/// - Parsing the `last_modified` timestamp of a file fails.
/// - Deleting a file fails due to insufficient permissions or other issues.
///
/// # Notes
/// - The `last_modified` property is parsed using RFC 3339 format, which is the standard format
///   for timestamps in S3 metadata.
/// - If the `last_modified` timestamp cannot be parsed for a file, a warning is logged, and the file
///   is skipped without affecting the rest of the process.
///
/// # Example
/// ```rust
/// let bucket: Bucket = /* Obtain the S3 bucket instance */;
/// let folder = "backup_folder".to_string();
/// let retention = 30; // Delete backups older than 30 days
/// check_outdated_s3_backups(&bucket, &folder, &retention).await?;
/// ```
pub async fn check_outdated_s3_backups(
    bucket: &Bucket,
    folder: &String,
    retention: &u64,
) -> Result<(), Box<dyn Error>> {
    let now = Local::now();
    let prefix = format!("{}/", folder);

    let results = bucket.list(prefix.clone(), None).await?;

    for result in results {
        let contents = result.contents;

        for object in contents {
            let last_modified_str = &object.last_modified;

            if let Ok(last_modified) = DateTime::parse_from_rfc3339(last_modified_str) {
                let file_age = now - last_modified.with_timezone(&Local);
                if file_age > Duration::days(*retention as i64) {
                    bucket.delete_object(&object.key).await?;
                    info!("Deleted outdated backup: {}", object.key);
                }
            } else {
                warn!(
                    "Failed to parse last_modified for object {}: {}",
                    object.key, last_modified_str
                );
            }
        }
    }

    info!("Check and delete outdated S3 backups completed");

    Ok(())
}
