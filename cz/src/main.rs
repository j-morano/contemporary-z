mod database;
mod data;
mod app;
mod config;
mod colors;
mod utils;
mod strings;
mod options;

use crate::database::{
    create_dirs_table_if_not_exist, create_current_dir_table_if_not_exist
};

use app::write_action;
use config::app_defaults_from_config;

use regex::Regex;
use rusqlite::{Connection, Result};
use std::env;
use std::path::Path;
use std::fs;
use crate::app::get_home_dir;
use crate::config::app_from_config;



fn main() -> Result<()> {
    // Collect command-line arguments 
    let args: Vec<_> = env::args().collect();

    // App configuration
    let app_defaults = app_defaults_from_config();
    let app = app_from_config();

    let mut database_dir_path = app.database_path.clone();
    // Replace typical environment variables
    let re = Regex::new(r"\$HOME").unwrap();
    let home_dir = env::var("HOME").unwrap();
    database_dir_path = String::from(
        re.replace_all(database_dir_path.as_str(), home_dir.clone())
        );
    // If config database not available, use default
    if !Path::new(database_dir_path.as_str()).exists() {
        database_dir_path = app_defaults.database_path.clone();
        database_dir_path = String::from(
            re.replace_all(database_dir_path.as_str(), home_dir.clone())
            );
    }

    // Create application user-specific data dir if it does not exist
    fs::create_dir_all(&database_dir_path).unwrap_or_else(
        |e| panic!("Error creating dir: {}", e));

    let database_file_path = format!(
        "{}{}", &database_dir_path, "directories.db");

    // Open connection with the database
    let conn = Connection::open(database_file_path)?;

    create_dirs_table_if_not_exist(&conn)?;
    create_current_dir_table_if_not_exist(&conn)?;

    write_action("empty", "".to_string());

    // If there is a dir argument, cd to the dir
    if args.len() > 1 {

        // Command option: clear table
        if args[1] == "--clear" {
            options::clear_database(&app, &conn)?;
        }
        // Command option: go to previous directory
        else if args[1] == "-" {
            options::go_to_previous_dir(&app, &conn);
        }
        // Command option: go to target directory
        else if args[1] == "=" {
            options::go_to_target_dir(&app, &conn);
        }
        // Command option: help
        else if args[1] == "--help" || args[1] == "-h" {
            options::show_help();
        }
        // Command option: run command
        else if args[1] == "-b" {
            options::opt_run_in_background(&app, &args);
        }
        // Command option: list directories
        else if args[1] == "-l" {
            options::list_dirs(&app, &conn, &args);
        }
        // Command option: interactive subdir selection
        else if args[1] == "-i" {
            options::interactive_select_dir(&app, &conn);
        }
        // Command option: remove directory
        else if args[1] == "-r" {
            options::opt_remove_dir(&app, &conn, &args);
        }
        // Command option: add alias
        else if args[1] == "-a" {
            options::add_alias(&app, &conn, &args);
        }
        // Command option: sync
        //  Remove directories which do not exist.
        else if args[1] == "--sync" {
            options::opt_sync_dirs(&app, &conn);
        }
        // Command option: list all dirs
        else if args[1] == "--list-all" {
            options::opt_list_all_dirs(&app, &conn);
        }
        else {
            options::do_cd(&app, &conn, &args);
        }

    } else { // if there is no argument, list stored dirs to select one
             //  interactively 
        
        options::interactive_cd(&app, &conn, &args);
    }
    Ok(())
}
