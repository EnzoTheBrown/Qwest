use std::collections::HashMap;
use anyhow::Context;
use regex::Regex;

pub type Vars = HashMap<String, String>;

pub fn merge_vars(
    env_file_vars: Vars,
    global_vars: Vars,
    project_vars: Vars,
    cli_vars: Vars,
) -> Vars {
    let mut merged = Vars::new();

    // lowest precedence
    merged.extend(env_file_vars);
    merged.extend(global_vars);
    merged.extend(project_vars);
    merged.extend(cli_vars); // highest precedence

    merged
}

/// Replace ${var} placeholders in strings.
pub fn render_placeholders(input: &str, vars: &Vars) -> anyhow::Result<String> {
    let re = Regex::new(r"\$\{([A-Za-z0-9_]+)\}")?;
    let result = re.replace_all(input, |caps: &regex::Captures| {
        let key = &caps[1];
        vars.get(key).cloned().unwrap_or_else(|| caps[0].to_string())
    });
    Ok(result.into_owned())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_vars_precedence() {
        let mut env_file = Vars::new();
        env_file.insert("A".into(), "env".into());
        env_file.insert("B".into(), "env".into());

        let mut global = Vars::new();
        global.insert("B".into(), "global".into());
        global.insert("C".into(), "global".into());

        let mut project = Vars::new();
        project.insert("C".into(), "project".into());
        project.insert("D".into(), "project".into());

        let mut cli = Vars::new();
        cli.insert("D".into(), "cli".into());
        cli.insert("E".into(), "cli".into());

        let merged = merge_vars(env_file, global, project, cli);

        assert_eq!(merged.get("A").unwrap(), "env");
        assert_eq!(merged.get("B").unwrap(), "global");
        assert_eq!(merged.get("C").unwrap(), "project");
        assert_eq!(merged.get("D").unwrap(), "cli");
        assert_eq!(merged.get("E").unwrap(), "cli");
    }

    #[test]
    fn render_placeholders_replaces_known() {
        let mut vars = Vars::new();
        vars.insert("name".into(), "Enzo".into());
        vars.insert("token".into(), "1234".into());

        let input = "Hello ${name}, token=${token}";
        let out = render_placeholders(input, &vars).unwrap();
        assert_eq!(out, "Hello Enzo, token=1234");
    }

    #[test]
    fn render_placeholders_keeps_unknown() {
        let vars = Vars::new();
        let input = "Hello ${name}";
        let out = render_placeholders(input, &vars).unwrap();
        // unknown placeholder stays as-is
        assert_eq!(out, "Hello ${name}");
    }
}

