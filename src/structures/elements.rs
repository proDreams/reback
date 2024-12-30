use crate::structures::backup_params::BackupParams;
use chrono::Local;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct Elements {
    pub element_title: String,
    pub s3_folder: String,
    pub backup_retention_days: u64,
    pub s3_backup_retention_days: u64,
    pub params: Option<BackupParams>,
}

impl Elements {
    pub async fn perform_backup(&self, path: &Path) -> Result<PathBuf, String> {
        let mut file_path: PathBuf = Default::default();
        match &self.params {
            Some(BackupParams::Postgresql {
                db_host,
                db_port,
                db_name,
                db_user,
                db_password,
            }) => {
                println!(
                    "Backing up PostgreSQL: host={}, port={}, db={}, user={}",
                    db_host, db_port, db_name, db_user
                );
            }
            Some(BackupParams::PostgresqlDocker {
                docker_container,
                db_name,
                db_user,
                db_password,
            }) => {
                println!(
                    "Backing up PostgreSQL Docker: docker_container={}, db={}, user={}",
                    docker_container, db_name, db_user
                );

                let now = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

                let file_name = format!("{}-{}.sql", self.element_title, now);
                file_path = path.join(&file_name);

                let command = format!(
                    "docker exec {} bash -c \"PGPASSWORD='{}' pg_dump -U {} {}\" > {}",
                    docker_container,
                    db_password,
                    db_user,
                    db_name,
                    file_path.display(),
                );
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()
                    .expect("Failed to execute backup command");

                if output.status.success() {
                    println!("Backup PostgreSQL Docker created successfully!");
                } else {
                    eprintln!("Backup failed!");
                    eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Some(BackupParams::Mongodb {
                db_name,
                db_user,
                db_password,
            }) => {
                println!("Backing up MongoDB: db={:?}, user={:?}", db_name, db_user);
            }
            Some(BackupParams::MongodbDocker {
                docker_container,
                db_name,
                db_user,
                db_password,
            }) => {
                println!(
                    "Backing up MongoDB: docker_container={}, db={:?}, user={:?}",
                    docker_container, db_name, db_user
                );
            }
            Some(BackupParams::Folder { path }) => {
                println!("Backing up folder: path={}", path);
            }
            None => {
                return Err(format!(
                    "No backup parameters provided for element '{}'",
                    self.element_title
                ));
            }
        }
        Ok(file_path)
    }
}
