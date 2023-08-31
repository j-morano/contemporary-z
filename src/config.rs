use std::fs;
use crate::data::Directory;
use crate::get_home_dir;
use crate::app::App;
use crate::strings::DEFAULT_CONFIG;
use std::path::Path;
use std::env;
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

    let current_seconds = get_current_seconds();
    // Current seconds minus 2 months
    let limit = current_seconds - (60 * 60 * 24 * 30 * 2);
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
        if last_access < limit {
            continue;
        }
        let score = 10000.0 * counter as f64 * (3.75 / ((0.0001 * (current_seconds - last_access) as f64 + 1.0) + 0.25));
        let mut alias = String::from("");
        if dir_string.len() == 4 {
            // alias is optional
            alias = dir_string[3].to_string();
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




// The same with lifetimes
fn parse_option_string<'a>(option_value: &Option<&'a (String, String)>) -> Value<'a> {
    match option_value {
        Some(x) => {
            let option = &x.0;
            let value = &x.1;
            if option == "max_results" || option == "nav_start_number" {
                Value::I64(value.parse::<i64>().unwrap())
            } else if option == "abs_paths" || option == "compact_paths" {
                Value::Bool(value.parse::<bool>().unwrap())
            } else {
                Value::String(&value)
            }
        }
        None => {
            panic!("Option not found in default config");
        }
    }
}


fn get_option<'a>(user_value: &'a Vec<(String, String)>, default_value: &'a Vec<(String, String)>, option: &str) -> Value<'a> {
    // If the option is not in the user config, use the default value
    let mut option_value = user_value.iter().find(|&x| x.0 == option);
    let default_value = default_value.iter().find(|&x| x.0 == option);
    if option_value.is_none() {
        option_value = default_value;
    }
    let mut value = parse_option_string(&option_value);
    // If the option is "database_path", also check if the path exists. If it
    //  does not exist, use the default value.
    if option == "database_path" {
        let path = value.clone();
        if !fs::metadata(path.to_string()).is_ok() {
            value = parse_option_string(&default_value);
        }
    }
    value
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
    let mut database_path = database_path.clone();

    // Replace typical environment variables
    let home_dir = env::var("HOME").unwrap();
    database_path = database_path.replace("$HOME", &home_dir);
    // If config database not available, use default
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
        let config_option = config_option.split(" = ").collect::<Vec<&str>>();
        let option = config_option[0];
        let value = config_option[1];
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
