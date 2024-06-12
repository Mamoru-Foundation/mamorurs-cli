use config::{Config as ConfigBuilder, File, FileFormat};
use serde::Deserialize;
use std::{
    error::Error,
    fs,
    io::{BufWriter, Write},
    path::Path,
};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mamoru_cli_auth0_domain: String,
    pub mamoru_cli_auth0_client_id: String,
    pub mamoru_cli_auth0_audience: String,
    pub mamoru_rpc_url: String,
    pub mamoru_private_key: String,
    pub mamoru_gas_limit: String,
    pub mamoru_graphql_url: String,
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
        let mut file = BufWriter::new(fs::File::create(config_path)?);

        file.write_all(
            toml::toml! {
                MAMORU_CLI_AUTH0_DOMAIN = "https://mamoru.us.auth0.com"
                MAMORU_CLI_AUTH0_CLIENT_ID = "DKVTdw1UnneGumqOAPrEJs8RqdGTDd2e"
                MAMORU_CLI_AUTH0_AUDIENCE = "https://mamoru.ai"
                MAMORU_RPC_URL = "http://localhost:9090"
                MAMORU_PRIVATE_KEY = ""
                MAMORU_GAS_LIMIT = "200000000"
                MAMORU_GRAPHQL_URL = "https://mamoru-be-production.mamoru.foundation/graphql"
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

        tmp_dir.close().unwrap();
    }
}
