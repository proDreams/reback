use chrono::{Duration, Local, NaiveDateTime};
use s3::bucket::Bucket;
use std::error::Error;
use tokio::fs::File;
use tokio::io::BufReader;
use std::path::Path;

/// Uploads a file to an S3 bucket asynchronously.
///
/// This function uploads the specified file to the given S3 bucket at the path determined by the
/// `s3_folder` and the file's name. It uses asynchronous I/O to open and read the file from the
/// provided local `path`, ensuring efficient resource usage and avoiding potential blocking
/// operations. The file is then streamed to the specified S3 folder.
///
/// # Arguments
/// - `bucket` - The S3 bucket where the file will be uploaded.
/// - `path` - The local path to the file that will be uploaded.
/// - `s3_folder` - The folder in the S3 bucket where the file will be stored.
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

    println!("File uploaded successfully to {}", s3_path);
    Ok(())
}

/// Checks for outdated backups in an S3 bucket and deletes them if they exceed the retention period.
///
/// This function lists the files in the specified S3 folder and checks their timestamps based on their
/// filenames. If a file's timestamp indicates it is older than the specified retention period, the file
/// is deleted from the bucket.
///
/// # Arguments
/// - `bucket` - The S3 bucket where the backups are stored.
/// - `title` - The prefix to identify the backup files (typically the name of the backup element).
/// - `folder` - The S3 folder containing the backup files.
/// - `retention` - The retention period in days. Files older than this period will be deleted.
///
/// # Errors
/// This function will return an error if:
/// - Listing the objects in the S3 bucket fails.
/// - Parsing the file timestamps or deleting a file fails.
///
/// # Example
/// ```rust
/// let bucket: Bucket = /* Obtain the S3 bucket instance */;
/// let title = "my_backup".to_string();
/// let folder = "backup_folder".to_string();
/// let retention = 30; // Delete backups older than 30 days
/// check_outdated_s3_backups(&bucket, &title, &folder, &retention).await?;
/// ```
pub async fn check_outdated_s3_backups(
    bucket: &Bucket,
    title: &String,
    folder: &String,
    retention: &u64,
) -> Result<(), Box<dyn Error>> {
    let now = Local::now();
    let prefix = format!("{}/", folder);

    let results = bucket.list(prefix.clone(), None).await?;

    for result in results {
        let contents = result.contents;

        for object in contents {
            let file_path = object.key;
            if let Some(file_name) = file_path.strip_prefix(&prefix) {
                if file_name.starts_with(title) {
                    if let Some(date_str) = file_name.strip_prefix(&format!("{}-", title)) {
                        if let Some(date_str) = date_str.split('.').next() {
                            if let Ok(file_date) =
                                NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d_%H-%M-%S")
                            {
                                if let Some(file_date_local) =
                                    file_date.and_local_timezone(Local).single()
                                {
                                    let file_age = now - file_date_local;
                                    if file_age > Duration::days(*retention as i64) {
                                        bucket.delete_object(file_path).await?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("Check and delete outdated S3 backups completed");

    Ok(())
}
