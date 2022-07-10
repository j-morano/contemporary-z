mod database;
mod data;
mod app;
mod config;
mod colors;
mod utils;

use crate::data::Directory;
use crate::database::{
    get_dir, get_valid_dirs, create_dirs_table_if_not_exist,
    drop_directories_table, insert_dir, create_current_dir_table_if_not_exist,
    obt_current_dir, drop_current_dir_table, obt_target_dir, remove_dir
};

use app::App;
use app::write;
use config::app_defaults_from_config;
use database::{get_dir_by_alias, insert_dir_alias, add_alias_to_directory};

use regex::Regex;
use rusqlite::{Connection, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::fs;
use std::fs::metadata;
use crate::app::{current_seconds, get_home_dir};
use crate::config::app_from_config;
use crate::utils::canonicalize_dir_str;



fn main() -> Result<()> {
    // Collect command-line arguments 
    let args: Vec<_> = env::args().collect();

    // App configuration
    let app_defaults = app_defaults_from_config();
    let app = app_from_config();

    let mut database_dir_path = app.database_path.clone();//format!(
        //"{}{}", get_home_dir(), "/.local/share/cz/");
    // Replace multiple contiguous slashes by a single slash
    let re = Regex::new(r"\$HOME").unwrap();
    database_dir_path = String::from(
        re.replace_all(database_dir_path.as_str(), get_home_dir())
        );
    if !Path::new(database_dir_path.as_str()).exists() {
        database_dir_path = app_defaults.database_path.clone();
        database_dir_path = String::from(
            re.replace_all(database_dir_path.as_str(), get_home_dir())
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

    write("empty", "".to_string());

    // Command option: clear table
    if args.len() > 1 && args[1] == "--clear" {
        // write(z_file, "clear#", "".to_string());
        drop_directories_table(&conn)?;
        drop_current_dir_table(&conn)?;
        app.show_exit_message("Cleared database");
    }

    // Command option: go to previous directory
    if args.len() > 1 && args[1] == "-" {
        // write(z_file, "clear#", "".to_string());
        match obt_current_dir(&conn) {
            Ok(current_dir) => {
                app.direct_cd(&conn, current_dir);
                exit(0);
            }
            Err(_) => {
                app.show_error("No previous directory", "");
                "".to_string()
            }
        };
    }

    // Command option: go to target directory
    if args.len() > 1 && args[1] == "=" {
        // write(z_file, "clear#", "".to_string());
        match obt_target_dir(&conn) {
            Ok(target_dir) => {
                app.direct_cd(&conn, target_dir);
                exit(0);
            }
            Err(_) => {
                app.show_error("No current directory", "");
                "".to_string()
            }
        };
    }

    // Command option: help
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        // Run command in a child process
        let help = fs::read_to_string("help.txt").expect("Unable to read file");
        println!("{}", help);
        exit(0);
    }

    // Command option: run command
    if args.len() > 1 && args[1] == "-b" {
        if args.len() < 3 {
            app.show_error("No command provided", "");
        }
        // Run command in a child process
        App::run_in_background(&args[2..]);
        exit(0);
    }

    // Command option: list directories
    if args.len() > 1 && args[1] == "-l" {
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
            &conn, Vec::new(), current_seconds(), num_results
        ).unwrap();

        app.list_dirs(&valid_dirs);
        exit(0);
    }

    // Command option: interactive subdir selection
    if args.len() > 1 && args[1] == "-i" {
        let mut dir_to_read = String::from(".");
        loop {
            let paths = fs::read_dir(dir_to_read.as_str()).unwrap();
            let mut valid_dirs: Vec<Directory> = Vec::new();

            for result_path in paths {
                let dir_path = result_path.unwrap().path();
                if dir_path.exists()
                    && dir_path.is_dir()
                {
                    let filename = String::from(
                        dir_path.file_name().unwrap().to_str().unwrap()
                    );
                    if !filename.starts_with(".") {
                        let directory = Directory{
                            name: filename.clone(),
                            counter: 0,
                            last_access: 0,
                            score: 0.0,
                            alias: String::new()
                        };
                        valid_dirs.push(directory);
                        // println!("Name: {}", filename);
                    }
                }
            }
            // Sort dirs by name
            valid_dirs.sort_by_key(|dir| dir.name.clone());

            if valid_dirs.is_empty() {
                break;
            }

            let dir_name: String; //= String::new();
            match app.select_valid_dir_no_exit(valid_dirs) {
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

            // TODO: repeated code
            // Check if dir is in the table
            let dir = get_dir(&conn, dir_str);

            // If the dir is not in the table and it does exists in the
            //   FS, add it
            if let Err(_err) = dir {
                // Do not store '..' or '.' dirs
                if !(dir_str == "." || dir_str == "..") {
                    let current_seconds = current_seconds();
                    insert_dir(&conn, dir_str, current_seconds)?;
                }
                // println!("{}", args[1]);
                //app.direct_cd(&conn, dir_str.to_string());
                dir_to_read = String::from(dir_str);

            } else { // if it is already present in the table, update its
                     // counter

                match dir {
                    Ok(dir_string)  => {
                        dir_to_read = dir_string;
                        //app.show_exit_message("cd");
                    }
                    Err(error) => {
                        app.show_error("Directory does not exist", error.to_string().as_str());
                    }
                };
                //app.direct_cd(&conn, dir?);
            }
            println!("{}", dir_to_read);
        }
        app.direct_cd(&conn, dir_to_read);
        exit(0);
    }

    // Command option: remove directory
    if args.len() > 1 && args[1] == "-r" {
        let valid_dirs = get_valid_dirs(
            &conn, Vec::from(&args[2..]), current_seconds(), app.max_results
        ).unwrap();

        let dir_name = app.select_valid_dir(valid_dirs).unwrap();

        match remove_dir(&conn, dir_name.clone()) {
            Ok(_)  => {
                app.show_exit_message("Removed directory");
            }
            Err(error) => {
                app.show_error("Could not remove directory", error.to_string().as_str());
            }
        };
        exit(0);
    }


    // Command option: add alias
    if args.len() > 1 && args[1] == "-a" {
        if args.len() < 3 {
            app.show_error("No alias nor directory provided", "");
        // } else if args.len() < 4 {
        //     app.show_error("No directory provided", "");
        } else {
            let mut alias = &String::from("");
            let mut dir_str;
            if args.len() < 4 {
                // Remove alias
                dir_str = args[2].as_str();
            } else {
                alias = &args[2];
                dir_str = args[3].as_str();
            }
            let canonical_dir = canonicalize_dir_str(dir_str);
            dir_str = canonical_dir.as_str();
            
            if Path::new(dir_str).exists()
                && metadata(dir_str).unwrap().is_dir()
            {
                // Check if dir is in the table
                let dir = get_dir(&conn, dir_str);

                // If the dir is not in the table and it does exists in the
                //   FS, add it
                if let Err(_err) = dir {
                    // Do not store '..' or '.' dirs
                    if !(dir_str == "." || dir_str == "..") {
                        let current_seconds = current_seconds();
                        insert_dir_alias(&conn, dir_str, current_seconds, alias.as_str())?;
                        app.show_exit_message("Added directory alias");
                    }
                } else {
                    add_alias_to_directory(&conn, dir_str, alias.as_str())?;
                    if args.len() < 4 {
                        app.show_exit_message("Removed directory alias");
                    } else {
                        app.show_exit_message("Added directory alias");
                    }
                }
            }
        }
        exit(0);
    }

    // If there is a dir argument, cd to the dir
    if args.len() > 1 {

        // Directory argument
        let mut dir_str = args[1].as_str();
        
        // If string is an alias, then cd to the directory, if exists
        match get_dir_by_alias(&conn, dir_str) {
            Ok(dir) => {
                let dir_str = dir.as_str();
                if Path::new(dir_str).exists()
                    && metadata(dir_str).unwrap().is_dir()
                {
                    app.direct_cd(&conn, dir);
                    exit(0);
                }
            },
            Err(_) => {},
        };

        // If it is a dir AND exists in the FS
        if Path::new(dir_str).exists()
            && metadata(dir_str).unwrap().is_dir()
        {
            let canonical_dir = canonicalize_dir_str(dir_str);
            dir_str = canonical_dir.as_str();

            // Check if dir is in the table
            let dir = get_dir(&conn, dir_str);

            // If the dir is not in the table and it does exists in the
            //   FS, add it
            if let Err(_err) = dir {
                // Do not store '..' or '.' dirs
                if !(dir_str == "." || dir_str == "..") {
                    let current_seconds = current_seconds();
                    insert_dir(&conn, dir_str, current_seconds)?;
                }
                // println!("{}", args[1]);
                // write("direct_cd", dir_str.to_string());
                app.direct_cd(&conn, dir_str.to_string());


            } else { // if it is already present in the table, update its
                     // counter
                // update_dir_counter(&conn, String::from(dir_str))?;

                // write("direct_cd", dir?);
                app.direct_cd(&conn, dir?);
            }
        } else { // if arguments are substrings, go to the parent folder of the
                 // top results that matches the substrings
            // Get shortest directory
            let valid_dirs = get_valid_dirs(
                // 100000 ~ no results limit
                &conn, Vec::from(&args[1..]), current_seconds(), app.max_results
            ).unwrap();

            if valid_dirs.is_empty() {
                app.show_exit_message("No dirs");
            } else {
                let mut selected_dir = valid_dirs[0].name.as_str();
                for dir in valid_dirs.iter() {
                    if dir.name.len() < selected_dir.len() {
                        selected_dir = dir.name.as_str();
                    }
                }
                app.direct_cd(&conn, selected_dir.to_string());
                exit(0);
            }
        }

        Ok(())

    } else { // if there is no argument, list stored dirs to select one
             //  interactively 
        
        let valid_dirs = get_valid_dirs(
            &conn, Vec::from(&args[1..]), current_seconds(), app.max_results
        ).unwrap();

        // Always list dirs
        let dir_name = app.select_valid_dir(valid_dirs).unwrap();
        // write("direct_cd", dir_name);
        app.direct_cd(&conn, dir_name.clone());

        Ok(())
    }
}
