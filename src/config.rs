use std::fs;
use crate::data::Directory;
use crate::get_home_dir;
use crate::app::App;
use crate::strings::DEFAULT_CONFIG;
use std::path::Path;
use crate::app::get_current_seconds;



fn init_dir_file(database_fn: String, dirs: &mut Vec<Directory>) {
    // println!("database_fn: {}", database_fn);
    // Create database_fn if it does not exist
    if !Path::new(database_fn.as_str()).exists() {
        let database_fn_parent = Path::new(database_fn.as_str()).parent().unwrap();
        fs::create_dir_all(database_fn_parent).unwrap_or_else(
            |e| panic!("Error creating dir: {}", e)
            );
        fs::write(database_fn.as_str(), "").unwrap_or_else(
            |e| panic!("Error creating file: {}", e)
            );
    }

    let db_string = fs::read_to_string(database_fn).unwrap();
    dirs.extend(parse_database(&db_string, get_current_seconds()));
}


/// Serialize directories to the database text format.
pub(crate) fn serialize_database(dirs: &[Directory]) -> String {
    let mut db_string = String::new();
    for dir in dirs {
        db_string.push_str(&format!("{}\n{}\n{}\n", dir.name, dir.counter, dir.last_access));
        if !dir.alias.is_empty() {
            db_string.push_str(&format!("{}\n", dir.alias));
        }
        db_string.push_str("---\n");
    }
    db_string
}


/// Parse the database text format (see `serialize_database`).
fn parse_database(db_string: &str, current_seconds: i64) -> Vec<Directory> {
    let mut dirs = Vec::new();
    // Current seconds minus 2 months
    let limit = current_seconds - (60 * 60 * 24 * 30 * 2);
    // Blocks are terminated by a line that is exactly "---". Malformed blocks
    // are skipped instead of aborting the program.
    let mut block: Vec<&str> = Vec::new();
    for line in db_string.lines().chain(std::iter::once("---")) {
        if line != "---" {
            block.push(line);
            continue;
        }
        let fields = std::mem::take(&mut block);
        if fields.len() < 3 {
            continue;
        }
        let (counter, last_access) = match (
            fields[1].parse::<i64>(),
            fields[2].parse::<i64>(),
        ) {
            (Ok(c), Ok(l)) => (c, l),
            _ => continue,
        };
        let alias = fields.get(3).map(|a| a.to_string()).unwrap_or_default();
        // Drop stale directories, but never aliased ones.
        if last_access < limit && alias.is_empty() {
            continue;
        }
        let score = 10000.0 * counter as f64 * (3.75 / ((0.0001 * (current_seconds - last_access) as f64 + 1.0) + 0.25));
        dirs.push(Directory {
            name: fields[0].to_string(),
            counter,
            last_access,
            score,
            alias,
        });
    }
    dirs
}

// Return either String, i64 or bool
enum Value<'a> {
    String(&'a str),
    I64(i64),
    Bool(bool),
}
// Implement Value to String
impl<'a> ToString for Value<'a> {
    fn to_string(&self) -> String {
        match self {
            Value::String(x) => { x.to_string() }
            Value::I64(x) => { x.to_string() }
            Value::Bool(x) => { x.to_string() }
        }
    }
}
// Method to_bool from Value
impl<'a> Value<'a> {
    fn to_bool(&self) -> bool {
        match self {
            Value::Bool(x) => { x.clone() }
            _ => { panic!("Value is not bool") }
        }
    }
}
impl<'a> Value<'a> {
    fn to_i64(&self) -> i64 {
        match self {
            Value::I64(x) => { x.clone() }
            _ => { panic!("Value is not i64") }
        }
    }
}
impl<'a> Clone for Value<'a> {
    fn clone(&self) -> Value<'a> {
        match self {
            Value::String(x) => { Value::String(x) }
            Value::I64(x) => { Value::I64(x.clone()) }
            Value::Bool(x) => { Value::Bool(x.clone()) }
        }
    }
}




fn parse_option_string<'a>(option_value: &Option<&'a (String, String)>) -> Option<Value<'a>> {
    let entry: &'a (String, String) = (*option_value)?;
    let (option, value) = (&entry.0, &entry.1);
    let value = value.trim();
    if option == "max_results" || option == "nav_start_number" {
        value.parse::<i64>().ok().filter(|v| *v >= 0).map(Value::I64)
    } else if option == "abs_paths" || option == "compact_paths" {
        value.parse::<bool>().ok().map(Value::Bool)
    } else {
        Some(Value::String(value))
    }
}


fn get_option<'a>(user_value: &'a Vec<(String, String)>, default_value: &'a Vec<(String, String)>, option: &str) -> Value<'a> {
    // Use the user value if present and valid; otherwise the default.
    let user = user_value.iter().find(|&x| x.0 == option);
    let default = default_value.iter().find(|&x| x.0 == option);
    if let Some(v) = parse_option_string(&user) {
        return v;
    }
    if user.is_some() {
        eprintln!("cz: invalid value for config option '{}', using default", option);
    }
    parse_option_string(&default).expect("Option not found in default config")
}


fn build_app(
    theme: String,
    abs_paths: bool,
    compact_paths: bool,
    max_results: i64,
    database_path: String,
    substring: String,
    show_files: String,
    nav_start_number: i64,
    dirs: &mut Vec<Directory>,
) -> App {
    // Replace typical environment variables
    let home_dir = get_home_dir();
    let database_path = database_path.replace("$HOME", &home_dir);
    // Avoid divisions by zero when paginating
    let max_results = max_results.max(1);
    // Create application user-specific data dir if it does not exist
    let database_file_parent = Path::new(database_path.as_str()).parent().unwrap();
    fs::create_dir_all(database_file_parent).unwrap_or_else(
        |e| panic!("Error creating dir: {}", e)
        );

    init_dir_file(database_path.clone(), dirs);
    let app = App {
        theme,
        abs_paths,
        compact_paths,
        max_results: max_results as usize,
        database_path,
        substring,
        show_files,
        nav_start_number: nav_start_number as usize,
        dirs,
    };
    return app;
}


fn parse_config(config_string: String) -> Vec<(String, String)> {
    /* File format:
     * theme = dark
     * max_results = 9
     * abs_paths = true
     * compact_paths = true
     * database_path = $HOME/.local/share/contemporary-z/directories.dir
     * substring = shortest
     * show_files = none
     * nav_start_number = 1
     */

    let mut option_values = Vec::new();
    let config_options = config_string.split("\n");
    for config_option in config_options {
        let config_option = config_option.trim();
        if config_option.len() == 0 {
            continue;
        }
        if config_option.starts_with('#') {
            continue;
        }
        let (option, value) = match config_option.split_once('=') {
            Some((o, v)) => (o.trim(), v.trim()),
            None => continue,
        };
        option_values.push((option.to_string(), value.to_string()));
    }
    return option_values;
}


pub(crate) fn app_from_config(dirs: &mut Vec<Directory>) -> App {
    let path = format!("{}/.config/contemporary-z/cz.conf", get_home_dir());
    let config_string = match fs::read_to_string(path) {
        Ok(contents) => { contents }
        Err(_) => { DEFAULT_CONFIG.to_string() }
    };
    let user_value = parse_config(config_string);
    let default_value = parse_config(DEFAULT_CONFIG.to_string());

    return build_app(
        get_option(&user_value, &default_value, "theme").to_string(),
        get_option(&user_value, &default_value, "abs_paths").to_bool(),
        get_option(&user_value, &default_value, "compact_paths").to_bool(),
        get_option(&user_value, &default_value, "max_results").to_i64(),
        get_option(&user_value, &default_value, "database_path").to_string(),
        get_option(&user_value, &default_value, "substring").to_string(),
        get_option(&user_value, &default_value, "show_files").to_string(),
        get_option(&user_value, &default_value, "nav_start_number").to_i64(),
        dirs,
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn dir(name: &str, counter: i64, last_access: i64, alias: &str) -> Directory {
        Directory {
            name: name.to_string(),
            counter,
            last_access,
            score: 0.0,
            alias: alias.to_string(),
        }
    }

    #[test]
    fn database_round_trip() {
        let dirs = vec![
            dir("/home/u/a", 3, NOW - 10, ""),
            dir("/home/u/b", 1, NOW - 20, "bee"),
        ];
        let parsed = parse_database(&serialize_database(&dirs), NOW);
        assert_eq!(parsed.len(), 2);
        for (p, d) in parsed.iter().zip(dirs.iter()) {
            assert_eq!(p.name, d.name);
            assert_eq!(p.counter, d.counter);
            assert_eq!(p.last_access, d.last_access);
            assert_eq!(p.alias, d.alias);
        }
        assert!(parsed[0].score > 0.0);
    }

    #[test]
    fn database_names_containing_separator() {
        let dirs = vec![dir("/home/u/my---proj", 2, NOW, "a-b")];
        let parsed = parse_database(&serialize_database(&dirs), NOW);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "/home/u/my---proj");
    }

    #[test]
    fn database_skips_malformed_blocks() {
        let db = format!("/a\nnot_a_number\n5\n---\n/b\n---\n/c\n1\n{}\n---\n", NOW);
        let parsed = parse_database(&db, NOW);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "/c");
        assert!(parse_database("", NOW).is_empty());
    }

    #[test]
    fn database_prunes_stale_but_keeps_aliased() {
        let old = NOW - 60 * 60 * 24 * 90;
        let dirs = vec![dir("/old", 1, old, ""), dir("/old_alias", 1, old, "x")];
        let parsed = parse_database(&serialize_database(&dirs), NOW);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "/old_alias");
    }

    #[test]
    fn config_skips_comments_and_bad_lines() {
        let opts = parse_config(
            "# comment\n\ntheme = bright\nnonsense\nmax_results=5\n".to_string()
        );
        assert_eq!(
            opts,
            vec![
                ("theme".to_string(), "bright".to_string()),
                ("max_results".to_string(), "5".to_string()),
            ]
        );
    }

    #[test]
    fn config_user_values_override_defaults() {
        let user = parse_config("theme = bright\nmax_results = 4".to_string());
        let defaults = parse_config(DEFAULT_CONFIG.to_string());
        assert_eq!(get_option(&user, &defaults, "theme").to_string(), "bright");
        assert_eq!(get_option(&user, &defaults, "max_results").to_i64(), 4);
        // Not set by the user: default
        assert!(get_option(&user, &defaults, "abs_paths").to_bool());
    }

    #[test]
    fn config_invalid_values_fall_back_to_default() {
        let user = parse_config("max_results = abc\nabs_paths = maybe\nnav_start_number = -3".to_string());
        let defaults = parse_config(DEFAULT_CONFIG.to_string());
        assert_eq!(get_option(&user, &defaults, "max_results").to_i64(), 9);
        assert!(get_option(&user, &defaults, "abs_paths").to_bool());
        assert_eq!(get_option(&user, &defaults, "nav_start_number").to_i64(), 1);
    }

    #[test]
    fn default_config_is_complete() {
        let defaults = parse_config(DEFAULT_CONFIG.to_string());
        for opt in ["theme", "abs_paths", "compact_paths", "max_results",
                    "database_path", "substring", "show_files", "nav_start_number"] {
            assert!(defaults.iter().any(|(o, _)| o == opt), "missing {}", opt);
        }
    }
}
