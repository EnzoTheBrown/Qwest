use crate::config::{ProjectConfig, Request};
use crate::scripting::{run_scripts_after, run_scripts_before};
use crate::templating::{render_placeholders, Vars};
use anyhow::Context;
use reqwest::blocking::Client;
use rusqlite::Connection;

pub fn run_route(
    conn: &Connection,
    cfg: &ProjectConfig,
    route: &str,
    mut vars: Vars,
    format: &str,
) -> anyhow::Result<()> {
    let api = &cfg.api;

    let request_names = if let Some(req) = cfg.requests.iter().find(|r| r.name == route) {
        vec![req.name.clone()]
    } else if let Some(seq) = api.scenarios.get(route) {
        seq.clone()
    } else {
        anyhow::bail!("unknown route or scenario `{route}`");
    };

    let client = Client::new();
    for req_name in request_names {
        let req_cfg = cfg
            .requests
            .iter()
            .find(|r| r.name == req_name)
            .ok_or_else(|| anyhow::anyhow!("request `{req_name}` not found"))?;

        run_single_request(conn, &client, api, req_cfg, &mut vars, format)?;
    }

    Ok(())
}

fn run_single_request(
    conn: &Connection,
    client: &Client,
    api: &crate::config::Api,
    request: &Request,
    vars: &mut Vars,
    format: &str,
) -> anyhow::Result<()> {
    println!("==> {}", request.name);

    // scripts BEFORE
    let scripts = &request.scripts;
    run_scripts_before(conn, &api.name, scripts, vars)?;

    let url = render_placeholders(&format!("{}{}", api.base_url, request.path), vars)?;

    let mut builder = client.request(request.method.parse()?, &url);

    if let Some(h) = &request.headers {
        let rendered = render_placeholders(h, vars)?;
        let map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&rendered)
            .context("invalid JSON in headers")?;
        for (k, v) in map {
            builder = builder.header(k, v.as_str().unwrap_or(&v.to_string()));
        }
    }

    if let Some(b) = &request.body {
        let rendered = render_placeholders(b, vars)?;
        builder = builder.body(rendered);
    }

    let resp = builder.send()?;
    let status = resp.status();
    let headers = resp.headers().clone();
    let text = resp.text()?;

    println!("Status: {}", status);
    println!("--- Response headers ---");
    for (k, v) in headers.iter() {
        println!("{}: {:?}", k, v);
    }

    println!("--- Response body ---");
    match format {
        "json" => {
            // Try to pretty print JSON, otherwise raw
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                println!("{}", text);
            }
        }
        "html" => {
            // Maybe just print raw; formatting can be added later
            println!("{}", text);
        }
        _ => {
            println!("{}", text);
        }
    }
    vars.insert("response_body".to_string(), text.clone());
    vars.insert("response_status".to_string(), status.as_u16().to_string());


    run_scripts_after(conn, &api.name, scripts, vars)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Api, ProjectConfig, Request, Script};
    use crate::storage::init_db;
    use crate::templating::Vars;
    use httpmock::prelude::*;
    use rusqlite::Connection;
    use std::collections::HashMap;

    #[test]
    fn run_single_request_get() {
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET).path("/docs");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(r#"{"message":"ok"}"#);
        });

        let api = Api {
            name: "test".into(),
            base_url: server.base_url(),
            scenarios: HashMap::new(),
        };

        let req = Request {
            name: "docs".into(),
            method: "GET".into(),
            path: "/docs".into(),
            headers: None,
            body: None,
            scripts: vec![],
        };

        let cfg = ProjectConfig {
            api,
            requests: vec![req],
        };

        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let vars = Vars::new();
        run_route(&conn, &cfg, "docs", vars, "json").unwrap();

        m.assert(); // ensure it was called
    }

    #[test]
    fn run_scenario_multiple_requests() {
        let server = MockServer::start();

        let m1 = server.mock(|when, then| {
            when.method(GET).path("/a");
            then.status(200).body("A");
        });

        let m2 = server.mock(|when, then| {
            when.method(GET).path("/b");
            then.status(200).body("B");
        });

        let mut scenarios = HashMap::new();
        scenarios.insert("scenario1".into(), vec!["first".into(), "second".into()]);

        let api = Api {
            name: "test".into(),
            base_url: server.base_url(),
            scenarios,
        };

        let r1 = Request {
            name: "first".into(),
            method: "GET".into(),
            path: "/a".into(),
            headers: None,
            body: None,
            scripts: vec![],
        };

        let r2 = Request {
            name: "second".into(),
            method: "GET".into(),
            path: "/b".into(),
            headers: None,
            body: None,
            scripts: vec![],
        };

        let cfg = ProjectConfig {
            api,
            requests: vec![r1, r2],
        };

        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let vars = Vars::new();
        run_route(&conn, &cfg, "scenario1", vars, "raw").unwrap();

        m1.assert();
        m2.assert();
    }
}

