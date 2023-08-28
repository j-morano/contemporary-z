use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};
use std::fs;
use crate::app::current_seconds;
use crate::data::Directory;
use crate::database::{
    get_dir, get_valid_dirs, drop_directories_table, insert_dir,
    obt_current_dir, drop_current_dir_table, obt_target_dir, remove_dir,
    get_all_dirs, remove_non_existent_dirs, remove_dir_alias
};

use crate::strings::HELP;
use crate::app::App;
// use crate::database::{add_alias_to_directory_unique};



pub(crate) fn clear_database(app: &App, conn: &Connection) -> Result<()> {
    drop_directories_table(&conn)?;
    drop_current_dir_table(&conn)?;
    app.show_exit_message("Cleared database");
    Ok(())
}


pub(crate) fn go_to_previous_dir(app: &App, conn: &Connection) {
    match obt_current_dir(&conn) {
        Ok(current_dir) => {
            if Path::new(current_dir.as_str()).exists() {
                app.direct_cd(&conn, current_dir);
            } else {
                app.show_error("No valid previous directory", "");
            }
        }
        Err(_) => {
            app.show_error("No previous directory", "");
        }
    };
}


pub(crate) fn go_to_target_dir(app: &App, conn: &Connection) {
    match obt_target_dir(&conn) {
        Ok(target_dir) => {
            if Path::new(target_dir.as_str()).exists() {
                app.direct_cd(&conn, target_dir);
            } else {
                app.show_error("No valid current directory", "");
            }
        }
        Err(_) => {
            app.show_error("No current directory", "");
        }
    };
}


pub(crate) fn list_dirs(app: &App, conn: &Connection, args: &[String]) {
    let mut num_results = app.max_results;
    if args.len() > 2 {
        // Select dir by number
        num_results = match args[2].parse::<usize>() {
            Ok(number)  => number,
            Err(error) => {
                app.show_error("Invalid number", error.to_string().as_str());
                1 as usize
            }
        };
    }

    let valid_dirs = get_valid_dirs(
        &conn, Vec::new(), current_seconds(), num_results, false
    ).unwrap();

    app.list_dirs(&valid_dirs, 0, 1);
}


pub(crate) fn interactive_navigation(
    app: &App,
    conn: &Connection,
    hidden: bool,
    force_dir_only: bool,
) {
    let mut dir_to_read = String::from(".");
    loop {
        let paths = fs::read_dir(dir_to_read.as_str()).unwrap();
        let mut valid_dirs: Vec<Directory> = Vec::new();
        let mut files: Vec<String> = Vec::new();

        for result_path in paths {
            let dir_path = result_path.unwrap().path();
            if dir_path.exists()
            {
                if dir_path.is_dir() {
                    let filename = String::from(
                        dir_path.file_name().unwrap().to_str().unwrap()
                    );
                    if (hidden && filename != ".")
                        || (!hidden  && !filename.starts_with("."))
                    {
                        let directory = Directory{
                            name: filename.clone(),
                            counter: 0,
                            last_access: 0,
                            score: 0.0,
                            alias: String::new()
                        };
                        valid_dirs.push(directory);
                    }
                } else {
                    if !force_dir_only {
                        // Add to files
                        let filename = String::from(
                            dir_path.file_name().unwrap().to_str().unwrap()
                        );
                        files.push(filename);
                    }
                }
            }
        }
        // Parent directory
        let directory = Directory{
            name: "..".to_string(),
            counter: 0,
            last_access: 0,
            score: 0.0,
            alias: String::new()
        };
        valid_dirs.push(directory);
        // Sort dirs by name
        valid_dirs.sort_by_key(|dir| dir.name.clone());

        if valid_dirs.is_empty() {
            break;
        }

        let dir_name: String; //= String::new();
        match app.select_valid_dir_no_exit(valid_dirs, usize::MAX, app.nav_start_number, files) {
            Ok(dir_string)  => {
                dir_name = dir_string
            }
            Err(_error) => {
                break;
            }
        };
        println!();

        let dir_name_str = dir_name.as_str();
        let base_dir_str = dir_to_read.as_str();
        let dir_path = Path::new(base_dir_str);
        let dir_path_buf = dir_path.join(dir_name_str);
        let mut dir_str = dir_path_buf.to_str().unwrap();

        let dir_pathbuf;
        if app.abs_paths {
            dir_pathbuf = PathBuf::from(dir_str).canonicalize().unwrap();
            dir_str = dir_pathbuf.to_str().unwrap();
        }

        // Check if dir is in the table
        let dir = get_dir(&conn, dir_str);

        // If the dir is not in the table and it does exists in the
        //   FS, add it
        if let Err(_err) = dir {
            // Do not store '..' or '.' dirs
            if !(dir_str == "." || dir_str == "..") {
                let current_seconds = current_seconds();
                match insert_dir(&conn, dir_str, current_seconds) {
                    Ok(_size)  => { }
                    Err(error) => {
                        app.show_error(
                            "Could not inser dir",
                            error.to_string().as_str()
                        );
                    }
                };
            }
            dir_to_read = String::from(dir_str);

        } else { // if it is already present in the table, update its
                 // counter

            match dir {
                Ok(dir_string)  => {
                    dir_to_read = dir_string;
                }
                Err(error) => {
                    app.show_error("Directory does not exist", error.to_string().as_str());
                }
            };
        }
        // print in bold dir_to_read
        println!("{}", app.format("bold", "", dir_to_read.to_string()));
        // println!("{}", dir_to_read);
    }
    app.direct_cd(&conn, dir_to_read);
}


pub(crate) fn opt_remove_dirs(app: &App, conn: &Connection, args: &[String]) {
    let valid_dirs = get_valid_dirs(
        &conn, Vec::from(&args[2..]), current_seconds(), app.max_results, false
    ).unwrap();

    let dir_names = app.select_valid_dirs(valid_dirs, 0).unwrap();

    let mut all_dirs_removed = true;
    for dir_name in dir_names {
        match remove_dir(&conn, dir_name.clone()) {
            Ok(_)  => { }
            Err(_error) => {
                all_dirs_removed = false;
            }
        };
    }
    if all_dirs_removed {
        app.show_exit_message("Removed directories");
    } else {
        app.show_error("Could not remove directories", "");
    }
}


pub(crate) fn remove_alias_interactive(app: &App, conn: &Connection) {
    println!("Select directory to remove alias");
    // app.show_error("The provided directory does not exist", "");
    let valid_dirs = get_valid_dirs(
        &conn, Vec::new(), current_seconds(), app.max_results, true
    ).unwrap();

    let dir_name = app.select_valid_dir(valid_dirs, 0).unwrap();
    let dir_str = dir_name.as_str();

    // Always list dirs
    remove_dir_alias(&conn, dir_str).unwrap();
    app.show_exit_detailed_message("Removed alias for dir", dir_str);
}


pub(crate) fn interactive_cd(app: &App, conn: &Connection, args: &[String]) {
    let valid_dirs = get_valid_dirs(
        &conn, Vec::from(&args[1..]), current_seconds(), app.max_results, false
    ).unwrap();

    // Always list dirs
    let dir_name = app.select_valid_dir(valid_dirs, 0).unwrap();
    app.direct_cd(&conn, dir_name.clone());
}


pub(crate) fn show_help() {
    println!("{}", HELP);
}


pub(crate) fn opt_sync_dirs(app: &App, conn: &Connection) {
    // Remove directories which do not exist
    remove_non_existent_dirs(&conn).unwrap();
    app.show_exit_message("Synced directories.");
}


pub(crate) fn opt_list_all_dirs(app: &App, conn: &Connection) {
    let all_dirs = get_all_dirs(&conn).unwrap();
    app.list_dirs(&all_dirs, 0, 1);
}
