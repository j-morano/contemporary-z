mod cli;
mod app;
mod config;
mod db;
mod lock;
mod colors;
mod utils;
mod strings;
mod ui;


use utils::write_dir;

use std::env;
use std::process::exit;
use std::time::Duration;
use crate::cli::Command;
use crate::app::App;
use crate::config::Config;
use crate::db::Database;
use crate::lock::Lock;
use crate::strings::HELP;



fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = match cli::parse(&args) {
        Ok(command) => command,
        Err(message) => {
            write_dir("".to_string());
            eprintln!("cz: {}\nTry 'cz --help' for more information.", message);
            exit(2);
        }
    };

    // Tell the shell not to cd anywhere unless a command says otherwise.
    // `--add` runs from shell hooks, possibly while another shell is waiting
    // for the cd target of its own command: it must not touch the file.
    if !matches!(command, Command::Add(_)) {
        write_dir("".to_string());
    }

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

    // Hold the lock from loading to saving. A hook must never delay the
    // shell for long, so it gives up quickly (losing one visit).
    let is_hook = matches!(command, Command::Add(_));
    let timeout = Duration::from_millis(if is_hook { 200 } else { 5000 });
    let _lock = match Lock::acquire(&config.database_path, timeout) {
        Some(lock) => lock,
        None if is_hook => return,
        None => {
            eprintln!("cz: the database is locked by another cz process");
            exit(1);
        }
    };

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
        Command::Add(dir) => app.add_visit(&dir),
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
