use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub api: Api,
    #[serde(default)]
    pub requests: Vec<Request>,
}

#[derive(Debug, Deserialize)]
pub struct Api {
    pub name: String,
    pub base_url: String,

    /// Map of scenario name -> ordered list of request names
    #[serde(default)]
    pub scenarios: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Request {
    pub name: String,
    pub method: String,
    pub path: String,

    /// Raw JSON as string, with placeholders.
    #[serde(default)]
    pub headers: Option<String>,

    /// Raw JSON as string, with placeholders.
    #[serde(default)]
    pub body: Option<String>,

    /// Scripts attached to this request.
    #[serde(default)]
    pub scripts: Vec<Script>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Script {
    /// true => run before request
    /// false => run after request
    pub before: bool,

    /// Rhai script source code
    pub script: String,

    /// Optional description shown before execution
    #[serde(default)]
    pub description: Option<String>,
}

// ---------- helper used from main.rs ----------

pub fn project_toml_path(project: &str) -> anyhow::Result<PathBuf> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("could not find local data dir"))?
        .join(".qwest")
        .join("adventures");

    Ok(base.join(format!("{project}.toml")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    #[test]
    fn parse_minimal_project_config() {
        let toml = r#"
            [api]
            name = "test"
            base_url = "https://example.com"

            [api.scenarios]
            first = ["docs", "login"]

            [[requests]]
            name = "docs"
            method = "GET"
            path = "/docs"

            [[requests]]
            name = "login"
            method = "POST"
            path = "/login"
        "#;

        let cfg: ProjectConfig = toml::from_str(toml).unwrap();

        assert_eq!(cfg.api.name, "test");
        assert_eq!(cfg.api.base_url, "https://example.com");
        assert_eq!(cfg.api.scenarios["first"], vec!["docs", "login"]);
        assert_eq!(cfg.requests.len(), 2);

        let docs = &cfg.requests[0];
        assert_eq!(docs.name, "docs");
        assert_eq!(docs.method, "GET");
        assert_eq!(docs.path, "/docs");
    }

    #[test]
    fn parse_request_with_scripts() {
        let toml = r#"
            [api]
            name = "test"
            base_url = "https://example.com"

            [[requests]]
            name = "create_job"
            method = "POST"
            path = "/jobs"

              [[requests.scripts]]
              before = false
              script = "return #{ job_id: 1 };"
              description = "Store job id"
        "#;

        let cfg: ProjectConfig = toml::from_str(toml).unwrap();

        assert_eq!(cfg.requests.len(), 1);
        let req = &cfg.requests[0];
        assert_eq!(req.scripts.len(), 1);
        assert_eq!(req.scripts[0].before, false);
        assert_eq!(req.scripts[0].description.as_deref(), Some("Store job id"));
    }
}

