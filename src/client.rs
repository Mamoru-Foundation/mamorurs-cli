use base64::{prelude::BASE64_STANDARD, Engine};
use config::Config;
use cosmrs::crypto::secp256k1;
use mamoru_chain_client::{
    AccountConfig, ChainConfig, ConnectionConfig, MessageClient, MessageClientConfig, QueryClient,
    QueryClientConfig, SendMode, DEFAULT_MAX_RECV_MESSAGE_SIZE,
};

use serde_json::json;
use url::Url;

use crate::errors::ResponseData;

#[allow(dead_code)]
pub async fn query_client(grpc_url: Url) -> QueryClient {
    QueryClient::connect(query_client_config(grpc_url))
        .await
        .expect("QueryClient::connect error.")
}

pub async fn message_client(
    prkey: String,
    grpc_url: &Url,
    gas_limit: u64,
    chain_id: String,
) -> MessageClient {
    MessageClient::connect(message_client_config(prkey, grpc_url, gas_limit, chain_id).await)
        .await
        .expect("MessageClient::connect error.")
}

#[allow(dead_code)]
pub fn query_client_config(grpc_url: Url) -> QueryClientConfig {
    QueryClientConfig {
        connection: ConnectionConfig {
            endpoint: grpc_url,
            max_decoding_message_size: DEFAULT_MAX_RECV_MESSAGE_SIZE,
        },
        sdk_versions: Some(mamoru_chain_client::VersionConfig {
            versions: Vec::new(),
        }),
    }
}

pub async fn message_client_config(
    key: String,
    grpc_url: &Url,
    gas_limit: u64,
    chain_id: String,
) -> MessageClientConfig {
    let private_key: cosmrs::crypto::secp256k1::SigningKey = string_to_signing_key(key.as_str());
    let builder = Config::builder()
        .set_default("max_decoding_message_size", (20 * 1024 * 1024).to_string())
        .unwrap()
        .add_source(
            config::Environment::with_prefix("MAMORU")
                .try_parsing(true)
                .separator("_"),
        );
    match builder.build() {
        Ok(config) => {
            let connect_config = ConnectionConfig {
                endpoint: grpc_url.to_owned(),
                max_decoding_message_size: config
                    .get::<usize>("max_decoding_message_size")
                    .unwrap(),
            };
            MessageClientConfig {
                connection: connect_config,
                chain: ChainConfig {
                    tx_gas_limit: gas_limit,
                    chain_id,
                    ..Default::default()
                },
                account: AccountConfig::new(private_key),
                send_mode: SendMode::Block,
            }
        }
        Err(e) => panic!("Error: {}", e),
    }
}

fn string_to_signing_key(private_key_str: &str) -> secp256k1::SigningKey {
    let secret_key_bytes = BASE64_STANDARD
        .decode(private_key_str)
        .expect("Can not parse private key base64");
    secp256k1::SigningKey::from_slice(&secret_key_bytes).expect("Can not parse private key bytes")
}

pub async fn register_daemon_to_organization(
    graphql_url: &str,
    token: &str,
    daemon_id: &str,
    organization_id: &str,
) -> std::result::Result<(), reqwest::Error> {
    ping_graphql(graphql_url, token).await?;

    println!("Registering agent to the organization...");

    let client = reqwest::Client::new();
    // GraphQL mutation query
    let query = r#"
    mutation assignDaemonOrganizationId($daemonId: String!, $organizationId: String!) {
        assignDaemonOrganizationId(daemonId: $daemonId, organizationId: $organizationId) {
            __typename
        }
    }
    "#;

    // Create the JSON body for the request
    let body = json!({
        "query": query,
        "variables": { "daemonId": daemon_id, "organizationId": organization_id }
    });

    loop {
        match client
            .post(graphql_url)
            .json(&body)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                let response_data: ResponseData = response.json::<ResponseData>().await?;
                if let Some(errors) = response_data.errors {
                    if !errors.is_empty() {
                        println!("Error register agent to the organization: {:?}", errors);
                        println!("Retrying...");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                }
                if status.is_success() {
                    println!("Agent successfully registered to the organization.");
                    return Ok(());
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
                println!("Error register agent to the organization. Retrying...");
            }
            Err(e) => {
                println!("Error register agent to the organization: {}", e);
                return Err(e);
            }
        };
    }
}

pub async fn ping_graphql(
    graphql_url: &str,
    token: &str,
) -> Result<reqwest::Response, reqwest::Error> {
    let query = r#"
    query {
        authPing {
          status
        }
      }
    "#;
    let client = reqwest::Client::new();
    let response = client
        .post(graphql_url)
        .json(&json!({ "query": query }))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    match response {
        Ok(response) => Ok(response),
        Err(e) => Err(e),
    }
}
