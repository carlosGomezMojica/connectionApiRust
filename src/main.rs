mod application;
mod domain;
mod infrastructure;

use crate::infrastructure::rest_api::client::RestApiClient;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env_exist = application::use_cases::uses_env::validar_env();

    if env_exist {
        println!("Se encontraron variables de entorno");
        return Ok(()); // opcional: salir si ya tienes env
    }

    // Si no existe .env: obtenemos metadatos (usa `?` porque main retorna Result)
    let info = application::use_cases::get_instance_identity::get_instance_metadata().await?;
    println!("instance_id={}", info.instance_id);
    println!("ami_id={}", info.image_id);
    println!("local_ipv4={}", info.private_ip);
    println!("mac_rapida={}", info.mac);
    println!("machin_key={}", info.machine_key); // OJO: revisa si es `machin_key` o `machine_key

    let client = RestApiClient::new_out_api_key(
        "https://v6dw91scv2.execute-api.us-east-1.amazonaws.com".to_string(),
    );

    let api_key = match client.inscribe(&info.machine_key).await {
        Ok(api_key) => api_key,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(e.into());
        }
    };

    println!("ApiKey recibida: {:?}", api_key);
    println!("Creacion de variables de entorno...");
    application::use_cases::uses_env::upsert_env_api_keys(
        &api_key.api_key,
        &api_key.synthetic_index,
    )
    .unwrap();
    Ok(())
}
