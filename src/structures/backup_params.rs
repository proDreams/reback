use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BackupParams {
    Postgresql {
        db_host: String,
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
        db_name: Option<String>,
        db_user: Option<String>,
        db_password: Option<String>,
    },
    MongodbDocker {
        docker_container: String,
        db_name: Option<String>,
        db_user: Option<String>,
        db_password: Option<String>,
    },
    Folder {
        path: String,
    },
}
