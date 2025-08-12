use aws_config::imds::client::Client as ImdsClient;
use std::error::Error;

//use crate::domain::entities::machine_key::MachineKey;

const PATH_INSTANCE_ID: &str = "/latest/meta-data/instance-id";
const PATH_AMI_ID: &str = "/latest/meta-data/ami-id";
const PATH_LOCAL_IPV4: &str = "/latest/meta-data/local-ipv4";
const PATH_PRIMARY_MAC: &str = "/latest/meta-data/mac";
const PATH_MACS: &str = "/latest/meta-data/network/interfaces/macs/";

pub async fn get_instance_metadata() -> Result<(), Box<dyn Error>> {
    // 1) Crear el cliente IMDS (usa IMDSv2 con token de forma transparente)
    let imds = ImdsClient::builder().build();

    // 2) Lecturas directas
    let instance_id: String = get_text(&imds, PATH_INSTANCE_ID).await?;
    let ami_id: String = get_text(&imds, PATH_AMI_ID).await?;
    let local_ipv4: String = get_text(&imds, PATH_LOCAL_IPV4).await?;

    // 3a) MAC rápida (si sabes que solo hay una interfaz)
    let mac_rapida: String = get_text(&imds, PATH_PRIMARY_MAC).await?;

    // 3b) MAC correcta asociada a tu IPv4 (si puede haber varias NICs)
    let mac_por_ip: String = find_mac_for_ip(&imds, &local_ipv4).await?;

    // Ejemplo: úsalo como necesites (evita loguear sensibles en producción)
    println!("instance_id={}", instance_id);
    println!("ami_id={}", ami_id);
    println!("local_ipv4={}", local_ipv4);
    println!("mac_rapida={}", mac_rapida);
    println!("mac_por_ip={}", mac_por_ip);

    Ok(())
}

fn calculate_machine_key() -> String {
    String::from("hola por el momento")
}

async fn get_text(imds: &ImdsClient, path: &str) -> Result<String, Box<dyn Error>> {
    // El tipo devuelto implementa AsRef<str>, así que no necesitamos SensitiveString
    let v = imds.get(path).await?;
    Ok(v.as_ref().to_string())
}

// Recorre las MACs y devuelve la que “posee” la IPv4 local
async fn find_mac_for_ip(imds: &ImdsClient, ip: &str) -> Result<String, Box<dyn Error>> {
    let macs_raw = get_text(imds, PATH_MACS).await?; // líneas que terminan con "/"
    for mac_line in macs_raw.lines().filter(|l| !l.trim().is_empty()) {
        let mac = mac_line.trim_end_matches('/');

        let path = format!(
            "/latest/meta-data/network/interfaces/macs/{}/local-ipv4s",
            mac
        );
        let ips = get_text(imds, &path).await?; // puede traer varias líneas

        if ips.lines().any(|line| line.trim() == ip) {
            return Ok(mac.to_string());
        }
    }
    Err("No se encontró una MAC asociada a la IPv4 indicada".into())
}

