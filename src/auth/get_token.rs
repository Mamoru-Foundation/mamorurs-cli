use crate::CommandContext;
use base64::Engine;
use cred_store::CredStore;
use serde::Deserialize;
use tracing::info;

use super::TokenResponse;

#[derive(Debug, Deserialize)]
struct Claims {
    exp: i64,
}

fn decode_claims_without_verification(token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = token.split('.').collect();

    if parts.len() != 3 {
        return Err("Token format is incorrect".into());
    }

    let payload = parts[1];
    let decoded_payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload)?;
    let claims: Claims = serde_json::from_slice(&decoded_payload)?;

    Ok(claims)
}

fn is_token_expired(token: &str) -> bool {
    let claims = match decode_claims_without_verification(token) {
        Ok(claims) => claims,
        Err(_) => return true,
    };

    let now = chrono::Utc::now().timestamp();

    claims.exp < now
}

pub async fn refresh_access_token(
    domain: &str,
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    let token_endpoint = format!("{}/oauth/token", domain);

    let response = reqwest::Client::new()
        .post(token_endpoint)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await
        .expect("send");

    match response.json::<TokenResponse>().await {
        Ok(response) => Ok(response),
        Err(e) => {
            println!("Error refreshing access token: {:?}", e);
            Err(Box::new(e))
        }
    }
}

pub async fn get_token<T: CredStore>(
    context: &mut CommandContext<'_, T>,
) -> Result<Option<String>, Box<dyn std::error::Error>>
where
    T: CredStore,
{
    let mut credentials = context.cred_store.load()?;
    let access_token = credentials.get("access_token").cloned();
    let refresh_token = credentials.get("refresh_token").cloned();

    let token = match (access_token, refresh_token) {
        (Some(at), Some(rt)) => {
            if is_token_expired(&at) {
                println!("Access token expired. Refreshing...");
                info!("Access token expired. Refreshing...");
                let token_response = refresh_access_token(
                    &context.config.mamoru_cli_auth0_domain,
                    &context.config.mamoru_cli_auth0_client_id,
                    &rt,
                )
                .await?;
                let new_access_token = match token_response.access_token {
                    Some(at) => at,
                    None => {
                        println!("Couldn't refresh access token.");
                        return Err("Couldn't refresh access token.".into());
                    }
                };
                let new_refresh_token = match token_response.refresh_token {
                    Some(rt) => rt,
                    None => "".to_string(),
                };

                info!("Access token refreshed.");
                credentials
                    .add("access_token".to_string(), new_access_token.clone())
                    .add("refresh_token".to_string(), new_refresh_token);

                credentials.save()?;

                Ok(Some(new_access_token))
            } else {
                Ok(Some(at))
            }
        }
        _ => Ok(None),
    };

    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_claims_without_verification() {
        let test_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ0ZW5hbnRfaWQiOiIxIiwidXNlcl9pZCI6IjEiLCJleHAiOjE2OTcxMTg2Nzh9.CYF2GjJ5T1xJSUM5T1gl9iFftufT8xe8cclGoU8kw_I";
        let claims = decode_claims_without_verification(test_token).unwrap();
        assert_eq!(claims.exp, 1697118678);
    }
}
