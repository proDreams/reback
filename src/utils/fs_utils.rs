use chrono::{Duration, Local, NaiveDateTime};
use std::fs;
use std::io;
use std::path::Path;

/// Checks for and deletes outdated local backup files based on the retention period.
///
/// This function scans the specified directory for files that match the given title, checks their
/// creation date (extracted from the filename), and compares it against the current date. If the file
/// is older than the specified retention period, it will be deleted.
///
/// The filenames must follow a pattern where the title is followed by a date in the format
/// `title-yyyy-mm-dd_HH-MM-SS.<extension>`. The function deletes files that are older than the given
/// retention period.
///
/// # Arguments
/// - `path` - The path to the directory where the backup files are stored.
/// - `title` - The title used in the filenames of the backup files. Files that start with this title
///             will be considered for deletion.
/// - `retention` - The retention period in days. Files older than this period will be deleted.
///
/// # Errors
/// This function will return an error if reading the directory or deleting a file fails.
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

    println!("Check and delete outdated local backups completed");

    Ok(())
}
