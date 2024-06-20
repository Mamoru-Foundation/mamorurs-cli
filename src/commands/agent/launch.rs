use crate::client::message_client;
use crate::daemon_builder::{build_daemon_parameters, check_supported_chains};
use crate::{input::input_user_params, manifest::read_manifest_file};
use inline_colorization::{color_green, color_reset};
use spinners::{Spinner, Spinners};
use std::{collections::HashMap, path::Path};
use url::Url;

pub async fn launch_agent(
    metadata_id: String,
    grpc: String,
    prkey: String,
    chain_name: String,
    dir_path: &Path,
    gas_limit: u64,
    chain_id: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let manifest = read_manifest_file(dir_path).expect("Manifest file not found");

    if !check_supported_chains(&manifest.supported_chains, &chain_name) {
        eprintln!(
            "Unsupported chain, please use one of the following: {:?}",
            manifest.supported_chains
        );
        std::process::exit(1);
    }

    let message_client =
        message_client(prkey, &grpc.parse::<Url>().unwrap(), gas_limit, chain_id).await;
    let mut user_params: HashMap<String, String> = HashMap::new();
    if let Some(manifest_params) = &manifest.parameters {
        input_user_params(manifest_params, &mut user_params);
    }
    let daemon_parameters =
        build_daemon_parameters(manifest.parameters, user_params, chain_name.clone());

    let mut sp = Spinner::new(Spinners::Triangle, "Publishing agent".into());

    let daemon = match message_client
        .register_daemon(metadata_id, chain_name.clone(), daemon_parameters, None)
        .await
    {
        Ok(daemon) => Some(daemon),
        Err(e) => {
            sp.stop();
            println!("Error registering agent: {:?}", e);
            None
        }
    };

    sp.stop();

    let daemon_id = daemon.unwrap().daemon_id;

    println!();
    println!("AgentId: {color_green}{}{color_reset}", daemon_id);
    println!("Agent successfully registered");

    Ok(daemon_id)
}
