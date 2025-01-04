use anyhow::Result;
use chrono::{DateTime, Duration, Local};
use log::{error, info, warn};
use s3::bucket::Bucket;
use s3::error::S3Error;
use s3::serde_types::ListBucketResult;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{BufReader};

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

/// Retrieves a list of objects from an S3 bucket in a specified folder asynchronously.
///
/// This function constructs a prefix using the provided `folder` and attempts to list the objects
/// in the S3 bucket under that prefix. It uses the `bucket.list()` method to retrieve the object list,
/// and if the request is successful, it returns the list of objects. If any error occurs during
/// the operation, the error is logged, and the function returns the error.
///
/// # Arguments
/// - `bucket` - The S3 bucket from which the list of objects will be retrieved.
/// - `folder` - The folder within the S3 bucket whose objects are to be listed.
///
/// # Returns
/// - `Ok(Vec<ListBucketResult>)` containing the list of objects in the specified folder if successful.
/// - `Err(S3Error)` if the request fails, with an error describing the failure.
///
/// # Errors
/// This function will return an error if:
/// - The request to list the objects from the S3 bucket fails.
///
/// # Example
/// ```rust
/// let bucket: Bucket = /* Obtain the S3 bucket instance */;
/// let folder = "path/to/folder".to_string();
/// match get_s3_objects_list(&bucket, &folder).await {
///     Ok(objects) => println!("Objects: {:?}", objects),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub async fn get_s3_objects_list(
    bucket: &Bucket,
    folder: &String,
) -> Result<Vec<ListBucketResult>, S3Error> {
    let prefix = format!("{}/", folder);

    // Попробуем получить список объектов
    match bucket.list(prefix.clone(), None).await {
        Ok(list) => Ok(list),
        Err(e) => {
            error!("Failed to get list of s3 objects: {}", e);
            Err(e)
        }
    }
}

/// Checks for outdated backups in an S3 bucket and deletes them if they exceed the specified retention period.
///
/// This function lists the objects in the specified S3 folder and checks each object's modification timestamp.
/// If an object’s modification time is older than the specified retention period (in days), it deletes the object
/// from the S3 bucket. The modification timestamp is retrieved from the `last_modified` property of each object.
///
/// # Arguments
/// - `bucket` - The S3 bucket where the backups are stored.
/// - `folder` - The folder within the S3 bucket containing the backup files to be checked.
/// - `retention` - The retention period in days. Any file older than this period will be deleted.
///
/// # Returns
/// - `Ok(())` if the outdated backups were successfully checked and deleted.
/// - `Err(Box<dyn Error>)` if an error occurs, such as an issue with listing objects or deleting files.
///
/// # Errors
/// This function will return an error if:
/// - Listing the objects in the S3 bucket fails.
/// - Parsing the `last_modified` timestamp of a file fails.
/// - Deleting a file fails due to permissions or other issues.
///
/// # Notes
/// - The `last_modified` property is expected to be in RFC 3339 format, which is the standard format for timestamps
///   in S3 metadata. If parsing fails, the file is skipped, and a warning is logged.
/// - Files older than the specified retention period are deleted from the S3 bucket.
///
/// # Example
/// ```rust
/// let bucket: Bucket = /* Obtain the S3 bucket instance */;
/// let folder = "backup_folder".to_string();
/// let retention = 30; // Retention period of 30 days
/// check_outdated_s3_backups(&bucket, &folder, &retention).await?;
/// ```
pub async fn check_outdated_s3_backups(
    bucket: &Bucket,
    folder: &String,
    retention: &u64,
) -> Result<(), Box<dyn Error>> {
    let now = Local::now();

    let results = match get_s3_objects_list(bucket, folder).await {
        Ok(results) => results,
        Err(e) => {
            return Err(e.into());
        }
    };

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

/// Finds the latest backup file in an S3 bucket folder based on the modification date.
///
/// This function lists all objects in the specified S3 folder and checks the `last_modified` timestamp
/// of each object to determine the most recent backup file. The latest file is returned as the file key (name).
/// If no backups are found in the folder, an error is returned.
///
/// # Arguments
/// - `bucket` - The S3 bucket from which the backup files are being checked.
/// - `folder` - The folder within the S3 bucket to search for backup files.
///
/// # Returns
/// - `Ok(String)` containing the key (name) of the latest backup file if found.
/// - `Err(Box<dyn Error>)` if an error occurs, such as failing to list objects or parse timestamps.
///
/// # Errors
/// This function will return an error if:
/// - Listing the objects in the S3 bucket fails.
/// - Parsing the `last_modified` timestamp of a file fails.
/// - No backups are found in the folder.
///
/// # Example
/// ```rust
/// let bucket: Bucket = /* Obtain the S3 bucket instance */;
/// let folder = "backup_folder".to_string();
/// match find_latest_s3_backup(&bucket, &folder).await {
///     Ok(latest_backup) => println!("Latest backup: {}", latest_backup),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub async fn find_latest_s3_backup(
    bucket: &Bucket,
    folder: &String,
) -> Result<String, Box<dyn Error>> {
    let results = match get_s3_objects_list(bucket, folder).await {
        Ok(results) => results,
        Err(e) => {
            return Err(e.into());
        }
    };

    let mut latest_backup: Option<(String, DateTime<Local>)> = None;

    for result in results {
        let contents = result.contents;

        for object in contents {
            let last_modified_str = &object.last_modified;

            if let Ok(last_modified) = DateTime::parse_from_rfc3339(last_modified_str) {
                let last_modified_local = last_modified.with_timezone(&Local);

                if latest_backup
                    .as_ref()
                    .map_or(true, |(_, latest_date)| last_modified_local > *latest_date)
                {
                    latest_backup = Some((object.key.clone(), last_modified_local));
                }
            } else {
                warn!(
                    "Failed to parse last_modified for object {}: {}",
                    object.key, last_modified_str
                );
            }
        }
    }

    if let Some((key, _)) = latest_backup {
        info!("Latest backup found: {})", key);
        Ok(key)
    } else {
        info!("No backups found in folder: {}", folder);
        Err("No backups found".into())
    }
}

/// Downloads the latest backup file from an S3 bucket to a local directory.
///
/// This function first retrieves the latest backup file by calling `find_latest_s3_backup` and then
/// downloads the file from the S3 bucket to the specified local path. If the local directory doesn't exist,
/// it is created before downloading the file.
///
/// # Arguments
/// - `bucket` - The S3 bucket containing the backup file to be downloaded.
/// - `path` - The local directory where the backup file will be saved.
/// - `file_key` - The folder in the S3 bucket where the backup files are stored (used to find the latest backup).
///
/// # Returns
/// - `Ok(PathBuf)` containing the path to the downloaded file if successful.
/// - `Err(Box<dyn Error>)` if any error occurs during the file download or directory creation.
///
/// # Errors
/// This function will return an error if:
/// - The latest backup cannot be found in the specified folder.
/// - The directory cannot be created.
/// - The file download fails due to S3 or network issues.
///
/// # Example
/// ```rust
/// let bucket: Bucket = /* Obtain the S3 bucket instance */;
/// let path = "local_backup_dir".to_string();
/// let folder = "backup_folder".to_string();
/// match get_file_from_s3(&bucket, &path, &folder).await {
///     Ok(file_path) => println!("Backup downloaded to: {}", file_path.display()),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub async fn get_file_from_s3(
    bucket: &Bucket,
    path: &String,
    file_key: &String,
) -> Result<PathBuf, Box<dyn Error>> {
    let file_key = find_latest_s3_backup(&bucket, &file_key).await?;

    let file_path = format!("{}/{}", &path, file_key);
    let path = Path::new(&file_path);

    if let Some(parent_dir) = path.parent() {
        if !parent_dir.exists() {
            if let Err(e) = fs::create_dir_all(parent_dir) {
                error!(
                    "Failed to create backup directory {}: {}",
                    parent_dir.display(),
                    e
                );
                return Err(e.into());
            }
            info!("Created backup directory {}", parent_dir.display());
        }
    }

    let mut async_output_file = File::create(&path).await?;

    bucket
        .get_object_to_writer(&file_key, &mut async_output_file)
        .await?;

    info!("File downloaded successfully: {}", &file_key);

    Ok(PathBuf::from(path))
}
