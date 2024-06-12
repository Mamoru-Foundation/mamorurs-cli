use crate::client::message_client;
use crate::daemon_builder::build_daemon_parameters;
use crate::{input::input_user_params, manifest::read_manifest_file};
use inline_colorization::{color_green, color_reset};
use std::{collections::HashMap, path::Path};
use url::Url;

pub async fn launch_agent(
    metadata_id: String,
    grpc: String,
    prkey: String,
    chain_name: String,
    dir_path: &Path,
    gas_limit: u64,
) -> Result<String, Box<dyn std::error::Error>> {
    let manifest = read_manifest_file(dir_path).expect("Manifest file not found");
    let message_client = message_client(prkey, &grpc.parse::<Url>().unwrap(), gas_limit).await;
    let mut user_params: HashMap<String, String> = HashMap::new();
    if let Some(manifest_params) = &manifest.parameters {
        input_user_params(manifest_params, &mut user_params);
    }
    let daemon_parameters =
        build_daemon_parameters(manifest.parameters, user_params, chain_name.clone());

    let daemon = match message_client
        .register_daemon(metadata_id, chain_name.clone(), daemon_parameters, None)
        .await
    {
        Ok(daemon) => Some(daemon),
        Err(e) => {
            println!("Error registering daemon: {:?}", e);
            None
        }
    };

    let daemon_id = daemon.unwrap().daemon_id;
    println!("DaemonId: {color_green}{}{color_reset}", daemon_id);
    println!("Agent successfully registered");

    Ok(daemon_id)
}
