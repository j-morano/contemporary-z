use std::fs;
use toml::Value;
use crate::get_home_dir;
use crate::app::App;
use crate::strings::DEFAULT_CONFIG;


fn get_option(user_value: Value, default_value: Value, option: &str) -> Value {
    let mut value = user_value.get(option);
    if value == None {
        value = default_value.get(option);
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
) -> App {
    return App {
        theme: theme.as_str().unwrap().to_string(),
        abs_paths: abs_paths.as_bool().unwrap(),
        compact_paths: compact_paths.as_bool().unwrap(),
        max_results: max_results.as_integer().unwrap() as usize,
        database_path: database_path.as_str().unwrap().to_string(),
        substring: substring.as_str().unwrap().to_string(),
    }
}


pub(crate) fn app_defaults_from_config() -> App {
    let default_value = DEFAULT_CONFIG.to_string().parse::<Value>().unwrap();

    return build_app(
        default_value.get("theme").unwrap().clone(),
        default_value.get("abs_paths").unwrap().clone(),
        default_value.get("compact_paths").unwrap().clone(),
        default_value.get("max_results").unwrap().clone(),
        default_value.get("database_path").unwrap().clone(),
        default_value.get("substring").unwrap().clone()
    )
}


pub(crate) fn app_from_config() -> App {
    let path = format!("{}/.config/cz/cz.toml", get_home_dir());
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
        get_option(user_value.clone(), default_value.clone(), "substring")
    )
}
