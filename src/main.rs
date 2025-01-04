use crate::structures::settings::Settings;
use crate::utils::process_backup::start_backup_process;
use crate::utils::process_restore::{restore_all_process, restore_selected_process};
use log::{error, LevelFilter};
use std::env;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

mod structures;
mod utils;

/// The entry point of the application.
///
/// This function serves as the main execution point for the program. It initializes logging, reads command-line
/// arguments, loads settings from a configuration file, creates an S3 bucket instance, and performs the operation
/// based on the provided argument.
///
/// # Arguments
/// The function expects at least one command-line argument in addition to the program name:
/// - `"backup"`: Starts the backup process using the provided settings and S3 bucket configuration.
/// - `"restore"`: Initiates the restore process. If no additional arguments are provided, it restores all backups.
///   If a backup file is specified, it restores the selected backup.
///
/// # Behavior
/// - Initializes logging with `env_logger::init()`.
/// - Reads and validates command-line arguments.
/// - Loads settings from a configuration file using `Settings::from_file()`.
/// - Creates an S3 bucket instance using `Settings::get_bucket()`.
/// - Based on the command-line argument, either initiates the backup process or restores the data from the S3 bucket.
///
/// # Returns
/// This function does not return any value. It exits after performing the specified operation or logging an error.
///
/// # Errors
/// This function handles and logs the following errors:
/// - No command-line argument is provided.
/// - Failure to read or parse the settings file.
/// - Failure to create the S3 bucket instance.
/// - An unknown command-line argument is provided.
///
/// # Execution Flow
/// The `main` function follows this sequence:
/// 1. Initializes logging with `env_logger::init()`.
/// 2. Reads and validates the command-line arguments.
/// 3. Loads settings from a configuration file using `Settings::from_file()`.
/// 4. Creates an S3 bucket instance using `Settings::get_bucket()`.
/// 5. Executes the corresponding action based on the command-line argument (`"backup"` or `"restore"`).
/// 6. Logs errors and exits if any issues occur during initialization or execution.
///
/// # Example
/// To start the backup process, run the program with the `"backup"` argument:
/// ```sh
/// cargo run backup
/// ```
/// To restore backups, use the `"restore"` argument:
/// ```sh
/// cargo run restore
/// ```
///
/// # Notes
/// This function uses the `#[tokio::main]` macro to enable asynchronous execution, allowing for
/// tasks such as backup or restore operations to be run asynchronously.
#[tokio::main]
async fn main() {
    let exe_path = env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();

    let log_dir = exe_dir.join("reback_logs");
    let log_file = log_dir.join("reback.log");

    let size_trigger = SizeTrigger::new(10 * 1024 * 1024);

    let roller = FixedWindowRoller::builder()
        .build(&format!("{}/reback.{{}}.log", log_dir.to_str().unwrap()), 5)
        .unwrap();

    let policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(roller));

    let file_appender = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} [{l}] {m}{n}")))
        .build(log_file, Box::new(policy))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .build(
            Root::builder()
                .appender("file")
                .build(LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        error!("No command argument provided. Exiting.");
        return;
    }

    let settings = match Settings::from_file() {
        Ok(s) => s,
        Err(err) => {
            error!("Failed to initialize settings: {}", err);
            return;
        }
    };

    let bucket = match settings.get_bucket() {
        Some(bucket) => bucket,
        None => {
            error!("Failed to create bucket.");
            return;
        }
    };

    match args[1].as_str() {
        "backup" => {
            start_backup_process(&settings, &bucket).await;
        }
        "restore" => {
            if args.len() > 2 {
                restore_selected_process(&settings, &bucket, &args).await
            } else {
                restore_all_process(&settings, &bucket).await;
            }
        }
        _ => {
            error!("Unknown argument provided. Exiting.");
            return;
        }
    }
}
