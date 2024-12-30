use chrono::{Duration, Local, NaiveDateTime};
use s3::bucket::Bucket;
use std::error::Error;
use std::fs;
use std::path::Path;

pub async fn upload_file_to_s3(
    bucket: &Bucket,
    path: &Path,
    s3_folder: &String,
) -> Result<(), Box<dyn Error>> {
    let file =
        fs::read(path).map_err(|e| format!("Failed to open file {}: {}", path.display(), e))?;

    let file_name = path
        .file_name()
        .ok_or_else(|| format!("Failed to extract file name from {}", path.display()))?;
    let file_name = file_name.to_string_lossy();

    let s3_path = format!("/{}/{}", s3_folder, file_name);

    bucket
        .put_object(&s3_path, &file)
        .await
        .map_err(|e| format!("Failed to upload file to S3 {}: {}", s3_path, e))?;

    println!("File uploaded successfully to {}", s3_path);
    Ok(())
}

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
