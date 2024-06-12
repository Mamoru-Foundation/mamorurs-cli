use crate::auth::TokenResponse;
use crate::{config::Config, CommandContext};
use inline_colorization::{color_green, color_reset, color_yellow};

use cred_store::CredStore;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::time::{Duration, Instant};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: usize,
    interval: usize,
}

pub async fn login(config: &Config) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    let client = Client::new();
    debug!("requesting device code {}", config.mamoru_cli_auth0_domain);
    let response = client
        .post(&format!(
            "{}/oauth/device/code",
            config.mamoru_cli_auth0_domain
        ))
        .form(&[
            ("client_id", config.mamoru_cli_auth0_client_id.as_str()),
            ("audience", config.mamoru_cli_auth0_audience.as_str()),
            ("scope", "openid profile email offline_access"),
        ])
        .send()
        .await?;

    let device_auth_response: DeviceAuthResponse = match response.json::<DeviceAuthResponse>().await
    {
        Ok(resp) => resp,
        Err(e) => return Err(e.into()),
    };

    println!(
        "Go to {color_green}{}{color_reset} and enter the code: {color_yellow}{}{color_reset}",
        device_auth_response.verification_uri, device_auth_response.user_code
    );

    let mut sp = Spinner::new(Spinners::Triangle, "Polling for token".into());

    _ = open::that(device_auth_response.verification_uri_complete);

    let token_endpoint = format!("https://{}/oauth/token", config.mamoru_cli_auth0_domain);

    let start_instant = Instant::now();
    let expiry_duration = Duration::from_secs(device_auth_response.expires_in as u64);

    loop {
        if Instant::now() >= start_instant + expiry_duration {
            sp.stop();
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Device code has expired",
            )));
        }

        let resp_result = client
            .post(&token_endpoint)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &device_auth_response.device_code),
                ("client_id", config.mamoru_cli_auth0_client_id.as_str()),
            ])
            .send()
            .await
            .unwrap();
        let resp_json = resp_result.json::<TokenResponse>().await;

        match resp_json {
            Ok(resp) => {
                if resp.access_token.is_some() {
                    sp.stop();
                    return Ok(resp);
                }
            }
            Err(e) => {
                sp.stop();
                return Err(Box::new(e));
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(
            device_auth_response.interval as u64,
        ));
    }
}

pub fn save_tokens<T: CredStore>(
    access_token: &str,
    refresh_token: &str,
    context: &mut CommandContext<T>,
) -> Result<(), std::io::Error> {
    context
        .cred_store
        .clear()
        .add("access_token".to_string(), access_token.to_string())
        .add("refresh_token".to_string(), refresh_token.to_string())
        .save()
}
