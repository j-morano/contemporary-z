mod cli;
mod data;
mod app;
mod config;
mod colors;
mod utils;
mod strings;


use utils::write_dir;

use std::env;
use std::fs;
use std::process::exit;
use crate::cli::Command;
use crate::config::app_from_config;
use crate::strings::HELP;



fn main() {
    // Tell the shell not to cd anywhere unless a command says otherwise
    write_dir("".to_string());

    let args: Vec<String> = env::args().skip(1).collect();
    let command = match cli::parse(&args) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("cz: {}\nTry 'cz --help' for more information.", message);
            exit(2);
        }
    };

    // These do not need the database
    match command {
        Command::Help => {
            println!("{}", HELP);
            return;
        }
        Command::Version => {
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        _ => {}
    }

    let app = &mut app_from_config();
    match command {
        Command::Help | Command::Version => unreachable!(),
        Command::DatabasePath => println!("{}", app.config.database_path),
        Command::Clear => app.clear_database(),
        Command::Previous => app.go_to_previous(),
        Command::Last => app.go_to_last(),
        Command::List(max_num) => app.list_existent(max_num),
        Command::ListAll => app.list_all(),
        Command::Navigate { hidden, dirs_only } => app.interactive_navigation(hidden, dirs_only),
        Command::Remove(patterns) => app.remove_dirs(&patterns),
        Command::Alias(alias_args) => app.add_alias(alias_args),
        Command::RemoveAlias => app.remove_alias_interactive(),
        // Remove directories which do not exist
        Command::Sync => app.sync_dirs(),
        Command::Find(patterns) => app.list_matching_dirs(&patterns),
        Command::Cd { patterns, mode } => app.do_cd(&patterns, mode),
        // No arguments: list stored dirs to select one interactively
        Command::Select => app.interactive_cd(),
    }
    // Write dirs to database_dir_fn
    let db_string = config::serialize_database(&app.dirs);
    // Write atomically: a crash mid-write must not truncate the database.
    let tmp_path = format!("{}.tmp", app.config.database_path);
    fs::write(&tmp_path, db_string)
        .and_then(|_| fs::rename(&tmp_path, &app.config.database_path))
        .unwrap_or_else(|e| panic!("Error writing file: {}", e));
}
