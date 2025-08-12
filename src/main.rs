mod application;
mod domain;
mod infrastructure;

#[tokio::main]
async fn main() {
    application::use_cases::get_instance_identity::get_instance_metadata();
}
