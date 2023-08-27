mod database;
mod data;
mod app;
mod config;
mod colors;
mod utils;
mod strings;
mod options;
mod directories;

use crate::database::{
    create_dirs_table_if_not_exist,
    create_current_dir_table_if_not_exist,
    remove_old_dirs
};

use app::write_dir;
use config::app_defaults_from_config;

use toml::Value;
use regex::Regex;
use rusqlite::{Connection, Result};
use std::env;
use std::path::Path;
use std::fs;
use crate::app::get_home_dir;
use crate::config::app_from_config;




use crate::data::Directory;


fn init_toml_file(database_toml_fn: String) -> Vec<Directory> {
    println!("database_toml_fn: {}", database_toml_fn);
    // Create database_toml_fn if it does not exist
    if !Path::new(database_toml_fn.as_str()).exists() {
        let database_toml_parent = Path::new(database_toml_fn.as_str()).parent().unwrap();
        fs::create_dir_all(database_toml_parent).unwrap_or_else(
            |e| panic!("Error creating dir: {}", e)
        );
        fs::write(database_toml_fn.as_str(), "").unwrap_or_else(
            |e| panic!("Error creating file: {}", e)
        );
    }
    // Read database_toml_fn and parse it
    let db_string = fs::read_to_string(database_toml_fn).unwrap();
    println!("db_string: {}", db_string);
    /* The string is like this:
     * [[dir]]
     * name = "dir1"
     * counter = 1
     * last_access = 1234567890
     * score = 0.0
     * alias = "d1"
     *
     * [[dir]]
     * ...
    */
    let db = db_string.parse::<Value>().unwrap();
    let mut dirs = Vec::new();
    for dir in db.as_table().unwrap().get("dir").unwrap().as_array().unwrap() {
        let name = dir.get("name").unwrap().as_str().unwrap().to_string();
        let counter = dir.get("counter").unwrap().as_integer().unwrap() as i64;
        let last_access = dir.get("last_access").unwrap().as_integer().unwrap() as i64;
        let score = dir.get("score").unwrap().as_float().unwrap();
        let alias = dir.get("alias").unwrap().as_str().unwrap().to_string();
        let dir = Directory {
            name,
            counter,
            last_access,
            score,
            alias,
        };
        dirs.push(dir);
    }

    dirs
}

fn main() -> Result<()> {
    // Collect command-line arguments
    let args: Vec<_> = env::args().collect();

    // App configuration
    let app_defaults = app_defaults_from_config();
    let app = app_from_config();

    let mut database_path = app.database_path.clone();
    // Replace typical environment variables
    let re = Regex::new(r"\$HOME").unwrap();
    let home_dir = env::var("HOME").unwrap();
    database_path = String::from(
        re.replace_all(database_path.as_str(), home_dir.clone())
        );
    // If config database not available, use default
    if !Path::new(database_path.as_str()).parent().unwrap().exists() {
        database_path = app_defaults.database_path.clone();
        database_path = String::from(
            re.replace_all(database_path.as_str(), home_dir.clone())
        );
        // Create application user-specific data dir if it does not exist
        let database_file_parent = Path::new(database_path.as_str()).parent().unwrap();
        fs::create_dir_all(database_file_parent).unwrap_or_else(
            |e| panic!("Error creating dir: {}", e)
        );
    }
    // The same as database_path but as toml file, not db
    let database_toml_fn = database_path.clone().replace(".db", ".toml");
    let dirs = &mut init_toml_file(database_toml_fn.clone());
    // mutable reference dirs
    directories::remove_old(dirs);

    // Open connection with the database
    let conn = Connection::open(database_path)?;

    create_dirs_table_if_not_exist(&conn)?;
    create_current_dir_table_if_not_exist(&conn)?;
    remove_old_dirs(&conn)?;

    write_dir("".to_string());

    // If there is a dir argument, cd to the dir
    if args.len() > 1 {
        if args[1] == "-v" || args[1] == "--version" {
            println!("{} {}", "Version:", env!("CARGO_PKG_VERSION"));
        }
        else if args[1] == "--clear" {
            options::clear_database(&app, &conn)?;
        }
        else if args[1] == "-" {
            options::go_to_previous_dir(&app, &conn);
        }
        else if args[1] == "=" {
            options::go_to_target_dir(&app, &conn);
        }
        else if args[1] == "--help" || args[1] == "-h" {
            options::show_help();
        }
        // Command option: list directories
        else if args[1] == "-l" {
            options::list_dirs(&app, &conn, &args);
        }
        else if args[1] == "-i" {
            // Command option: interactive subdir selection
            options::interactive_navigation(&app, &conn, false, false);
        }
        else if args[1] == "--id" {
            // Command option: interactive subdir selection
            options::interactive_navigation(&app, &conn, false, true);
        }
        else if args[1] == "--ih" {
            // Interactive subdir selection (including hidden)
            options::interactive_navigation(&app, &conn, true, false);
        }
        else if args[1] == "-r" {
            options::opt_remove_dirs(&app, &conn, &args);
        }
        else if args[1] == "-a" {
            options::add_alias(&app, &conn, &args);
        }
        else if args[1] == "--remove-alias" {
            options::remove_alias_interactive(&app, &conn)
        }
        else if args[1] == "--sync" {
            //  Remove directories which do not exist.
            options::opt_sync_dirs(&app, &conn);
        }
        else if args[1] == "--list-all" {
            options::opt_list_all_dirs(&app, &conn);
        }
        else if args[1] == "-f" {
            options::list_matching_dirs(&app, &conn, &args);
        }
        else if args[1] == "-t" {
            options::do_cd(&app, &conn, &args, "shortest", dirs);
        }
        else if args[1] == "-e" {
            options::do_cd(&app, &conn, &args, "score", dirs);
        }
        else {
            options::do_cd(&app, &conn, &args, "none", dirs);
        }
    } else {
        // If there is no argument, list stored dirs to select one interactively
        options::interactive_cd(&app, &conn, &args);
    }
    // Write dirs to database_toml_fn
    let mut db_string = String::new();
    for dir in dirs.iter() {
        db_string.push_str("[[dir]]\n");
        db_string.push_str(&format!("name = \"{}\"\n", dir.name));
        db_string.push_str(&format!("counter = {}\n", dir.counter));
        db_string.push_str(&format!("last_access = {}\n", dir.last_access));
        // Score must be a float in the format x.y
        db_string.push_str(&format!("score = {:.10}\n", dir.score));
        db_string.push_str(&format!("alias = \"{}\"\n", dir.alias));
        db_string.push_str("\n");
    }
    fs::write(database_toml_fn, db_string).unwrap_or_else(
        |e| panic!("Error writing file: {}", e)
    );
    Ok(())
}
