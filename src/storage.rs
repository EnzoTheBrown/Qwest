use rusqlite::{params, Connection};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Variable {
    pub label: String,
    pub value: String,
    pub project: Option<String>,
}

pub fn db_path() -> anyhow::Result<PathBuf> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("could not find local data dir"))?
        .join(".qwest");
    Ok(base.join("qwest.sqlite"))
}

pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS variables (
            label   TEXT NOT NULL,
            value   TEXT NOT NULL,
            project TEXT NULL
        );
        "#,
    )
}

/// Insert or update variable.
pub fn set_variable(
    conn: &Connection,
    label: &str,
    value: &str,
    project: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        r#"
        DELETE FROM variables WHERE label = ?1 AND project IS ?2
            OR label = ?1 AND project = ?2;
        "#,
        params![label, project],
    )?;
    conn.execute(
        r#"
        INSERT INTO variables (label, value, project)
        VALUES (?1, ?2, ?3);
        "#,
        params![label, value, project],
    )?;
    Ok(())
}

/// Load variables for a given project.
/// Returns (global, project-specific)
pub fn load_variables(
    conn: &Connection,
    project: &str,
) -> rusqlite::Result<(Vec<Variable>, Vec<Variable>)> {
    let mut stmt = conn.prepare(
        "SELECT label, value, project FROM variables WHERE project IS NULL",
    )?;
    let global = stmt
        .query_map([], |row| {
            Ok(Variable {
                label: row.get(0)?,
                value: row.get(1)?,
                project: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT label, value, project FROM variables WHERE project = ?1",
    )?;
    let project_vars = stmt
        .query_map([project], |row| {
            Ok(Variable {
                label: row.get(0)?,
                value: row.get(1)?,
                project: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok((global, project_vars))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn set_and_load_global_and_project_variables() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        // global
        set_variable(&conn, "token", "global-token", None).unwrap();
        // project-specific
        set_variable(&conn, "token", "project-token", Some("my_project")).unwrap();
        set_variable(&conn, "url", "https://example.com", Some("my_project")).unwrap();

        let (global, project) = load_variables(&conn, "my_project").unwrap();

        assert_eq!(global.len(), 1);
        assert_eq!(global[0].label, "token");
        assert_eq!(global[0].value, "global-token");
        assert!(global[0].project.is_none());

        assert_eq!(project.len(), 2);
        let token = project.iter().find(|v| v.label == "token").unwrap();
        assert_eq!(token.value, "project-token");
        assert_eq!(token.project.as_deref(), Some("my_project"));

        let url = project.iter().find(|v| v.label == "url").unwrap();
        assert_eq!(url.value, "https://example.com");
    }

    #[test]
    fn set_overwrites_existing() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        set_variable(&conn, "token", "first", Some("project")).unwrap();
        set_variable(&conn, "token", "second", Some("project")).unwrap();

        let (_g, project) = load_variables(&conn, "project").unwrap();
        assert_eq!(project.len(), 1);
        assert_eq!(project[0].value, "second");
    }
}

