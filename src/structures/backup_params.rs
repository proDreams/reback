use serde::Deserialize;

/// Enum representing the different types of backup parameters.
///
/// This enum defines the configuration for various backup types, including:
/// PostgreSQL (both normal and Docker-based), MongoDB (normal and Docker-based), and Folder backups.
/// Each variant contains the relevant configuration for connecting to the database or specifying the folder
/// to be backed up.
///
/// # Variants
/// - `Postgresql` - Represents a PostgreSQL backup, with details about the database host, port, name,
///   user, and password.
/// - `PostgresqlDocker` - Represents a PostgreSQL backup from a Docker container, with details about the
///   Docker container, database name, user, and password.
/// - `Mongodb` - Represents a MongoDB backup, with details about the host, port, and optional user/password.
/// - `MongodbDocker` - Represents a MongoDB backup from a Docker container, with optional user/password.
/// - `Folder` - Represents a folder backup, with a path to the folder to back up.
///
/// # Example
/// ```rust
/// let backup_config = BackupParams::Postgresql {
///     db_host: "localhost".to_string(),
///     db_port: 5432,
///     db_name: "my_db".to_string(),
///     db_user: "user".to_string(),
///     db_password: "password".to_string(),
/// };
/// ```
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BackupParams {
    Postgresql {
        db_host: Option<String>,
        db_port: u16,
        db_name: String,
        db_user: String,
        db_password: String,
    },
    PostgresqlDocker {
        docker_container: String,
        db_name: String,
        db_user: String,
        db_password: String,
    },
    Mongodb {
        db_host: Option<String>,
        db_port: u16,
        db_user: Option<String>,
        db_password: Option<String>,
    },
    MongodbDocker {
        docker_container: String,
        db_user: Option<String>,
        db_password: Option<String>,
    },
    Folder {
        target_path: String,
    },
    MySQL {
        db_host: Option<String>,
        db_port: u16,
        db_name: String,
        db_user: String,
        db_password: String,
    },
    MySQLDocker {
        docker_container: String,
        db_name: String,
        db_user: String,
        db_password: String,
    },
}
