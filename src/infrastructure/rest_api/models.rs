use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ResponseApiKey {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "syntheticIndex")]
    pub synthetic_index: String,
}
