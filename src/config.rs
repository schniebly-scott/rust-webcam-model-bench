use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::cv::InfType;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub model: ModelConfig,
    pub camera: CameraConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub model_path: String,
    pub inference_type: InfType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraConfig {
    pub width: u32,
    pub height: u32,
    pub device: String,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let toml = toml::to_string_pretty(self)?;
        fs::write(path, toml)?;
        Ok(())
    }
}