use crate::{
    client::message_client,
    daemon_builder::{
        build_daemon_metadata_request, build_daemon_parameters, check_supported_chains,
    },
    input::{input_user_params, select_user_input},
    manifest::read_manifest_file,
};
use inline_colorization::{color_green, color_reset};
use spinners::{Spinner, Spinners};
use std::{collections::HashMap, fs, path::Path, time::Duration};
use tokio::time;
use url::Url;

/// Publishes an agent to a specified chain.
///
/// This function reads a manifest file, collects user parameters, registers daemon metadata,
/// and finally registers the daemon itself. It uses the `message_client` to communicate with the chain.
pub async fn publish_agent(
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

    let mut user_params: HashMap<String, String> = HashMap::new();
    if let Some(manifest_params) = &manifest.parameters {
        input_user_params(manifest_params, &mut user_params);
    }
    
    println!("Publishing agent...");

    let message_client =
        message_client(prkey, &grpc.parse::<Url>().unwrap(), gas_limit, chain_id).await;
    let module_content = read_wasm_file(dir_path)?;
    let request = build_daemon_metadata_request(&manifest, &module_content);

    let mut sp = Spinner::new(Spinners::Triangle, "Publishing metadata".into());

    let dm_response = match message_client.register_daemon_metadata(request).await {
        Ok(response) => response,
        Err(e) => {
            sp.stop();
            println!("Error registering daemon metadata: {:?}", e);
            return Err(Box::new(e));
        }
    };

    time::sleep(Duration::from_millis(1000)).await;

    let daemon_metadata_id = dm_response.daemon_metadata_id;
    println!(
        "DaemonMetadataId: {color_green}{}{color_reset}",
        daemon_metadata_id
    );

    println!("DaemonMetadata successfully registered");

    let daemon_parameters =
        build_daemon_parameters(manifest.parameters, user_params, chain_name.clone());
    let relay = None;
    let daemon = match message_client
        .register_daemon(
            daemon_metadata_id,
            chain_name.clone(),
            daemon_parameters,
            relay,
        )
        .await
    {
        Ok(daemon) => Some(daemon),
        Err(e) => {
            sp.stop();
            println!("Error registering daemon: {:?}", e);
            None
        }
    };

    time::sleep(Duration::from_millis(1000)).await;
    sp.stop_with_message("Completed".into());
    let daemon_id = daemon.unwrap().daemon_id;
    println!("DaemonId: {color_green}{}{color_reset}", daemon_id);
    println!("Agent successfully registered");

    Ok(daemon_id)
}

fn read_wasm_file(dir_path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let wasm_dir_path = dir_path.join("target/wasm32-wasi/release/");
    let wasm_files = fs::read_dir(wasm_dir_path.clone())?
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "wasm"))
        .collect::<Vec<_>>();

    match wasm_files.len() {
        0 => Err("no Wasm file found".into()),
        1 => {
            let wasm_file = wasm_files.first().expect("Wasm file not found");
            let wasm_file_path = wasm_file.canonicalize()?;
            println!("wasm_file: {:?}", wasm_file_path);
            Ok(std::fs::read(wasm_file_path)?)
        }
        _ => {
            let items = wasm_files
                .iter()
                .map(|path| path.file_name().unwrap().to_str().unwrap().to_string())
                .collect();
            let index = select_user_input(items);
            let wasm_file = wasm_files.get(index).expect("Wasm file not found");
            let wasm_file_path = wasm_file.canonicalize()?;
            Ok(std::fs::read(wasm_file_path)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{daemon_builder::build_daemon_parameters, manifest::ManifestParameter};
    use std::collections::HashMap;

    #[test]
    fn test_build_daemon_parameters() {
        let manifest_params = Some(vec![
            ManifestParameter {
                key: "param1".to_string(),
                type_: "type".to_string(),
                title: "title".to_string(),
                description: "description".to_string(),
                default_value: "param1_default_value".to_string(),
                required_for: None,
                hidden_for: vec!["Sui".to_string()].into(),
                symbol: None,
                min: None,
                max: None,
                min_len: None,
                max_len: None,
            },
            ManifestParameter {
                key: "param2".to_string(),
                type_: "type".to_string(),
                title: "title".to_string(),
                description: "description".to_string(),
                default_value: "param2_default_value".to_string(),
                required_for: None,
                hidden_for: None,
                symbol: None,
                min: None,
                max: None,
                min_len: None,
                max_len: None,
            },
        ]);
        let user_params = HashMap::from([("param2".to_string(), "user_param2_value".to_string())]);
        let daemon_params =
            build_daemon_parameters(manifest_params, user_params, "Sui".to_string());

        assert_eq!(daemon_params.len(), 1);
        assert_eq!(daemon_params[0].key, "param2");
        assert_eq!(daemon_params[0].value, "user_param2_value");
    }
}
