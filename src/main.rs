mod database;
mod data;
mod app;
mod config;
mod colors;
mod utils;
mod strings;
mod options;

use crate::database::{
    create_dirs_table_if_not_exist,
    create_current_dir_table_if_not_exist,
    remove_old_dirs
};

use utils::write_dir;
use config::app_defaults_from_config;

use regex::Regex;
use rusqlite::{Connection, Result};
use std::env;
use std::path::Path;
use std::fs;
use crate::app::get_home_dir;
use crate::config::app_from_config;
use crate::data::Directory;
use crate::app::current_seconds;



fn init_dir_file(database_fn: String) -> Vec<Directory> {
    println!("database_fn: {}", database_fn);
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
    println!("db_string: {}", db_string);
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
    let mut dirs = Vec::new();
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
    dirs
}


fn main() -> Result<()> {
    // Collect command-line arguments
    let args: Vec<_> = env::args().collect();

    let def_dirs = &mut Vec::new();
    // App configuration
    let app_defaults = app_defaults_from_config(def_dirs);
    let dirs = &mut Vec::new();
    let app = &mut app_from_config(dirs);

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
    let database_dir_fn = database_path.clone().replace(".db", ".dir");
    let dirs = &mut init_dir_file(database_dir_fn.clone());
    // mutable reference dirs
    app.remove_old();
    app.compute_score(current_seconds());

    app.dirs = dirs;

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
            app.add_alias(&args);
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
            app.list_matching_dirs(&args);
        }
        else if args[1] == "-t" {
            app.do_cd(&args, "shortest");
        }
        else if args[1] == "-e" {
            app.do_cd(&args, "score");
        }
        else {
            // options::do_cd(app, &args, "none");
            app.do_cd(&args, "none");
        }
    } else {
        // If there is no argument, list stored dirs to select one interactively
        options::interactive_cd(&app, &conn, &args);
    }
    // Write dirs to database_dir_fn
    let mut db_string = String::new();
    for dir in dirs.iter() {
        db_string.push_str(&format!("{}\n", dir.name));
        db_string.push_str(&format!("{}\n", dir.counter));
        db_string.push_str(&format!("{}\n", dir.last_access));
        // Score must be a float in the format x.y
        db_string.push_str(&format!("{:.10}\n", dir.score));
        if dir.alias.len() > 0 {
            db_string.push_str(&format!("{}\n", dir.alias));
        }
        db_string.push_str("---\n");
    }
    fs::write(database_dir_fn, db_string).unwrap_or_else(
        |e| panic!("Error writing file: {}", e)
    );
    Ok(())
}
