use crate::structures::backup_params::BackupParams;
use chrono::Local;
use log::{error, info};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Represents an element to be backed up, along with its backup configuration.
///
/// This structure contains the title, S3 folder, retention settings, and optional
/// backup parameters for various types of elements (e.g., PostgreSQL, MongoDB, folder).
/// It also provides functionality to perform backups based on the provided parameters.
///
/// # Fields
/// - `element_title` - A descriptive title for the element being backed up.
/// - `s3_folder` - The folder in the S3 bucket where the backup should be stored.
/// - `backup_retention_days` - The number of days to retain the backup locally.
/// - `s3_backup_retention_days` - The number of days to retain the backup in the S3 bucket.
/// - `params` - Optional parameters describing the type of backup (e.g., database or folder).
#[derive(Debug, Deserialize)]
pub struct Elements {
    pub element_title: String,
    pub s3_folder: String,
    pub backup_retention_days: u64,
    pub s3_backup_retention_days: u64,
    pub params: Option<BackupParams>,
}

impl Elements {
    /// Performs a backup based on the specified parameters for the element.
    ///
    /// This function generates a backup for the element using the appropriate method: PostgreSQL, MongoDB, Docker-based PostgreSQL, Docker-based MongoDB, or folder backup.
    /// It constructs the required backup command, executes it, and returns the path to the backup file.
    ///
    /// The filename is formatted with a timestamp (e.g., `element-title-YYYY-MM-DD_HH-MM-SS.sql`) to avoid overwriting files.
    /// The backup command is executed for each type of backup, depending on the provided parameters.
    ///
    /// # Arguments
    /// - `path` - The base directory path where the backup file will be stored.
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - The path of the generated backup file.
    /// - `Err(String)` - An error message if backup parameters are not provided or an error occurs during backup.
    ///
    /// # Behavior
    /// - Executes a backup command based on the backup type specified in `self.params`.
    /// - If no backup parameters are provided (`None`), it returns an error with the element's title.
    /// - The method handles PostgreSQL, MongoDB, Docker-based backups, and folder backups.
    /// - For Docker-based backups, the appropriate `docker exec` commands are used to run the backups inside containers.
    /// - For folder backups, a `tar` command is used to create compressed archive files.
    ///
    /// # Example
    /// ```rust
    /// let backup_path = element.perform_backup(&backup_dir).await?;
    /// ```
    pub async fn perform_backup(&self, path: &Path) -> Result<PathBuf, String> {
        let now = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let file_path: PathBuf;

        match &self.params {
            Some(BackupParams::Postgresql {
                db_host,
                db_port,
                db_name,
                db_user,
                db_password,
            }) => {
                info!(
                    "Backing up PostgreSQL: host={}, port={}, db={}, user={}",
                    db_host, db_port, db_name, db_user
                );

                let file_name = format!("{}-{}.sql", self.element_title, now);
                file_path = path.join(&file_name);

                let command = format!(
                    "PGPASSWORD=\"{}\" pg_dump -U {} -h {} -p {} {} > {}",
                    db_password,
                    db_user,
                    db_host.clone().unwrap_or(String::from("localhost")),
                    db_port,
                    db_name,
                    file_path.display(),
                );

                self.execute_command(&command).await;
            }

            Some(BackupParams::PostgresqlDocker {
                docker_container,
                db_name,
                db_user,
                db_password,
            }) => {
                info!(
                    "Backing up PostgreSQL Docker: docker_container={}, db={}, user={}",
                    docker_container, db_name, db_user
                );

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

                self.execute_command(&command).await;
            }

            Some(BackupParams::Mongodb {
                db_host,
                db_port,
                db_user,
                db_password,
            }) => {
                info!("Backing up MongoDB");

                let file_name = format!("{}-{}.gz", self.element_title, now);
                file_path = path.join(&file_name);

                let command = match db_user {
                    Some(user) => {
                        format!(
                            "mongodump --host {} --port {} --username {} --password {:?} --authenticationDatabase admin --archive={} --gzip",
                            db_host.clone().unwrap_or(String::from("localhost")),
                            db_port,
                            user,
                            db_password,
                            file_path.display(),
                        )
                    }
                    None => {
                        format!(
                            "mongodump --host {} --port {} --archive={} --gzip",
                            db_host.clone().unwrap_or("localhost".to_string()),,
                            db_port,
                            file_path.display(),
                        )
                    }
                };

                self.execute_command(&command).await;
            }

            Some(BackupParams::MongodbDocker {
                docker_container,
                db_user,
                db_password,
            }) => {
                info!("Backing up MongoDB: docker_container={}", docker_container);

                let file_name = format!("{}-{}.gz", self.element_title, now);
                file_path = path.join(&file_name);

                let command = match db_user {
                    Some(user) => {
                        format!(
                            "docker exec {} mongodump --username {} --password {:?} --authenticationDatabase admin --archive=/backup/backup.gz --gzip",
                            docker_container,
                            user,
                            db_password,
                        )
                    }
                    None => {
                        format!(
                            "docker exec {} mongodump --archive=/backup/backup.gz --gzip",
                            docker_container,
                        )
                    }
                };

                self.execute_command(&command).await;

                let copy_backup_command = format!(
                    "docker cp {}:/backup/backup.gz {}",
                    docker_container,
                    file_path.display()
                );

                self.execute_command(&copy_backup_command).await;
            }

            Some(BackupParams::Folder { target_path }) => {
                info!("Backing up folder: path={}", target_path);

                let file_name = format!("{}-{}.tar.gz", self.element_title, now);
                file_path = path.join(&file_name);

                let command = format!("tar -czvf {} -C {} .", file_path.display(), target_path);

                self.execute_command(&command).await;
            }

            Some(BackupParams::MySQL {
                db_host,
                db_port,
                db_name,
                db_user,
                db_password,
            }) => {
                info!(
                    "Backing up MySQL: host={}, port={}, db={}, user={}",
                    db_host, db_port, db_name, db_user
                );

                let file_name = format!("{}-{}.sql", self.element_title, now);
                file_path = path.join(&file_name);

                let command = format!(
                    "MYSQL_PWD={} mysqldump -u {} -h {} -P {} {} > {}",
                    db_password,
                    db_user,
                    db_host.clone().unwrap_or(String::from("localhost")),
                    db_port,
                    db_name,
                    file_path.display(),
                );

                self.execute_command(&command).await;
            }

            Some(BackupParams::MySQLDocker {
                docker_container,
                db_name,
                db_user,
                db_password,
            }) => {
                info!(
                    "Backing up MySQL Docker: docker_container={}, db={}, user={}",
                    docker_container, db_name, db_user
                );

                let file_name = format!("{}-{}.sql", self.element_title, now);
                file_path = path.join(&file_name);

                let command = format!(
                    "docker exec {} bash -c \"MYSQL_PWD='{}' mysqldump -u {} {}\" > {}",
                    docker_container,
                    db_password,
                    db_user,
                    db_name,
                    file_path.display(),
                );

                self.execute_command(&command).await;
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

    /// Executes a shell command asynchronously to perform a backup.
    ///
    /// This function runs a shell command (using `sh -c`) to execute the backup operation.
    /// It captures the output and checks whether the command succeeded or failed, printing
    /// appropriate messages based on the result.
    ///
    /// # Arguments
    /// - `command` - The shell command to execute. This should be a valid shell command string
    ///   that performs the backup operation.
    ///
    /// # Returns
    /// - `()` - This function does not return any value. It logs success or failure messages
    ///   based on the command's execution status.
    ///
    /// # Behavior
    /// - If the command executes successfully (i.e., the exit status is `0`), it logs a success message.
    /// - If the command fails, it logs an error message along with the `stderr` output to provide error details.
    ///
    /// # Example
    /// ```rust
    /// element.execute_command(&command).await;
    /// ```
    async fn execute_command(&self, command: &str) {
        let output = match Command::new("sh").arg("-c").arg(command).output() {
            Ok(o) => o,
            Err(e) => {
                error!("Failed to execute backup command '{}': {}", command, e);
                return;
            }
        };

        if output.status.success() {
            info!("Backup created successfully!");
        } else {
            error!("Backup failed!");
            error!("Error: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
}
