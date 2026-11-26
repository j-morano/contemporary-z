mod cli;
mod app;
mod config;
mod db;
mod colors;
mod utils;
mod strings;
mod ui;


use utils::write_dir;

use std::env;
use std::process::exit;
use crate::cli::Command;
use crate::app::App;
use crate::config::Config;
use crate::db::Database;
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

    let config = Config::load();
    let db = Database::load(&config.database_path);
    let app = &mut App::new(config, db);
    match command {
        Command::Help | Command::Version => unreachable!(),
        Command::DatabasePath => app.show_database_path(),
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
    app.save();
}
