use std::collections::HashMap;

use mamoru_chain_client::{
    proto::validation_chain::{
        daemon_metadata_paremeter::DaemonParemeterType, Chain, DaemonMetadataParemeter,
    },
    DaemonMetadataContent, DaemonMetadataType, DaemonParameter, RegisterDaemonMetadataRequest,
};

use crate::manifest::{self, ManifestParameter};

pub fn build_daemon_metadata_request(
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

pub fn build_daemon_parameters(
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

pub fn check_supported_chains(supported_chains: &[String], chain_name: &String) -> bool {
    if !supported_chains.contains(chain_name) {
        return false;
    }

    true
}

#[cfg(test)]
mod test {
    use crate::manifest::ManifestParameter;
    use std::collections::HashMap;

    #[test]
    fn test_build_daemon_metadata_request() {
        let manifest = crate::manifest::Manifest {
            name: "test".to_string(),
            description: "test".to_string(),
            parameters: Some(vec![ManifestParameter {
                type_: "NUMBER".to_string(),
                title: "test".to_string(),
                key: "test".to_string(),
                description: "test".to_string(),
                default_value: "test".to_string(),
                required_for: None,
                hidden_for: None,
                symbol: None,
                min: None,
                max: None,
                min_len: None,
                max_len: None,
            }]),
            supported_chains: vec!["TEST_CHAIN".to_string()],
            tags: vec![],
            subscribable: true,
            logo_url: "https://mamoru.ai/default-agent-logo.png".to_string(),
            version: HashMap::new(),
        };

        let wasm_content = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        let request =
            crate::daemon_builder::build_daemon_metadata_request(&manifest, &wasm_content);

        assert_eq!(request.title, "test");
        assert_eq!(request.logo_url, "https://mamoru.ai/default-agent-logo.png");

        assert_eq!(
            request.kind,
            crate::daemon_builder::DaemonMetadataType::Subcribable
        );

        assert_eq!(request.supported_chains.len(), 1);
        assert_eq!(request.tags.len(), 0);
        assert_eq!(request.versions.len(), 0);
        assert_eq!(request.parameters.len(), 1);

        assert_eq!(request.parameters[0].key, "test");
        assert_eq!(request.parameters[0].title, "test");
    }
}
