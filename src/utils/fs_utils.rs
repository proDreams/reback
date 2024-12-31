use chrono::{Duration, Local, NaiveDateTime};
use std::fs;
use std::io;
use std::path::Path;
use log::info;

/// Checks for and deletes outdated local backup files based on the retention period.
///
/// This function scans the specified directory for files matching the given title, extracts the
/// creation date from the filename, and compares it to the current date. Files older than the specified
/// retention period will be deleted.
///
/// The filenames must follow a specific pattern: they should start with the `title` followed by a date in
/// the format `title-yyyy-mm-dd_HH-MM-SS.<extension>`. The function will delete files that are older than
/// the specified retention period.
///
/// # Arguments
/// - `path` - The path to the directory containing the backup files.
/// - `title` - The prefix used in the filenames of the backup files. Only files starting with this title
///             will be considered for deletion.
/// - `retention` - The retention period in days. Files older than this period will be deleted.
///
/// # Returns
/// - `Ok(())` if the function completes successfully, i.e., the outdated backup files are checked and
///   deleted as necessary.
/// - An error of type `io::Error` if reading the directory or deleting a file fails.
///
/// # Example
/// ```rust
/// let backup_dir: Path = /* directory path */;
/// let title: String = "backup_title".to_string();
/// let retention_days: u64 = 30;
/// check_outdated_local_backups(&backup_dir, &title, &retention_days)?;
/// ```
pub fn check_outdated_local_backups(path: &Path, title: &String, retention: &u64) -> io::Result<()> {
    let now = Local::now();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.is_file() {
            if let Some(file_name) = file_path.file_name().and_then(|name| name.to_str()) {
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
                                        fs::remove_file(file_path)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    info!("Check and delete outdated local backups completed");

    Ok(())
}
