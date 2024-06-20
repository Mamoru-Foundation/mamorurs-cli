mod auth;
mod client;
mod commands;
mod config;
mod daemon_builder;
mod errors;
mod input;
mod manifest;

use auth::{get_token::get_token, jwtverifier::JwtVerifier, Claims};
use client::register_daemon_to_organization;
use config::Config;
use cred_store::{CredStore, Credentials};

use clap::{arg, command, value_parser, Arg, ArgMatches};
use std::{env, panic, path::PathBuf};
use tracing::{debug, info};

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
        .set_file_name(
            credentials_file
                .to_str()
                .expect("failed to get credentials")
                .to_string(),
        )
        .build()
        .load()
        .expect("failed to load credentials");

    let mut context = CommandContext {
        config: &config,
        cred_store: &mut credentials,
    };

    debug!("DEBUG");
    info!("INFO");

    let matches = command!()
        .about("mamoru cli tool")
        .arg_required_else_help(true)
        .subcommand(
            command!("agent")
                .about("Manage agents")
                .arg_required_else_help(true)
                .subcommand(
                    command!("publish")
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
                        .arg(arg!(--"chain-id" <CHAIN_ID> "Chain ID").required(false))
                        .arg(
                            arg!(-o --"organization-id" <ORGANIZATION_ID> "Organization ID")
                                .required(false),
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
                        )
                        .arg(arg!(--"chain-id" <CHAIN_ID> "Chain ID").required(false))
                        .arg(
                            arg!(-o --"organization-id" <ORGANIZATION_ID> "Organization ID")
                                .required(false),
                        ),
                )
                .subcommand(
                    command!("assign")
                        .about("Assign an agent to an organization")
                        .arg(arg!(-d --"daemon-id" <DAEMON_ID> "Daemon ID").required(true))
                        .arg(
                            arg!(-o --"organization-id" <ORGANIZATION_ID> "Organization ID")
                                .required(false),
                        )
                        .arg(arg!(--"graphql-url" <GRAPHQL_URL> "GraphQL URL").required(false)),
                )
                .subcommand(
                    command!("unregister")
                        .about("Unregister an agent")
                        .arg(arg!(-d --"daemon-id" <DAEMON_ID> "Daemon ID").required(true))
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
                            arg!(--grpc <GRPC> "gRPC URL")
                                .required(false)
                                .env("MAMORU_RPC_URL"),
                        )
                        .arg(arg!(--"chain-id" <CHAIN_ID> "Chain ID").required(false)),
                ),
        )
        .subcommand(command!("logout").about("Logout from mamoru"))
        .subcommand(command!("login").about("Login to mamoru"))
        .get_matches();

    if let Some(agent_matches) = matches.subcommand_matches("agent") {
        if let Some(publish_matches) = agent_matches.subcommand_matches("publish") {
            check_auth(&mut context).await?;

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

            let chain_id: String = match publish_matches.get_one::<String>("chain-id") {
                Some(chain_id) => chain_id.to_string(),
                None => {
                    if context.config.mamoru_chain_id.is_empty() {
                        eprintln!("Chain ID required");
                        std::process::exit(1);
                    } else {
                        context.config.mamoru_chain_id.clone()
                    }
                }
            };

            let organization_id = get_organization_id(publish_matches, &context);

            let publish_result = commands::agent::publish::publish_agent(
                grpc, prkey, chain_name, &file_path, gas_limit, chain_id,
            )
            .await;

            let token = context
                .cred_store
                .get("access_token")
                .expect("access_token required");
            match publish_result {
                Ok(daemon_id) => {
                    match register_daemon_to_organization(
                        context.config.mamoru_graphql_url.as_str(),
                        token,
                        daemon_id.as_str(),
                        organization_id.as_str(),
                    )
                    .await
                    {
                        Ok(_) => (),
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

            let chain_id: String = match launch_matches.get_one::<String>("chain-id") {
                Some(chain_id) => chain_id.to_string(),
                None => {
                    if context.config.mamoru_chain_id.is_empty() {
                        eprintln!("Gas limit required");
                        std::process::exit(1);
                    } else {
                        context.config.mamoru_chain_id.clone()
                    }
                }
            };

            let organization_id = get_organization_id(launch_matches, &context);

            let publish_result = commands::agent::launch::launch_agent(
                metadata_id,
                grpc,
                prkey,
                chain_name,
                &file_path,
                gas_limit,
                chain_id,
            )
            .await;

            let token = context
                .cred_store
                .get("access_token")
                .expect("access_token required");

            match publish_result {
                Ok(daemon_id) => {
                    match register_daemon_to_organization(
                        context.config.mamoru_graphql_url.as_str(),
                        token,
                        daemon_id.as_str(),
                        organization_id.as_str(),
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

        if let Some(assign_matches) = agent_matches.subcommand_matches("assign") {
            match check_auth(&mut context).await {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    std::process::exit(1);
                }
            };

            let organization_id = get_organization_id(assign_matches, &context);

            let daemon_id = assign_matches
                .get_one::<String>("daemon-id")
                .expect("daemon-id required")
                .to_string();

            let graphql_url = assign_matches
                .get_one::<String>("graphql-url")
                .map(|s| s.as_str());
            let graphql_url = match graphql_url {
                Some(url) => url.to_string(),
                None => {
                    if context.config.mamoru_graphql_url.is_empty() {
                        eprintln!("GraphQL URL required");
                        std::process::exit(1);
                    } else {
                        context.config.mamoru_graphql_url.clone()
                    }
                }
            };

            commands::agent::assign::assign_to_organization(
                graphql_url,
                daemon_id,
                organization_id,
                context.cred_store,
            )
            .await?;
        }

        if let Some(unregister_matches) = agent_matches.subcommand_matches("unregister") {
            check_auth(&mut context).await?;

            let daemon_id = unregister_matches
                .get_one::<String>("daemon-id")
                .expect("daemon-id required")
                .to_string();

            let prkey: String = match unregister_matches.get_one::<String>("key") {
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

            let grpc: String = match unregister_matches.get_one::<String>("grpc") {
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
            let gas_limit: String = match unregister_matches.get_one::<String>("gas-limit") {
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

            let chain_id: String = match unregister_matches.get_one::<String>("chain-id") {
                Some(chain_id) => chain_id.to_string(),
                None => {
                    if context.config.mamoru_chain_id.is_empty() {
                        eprintln!("Chain ID required");
                        std::process::exit(1);
                    } else {
                        context.config.mamoru_chain_id.clone()
                    }
                }
            };

            match commands::agent::unregister::unregister_agent(
                prkey, grpc, chain_id, gas_limit, daemon_id,
            )
            .await
            {
                Ok(response) => println!("Success unregister agent: {}", response),
                Err(e) => println!("Error unregister agent: {:?}", e),
            };
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
                println!("Access token received!");
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
    let access_token = match get_token(context).await {
        Ok(token) => match token {
            Some(token) => token,
            None => {
                eprintln!("You must login first, please run 'mamorurs-cli login'.");
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

fn get_organization_id(
    matcher: &ArgMatches,
    context: &CommandContext<'_, impl CredStore>,
) -> String {
    let organization_id = matcher.get_one::<String>("organization-id");

    match organization_id {
        Some(organization_id) => organization_id.to_string(),
        None => {
            if context.config.mamoru_organization_id.is_empty() {
                eprintln!("Organization ID required");
                std::process::exit(1);
            } else {
                context.config.mamoru_organization_id.clone()
            }
        }
    }
}
