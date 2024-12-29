use crate::structures::settings::Settings;
use std::{fs, io};

pub fn read_json() -> io::Result<Settings> {
    let file_path = "settings.json";

    let file_content = fs::read_to_string(file_path)?;

    let json_data: Settings =
        serde_json::from_str(&file_content).expect("Cannot parse settings file");

    Ok(json_data)
}
