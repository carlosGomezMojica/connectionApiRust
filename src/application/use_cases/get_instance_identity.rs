use aws_config::imds::client::Client as ImdsClient;
use sha2::{Digest, Sha256};
use std::error::Error;

use crate::domain::entities::machine_key::MachineKey;

const PATH_INSTANCE_ID: &str = "/latest/meta-data/instance-id";
const PATH_AMI_ID: &str = "/latest/meta-data/ami-id";
const PATH_LOCAL_IPV4: &str = "/latest/meta-data/local-ipv4";
//const PATH_PRIMARY_MAC: &str = "/latest/meta-data/mac";
const PATH_MACS: &str = "/latest/meta-data/network/interfaces/macs/";

pub async fn get_instance_metadata() -> Result<MachineKey, Box<dyn Error>> {
    // 1) Crear el cliente IMDS (usa IMDSv2 con token de forma transparente)
    let imds = ImdsClient::builder().build();

    // 2) Lecturas directas
    let instance_id: String = get_text(&imds, PATH_INSTANCE_ID).await?;
    let ami_id: String = get_text(&imds, PATH_AMI_ID).await?;
    let local_ipv4: String = get_text(&imds, PATH_LOCAL_IPV4).await?;

    // 3a) MAC rápida (si sabes que solo hay una interfaz)
    //let mac_rapida: String = get_text(&imds, PATH_PRIMARY_MAC).await?;

    // 3b) MAC correcta asociada a tu IPv4 (si puede haber varias NICs)
    let mac_por_ip: String = find_mac_for_ip(&imds, &local_ipv4).await?;

    let machine_key = calculate_machine_key(&mac_por_ip, &local_ipv4, &ami_id, &instance_id);

    Ok(MachineKey {
        mac: mac_por_ip,
        private_ip: local_ipv4,
        image_id: ami_id,
        instance_id: instance_id,
        machine_key: machine_key,
    })
}

fn calculate_machine_key(
    mac: &String,
    private_ip: &String,
    image_id: &String,
    instance_id: &String,
) -> String {
    let cadena = format!("{}:{}:{}:{}", mac, private_ip, image_id, instance_id);
    let mut hasher = Sha256::new();
    hasher.update(cadena.as_bytes());
    let result = hasher.finalize();
    let machine_key = format!("{:x}", result);

    machine_key
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
