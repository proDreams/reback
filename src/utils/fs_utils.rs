use chrono::{Duration, Local, TimeZone};
use log::{info, warn};
use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

/// Checks for and deletes outdated local backup files based on their last modified time.
///
/// This function scans the specified directory for files, retrieves their last modified time from
/// the filesystem metadata, and compares it to the current date. Files older than the specified
/// retention period will be deleted.
///
/// # Arguments
/// - `path` - The path to the directory containing the backup files.
/// - `retention` - The retention period in days. Files older than this period will be deleted.
///
/// # Returns
/// - `Ok(())` if the function completes successfully, i.e., the outdated backup files are checked and
///   deleted as necessary.
/// - An error of type `io::Error` if reading the directory, retrieving metadata, or deleting a file fails.
///
/// # Notes
/// - The function uses the `modified` time from the file metadata, which represents the last time the file
///   was modified. On most systems, this is more reliable than parsing timestamps from filenames.
/// - If retrieving metadata or the modified time fails for a file, a warning is logged, and the file
///   is skipped without affecting the rest of the process.
///
/// # Example
/// ```rust
/// let backup_dir: Path = /* directory path */;
/// let retention_days: u64 = 30;
/// check_outdated_local_backups(&backup_dir, &retention_days)?;
/// ```
pub fn check_outdated_local_backups(path: &Path, retention: &u64) -> io::Result<()> {
    let now = Local::now();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.is_file() {
            if let Ok(metadata) = fs::metadata(&file_path) {
                if let Ok(modified_time) = metadata.modified() {
                    if let Ok(file_date) = modified_time.duration_since(SystemTime::UNIX_EPOCH) {
                        if let Some(file_date) = Local.timestamp_opt(file_date.as_secs() as i64, 0).single() {
                            let file_age = now - file_date;
                            if file_age > Duration::days(*retention as i64) {
                                fs::remove_file(&file_path)?;
                                info!("Deleted outdated backup: {:?}", file_path);
                            }
                        }
                    }
                } else {
                    warn!("Failed to get modified time for file: {:?}", file_path);
                }
            } else {
                warn!("Failed to get metadata for file: {:?}", file_path);
            }
        }
    }

    info!("Check and delete outdated local backups completed");

    Ok(())
}
