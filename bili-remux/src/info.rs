use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug)]
pub struct Sources {
    pub video: PathBuf,
    pub audio: PathBuf,
    pub output: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct VideoInfo {
    #[serde(rename = "p")]
    pub fragment: u8,
    #[serde(rename = "tabName")]
    pub tab_name: String,
}

impl VideoInfo {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes).map_err(|e| e.into())
    }
}
