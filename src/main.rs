use crate::structures::settings::Settings;

mod structures;

#[tokio::main]
async fn main() {
    let settings = match Settings::from_file() {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Failed to initialize settings: {}", err);
            return;
        }
    };

    println!("{:?}", settings);
}
