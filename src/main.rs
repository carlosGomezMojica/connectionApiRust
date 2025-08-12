mod application;
mod domain;
mod infrastructure;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let info = application::use_cases::get_instance_identity::get_instance_metadata().await?;
    println!("instance_id={}", info.instance_id);
    println!("ami_id={}", info.image_id);
    println!("local_ipv4={}", info.private_ip);
    println!("mac_rapida={}", info.mac);
    println!("mac_por_ip={}", info.machin_key);

    Ok(())
}
