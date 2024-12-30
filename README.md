# Universal Backup and Restore Utility

## [Русская версия](./README-RU.md)

This utility allows you to perform backups of databases (PostgreSQL, MongoDB) and folders, both locally and to Amazon
S3. It tracks backup retention and organizes backups in directories based on the specified configuration. The restore
functionality is planned for future releases.

## Features

- Supports backup of PostgreSQL, MongoDB and Dockerized databases.
- Backs up folders to local directories or Amazon S3/Minio/Other S3-like.
- Retains backups according to the specified retention periods.
- Logs each step in the backup process.
- Configurable through a `settings.json` file.

## Prerequisites

- Docker (required for PostgreSQL Docker and MongoDB Docker backups)
- S3 access credentials

## Installation

### Option 1: Download from Releases

You can download the latest release from the [Releases page](https://github.com/yourusername/backup-utility/releases).

1. Go to the [Releases page](https://github.com/yourusername/backup-utility/releases).
2. Download the appropriate binary for your operating system.
3. Extract the archive and make the binary executable (if needed):

    ```bash
    chmod +x universal-backup-restore
    ```

### Option 2: Build Locally

If you'd like to build the application yourself, follow these steps:

1. Clone the repository to your local machine:

    ```bash
    git clone https://github.com/yourusername/backup-utility.git
    cd backup-utility
    ```

2. Build the application (you need Rust installed):

    ```bash
    cargo build --release
    ```

3. Make the binary executable:

    ```bash
    chmod +x target/release/universal-backup-restore
    ```

4. You can now run the binary from the `target/release` folder or move it to a directory of your choice.

## Configuration

The application is configured through a `settings.json` file, which should be placed in the same directory as the
executable (`universal-backup-restore`). Below is an example of the configuration:

1. Create a `settings.json` file next to the executable (`universal-backup-restore`).
2. Copy the following template into your `settings.json` and modify the values to suit your environment:

```json
{
  "s3_endpoint": "https://s3.example.com",
  "s3_region": "us-east-1",
  "s3_bucket": "my-bucket",
  "s3_access": "access-key",
  "s3_secret": "secret-key",
  "backup_dir": "/tmp/backups",
  "elements": [
    {
      "element_title": "my_pg_db",
      "s3_folder": "postgres_backups",
      "backup_retention_days": 30,
      "s3_backup_retention_days": 90,
      "params": {
        "type": "postgresql",
        "db_host": "localhost",
        "db_port": 5432,
        "db_name": "my_database",
        "db_user": "user",
        "db_password": "password"
      }
    },
    {
      "element_title": "my_mongo_db",
      "s3_folder": "mongodb_backups",
      "backup_retention_days": 30,
      "s3_backup_retention_days": 90,
      "params": {
        "type": "mongodb",
        "db_host": "localhost",
        "db_port": 27017,
        "db_name": "my_mongo_database",
        "db_user": "mongo_user",
        "db_password": "mongo_password"
      }
    },
    {
      "element_title": "project_folder",
      "s3_folder": "folder_backups",
      "backup_retention_days": 30,
      "s3_backup_retention_days": 90,
      "params": {
        "type": "folder",
        "target_path": "/path/to/folder"
      }
    }
  ]
}
```

### Available Backup Types

- **Postgresql**: PostgreSQL database backup.
- **PostgresqlDocker**: PostgreSQL database backup from a Docker container.
- **Mongodb**: MongoDB database backup.
- **MongodbDocker**: MongoDB database backup from a Docker container.
- **Folder**: Backup of a local folder.

Each type has its own set of parameters, as defined in the configuration file.

## Usage

To start the backup process, run the following command:

```bash
./universal-backup-restore backup
```

The utility will:

1. Backup databases and folders locally and to S3.
2. Organize backups in subdirectories based on the configuration.
3. Retain backups according to the specified retention settings.

### Running as a Cron Job

You can automate the backup process by creating a Cron job. For example, to run the backup every day at 2:00 AM, add the
following line to your crontab file:

```bash
0 2 * * * /path/to/universal-backup-restore backup >> /path/to/logs/backup.log 2>&1
```

This Cron job will:

- Execute the `universal-backup-restore backup` command at 2:00 AM every day.
- Redirect both standard output and error output to a log file (`backup.log`).

To edit your crontab, run:

```bash
crontab -e
```

Then add the Cron job line. Make sure the path to the `universal-backup-restore` binary and log file is correct.

## Backup Retention

Backups are retained based on the configuration in the `settings.json` file. The `backup_retention_days` parameter
specifies how long backups are kept locally, while the `s3_backup_retention_days` parameter specifies how long backups
are kept in S3.

## Author

This project was created and is maintained by Ivan Ashikhmin.
Feel free to open an issue or contribute to the project.

## Donate

If you appreciate this project and want to support further development, consider donating:

Donate via TON: `UQBU8rJEfUcBvJUbz6NbXiWxaOO_NoXHK_pXOWv7qsOBWbFp`

Your support helps keep the project alive and improve future features!

