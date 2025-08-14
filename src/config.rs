use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub timescale: TimescaleConfig,
}

#[derive(Debug, Deserialize)]
pub struct MqttConfig {
    pub hostname: String,
    pub port: u16,
    pub client_id: String,
    pub user: String,
    pub password: String,
    pub schema_topic: String,
    pub data_topic: String,
}

#[derive(Debug, Deserialize)]
pub struct TimescaleConfig {
    pub hostname: String,
    pub port: usize,
    pub db_name: String,
    pub user: String,
    pub password: String,
}

impl Config {
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let cfg: Self = toml::from_str(&content)?;
        Ok(cfg)
    }
}
