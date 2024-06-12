use crate::manifest::ManifestParameter;
use dialoguer::Input;
use inline_colorization::{color_reset, color_yellow};
use std::collections::HashMap;

pub fn input_user_params(
    manifest_params: &Vec<ManifestParameter>,
    user_params: &mut HashMap<String, String>,
) {
    for param in manifest_params {
        let param_name = param.key.as_str();
        let user_input: String = Input::new()
            .with_prompt(format!(
                "Enter value for {color_yellow}{}{color_reset}",
                param_name
            ))
            .default(param.default_value.as_str().into())
            .interact_text()
            .unwrap();
        user_params.insert(param_name.to_string(), user_input);
    }
}
