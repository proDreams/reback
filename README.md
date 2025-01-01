# Napkin Tools: ReBack (Restore Backup Utility)

![GitHub License](https://img.shields.io/github/license/proDreams/reback)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/proDreams/reback/build-release.yml)
![GitHub Release](https://img.shields.io/github/v/release/proDreams/reback)
![GitHub Downloads](https://img.shields.io/github/downloads/proDreams/reback/total)

## [Русская версия](./README-RU.md)

This utility is designed for server users who need to regularly create backups of databases (PostgreSQL, MongoDB and
MySQL) and directories. ReBack supports saving backups both locally and in S3-compatible storage, organizing them in a
convenient structure and automatically tracking their retention period.

Future versions will include functionality for restoring backups, including full restoration or restoration of specific
elements.

## Table of contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
    - [Option 1: Download from Releases](#option-1-download-from-releases)
    - [Option 2: Build Locally](#option-2-build-locally)
- [Configuration](#configuration)
    - [Configuration Parameters](#configuration-parameters)
    - [Adding Backup Elements](#adding-backup-elements)
    - [Available Element Types](#available-element-types)
    - [Backup Element Parameters Table](#backup-element-parameters-table)
    - [Common Parameters for All Elements](#common-parameters-for-all-elements)
- [Usage](#usage)
    - [Running as a Cron Job](#running-as-a-cron-job)
- [Author](#author)
- [Support](#support)
- [License](#license)

## Features

- Support for backing up PostgreSQL, MongoDB and MySQL, both locally installed and in Docker containers.
- Backup of local directories.
- Saving backups locally and in S3-compatible storage.
- Organizing backups in subdirectories based on element names specified in the configuration.
- Generating backup file names based on the element name and creation time.
- Logging all steps of the process.
- Configuration via the `settings.json` JSON file.

## Requirements

- Docker installed (for backing up from containers).
- S3-compatible storage (for remote backup storage).

## Installation

### Option 1: Download from Releases

You can download the latest version from the [releases page](https://github.com/proDreams/reback/releases/latest).  
The following platforms are supported:

- **macOS (Intel and ARM)**: `reback_macos_intel`, `reback_macos_aarch`
- **Linux**: `reback_linux`
- **Windows**: `reback.exe`

1. Go to the [releases page](https://github.com/proDreams/reback/releases/latest).
2. Download the appropriate binary for your operating system.
3. **For Linux and macOS**: Make the file executable by running the following command:

    ```bash
    chmod +x reback
    ```

### Option 2: Build Locally

If you want to build the application yourself, follow these steps:

**Requirements:**

- Installed Rust (recommended version: `rustc 1.83.0`).
- Rust toolchain manager: `rustup 1.27.1` or higher.

1. Clone the repository:

    ```bash
    git clone https://github.com/proDreams/reback.git
    cd reback
    ```

2. Build the application in release mode:

    ```bash
    cargo build --release
    ```

3. Make the binary executable (for Linux/macOS):

    ```bash
    chmod +x target/release/reback
    ```

## Configuration

The application is configured through the `settings.json` file, which should be located in the same directory as the
executable file (`reback`).

1. Create the `settings.json` file next to the executable (`reback`).
2. Copy the following template into your `settings.json` and adjust the values to match your environment:

```json
{
  "s3_endpoint": "https://s3.example.com",
  "s3_region": "us-east-1",
  "s3_bucket": "my-bucket",
  "s3_access": "access-key",
  "s3_secret": "secret-key",
  "s3_path_style": "path",
  "backup_dir": "/tmp/backups",
  "elements": []
}
```

### Configuration Parameters:

- **s3_endpoint**: URL of your S3-compatible storage.
- **s3_region**: Region of your S3 storage.
- **s3_bucket**: The name of the S3 bucket where backups will be saved.
- **s3_access**: Access key for connecting to S3.
- **s3_secret**: Secret key for connecting to S3.
- **s3_path_style**: Specifies the path style for S3 (e.g., "path" or "virtual-host").
- **backup_dir**: Absolute path to the directory where local backups will be stored. If the directory doesn't exist, it
  will be created automatically.
- **elements**: An array of objects, each representing an element for backup (e.g., a database or directory).

### Adding Backup Elements:

You can add backup elements to the `elements` array in the configuration file. Each element is an object with parameters
for backing up a specific type. Example:

```json
{
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

A complete configuration example is available in the file [settings.json.example](settings.json.example).

### Available Element Types:

- `postgresql` — Backup of a PostgreSQL database.
- `postgresql_docker` — Backup of a PostgreSQL database from a Docker container.
- `mongodb` — Backup of a MongoDB database.
- `mongodb_docker` — Backup of a MongoDB database from a Docker container.
- `mysql` — Backup of a MySQL database.
- `mysql_docker` — Backup of a MySQL database from a Docker container.
- `folder` — Backup of a local directory.

### Backup Element Parameters Table:

| Element Type          | Parameter          | Description                                   | Required |
|-----------------------|--------------------|-----------------------------------------------|----------|
| **postgresql**        | `db_host`          | Database host. Default: `localhost`.          | Optional |  
|                       | `db_port`          | Port for connection.                          | Required |  
|                       | `db_name`          | Name of the database.                         | Required |  
|                       | `db_user`          | Database user.                                | Required |  
|                       | `db_password`      | User password.                                | Required |  
|                       |                    |                                               |          |  
| **postgresql_docker** | `docker_container` | Name of the Docker container with PostgreSQL. | Required |  
|                       | `db_name`          | Name of the database.                         | Required |  
|                       | `db_user`          | Database user.                                | Required |  
|                       | `db_password`      | User password.                                | Required |  
|                       |                    |                                               |          |  
| **mongodb**           | `db_host`          | Database host. Default: `localhost`.          | Optional |  
|                       | `db_port`          | Port for connection.                          | Required |  
|                       | `db_user`          | Database user.                                | Optional |  
|                       | `db_password`      | User password.                                | Optional |  
|                       |                    |                                               |          |  
| **mongodb_docker**    | `docker_container` | Name of the Docker container with MongoDB.    | Required |  
|                       | `db_user`          | Database user.                                | Optional |  
|                       | `db_password`      | User password.                                | Optional |  
|                       |                    |                                               |          |  
| **mysql**             | `db_host`          | Database host. Default: `localhost`.          | Optional |  
|                       | `db_port`          | Port for connection.                          | Required |  
|                       | `db_name`          | Name of the database.                         | Required |  
|                       | `db_user`          | Database user.                                | Required |  
|                       | `db_password`      | User password.                                | Required |  
|                       |                    |                                               |          |  
| **mysql_docker**      | `docker_container` | Name of the Docker container with MySQL.      | Required |  
|                       | `db_name`          | Name of the database.                         | Required |  
|                       | `db_user`          | Database user.                                | Required |  
|                       | `db_password`      | User password.                                | Required |  
|                       |                    |                                               |          |  
| **folder**            | `target_path`      | Path to the directory to be backed up.        | Required |  

### Common Parameters for All Elements:

| Parameter                    | Description                                                 |
|------------------------------|-------------------------------------------------------------|
| **element_title**            | Name of the element (used in the directory and file names). |
| **s3_folder**                | Folder in S3 for storing backups.                           |
| **backup_retention_days**    | Number of days to retain local backups.                     |
| **s3_backup_retention_days** | Number of days to retain backups in S3.                     |

## Usage

To start the backup process, run the following command:

```bash
./reback backup
```

**Important**: The `backup` argument is required to start the backup process. Without it, the program will not run and
you will get an error.

### Running as a Cron Job

You can automate the backup process by creating a Cron job. For example, to run the backup every day at 2:00 AM, add the
following line to your crontab file:

```bash
0 2 * * * /path/to/reback backup >> /path/to/logs/backup.log 2>&1
```

This Cron job will:

- Run the `./reback backup` command every day at 2:00 AM.
- Redirect both standard output and error output to a log file (`backup.log`).

To edit your crontab, use the command:

```bash
crontab -e
```

Then, add the Cron job line. Make sure the path to the `reback` binary and the log file are correct.

## Author

Program author: Ivan Ashikhmin  
Telegram for contact: [https://t.me/proDreams](https://t.me/proDreams)

The program was created as part of the "Code on a Napkin" project.

- Website: [https://pressanybutton.ru/](https://pressanybutton.ru/)
- Telegram channel: [https://t.me/press_any_button](https://t.me/press_any_button)

## Support

If you like this project and want to support its further development, consider donating:

- Donation via TON: `UQBU8rJEfUcBvJUbz6NbXiWxaOO_NoXHK_pXOWv7qsOBWbFp`
- [Support on Boosty](https://boosty.to/prodream)
- [In our Telegram bot for Telegram Stars](https://t.me/press_any_button_bot?start=donate)

Your support helps the project grow and improve future features!

## License

This project is licensed under the MIT License. Details can be found in the [LICENSE](LICENSE) file.