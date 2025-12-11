use clap::{Parser, Subcommand};

/// Qwest - a CLI-based HTTP client with TOML projects and scripted flows.
#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new project and open it in the default editor.
    New {
        /// The project name (also the TOML filename).
        project: String,
    },

    /// Run a request (or scenario) from a project.
    Run {
        /// Optional .env file to load variables from.
        #[arg(long = "env-file")]
        env_file: Option<String>,

        /// Extra variables, e.g. -e token=1234
        #[arg(short = 'e', long = "env", value_parser = parse_key_val::<String, String>)]
        env: Vec<(String, String)>,

        /// Project name (TOML file: ~/.local/share/.qwest/adventures/<project>.toml)
        project: String,

        /// Route or scenario name
        ///
        /// - If matches a request name => single request
        /// - If matches a scenario name => chained requests
        route: String,

        /// Optional output format: json, html, raw…
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Set a variable in the SQLite DB.
    Set {
        /// label of the variable
        label: String,
        /// value of the variable
        value: String,
        /// optional project name (NULL means global variable)
        #[arg(long)]
        project: Option<String>,
    },

    /// Edit an existing project in the default editor.
    Edit {
        project: String,
    },

    /// Delete a project (remove the TOML file).
    Delete {
        project: String,
    },
}

/// Parse KEY=VALUE pairs
fn parse_key_val<K, V>(s: &str) -> Result<(K, V), String>
where
    K: std::str::FromStr,
    V: std::str::FromStr,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no `=` found in `{s}`"))?;
    let (key, value) = s.split_at(pos);
    let value = &value[1..];
    Ok((
        key.parse().map_err(|_| format!("invalid key: `{key}`"))?,
        value.parse().map_err(|_| format!("invalid value: `{value}`"))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_run_with_env_and_format() {
        let cli = Cli::try_parse_from([
            "qwest",
            "run",
            "--env-file",
            ".env",
            "-e",
            "token=1234",
            "my_project",
            "my_route",
            "--format",
            "json",
        ])
        .unwrap();

        match cli.command {
            Command::Run {
                env_file,
                env,
                project,
                route,
                format,
            } => {
                assert_eq!(env_file.as_deref(), Some(".env"));
                assert_eq!(project, "my_project");
                assert_eq!(route, "my_route");
                assert_eq!(format, "json");
                assert_eq!(env.len(), 1);
                assert_eq!(env[0].0, "token");
                assert_eq!(env[0].1, "1234");
            }
            _ => panic!("expected Run command"),
        }
    }

    #[test]
    fn parse_set_command() {
        // this matches: qwest set <LABEL> <VALUE> --project <PROJECT>
        let cli = Cli::try_parse_from([
            "qwest",
            "set",
            "token",
            "xx",
            "--project",
            "proj",
        ])
        .unwrap();

        match cli.command {
            Command::Set {
                label,
                value,
                project,
            } => {
                assert_eq!(label, "token");
                assert_eq!(value, "xx");
                assert_eq!(project.as_deref(), Some("proj"));
            }
            _ => panic!("expected Set command"),
        }
    }
}

