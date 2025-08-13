use rand::Rng;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::entities::api_key::ApiKey;
use crate::infrastructure::rest_api::mappers::map_api_key;
use crate::infrastructure::rest_api::models::ResponseApiKey;

#[derive(Clone)]
pub struct RestApiClient {
    http: Client,
    base_url: String,
}

impl RestApiClient {
    pub fn new_out_api_key(base_url: String) -> Self {
        Self {
            http: Client::new(),
            base_url,
        }
    }

    fn generate_timestamp_and_nounce() -> (String, String) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        let nounce = format!("{:06}", rand::rng().random_range(0..1_000_000));

        (timestamp, nounce)
    }

    fn calculate_hash(
        timestamp: &str,
        nounce: &str,
        jwt: Option<&str>,
        body: Option<&serde_json::Value>,
        url: &str,
    ) -> String {
        // 🔒 Construimos manualmente el string JSON, asegurando orden exacto
        let mut json_string = format!(
            "{{\"url\":\"{}\",\"timestamp\":\"{}\",\"nounce\":\"{}\"", // ,\"body\":\"{}\"}}   body_str
            url, timestamp, nounce
        );

        if let Some(jwt_val) = jwt {
            json_string.push_str(&format!(",\"jwt\":\"{}\"", jwt_val));
        }

        if let Some(body_val) = body {
            let body_str = body_val.to_string();
            json_string.push_str(&format!(",\"body\":{}", body_str));
        }
        json_string.push('}');

        println!("[API_]    📦 Payload para SHA256:\n{}", json_string);

        let mut hasher = Sha256::new();
        hasher.update(json_string.as_bytes());
        let result = hasher.finalize();

        hex::encode(result)
    }

    pub async fn inscribe(&self, machine_key: &str) -> Result<ApiKey, String> {
        let url_path = "/price-engine/inscribe";
        let url = format!("{}{}", self.base_url, url_path);

        let (timestamp, nounce) = Self::generate_timestamp_and_nounce();

        let body = serde_json::json!({ "machineKey": machine_key });
        let hash = Self::calculate_hash(&timestamp, &nounce, None, Some(&body), url_path);

        let response = self
            .http
            .post(&url)
            .header("timestamp", &timestamp)
            .header("nounce", &nounce)
            .header("hash", &hash)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();

        if status.is_success() {
            // 1) Deserializa DTO
            let dto: ResponseApiKey = response
                .json()
                .await
                .map_err(|e| format!("Invalid JSON: {}", e))?;

            // 2) Mapea a dominio
            let api_key = map_api_key(dto).map_err(|e| format!("Mapping error: {}", e))?;

            // 3) Retorna entidad
            Ok(api_key)
        } else {
            let error_body = response.text().await.unwrap_or_default();
            Err(format!(
                "Auth failed with status {}: {}",
                status, error_body
            ))
        }
    }
}
