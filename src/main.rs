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

use app::write_dir;
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
            options::do_cd(&app, &conn, &args, "shortest");
        }
        else if args[1] == "-e" {
            options::do_cd(&app, &conn, &args, "score");
        }
        else {
            options::do_cd(&app, &conn, &args, "none");
        }
    } else {
        // If there is no argument, list stored dirs to select one interactively
        options::interactive_cd(&app, &conn, &args);
    }
    Ok(())
}
