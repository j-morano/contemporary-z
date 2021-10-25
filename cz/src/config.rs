use std::fs;
use toml::Value;
use crate::{get_home_dir};
use crate::app::App;


const DEFAULT_CONFIG: &str = "
theme = 'dark'
max_results = 9
abs_paths = true
";


// fn config(option: &str) -> Value {
//     let path = format!("{}/.config/cz.toml", get_home_dir());
//     let config_string = match fs::read_to_string(path) {
//         Ok(contents) => {contents}
//         Err(_) => { DEFAULT_CONFIG.to_string() }
//     };
//     let user_value = config_string.parse::<Value>().unwrap();
//     let default_value = DEFAULT_CONFIG.to_string().parse::<Value>().unwrap();
//
//     let mut value = user_value.get(option);
//     if value == None {
//         value = default_value.get(option);
//     }
//
//     if value == None {
//         show_error("Invalid option", option);
//         Value::String("".to_string())
//     } else {
//         value.unwrap().clone()
//     }
// }

// pub(crate) fn max_results() -> usize {
//     config("max_results").as_integer().unwrap() as usize
// }
//
// pub(crate) fn theme() -> String {
//     config("theme").to_string()
// }
//
// pub(crate) fn abs_paths() -> bool {
//     config("abs_paths").as_bool().unwrap()
// }
//

fn get_option(user_value: Value, default_value: Value, option: &str) -> Value {
    let mut value = user_value.get(option);
    if value == None {
        value = default_value.get(option);
    }
    value.unwrap().clone()
}

pub(crate) fn app_from_config() -> App {
    let path = format!("{}/.config/cz.toml", get_home_dir());
    let config_string = match fs::read_to_string(path) {
        Ok(contents) => {contents}
        Err(_) => { DEFAULT_CONFIG.to_string() }
    };
    let user_value = config_string.parse::<Value>().unwrap();
    let default_value = DEFAULT_CONFIG.to_string().parse::<Value>().unwrap();


    let theme = get_option(
        user_value.clone(), default_value.clone(), "theme");
    let abs_paths = get_option(
        user_value.clone(), default_value.clone(), "abs_paths");
    let max_results = get_option(
        user_value.clone(), default_value.clone(), "max_results");

    App {
        theme: theme.to_string(),
        abs_paths: abs_paths.as_bool().unwrap(),
        max_results: max_results.as_integer().unwrap() as usize
    }

}