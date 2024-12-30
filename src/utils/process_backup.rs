use crate::structures::settings::Settings;
use crate::utils::fs_utils::check_outdated_local_backups;
use crate::utils::s3_utils::{check_outdated_s3_backups, upload_file_to_s3};
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
            fs::create_dir_all(path).expect("Failed to create backup dir");
            println!("Created backup dir {}", path.display());
        }

        let file_path = match element.perform_backup(&path).await {
            Ok(f) => f,
            Err(_) => {
                continue;
            }
        };

        upload_file_to_s3(&bucket, &file_path, &element.s3_folder)
            .await
            .expect("Failed to upload file");

        check_outdated_local_backups(
            &path,
            &element.element_title,
            &element.backup_retention_days,
        )
        .expect("Failed to delete outdated local backups");

        check_outdated_s3_backups(
            &bucket,
            &element.element_title,
            &element.s3_folder,
            &element.s3_backup_retention_days,
        )
        .await
        .expect("Failed to delete outdated backups from S3");
    }
}
