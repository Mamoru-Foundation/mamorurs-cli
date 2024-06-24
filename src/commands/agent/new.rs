#[cfg(feature = "no-ssl")]
use cargo_generate::{generate, GenerateArgs, TemplatePath, Vcs};

const GIT_TEMPLATE_PATH: &str = "https://github.com/Mamoru-Foundation/mamoru-wit-agent-templat.git";

/// This function creates a new agent with the given name.
///
/// It uses the `cargo_generate` crate to generate a new project from a git template.
/// The template is specified by the `GIT_TEMPLATE_PATH` constant.
///
/// # Arguments
///
/// * `name` - A string that holds the name of the new agent.
///
/// # Examples
///
/// ```sh
/// cargo generate --git https://github.com/Mamoru-Foundation/mamoru-wit-agent-templat.git --name my-project
/// ```
///
/// # Panics
///
/// This function will panic if the generation of the new agent fails for any reason.
#[cfg(feature = "no-ssl")]
pub fn create_new_agent(name: String) {
    println!("Creating new agent at {}", name);

    let agent_args = GenerateArgs {
        name: Some(name),
        vcs: Some(Vcs::Git),
        template_path: TemplatePath {
            git: Some(GIT_TEMPLATE_PATH.to_string()),
            ..TemplatePath::default()
        },
        ..GenerateArgs::default()
    };

    let path = generate(agent_args).expect("something went wrong!");
    println!("Generated agent at {:?}", path.display());
}
