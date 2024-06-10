mod auth;
mod client;
mod commands;
mod config;
mod manifest;

use auth::{get_token::get_token, jwtverifier::JwtVerifier, Claims};
use client::register_daemon_to_graphql;
use config::Config;
use cred_store::{CredStore, Credentials};

use std::{env, path::PathBuf};
use url::Url;

use clap::{arg, command, value_parser, Arg, Command};

pub struct CommandContext<'a, T: CredStore> {
    pub config: &'a Config,
    pub cred_store: &'a mut T,
}

const MAMORU_CONFIG_DIR: &str = ".mamorurc";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::from_env(Some(MAMORU_CONFIG_DIR)).expect("failed to load config");

    let mut credentials = Credentials::new()
        .set_file_name(format!("{}/{}", MAMORU_CONFIG_DIR, ".credentials"))
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
                                .env("MAMORU_RPC_URL")
                                .default_value("http://127.0.0.1:9090")
                                .value_parser(value_parser!(Url)),
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
                                .help("WASM file to publish")
                                .required(true)
                                .value_parser(value_parser!(PathBuf)),
                        ),
                )
                .subcommand(
                    command!("new")
                        // .arg_required_else_help(true)
                        .about("Create a new agent")
                        .arg(arg!(-n [name] "Agent name").default_value("new-agent")),
                ),
        )
        .subcommand(Command::new("logout").about("Logout from mamoru"))
        .subcommand(command!("login").about("Login to mamoru"))
        .get_matches();

    if let Some(agent_matches) = matches.subcommand_matches("agent") {
        if let Some(publish_matches) = agent_matches.subcommand_matches("publish") {
            let access_token = match get_token(&mut context) {
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
            let verifier = JwtVerifier::new(&config.domain)
                .validate_aud(&config.audience)
                .use_cache(true)
                .build();
            let _ver_res = verifier
                .verify::<Claims>(access_token.as_str())
                .await
                .unwrap();

            let file_path = publish_matches
                .get_one::<PathBuf>("file")
                .expect("filepath required")
                .canonicalize()
                .expect("invalid file path");
            let prkey = publish_matches
                .get_one::<String>("key")
                .expect("key required")
                .to_string();
            let grpc: &Url = publish_matches
                .get_one::<Url>("grpc")
                .expect("grpc required");
            let chain_name = publish_matches
                .get_one::<String>("chain-name")
                .expect("chain-name required")
                .to_string();
            let gas_limit = publish_matches
                .get_one::<String>("gas-limit")
                .expect("gas-limit required");

            let gas_limit = gas_limit.parse::<u64>().unwrap();

            let result = commands::agent::publish::publish_agent(
                grpc, prkey, chain_name, &file_path, gas_limit,
            )
            .await;
            let token = context.cred_store.get("access_token").unwrap();
            match result {
                Ok(result) => match register_daemon_to_graphql(token, result.as_str()).await {
                    Ok(_) => println!("Agent successfully registered"),
                    Err(e) => println!("Error: {:?}", e),
                },
                Err(e) => println!("Error: {:?}", e),
            }
        }

        if let Some(new_matches) = agent_matches.subcommand_matches("new") {
            let name = new_matches.get_one::<String>("name").unwrap().to_string();
            commands::agent::new::create_new_agent(name);
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
