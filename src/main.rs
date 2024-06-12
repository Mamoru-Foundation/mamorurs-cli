mod auth;
mod client;
mod commands;
mod config;
mod daemon_builder;
mod input;
mod manifest;

use auth::{get_token::get_token, jwtverifier::JwtVerifier, Claims};
use client::register_daemon_to_graphql;
use config::Config;
use cred_store::{CredStore, Credentials};

use clap::{arg, command, value_parser, Arg, Command};
use std::{env, panic, path::PathBuf};

#[derive(Debug)]
pub struct CommandContext<'a, T: CredStore> {
    pub config: &'a Config,
    pub cred_store: &'a mut T,
}

const MAMORU_CONFIG_DIR: &str = ".mamorurc";
const CONFIG_NAME: &str = "settings.toml";
const CREDENTIALS: &str = ".credentials";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = match dirs::home_dir() {
        Some(home) => home,
        None => {
            panic!("failed to get home directory");
        }
    };

    let mamoru_dir_path = home_dir.join(MAMORU_CONFIG_DIR);
    let settings_file = mamoru_dir_path.join(CONFIG_NAME);
    let credentials_file = mamoru_dir_path.join(CREDENTIALS);

    let config = config::Config::from_env(settings_file.to_str()).expect("failed to load config");

    let mut credentials = Credentials::new()
        .set_file_name(credentials_file.to_str().unwrap().to_string())
        .build()
        .load()
        .expect("failed to load credentials");

    let mut context = CommandContext {
        config: &config,
        cred_store: &mut credentials,
    };

    let matches = command!()
        .about("mamoru cli tool")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("agent")
                .about("Manage agents")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("publish")
                        .about("Publish an agent")
                        .arg_required_else_help(true)
                        .arg(
                            arg!(--grpc <GRPC> "gRPC URL")
                                .required(false)
                                .env("MAMORU_RPC_URL"),
                        )
                        .arg(
                            arg!(-k --key <KEY> "Private key")
                                .required(false)
                                .env("MAMORU_PRIVATE_KEY"),
                        )
                        .arg(arg!(-c --"chain-name" <CHAIN_NAME> "Chain name").required(true))
                        .arg(
                            arg!(--"gas-limit" <GAS_LIMIT> "Gas limit")
                                .default_value("200000000")
                                .env("MAMORU_GAS_LIMIT"),
                        )
                        .arg(
                            Arg::new("file")
                                .help("Path to Agent directory")
                                .required(true)
                                .value_parser(value_parser!(PathBuf)),
                        ),
                )
                .subcommand(
                    command!("new")
                        .about("Create a new agent")
                        .arg(arg!(-n [name] "Agent name").default_value("new-agent")),
                )
                .subcommand(
                    command!("launch")
                        .about("Publish an agent to existing metadata")
                        .arg(arg!(-m --"metadata-id" <METADATA_ID> "Metadata ID").required(true))
                        .arg(arg!(-c --"chain-name" <CHAIN_NAME> "Chain name").required(true))
                        .arg(
                            arg!(--"gas-limit" <GAS_LIMIT> "Gas limit")
                                .default_value("200000000")
                                .env("MAMORU_GAS_LIMIT"),
                        )
                        .arg(
                            arg!(-k --key <KEY> "Private key")
                                .required(false)
                                .env("MAMORU_PRIVATE_KEY"),
                        )
                        .arg(
                            Arg::new("file")
                                .help("Path to Agent directory")
                                .required(true)
                                .value_parser(value_parser!(PathBuf)),
                        ),
                ),
        )
        .subcommand(Command::new("logout").about("Logout from mamoru"))
        .subcommand(command!("login").about("Login to mamoru"))
        .get_matches();

    if let Some(agent_matches) = matches.subcommand_matches("agent") {
        if let Some(publish_matches) = agent_matches.subcommand_matches("publish") {
            check_auth(&mut context).await?;
            dbg!(&context);

            let file_path = publish_matches
                .get_one::<PathBuf>("file")
                .expect("filepath required")
                .canonicalize()
                .expect("invalid file path");

            let prkey: String = match publish_matches.get_one::<String>("key") {
                Some(key) => key.to_string(),
                None => {
                    if context.config.mamoru_private_key.is_empty() {
                        eprintln!("Private key required");
                        std::process::exit(1);
                    } else {
                        context.config.mamoru_private_key.clone()
                    }
                }
            };

            let grpc: String = match publish_matches.get_one::<String>("grpc") {
                Some(grpc) => grpc.to_string(),
                None => {
                    if context.config.mamoru_rpc_url.is_empty() {
                        eprintln!("gRPC URL required");
                        std::process::exit(1);
                    } else {
                        context.config.mamoru_rpc_url.clone()
                    }
                }
            };

            let gas_limit: String = match publish_matches.get_one::<String>("gas-limit") {
                Some(gas_limit) => gas_limit.to_string(),
                None => {
                    if context.config.mamoru_gas_limit.is_empty() {
                        eprintln!("Gas limit required");
                        std::process::exit(1);
                    } else {
                        context.config.mamoru_gas_limit.clone()
                    }
                }
            };

            let chain_name = publish_matches
                .get_one::<String>("chain-name")
                .expect("chain-name required")
                .to_string();

            let gas_limit = gas_limit
                .parse::<u64>()
                .expect("gas limit must be a number");

            let publish_result = commands::agent::publish::publish_agent(
                grpc, prkey, chain_name, &file_path, gas_limit,
            )
            .await;

            let token = context
                .cred_store
                .get("access_token")
                .expect("access_token required");
            match publish_result {
                Ok(daemon_id) => {
                    match register_daemon_to_graphql(
                        context.config.mamoru_graphql_url.as_str(),
                        token,
                        daemon_id.as_str(),
                    )
                    .await
                    {
                        Ok(_) => println!("Agent successfully registered to the organization."),
                        Err(e) => println!("Error graphql: {:?}", e),
                    }
                }
                Err(e) => println!("Error publish agent: {:?}", e),
            }
        }

        if let Some(new_matches) = agent_matches.subcommand_matches("new") {
            let name = new_matches.get_one::<String>("name").unwrap().to_string();
            commands::agent::new::create_new_agent(name);
        }

        if let Some(launch_matches) = agent_matches.subcommand_matches("launch") {
            check_auth(&mut context).await?;

            let file_path = launch_matches
                .get_one::<PathBuf>("file")
                .expect("filepath required")
                .canonicalize()
                .expect("invalid file path");
            let metadata_id = launch_matches
                .get_one::<String>("metadata-id")
                .expect("metadata-id required")
                .to_string();
            let chain_name = launch_matches
                .get_one::<String>("chain-name")
                .expect("chain-name required")
                .to_string();

            let prkey: String = match launch_matches.get_one::<String>("key") {
                Some(key) => key.to_string(),
                None => {
                    if context.config.mamoru_private_key.is_empty() {
                        eprintln!("Private key required");
                        std::process::exit(1);
                    } else {
                        context.config.mamoru_private_key.clone()
                    }
                }
            };

            let grpc: String = match launch_matches.get_one::<String>("grpc") {
                Some(grpc) => grpc.to_string(),
                None => {
                    if context.config.mamoru_rpc_url.is_empty() {
                        eprintln!("gRPC URL required");
                        std::process::exit(1);
                    } else {
                        context.config.mamoru_rpc_url.clone()
                    }
                }
            };

            let gas_limit: String = match launch_matches.get_one::<String>("gas-limit") {
                Some(gas_limit) => gas_limit.to_string(),
                None => {
                    if context.config.mamoru_gas_limit.is_empty() {
                        eprintln!("Gas limit required");
                        std::process::exit(1);
                    } else {
                        context.config.mamoru_gas_limit.clone()
                    }
                }
            };
            let gas_limit = gas_limit
                .parse::<u64>()
                .expect("gas limit must be a number");

            let publish_result = commands::agent::launch::launch_agent(
                metadata_id,
                grpc,
                prkey,
                chain_name,
                &file_path,
                gas_limit,
            )
            .await;

            let token = context
                .cred_store
                .get("access_token")
                .expect("access_token required");

            match publish_result {
                Ok(daemon_id) => {
                    match register_daemon_to_graphql(
                        context.config.mamoru_graphql_url.as_str(),
                        token,
                        daemon_id.as_str(),
                    )
                    .await
                    {
                        Ok(_) => println!("Agent successfully registered to the organization."),
                        Err(e) => println!("Error graphql: {:?}", e),
                    }
                }
                Err(e) => println!("Error publish agent: {:?}", e),
            }
        }
    }
    if let Some(_logout_matches) = matches.subcommand_matches("logout") {
        commands::logout::logout(&mut context);
    }

    if let Some(_login_matches) = matches.subcommand_matches("login") {
        match dialoguer::Confirm::new()
            .with_prompt("Do you want to create a new token?")
            .default(false)
            .show_default(true)
            .interact()
            .unwrap()
        {
            true => (),
            false => std::process::exit(0),
        };
        match commands::login::login(&config).await {
            Ok(resp) => {
                let access_token = resp.access_token.clone().unwrap();
                let refresh_token = resp.refresh_token.clone().unwrap_or_default();
                println!();
                println!("Access Token: {}", access_token);
                if commands::login::save_tokens(&access_token, &refresh_token, &mut context)
                    .is_err()
                {
                    eprintln!("Couldn't configure credentials.");
                    std::process::exit(1);
                }
            }
            Err(e) => println!("Error logging in: {}", e),
        }
    }

    Ok(())
}

async fn check_auth<T>(
    context: &mut CommandContext<'_, T>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: CredStore + Send + Sync + 'static,
{
    let access_token = match get_token(context) {
        Ok(token) => match token {
            Some(token) => token,
            None => {
                eprintln!("You must login first.");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Couldn't get credentials: {}.  Try to login again.", e);
            std::process::exit(1);
        }
    };
    // verify token
    let verifier = JwtVerifier::new(&context.config.mamoru_cli_auth0_domain)
        .validate_aud(&context.config.mamoru_cli_auth0_audience)
        .use_cache(true)
        .build();
    let _ver_res = verifier.verify::<Claims>(access_token.as_str()).await?;

    Ok(())
}
