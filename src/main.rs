mod data;
mod app;
mod config;
mod colors;
mod utils;
mod strings;


use utils::write_dir;

use std::env;
use std::fs;
use crate::config::{app_from_config, Mode};
use crate::strings::HELP;



fn main() {
    // Collect command-line arguments
    let args: Vec<_> = env::args().collect();

    // Initialize dirs and app
    let app = &mut app_from_config();

    write_dir("".to_string());

    // If there is a dir argument, cd to the dir
    if args.len() > 1 {
        if args[1] == "-v" || args[1] == "--version" {
            println!("{} {}", "Version:", env!("CARGO_PKG_VERSION"));
        }
        else if args[1] == "--database-path" {
            println!("{}", app.config.database_path);
        }
        else if args[1] == "--clear" {
            app.clear_database();
        }
        else if args[1] == "-" {
            app.go_to_previous();
        }
        else if args[1] == "=" {
            app.go_to_last();
        }
        else if args[1] == "--help" || args[1] == "-h" {
            println!("{}", HELP);
        }
        // Command option: list directories
        else if args[1] == "-l" {
            app.list_existent(&args);
        }
        else if args[1] == "-i" {
            // Command option: interactive subdir selection
            app.interactive_navigation(false, false);
        }
        else if args[1] == "--id" {
            // Command option: interactive subdir selection
            app.interactive_navigation(false, true);
        }
        else if args[1] == "--ih" {
            // Interactive subdir selection (including hidden)
            app.interactive_navigation(true, false);
        }
        else if args[1] == "-r" {
            app.remove_dirs(&args);
        }
        else if args[1] == "-a" {
            app.add_alias(&args);
        }
        else if args[1] == "--remove-alias" {
            app.remove_alias_interactive();
        }
        else if args[1] == "--sync" {
            //  Remove directories which do not exist.
            app.sync_dirs();
        }
        else if args[1] == "--list-all" {
            // options::opt_list_all_dirs(&app, &conn);
            app.list_all();
        }
        else if args[1] == "-f" {
            app.list_matching_dirs(&args);
        }
        else if args[1] == "-t" {
            app.do_cd(&args[2..], Some(Mode::Shortest));
        }
        else if args[1] == "-e" {
            app.do_cd(&args[2..], Some(Mode::Score));
        }
        else if args[1] == "-b" {
            app.do_cd(&args[2..], Some(Mode::Basename));
        }
        else {
            app.do_cd(&args[1..], None);
        }
    } else {
        // If there is no argument, list stored dirs to select one interactively
        app.interactive_cd(&args);
    }
    // Write dirs to database_dir_fn
    let db_string = config::serialize_database(&app.dirs);
    // Write atomically: a crash mid-write must not truncate the database.
    let tmp_path = format!("{}.tmp", app.config.database_path);
    fs::write(&tmp_path, db_string)
        .and_then(|_| fs::rename(&tmp_path, &app.config.database_path))
        .unwrap_or_else(|e| panic!("Error writing file: {}", e));
}
