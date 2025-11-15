use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub redis_url: String,
    pub ws_port: u16,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // TODO: Implement configuration loading from environment variables
        todo!("Load configuration from environment")
    }
}
