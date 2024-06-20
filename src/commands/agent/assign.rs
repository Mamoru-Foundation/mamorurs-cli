use cred_store::CredStore;

use crate::client::{ping_graphql, register_daemon_to_organization};

pub async fn assign_to_organization(
    graphql_url: String,
    daemon_id: String,
    organization_id: String,
    cred_store: &impl CredStore,
) -> Result<String, reqwest::Error> {
    let token = cred_store
        .get("access_token")
        .expect("access_token required");
    // Ping graphql
    let resp = ping_graphql(&graphql_url, token).await;

    if resp.is_err() {
        println!("Error ping graphql");
        return Err(resp.err().unwrap());
    }

    // Assign agent to organization

    println!("Assign agent to organization: {}", organization_id);

    match register_daemon_to_organization(
        graphql_url.as_str(),
        token,
        daemon_id.as_str(),
        organization_id.as_str(),
    )
    .await
    {
        Ok(response) => Ok(response.text().await?),
        Err(e) => Err(e),
    }
}
