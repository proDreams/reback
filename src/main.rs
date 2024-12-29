use crate::structures::settings::Settings;

mod structures;
mod utils;

#[tokio::main]
async fn main() {
    let settings = match Settings::new() {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Failed to initialize settings: {}", err);
            return;
        }
    };
}
