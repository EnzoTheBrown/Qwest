use crate::templating::Vars;
use crate::storage::set_variable;
use anyhow::Context;
use rhai::{Dynamic, Engine, Map};
use rusqlite::Connection;

pub fn run_scripts_before(
    conn: &Connection,
    project: &str,
    scripts: &[crate::config::Script],
    vars: &mut Vars,
) -> anyhow::Result<()> {
    run_scripts(conn, project, scripts.iter().filter(|s| s.before), vars)
}

pub fn run_scripts_after(
    conn: &Connection,
    project: &str,
    scripts: &[crate::config::Script],
    vars: &mut Vars,
) -> anyhow::Result<()> {
    run_scripts(conn, project, scripts.iter().filter(|s| !s.before), vars)
}

fn run_scripts<'a, I>(
    conn: &Connection,
    project: &str,
    scripts: I,
    vars: &mut Vars,
) -> anyhow::Result<()>
where
    I: Iterator<Item = &'a crate::config::Script>,
{
    let engine = Engine::new();

    for script in scripts {
        if let Some(desc) = &script.description {
            println!(">> {}", desc);
        }

        let mut scope = rhai::Scope::new();
        for (k, v) in vars.iter() {
            scope.push(k.as_str(), v.clone());
        }

        // 👉 Key change: map Rhai error to String, then into anyhow
        let result = engine
            .eval_with_scope::<Dynamic>(&mut scope, &script.script)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        // If result is a map, convert keys to variables and persist them
        if let Some(map) = result.clone().try_cast::<Map>() {
            for (k, v) in map.into_iter() {
                let val = v.to_string();
                vars.insert(k.to_string(), val.clone());
                set_variable(conn, &k, &val, Some(project))
                    .with_context(|| format!("failed to persist script variable `{k}`"))?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Script;
    use crate::storage::{init_db, load_variables};
    use crate::templating::Vars;
    use rusqlite::Connection;

    #[test]
    fn script_can_store_variables_in_db() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let script = Script {
            before: true,
            script: r#"return #{ token: "abc123", answer: 42 };"#.into(),
            description: None,
        };

        let mut vars = Vars::new();

        run_scripts_before(&conn, "my_project", &[script], &mut vars).unwrap();

        // in-memory vars
        assert_eq!(vars.get("token").unwrap(), "abc123");
        assert_eq!(vars.get("answer").unwrap(), "42"); // to_string()

        // DB should also contain project vars
        let (_global, proj) = load_variables(&conn, "my_project").unwrap();
        let token = proj.iter().find(|v| v.label == "token").unwrap();
        assert_eq!(token.value, "abc123");
    }

    #[test]
    fn script_can_access_existing_vars() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let script = Script {
            before: true,
            script: r#"
                // existing var 'name' is available in scope
                return #{ greeting: "Hello, " + name };
            "#
            .into(),
            description: None,
        };

        let mut vars = Vars::new();
        vars.insert("name".into(), "Enzo".into());

        run_scripts_before(&conn, "proj", &[script], &mut vars).unwrap();
        assert_eq!(vars.get("greeting").unwrap(), "Hello, Enzo");
    }
}

