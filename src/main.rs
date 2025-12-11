mod cli;
mod config;
mod storage;
mod runner;
mod scripting;
mod templating;

use crate::cli::{Cli, Command};
use clap::Parser;
use rusqlite::Connection;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::New { project } => cmd_new(&project)?,
        Command::Run {
            env_file,
            env,
            project,
            route,
            format,
        } => cmd_run(env_file, env, &project, &route, &format)?,
        Command::Set {
            label,
            value,
            project,
        } => cmd_set(&label, &value, project.as_deref())?,
        Command::Edit { project } => cmd_edit(&project)?,
        Command::Delete { project } => cmd_delete(&project)?,
    }

    Ok(())
}

fn open_db() -> anyhow::Result<Connection> {
    let path = storage::db_path()?;
    std::fs::create_dir_all(path.parent().unwrap())?;
    let conn = Connection::open(path)?;
    storage::init_db(&conn)?;
    Ok(conn)
}

// qwest new my_project
fn cmd_new(project: &str) -> anyhow::Result<()> {
    use std::io::Write;

    let path = config::project_toml_path(project)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if path.exists() {
        anyhow::bail!("project `{project}` already exists");
    }

    let mut file = std::fs::File::create(&path)?;
    writeln!(
        file,
        r#"[api]
name = "{project}"
base_url = ""

[api.scenarios]

[[requests]]
name   = "docs"
method = "GET"
path   = "/docs"
"#
    )?;

    // open in default editor (e.g. $EDITOR)
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".into());
    std::process::Command::new(editor)
        .arg(&path)
        .status()?;

    Ok(())
}

// qwest run ...
fn cmd_run(
    env_file: Option<String>,
    cli_env: Vec<(String, String)>,
    project: &str,
    route: &str,
    format: &str,
) -> anyhow::Result<()> {
    let conn = open_db()?;

    // Load TOML
    let path = config::project_toml_path(project)?;
    let content = std::fs::read_to_string(path)?;
    let cfg: config::ProjectConfig = toml::from_str(&content)?;

    // Load variables from all sources and merge
    let env_file_vars = if let Some(path) = env_file {
        dotenvy::from_filename_iter(path)?
            .map(|item| {
                let (k, v) = item?;
                Ok((k, v))
            })
            .collect::<Result<templating::Vars, dotenvy::Error>>()?
    } else {
        templating::Vars::new()
    };

    let (global_vars, project_vars) = storage::load_variables(&conn, project)?;
    let global_vars = global_vars
        .into_iter()
        .map(|v| (v.label, v.value))
        .collect();
    let project_vars = project_vars
        .into_iter()
        .map(|v| (v.label, v.value))
        .collect();

    let cli_vars = cli_env.into_iter().collect();

    let vars = templating::merge_vars(env_file_vars, global_vars, project_vars, cli_vars);

    runner::run_route(&conn, &cfg, route, vars, format)
}

// qwest set variable my_var my_val --project my_project
fn cmd_set(label: &str, value: &str, project: Option<&str>) -> anyhow::Result<()> {
    let conn = open_db()?;
    storage::set_variable(&conn, label, value, project)?;
    Ok(())
}

// qwest edit my_project
fn cmd_edit(project: &str) -> anyhow::Result<()> {
    let path = config::project_toml_path(project)?;
    if !path.exists() {
        anyhow::bail!("project `{project}` does not exist");
    }

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".into());
    std::process::Command::new(editor)
        .arg(&path)
        .status()?;

    Ok(())
}

// qwest delete my_project
fn cmd_delete(project: &str) -> anyhow::Result<()> {
    let path = config::project_toml_path(project)?;
    if path.exists() {
        std::fs::remove_file(path)?;
        println!("Deleted project `{project}`");
    } else {
        println!("Project `{project}` does not exist");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn cmd_set_creates_variable_in_db() {
        let conn = super::open_db().unwrap();
        // use a unique label to avoid interference across runs
        let label = "test_main_cmd_set";
        super::cmd_set(label, "value", Some("proj")).unwrap();

        let (_global, proj) = crate::storage::load_variables(&conn, "proj").unwrap();
        let v = proj
            .iter()
            .find(|v| v.label == label)
            .expect("variable not found");
        assert_eq!(v.value, "value");
    }

    #[test]
    fn cmd_new_creates_toml_file() {
        // avoid opening nvim -> set EDITOR to `true` (no-op)
        env::set_var("EDITOR", "true");

        let project = "test_project_cmd_new";
        super::cmd_new(project).unwrap();

        let path = crate::config::project_toml_path(project).unwrap();
        assert!(path.exists());

        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("[api]"));
        assert!(content.contains("name = \"test_project_cmd_new\""));
    }
}

