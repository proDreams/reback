use crate::structures::settings::Settings;
use crate::utils::fs_utils::check_outdated_local_backups;
use crate::utils::s3_utils::{check_outdated_s3_backups, upload_file_to_s3};
use s3::Bucket;
use std::fs;
use std::path::Path;

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
