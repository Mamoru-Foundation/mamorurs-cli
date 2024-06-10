use crate::{
    client::message_client,
    manifest::{self, read_manifest_file, ManifestParameter},
};
use mamoru_chain_client::{
    proto::validation_chain::{
        daemon_metadata_paremeter::DaemonParemeterType, Chain, DaemonMetadataParemeter,
    },
    DaemonMetadataContent, DaemonMetadataType, DaemonParameter, RegisterDaemonMetadataRequest,
};
use std::{collections::HashMap, fs, io, path::Path, time::Duration};
use tokio::time;
use url::Url;

/// Publishes an agent to a specified chain.
///
/// This function reads a manifest file, collects user parameters, registers daemon metadata,
/// and finally registers the daemon itself. It uses the `message_client` to communicate with the chain.
pub async fn publish_agent(
    grpc: &Url,
    prkey: String,
    chain_name: String,
    dir_path: &Path,
    gas_limit: u64,
) -> Result<String, Box<dyn std::error::Error>> {
    let manifest = read_manifest_file(dir_path).expect("Manifest file not found");

    let mut user_params: HashMap<String, String> = HashMap::new();
    if let Some(manifest_params) = &manifest.parameters {
        for param in manifest_params {
            let param_name = param.key.as_str();
            let user_input = get_input(param_name);
            let param_type: DaemonParemeterType =
                DaemonParemeterType::from_str_name(param.type_.as_str()).unwrap();
            println!("{:?}={}", param_type, user_input);
            user_params.insert(param_name.to_string(), user_input);
        }
        println!("user_params: {:?}", user_params);
    }

    let message_client = message_client(prkey, grpc, gas_limit).await;
    let module_content = read_wasm_file(dir_path)?;
    let request = build_daemon_metadata_request(&manifest, &module_content);
    let dm_response = message_client.register_daemon_metadata(request).await;

    time::sleep(Duration::from_millis(1000)).await;

    let daemon_metadata_id = dm_response.unwrap().daemon_metadata_id;
    println!("DaemonMetadataId: {:?}", daemon_metadata_id);
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
            println!("Error registering daemon: {:?}", e);
            None
        }
    };

    let daemon_id = daemon.unwrap().daemon_id;
    println!("DaemonId: {:?}", daemon_id);
    println!("Agent successfully registered");

    Ok(daemon_id)
}

fn read_wasm_file(dir_path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let wasm_dir_path = dir_path.join("target/wasm32-unknown-unknown/release");
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
            panic!("Multiple Wasm files found in {}", wasm_dir_path.display());
        }
    }
}

fn build_daemon_metadata_request(
    manifest: &manifest::Manifest,
    wasm_content: &[u8],
) -> RegisterDaemonMetadataRequest {
    let mut parameters: Vec<DaemonMetadataParemeter> = vec![];
    if let Some(manifest_params) = &manifest.parameters {
        for parameter in manifest_params {
            parameters.push(DaemonMetadataParemeter {
                r#type: DaemonParemeterType::from_str_name(parameter.type_.as_str())
                    .unwrap()
                    .into(),
                title: parameter.title.clone(),
                key: parameter.key.clone(),
                description: parameter.description.clone(),
                default_value: parameter.default_value.clone(),
                required_for: parameter
                    .required_for
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| Chain { name: s })
                    .collect(),
                hidden_for: parameter
                    .hidden_for
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| Chain { name: s })
                    .collect(),
                symbol: parameter.symbol.clone().unwrap_or_default(),
                min: parameter.min.clone().unwrap_or_default(),
                max: parameter.max.clone().unwrap_or_default(),
                min_len: parameter.min_len.unwrap_or_default(),
                max_len: parameter.max_len.unwrap_or_default(),
            });
        }
    }

    RegisterDaemonMetadataRequest {
        kind: match manifest.subscribable {
            true => DaemonMetadataType::Subcribable,
            false => DaemonMetadataType::Sole,
        },
        logo_url: manifest.logo_url.to_string(),
        title: manifest.name.to_string(),
        description: manifest.description.to_string(),
        tags: manifest.tags.clone(),
        supported_chains: manifest.supported_chains.clone(),
        parameters,
        versions: manifest.version.clone(),
        content: DaemonMetadataContent::Wasm {
            module: wasm_content.to_owned(),
        },
    }
}

fn build_daemon_parameters(
    manifest_parameters: Option<Vec<ManifestParameter>>,
    user_params: HashMap<String, String>,
    chain_name: String,
) -> Vec<DaemonParameter> {
    let mut parameters: Vec<DaemonParameter> = vec![];
    match manifest_parameters {
        None => parameters,
        Some(manifest_parameters) => {
            manifest_parameters
                .into_iter()
                .filter(|x| {
                    if let Some(hidden_for) = &x.hidden_for {
                        return !hidden_for.contains(&chain_name);
                    }
                    true
                })
                .for_each(|x| {
                    parameters.push(DaemonParameter {
                        key: x.key,
                        value: x.default_value,
                    });
                });

            for (key, value) in user_params {
                if let Some(parameter) = parameters.iter_mut().find(|x| x.key == key) {
                    parameter.value = value;
                }
            }

            parameters
        }
    }
}

pub fn get_input(prompt: &str) -> String {
    println!("{}: ", prompt);
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {}
        Err(_no_updates_is_fine) => {}
    }
    input.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::build_daemon_parameters;
    use crate::manifest::ManifestParameter;
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
