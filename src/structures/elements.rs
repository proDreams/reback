use crate::structures::backup_params::BackupParams;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Elements {
    pub element_title: String,
    pub element_type: String,
    pub s3_folder: String,
    pub backup_retention_days: u64,
    pub s3_backup_retention_days: u64,
    pub params: Option<BackupParams>,
}

impl Elements {
    pub async fn perform_backup(&self) -> Result<(), String> {
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
            }
            Some(BackupParams::Mongodb {
                db_name,
                db_user,
                db_password,
            }) => {
                println!("Backing up MongoDB: db={}, user={}", db_name, db_user);
            }
            Some(BackupParams::MongodbDocker {
                docker_container,
                db_name,
                db_user,
                db_password,
            }) => {
                println!(
                    "Backing up MongoDB: docker_container={}, db={}, user={}",
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
        Ok(())
    }
}
