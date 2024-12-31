use crate::structures::settings::Settings;
use crate::utils::fs_utils::check_outdated_local_backups;
use crate::utils::s3_utils::{check_outdated_s3_backups, upload_file_to_s3};
use log::{error, info, warn};
use s3::Bucket;
use std::fs;
use std::path::Path;

/// Starts the backup process for all elements in the provided settings.
///
/// This function iterates over the elements defined in the `settings` and performs the following tasks
/// for each element:
/// - Creates a backup directory if it does not already exist.
/// - Performs the backup using the parameters defined for the element.
/// - Uploads the resulting backup file to the specified S3 bucket.
/// - Deletes outdated local backups based on the retention days specified.
/// - Deletes outdated backups from the S3 bucket based on the retention days specified for S3 backups.
///
/// # Arguments
/// - `settings` - The configuration containing backup settings and elements to back up.
/// - `bucket` - The S3 bucket where the backup files will be uploaded.
///
/// # Behavior
/// - The function will attempt to process each element in the `settings`. If any operation fails (directory creation,
///   backup creation, file upload, or outdated backup deletion), the error is logged, and the function continues with
///   the next element. This ensures that a failure in one element does not stop the backup process for other elements.
///
/// # Errors
/// This function will panic if:
/// - A directory creation or backup operation fails.
/// - Uploading the backup file to S3 fails.
/// - Deleting outdated local or S3 backups fails.
///
/// # Example
/// ```rust
/// let settings: Settings = /* Obtain backup settings */;
/// let bucket: Bucket = /* Obtain the S3 bucket instance */;
/// start_backup_process(&settings, &bucket).await;
/// ```
pub async fn start_backup_process(settings: &Settings, bucket: &Bucket) {
    for element in &settings.elements {
        let path_str = format!("{}/{}", settings.backup_dir, element.element_title);
        let path = Path::new(&path_str);

        if !path.exists() {
            if let Err(e) = fs::create_dir_all(path) {
                error!("Failed to create backup dir {}: {}", path.display(), e);
                continue;
            }
            info!("Created backup dir {}", path.display());
        }

        let file_path = match element.perform_backup(&path).await {
            Ok(f) => f,
            Err(e) => {
                warn!(
                    "Backup process encountered an error for {}: {}",
                    element.element_title, e
                );
                continue;
            }
        };

        if let Err(e) = upload_file_to_s3(&bucket, &file_path, &element.s3_folder).await {
            error!(
                "Failed to upload file to S3 for {}: {}",
                element.element_title, e
            );
            continue;
        }

        if let Err(e) = check_outdated_local_backups(
            &path,
            &element.element_title,
            &element.backup_retention_days,
        ) {
            error!(
                "Failed to delete outdated local backups for {}: {}",
                element.element_title, e
            );
            continue;
        }

        if let Err(e) = check_outdated_s3_backups(
            &bucket,
            &element.element_title,
            &element.s3_folder,
            &element.s3_backup_retention_days,
        )
        .await
        {
            error!(
                "Failed to delete outdated backups from S3 for {}: {}",
                element.element_title, e
            );
            continue;
        }
    }
}
