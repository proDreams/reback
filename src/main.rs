use crate::structures::settings::Settings;
use crate::utils::process_backup::start_backup_process;

mod structures;
mod utils;

#[tokio::main]
async fn main() {
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

    start_backup_process(&settings, &bucket).await;
}
