use url::Url;

use crate::client::message_client;

pub async fn unregister_agent(
    prkey: String,
    grpc: String,
    chain_id: String,
    gas_limit: u64,
    daemon_id: String,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Unresisting agent...");
    let message_client =
        message_client(prkey, &grpc.parse::<Url>().unwrap(), gas_limit, chain_id).await;

    match message_client.unregister_daemon(daemon_id).await {
        Ok(response) => Ok(response.daemon_id),
        Err(e) => Err(Box::new(e)),
    }
}
