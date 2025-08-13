use crate::infrastructure::rest_api::models::*;

use crate::domain::entities::api_key::ApiKey;

pub fn map_api_key(response: ResponseApiKey) -> Result<ApiKey, String> {
    // Validación opcional: asegurarte de que no vengan vacíos
    if response.api_key.trim().is_empty() {
        return Err("api_key está vacío".into());
    }
    if response.synthetic_index.trim().is_empty() {
        return Err("synthetic_index está vacío".into());
    }

    Ok(ApiKey {
        api_key: response.api_key,
        synthetic_index: response.synthetic_index,
    })
}
