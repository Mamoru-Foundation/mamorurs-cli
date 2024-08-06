use config::{Config as ConfigBuilder, File, FileFormat};
use serde::Deserialize;
use std::{error::Error, fs, io::Write, path::Path};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mamoru_cli_auth0_domain: String,
    pub mamoru_cli_auth0_client_id: String,
    pub mamoru_cli_auth0_audience: String,
    pub mamoru_rpc_url: String,
    pub mamoru_private_key: String,
    pub mamoru_gas_limit: String,
    pub mamoru_graphql_url: String,
    pub mamoru_chain_id: String,
    pub mamoru_organization_id: String,
}

impl Config {
    pub fn from_env(config_path: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let mut builder = ConfigBuilder::builder();
        if let Some(config_path) = config_path {
            create_config_file(config_path)?;
            builder =
                builder.add_source(File::from(Path::new(config_path)).format(FileFormat::Toml));
        }

        builder = builder.add_source(config::Environment::default());

        let config = match builder.build() {
            Ok(config) => config,
            Err(e) => {
                println!("Error build config: {}", e);
                Err(Box::new(e))?
            }
        };

        match config.try_deserialize::<Config>() {
            Ok(settings) => Ok(settings),
            Err(e) => {
                println!("Error: {}", e);
                Err(Box::new(e))
            }
        }
    }
}

fn create_config_file(config_path: &str) -> Result<(), Box<dyn Error>> {
    //create config file if it doesn't exist
    if !Path::new(config_path).exists() {
        let mut file = fs::File::create(config_path)?;

        file.write_all(
            toml::toml! {
                MAMORU_CLI_AUTH0_DOMAIN = "https://dev-xp12liakgecl7vlc.us.auth0.com"
                MAMORU_CLI_AUTH0_CLIENT_ID = "dwauk7iBT36rlvE4XTh3QJ0IxWAv8AGc"
                MAMORU_CLI_AUTH0_AUDIENCE = "https://mamoru.ai"
                MAMORU_RPC_URL = "https://devnet.chain.mamoru.foundation:9090"
                MAMORU_PRIVATE_KEY = ""
                MAMORU_GAS_LIMIT = "200000000"
                MAMORU_GRAPHQL_URL = "https://mamoru-be-development.mamoru.foundation/graphql"
                MAMORU_CHAIN_ID = "devnet"
                MAMORU_ORGANIZATION_ID = "cbcb995c-aa56-4edb-a305-57a66edf5480"
            }
            .to_string()
            .as_bytes(),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::tests::tempfile::TempDir;
    use crate::config::Config;
    use sealed_test::prelude::*;
    use std::{env, fs::File, io::Write};

    #[sealed_test]
    fn test_config_from_env() {
        env::set_var("MAMORU_CLI_AUTH0_DOMAIN", "http://localhost");
        env::set_var("MAMORU_CLI_AUTH0_CLIENT_ID", "some_client_id");
        env::set_var("MAMORU_CLI_AUTH0_AUDIENCE", "http://localhost");
        env::set_var("MAMORU_RPC_URL", "http://localhost:9090");
        env::set_var("MAMORU_PRIVATE_KEY", "private_key");
        env::set_var("MAMORU_GAS_LIMIT", "200000000");
        env::set_var("MAMORU_GRAPHQL_URL", "http://localhost:1234");
        env::set_var("MAMORU_CHAIN_ID", "validationchain");
        env::set_var("MAMORU_ORGANIZATION_ID", "some_organization_id");
        let config = Config::from_env(None).unwrap();
        assert_eq!(
            config.mamoru_cli_auth0_domain, "http://localhost",
            "domain should be equal"
        );
        assert_eq!(
            config.mamoru_cli_auth0_client_id, "some_client_id",
            "client_id should be equal"
        );
        assert_eq!(
            config.mamoru_cli_auth0_audience, "http://localhost",
            "audience should be equal"
        );
        assert_eq!(
            config.mamoru_rpc_url, "http://localhost:9090",
            "grpc_url should be equal"
        );
        assert_eq!(
            config.mamoru_private_key, "private_key",
            "private_key should be equal"
        );
        assert_eq!(
            config.mamoru_gas_limit, "200000000",
            "gas_limit should be equal"
        );
        assert_eq!(
            config.mamoru_graphql_url, "http://localhost:1234",
            "graphql_url should be equal"
        );
        assert_eq!(
            config.mamoru_chain_id, "validationchain",
            "chain_id should be equal"
        );
        assert_eq!(
            config.mamoru_organization_id, "some_organization_id",
            "organization_id should be equal"
        );
    }

    #[sealed_test]
    fn test_override_config_from_file() {
        env::set_var("MAMORU_CLI_AUTH0_DOMAIN", "http://localhost:0000");
        env::set_var("MAMORU_CLI_AUTH0_CLIENT_ID", "some_client_id0");
        env::set_var("MAMORU_CLI_AUTH0_AUDIENCE", "http://localhost:0000");
        env::set_var("MAMORU_RPC_URL", "http://localhost:0000");
        env::set_var("MAMORU_PRIVATE_KEY", "private_key0");
        env::set_var("MAMORU_GAS_LIMIT", "1234567890");
        env::set_var("MAMORU_GRAPHQL_URL", "http://localhost:1234");
        env::set_var("MAMORU_CHAIN_ID", "validationchain");
        env::set_var("MAMORU_ORGANIZATION_ID", "some_organization_id");
        let tmp_dir = TempDir::new().unwrap();
        let config_file = tmp_dir.path().join("mamoru.toml");
        let mut tmp_file = File::create(&config_file).unwrap();
        let config_data = toml::toml! {
            MAMORU_CLI_AUTH0_DOMAIN = "http://cli_auth0_domain"
            MAMORU_CLI_AUTH0_CLIENT_ID = "cli_auth0_client_id"
            MAMORU_CLI_AUTH0_AUDIENCE = "http://cli_auth0_audience"
            MAMORU_RPC_URL = "http://rpc_url"
            MAMORU_PRIVATE_KEY = "private_key"
            MAMORU_GAS_LIMIT = "9000000"
            MAMORU_GRAPHQL_URL = "http://graphql_url"
            MAMORU_CHAIN_ID = "chain_id"
            MAMORU_ORGANIZATION_ID = "some_organization_id"
        };
        tmp_file
            .write_all(config_data.to_string().as_bytes())
            .unwrap();

        let config = Config::from_env(config_file.to_str()).unwrap();
        assert_eq!(
            config.mamoru_cli_auth0_domain, "http://localhost:0000",
            "domain should be equal"
        );
        assert_eq!(
            config.mamoru_cli_auth0_client_id, "some_client_id0",
            "client_id should be equal"
        );
        assert_eq!(
            config.mamoru_cli_auth0_audience, "http://localhost:0000",
            "audience should be equal"
        );
        assert_eq!(
            config.mamoru_rpc_url, "http://localhost:0000",
            "grpc_url should be equal"
        );
        assert_eq!(
            config.mamoru_private_key, "private_key0",
            "private_key should be equal"
        );
        assert_eq!(
            config.mamoru_gas_limit, "1234567890",
            "gas_limit should be equal"
        );
        assert_eq!(
            config.mamoru_graphql_url, "http://localhost:1234",
            "graphql_url should be equal"
        );
        assert_eq!(
            config.mamoru_chain_id, "validationchain",
            "chain_id should be equal"
        );
        assert_eq!(
            config.mamoru_organization_id, "some_organization_id",
            "organization_id should be equal"
        );

        tmp_dir.close().unwrap();
    }
}
