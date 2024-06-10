use config::Config as ConfigBuilder;
use std::{env, error::Error, fs, path::PathBuf};

const CONFIG_NAME: &str = ".env";

#[derive(Debug)]
pub struct Config {
    pub domain: String,
    pub client_id: String,
    pub audience: String,
    pub grpc_url: String,
    pub private_key: String,
    pub gas_limit: u64,
}

impl Config {
    pub fn from_env(config_path: Option<&str>) -> Result<Self, Box<dyn Error>> {
        if let Some(config_path) = config_path {
            Self::create_config_file(config_path);
            dotenv::from_filename(PathBuf::from(config_path).join(CONFIG_NAME)).ok();
        } else {
            dotenv::dotenv().ok();
        }

        let builder = ConfigBuilder::builder()
            .set_default(
                "CLI_AUTH0_DOMAIN",
                env::var("MAMORU_CLI_AUTH0_DOMAIN").unwrap_or_default(),
            )
            .unwrap()
            .set_default(
                "CLI_AUTH0_CLIENT_ID",
                env::var("MAMORU_CLI_AUTH0_CLIENT_ID").unwrap_or_default(),
            )
            .unwrap()
            .set_default(
                "CLI_AUTH0_AUDIENCE",
                env::var("MAMORU_CLI_AUTH0_AUDIENCE").unwrap_or_default(),
            )
            .unwrap()
            .set_default("RPC_URL", env::var("MAMORU_RPC_URL").unwrap_or_default())
            .unwrap()
            .set_default(
                "PRIVATE_KEY",
                env::var("MAMORU_PRIVATE_KEY").unwrap_or_default(),
            )
            .unwrap()
            .set_default(
                "GAS_LIMIT",
                env::var("MAMORU_GAS_LIMIT").unwrap_or_default(),
            )
            .unwrap()
            .add_source(
                config::Environment::with_prefix("MAMORU")
                    .try_parsing(true)
                    .separator("_"),
            );

        let config = match builder.build() {
            Ok(config) => config,
            Err(e) => {
                println!("Error: {}", e);
                return Err(Box::new(e));
            }
        };

        Ok(Self {
            domain: config.get("CLI_AUTH0_DOMAIN").unwrap_or_default(),
            client_id: config.get("CLI_AUTH0_CLIENT_ID").unwrap_or_default(),
            audience: config.get("CLI_AUTH0_AUDIENCE").unwrap_or_default(),
            grpc_url: config.get("RPC_URL").unwrap_or_default(),
            private_key: config.get("PRIVATE_KEY").unwrap_or_default(),
            gas_limit: config.get::<u64>("GAS_LIMIT").unwrap_or_default(),
        })
    }

    fn create_config_file(dir_path: &str) {
        // look for a config file on home directory
        if let Some(path) = dirs::home_dir() {
            let config_file = path.join(dir_path).join(CONFIG_NAME);
            if !config_file.exists() {
                // create a config file
                let config = r#"
MAMORU_CLI_AUTH0_DOMAIN = "https://mamoru.us.auth0.com"
MAMORU_CLI_AUTH0_CLIENT_ID = "DKVTdw1UnneGumqOAPrEJs8RqdGTDd2e"
MAMORU_CLI_AUTH0_AUDIENCE = "https://mamoru.ai"
MAMORU_GAS_LIMIT = "200000000"
                "#;
                fs::write(config_file, config).expect("Unable to write file");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sealed_test::prelude::*;
    use std::env;

    use crate::config::Config;

    #[sealed_test]
    fn test_config() {
        env::set_var("MAMORU_CLI_AUTH0_DOMAIN", "http://localhost");
        env::set_var("MAMORU_CLI_AUTH0_CLIENT_ID", "some_client_id");
        env::set_var("MAMORU_CLI_AUTH0_AUDIENCE", "http://localhost");
        env::set_var("MAMORU_RPC_URL", "http://localhost:9090");
        env::set_var("MAMORU_PRIVATE_KEY", "private_key");
        env::set_var("MAMORU_GAS_LIMIT", "200000000");
        let config = Config::from_env(None).unwrap();
        assert_eq!(config.domain, "http://localhost", "domain should be equal");
        assert_eq!(
            config.client_id, "some_client_id",
            "client_id should be equal"
        );
        assert_eq!(
            config.audience, "http://localhost",
            "audience should be equal"
        );
        assert_eq!(
            config.grpc_url, "http://localhost:9090",
            "grpc_url should be equal"
        );
        assert_eq!(
            config.private_key, "private_key",
            "private_key should be equal"
        );
        assert_eq!(config.gas_limit, 200000000, "gas_limit should be equal");
    }

    #[sealed_test]
    fn test_empy_env() {
        let config = Config::from_env(None).unwrap();
        assert_eq!(config.domain, "", "domain should be equal");
        assert_eq!(config.client_id, "", "client_id should be equal");
        assert_eq!(config.audience, "", "audience should be equal");
        assert_eq!(config.grpc_url, "", "grpc_url should be equal");
        assert_eq!(config.private_key, "", "private_key should be equal");
        assert_eq!(config.gas_limit, 0, "gas_limit should be equal");
    }
}
