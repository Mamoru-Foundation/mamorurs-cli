use base64::{prelude::BASE64_STANDARD, Engine};
use config::Config;
use cosmrs::crypto::secp256k1;
use mamoru_chain_client::{
    AccountConfig, ChainConfig, ConnectionConfig, MessageClient, MessageClientConfig, QueryClient,
    QueryClientConfig, SendMode, DEFAULT_MAX_RECV_MESSAGE_SIZE,
};

use serde_json::json;
use url::Url;

const GRAPHQL_URL: &str = "https://mamoru-be-production.mamoru.foundation/graphql";

#[allow(dead_code)]
pub async fn query_client(grpc_url: Url) -> QueryClient {
    QueryClient::connect(query_client_config(grpc_url))
        .await
        .expect("QueryClient::connect error.")
}

pub async fn message_client(prkey: String, grpc_url: &Url, gas_limit: u64) -> MessageClient {
    MessageClient::connect(message_client_config(prkey, grpc_url, gas_limit).await)
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
        sdk_versions: mamoru_chain_client::VersionConfig {
            versions: Vec::new(),
        },
    }
}

pub async fn message_client_config(
    key: String,
    grpc_url: &Url,
    gas_limit: u64,
) -> MessageClientConfig {
    let private_key: cosmrs::crypto::secp256k1::SigningKey = string_to_signing_key(key.as_str());
    let builder = Config::builder()
        .set_default(
            "max_decoding_message_size",
            DEFAULT_MAX_RECV_MESSAGE_SIZE.to_string(),
        )
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

pub async fn register_daemon_to_graphql(
    token: &str,
    daemon_id: &str,
) -> std::result::Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();

    let query = json!({
        "query": "mutation assignDaemonOrganizationId($daemonId: String!) { assignDaemonOrganizationId(daemonId: $daemonId) { __typename } }",
        "variables": { "daemonId": daemon_id }
    });

    let body = query;
    let body_string = serde_json::to_string(&body).unwrap();

    loop {
        match client
            .post(GRAPHQL_URL)
            .body(body_string.clone())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
        {
            Ok(response) => {
                return Ok(response);
            }
            Err(e) => {
                println!("Error register agent to the organisation: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1000));
            }
        };
    }
}
