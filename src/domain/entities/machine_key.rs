#[derive(Debug, Clone)]
pub struct MachineKey {
    pub mac: String,
    pub private_ip: String,
    pub image_id: String,
    pub instance_id: String,
    pub machine_key: String,
}
