mod cli;
mod app;
mod config;
mod db;
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



/// Report a wrong usage and exit (telling the shell not to cd).
fn usage_error(message: &str) -> ! {
    write_dir("".to_string());
    eprintln!("cz: {}\nTry 'cz --help' for more information.", message);
    exit(2);
}


fn main() {
    // Not env::args(): it panics on arguments that are not valid UTF-8
    let mut args = Vec::new();
    for arg in env::args_os().skip(1) {
        match arg.into_string() {
            Ok(arg) => args.push(arg),
            Err(_) => usage_error("arguments must be valid UTF-8"),
        }
    }
    let command = match cli::parse(&args) {
        Ok(command) => command,
        Err(message) => usage_error(&message),
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
