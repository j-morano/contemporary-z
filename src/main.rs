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
use crate::strings::HELP;



fn main() {
    // Not env::args(): it panics on arguments that are not valid UTF-8
    let args: Result<Vec<String>, _> = env::args_os().skip(1).map(|a| a.into_string()).collect();
    let command = match args.map_err(|_| "arguments must be valid UTF-8".to_string())
        .and_then(|args| cli::parse(&args))
    {
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
    // A hook must never delay the shell for long: if the database is busy, it
    // gives up quickly (losing one visit)
    let is_hook = matches!(command, Command::Add(_));
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
    let timeout = Duration::from_millis(if is_hook { 200 } else { 5000 });
    if !app.save(timeout) && !is_hook {
        eprintln!("cz: could not save the database: it is locked by another cz process");
        exit(1);
    }
}
