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

use rusqlite::{Connection, Result};
use std::env;
use std::fs;
use crate::app::get_home_dir;
use crate::config::app_from_config;
use crate::app::current_seconds;



fn main() -> Result<()> {
    // Collect command-line arguments
    let args: Vec<_> = env::args().collect();

    // Initialize dirs and app
    let dirs = &mut Vec::new();
    let app = &mut app_from_config(dirs);

    // mutable reference dirs
    app.remove_old();
    app.compute_score(current_seconds());


    // Open connection with the database
    // let conn = Connection::open(app.database_path.clone())?;

    // create_dirs_table_if_not_exist(&conn)?;
    // create_current_dir_table_if_not_exist(&conn)?;
    // remove_old_dirs(&conn)?;

    write_dir("".to_string());

    // If there is a dir argument, cd to the dir
    if args.len() > 1 {
        if args[1] == "-v" || args[1] == "--version" {
            println!("{} {}", "Version:", env!("CARGO_PKG_VERSION"));
        }
        else if args[1] == "--clear" {
            // options::clear_database(&app, &conn)?;
        }
        else if args[1] == "-" {
            // options::go_to_previous_dir(&app, &conn);
        }
        else if args[1] == "=" {
            // options::go_to_target_dir(&app, &conn);
        }
        else if args[1] == "--help" || args[1] == "-h" {
            options::show_help();
        }
        // Command option: list directories
        else if args[1] == "-l" {
            // options::list_dirs(&app, &conn, &args);
        }
        else if args[1] == "-i" {
            // Command option: interactive subdir selection
            // options::interactive_navigation(&app, &conn, false, false);
        }
        else if args[1] == "--id" {
            // Command option: interactive subdir selection
            // options::interactive_navigation(&app, &conn, false, true);
        }
        else if args[1] == "--ih" {
            // Interactive subdir selection (including hidden)
            // options::interactive_navigation(&app, &conn, true, false);
        }
        else if args[1] == "-r" {
            // options::opt_remove_dirs(&app, &conn, &args);
        }
        else if args[1] == "-a" {
            app.add_alias(&args);
        }
        else if args[1] == "--remove-alias" {
            // options::remove_alias_interactive(&app, &conn)
        }
        else if args[1] == "--sync" {
            //  Remove directories which do not exist.
            // options::opt_sync_dirs(&app, &conn);
        }
        else if args[1] == "--list-all" {
            // options::opt_list_all_dirs(&app, &conn);
            app.list_all();
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
        // options::interactive_cd(&app, &conn, &args);
    }
    // Write dirs to database_dir_fn
    let mut db_string = String::new();
    for dir in app.dirs.iter() {
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
    fs::write(app.database_path.clone(), db_string).unwrap_or_else(
        |e| panic!("Error writing file: {}", e)
        );
    Ok(())
}
