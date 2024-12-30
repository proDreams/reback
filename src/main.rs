use crate::structures::settings::Settings;
use crate::utils::process_backup::start_backup_process;
use std::env;

mod structures;
mod utils;

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
