use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub on_new_episode: Vec<Command>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    pub command: String,
    pub args: Vec<String>,
}

impl Config {
    pub fn read(config_dir: &str) -> Result<Config, String> {
        let config_path = Path::new(config_dir).join("config.json");

        if !config_path.is_file() {
            return Err(format!("config file not found: {:?}", config_path))
        }

        let config_contents = fs::read_to_string(config_path)
            .unwrap();

        let settings = serde_json::from_str(&config_contents)
            .map_err(|e| format!("Failed to deserialize config file: {}", e))?;


        Ok(settings)
    }
}
