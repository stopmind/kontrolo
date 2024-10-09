use std::error::Error;
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub address: String
}

impl Config {
    pub fn read(path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str::<Config>(content.as_str())?)
    }
}