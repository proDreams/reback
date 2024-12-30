use crate::structures::backup_params::BackupParams;
use chrono::Local;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
    /// This function generates a backup for the element using the appropriate method:
    /// PostgreSQL, MongoDB, Docker-based PostgreSQL, Docker-based MongoDB, or folder backup.
    /// It constructs the required backup command, executes it, and returns the path to the backup file.
    ///
    /// # Arguments
    /// - `path` - The base directory path where the backup file will be stored.
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - The path of the generated backup file.
    /// - `Err(String)` - An error message if backup parameters are not provided or an error occurs during backup.
    ///
    /// # Behavior
    /// - The function formats the backup filename with a timestamp (e.g., `element-title-YYYY-MM-DD_HH-MM-SS.sql`).
    /// - It executes different backup commands depending on the backup type (PostgreSQL, MongoDB, Docker-based backups, etc.).
    /// - If no backup parameters are provided, it returns an error with the element title.
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
                println!(
                    "Backing up PostgreSQL: host={}, port={}, db={}, user={}",
                    db_host, db_port, db_name, db_user
                );

                let file_name = format!("{}-{}.sql", self.element_title, now);
                file_path = path.join(&file_name);

                let command = format!(
                    "PGPASSWORD=\"{}\" pg_dump -U {} -h {} -p {} {} > {}",
                    db_password,
                    db_user,
                    db_host,
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
                println!(
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
                println!("Backing up MongoDB");

                let file_name = format!("{}-{}.gz", self.element_title, now);
                file_path = path.join(&file_name);

                let command = match db_user {
                    Some(user) => {
                        format!(
                            "mongodump --host {} --port {} --username {} --password {:?} --authenticationDatabase admin --archive={} --gzip",
                            db_host,
                            db_port,
                            user,
                            db_password,
                            file_path.display(),
                        )
                    }
                    None => {
                        format!(
                            "mongodump --host {} --port {} --archive={} --gzip",
                            db_host,
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
                println!("Backing up MongoDB: docker_container={}", docker_container);

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
                println!("Backing up folder: path={}", target_path);

                let file_name = format!("{}-{}.tar.gz", self.element_title, now);
                file_path = path.join(&file_name);

                let command = format!("tar -czvf {} -C {} .", file_path.display(), target_path);

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
    /// This function runs a shell command (using `sh -c`) to perform the backup operation,
    /// printing a success or error message depending on the command's result.
    ///
    /// # Arguments
    /// - `command` - The shell command to execute.
    ///
    /// # Returns
    /// - `Output` - The output of the executed command, containing the status and any stdout/stderr.
    ///
    /// # Behavior
    /// - If the command succeeds, the function prints a success message.
    /// - If the command fails, it prints an error message and the error details.
    ///
    /// # Example
    /// ```rust
    /// let output = element.execute_command(&command).await;
    /// ```
    async fn execute_command(&self, command: &str) -> Output {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .expect("Failed to execute backup command");

        if output.status.success() {
            println!("Backup created successfully!");
        } else {
            eprintln!("Backup failed!");
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        }

        output
    }
}
