use crate::manifest::ManifestParameter;
use dialoguer::Input;
use inline_colorization::{color_green, color_reset};
use mamoru_chain_client::proto::validation_chain::daemon_metadata_paremeter::DaemonParemeterType;
use std::collections::HashMap;

pub fn input_user_params(
    manifest_params: &Vec<ManifestParameter>,
    user_params: &mut HashMap<String, String>,
) {
    for param in manifest_params {
        let param_name = param.key.as_str();
        let user_input: String = Input::new()
            .with_prompt(format!(
                "Enter value for {color_green}{}{color_reset}",
                param_name
            ))
            .interact_text()
            .unwrap();

        let param_type: DaemonParemeterType =
            DaemonParemeterType::from_str_name(param.type_.as_str()).unwrap();
        println!("{:?}={}", param_type, user_input);
        user_params.insert(param_name.to_string(), user_input);
    }
}
