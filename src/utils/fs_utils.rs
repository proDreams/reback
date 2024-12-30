use chrono::{Duration, Local, NaiveDateTime};
use std::fs;
use std::io;
use std::path::Path;

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
