use crate::structures::settings::Settings;
use crate::utils::process_backup::start_backup_process;
use std::env;

mod structures;
mod utils;

/// The entry point of the application.
///
/// This function initializes the settings, validates the command-line arguments,
/// and performs the appropriate operation based on the provided argument.
///
/// # Arguments
/// The function expects at least one command-line argument in addition to the program name:
/// - `"backup"`: Starts the backup process using the provided settings and S3 bucket configuration.
///
/// # Behavior
/// - Reads settings from a configuration file using `Settings::from_file()`.
/// - Creates an S3 bucket instance using `Settings::get_bucket()`.
/// - Validates the command-line arguments and executes the corresponding action.
/// - Logs errors to `stderr` if initialization or validation fails.
///
/// # Example
/// Run the program with the `"backup"` argument to start the backup process:
/// ```sh
/// cargo run -- backup
/// ```
///
/// # Errors
/// - If no command-line argument is provided, the function logs an error and exits.
/// - If the settings file cannot be read or parsed, the function logs an error and exits.
/// - If the S3 bucket initialization fails, the function logs an error and exits.
/// - If an unknown argument is provided, the function logs an error and exits.
///
/// This function uses the `#[tokio::main]` macro to enable asynchronous execution.
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("No command argument provided. Exiting.");
        return;
    }

    let settings = match Settings::from_file() {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Failed to initialize settings: {}", err);
            return;
        }
    };

    let bucket = match settings.get_bucket() {
        Some(bucket) => bucket,
        None => {
            eprintln!("Failed to create bucket.");
            return;
        }
    };

    match args[1].as_str() {
        "backup" => {
            start_backup_process(&settings, &bucket).await;
        }
        _ => {
            eprintln!("Unknown argument provided. Exiting.");
            return;
        }
    }
}
