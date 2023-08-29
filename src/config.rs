use std::fs;
use toml::Value;
use crate::data::Directory;
use crate::get_home_dir;
use crate::app::App;
use crate::strings::DEFAULT_CONFIG;
use std::path::Path;
use std::env;
use regex::Regex;



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
    // Read database_fn and parse it
    let db_string = fs::read_to_string(database_fn).unwrap();
    // println!("db_string:\n{}", db_string);
    /* The string is like this:
     * name1
     * counter1
     * last_access1
     * score1
     * alias1 // alias is optional
     * ---
     * name2
     * counter2
     * last_access2
     * score2
     * ---
     * ...
     */
    let dir_strings = db_string.split("---");
    for dir_string in dir_strings {
        let dir_string = dir_string.trim();
        if dir_string.len() == 0 {
            continue;
        }
        let dir_string = dir_string.split("\n").collect::<Vec<&str>>();
        let name = dir_string[0].to_string();
        let counter = dir_string[1].parse::<i64>().unwrap();
        let last_access = dir_string[2].parse::<i64>().unwrap();
        let score = dir_string[3].parse::<f64>().unwrap();
        let mut alias = String::from("");
        if dir_string.len() == 5 {
            // alias is optional
            alias = dir_string[4].to_string();
        }
        let dir = Directory {
            name,
            counter,
            last_access,
            score,
            alias,
        };
        dirs.push(dir);
    }
}


fn get_option(user_value: Value, default_value: Value, option: &str) -> Value {
    let mut value = user_value.get(option);
    // Check if value is None
    if value == None {
        value = default_value.get(option);
    }
    // If the option is "database_path", also check if the path exists. If it
    //  does not exist, use the default value.
    else if option == "database_path" {
        let path = value.unwrap().as_str().unwrap();
        if !fs::metadata(path).is_ok() {
            value = default_value.get(option);
        }
    }
    value.unwrap().clone()
}


fn build_app(
    theme: Value,
    abs_paths: Value,
    compact_paths: Value,
    max_results: Value,
    database_path: Value,
    substring: Value,
    show_files: Value,
    nav_start_number: Value,
    dirs: &mut Vec<Directory>,
) -> App {
    let mut database_path = database_path.as_str().unwrap().to_string();

    // Replace typical environment variables
    let re = Regex::new(r"\$HOME").unwrap();
    let home_dir = env::var("HOME").unwrap();
    database_path = String::from(
        re.replace_all(database_path.as_str(), home_dir.clone())
        );
    // If config database not available, use default
    // Create application user-specific data dir if it does not exist
    let database_file_parent = Path::new(database_path.as_str()).parent().unwrap();
    fs::create_dir_all(database_file_parent).unwrap_or_else(
        |e| panic!("Error creating dir: {}", e)
        );
    // The same as database_path but as toml file, not db
    // let database_dir_fn = database_path.clone().replace(".db", ".dir");

    init_dir_file(database_path.clone(), dirs);
    let app = App {
        theme: theme.as_str().unwrap().to_string(),
        abs_paths: abs_paths.as_bool().unwrap(),
        compact_paths: compact_paths.as_bool().unwrap(),
        max_results: max_results.as_integer().unwrap() as usize,
        database_path,
        substring: substring.as_str().unwrap().to_string(),
        show_files: show_files.as_str().unwrap().to_string(),
        nav_start_number: nav_start_number.as_integer().unwrap() as usize,
        dirs,
    };
    return app;
}


pub(crate) fn app_from_config(dirs: &mut Vec<Directory>) -> App {
    let path = format!("{}/.config/contemporary-z/cz.toml", get_home_dir());
    let config_string = match fs::read_to_string(path) {
        Ok(contents) => { contents }
        Err(_) => { DEFAULT_CONFIG.to_string() }
    };
    let user_value = config_string.parse::<Value>().unwrap();
    let default_value = DEFAULT_CONFIG.to_string().parse::<Value>().unwrap();

    return build_app(
        get_option(user_value.clone(), default_value.clone(), "theme"),
        get_option(user_value.clone(), default_value.clone(), "abs_paths"),
        get_option(user_value.clone(), default_value.clone(), "compact_paths"),
        get_option(user_value.clone(), default_value.clone(), "max_results"),
        get_option(user_value.clone(), default_value.clone(), "database_path"),
        get_option(user_value.clone(), default_value.clone(), "substring"),
        get_option(user_value.clone(), default_value.clone(), "show_files"),
        get_option(user_value.clone(), default_value.clone(), "nav_start_number"),
        dirs,
    )
}
