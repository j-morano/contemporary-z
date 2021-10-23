mod database;
mod data;

use crate::data::Directory;
use crate::database::{
    get_dir,get_valid_dirs,create_dirs_table_if_not_exist,
    update_dir_counter, drop_directories_table, insert_dir
};

use std::borrow::Borrow;
use rusqlite::{Connection, Result};
use std::env;
use std::path::Path;
use std::process::exit;
use std::fs;
use home::home_dir;
use std::fs::{File, metadata};
use std::io::prelude::*;
use std::io;
use regex::Regex;


fn write(action:&str, text: String) {
    // https://stackoverflow.com/questions/65782872/
    let mut z_file = File::create(
        "/tmp/cz_path"
    ).expect("Could not open file");
    z_file.write_all(
        format!("{}|{}", action, text).as_bytes()
    ).expect("Could not write to file");
    // println!("{}", format!("{}|{}", action, text));
}

fn select_dir() -> String {
    let mut line = String::new();
    print!("Number: ");
    io::stdout().flush().expect("Could not flush output");
    std::io::stdin().read_line(&mut line).unwrap();
    return line.replace('\n', "");
}

fn bold_blue(text: String) -> String {
    return format!("\x1b[1;34m{}\x1b[0m", text);
}

fn bold(text: String) -> String {
    return format!("\x1b[1m{}\x1b[0m", text);
}

fn bold_magenta(text: String) -> String {
    return format!("\x1b[1;35m{}\x1b[0m", text);
}

fn bold_green(text: String) -> String {
    return format!("\x1b[1;32m{}\x1b[0m", text);
}

fn show_error(text: &str, error: &str) {
    write("error", text.to_string());
    println!(
        "{}: {}",
        bold_magenta(text.to_string()),
        error
    );
    exit(1);
}

fn show_error_usize(text: &str, error: &str) -> usize {
    show_error(text, error);
    return 1;
}

fn show_exit_message(text: &str) {
    println!("{}", bold_green(String::from(text)));
    exit(0);
}

fn select_valid_dir(
    conn: &Connection,
    valid_dirs: Vec<Directory>
) -> Result<String> {
    // If there are no dirs, exit
    if valid_dirs.len() == 0 {
        show_exit_message("No dirs");
    }

    // Show valid dirs
    for (i, dir) in valid_dirs.iter().enumerate() {
        println!(
            "{}) {} [{}]",
            bold((i+1).to_string()),
            bold_blue(dir.name.clone()),
            dir.counter
        );
    }
    println!();

    // Select dir by number
    let selected_dir = match select_dir().parse::<usize>() {
        Ok(number)  => number,
        Err(error) => {
            show_error_usize("No dir selected", error.to_string().as_str())
        },
    };

    // Check if the introduced number is valid
    if selected_dir > valid_dirs.len() || selected_dir < 1{
        show_error(
            "Invalid number",
            format!(
                "{} > {}",
                selected_dir, valid_dirs.len()
            ).as_str()
        );
    }

    // Get name of the selected dir
    let dir_name =
        format!("{}", valid_dirs[selected_dir-1].name);

    update_dir_counter(conn, dir_name.clone())?;
    // println!("{}", dir_name);

    return Ok(dir_name);
}


fn main() -> Result<()> {
    // Collect command-line arguments 
    let args: Vec<_> = env::args().collect();

    // Get user home dir
    let home_dir_o = home_dir().unwrap();
    let home_dir_d = home_dir_o.display();

    let database_dir_path = format!(
        "{}{}", home_dir_d, "/.local/share/cz/");

    // Create application user-specific data dir if it does not exist
    fs::create_dir_all(&database_dir_path).unwrap_or_else(
        |e| panic!("Error creating dir: {}", e));

    let database_file_path = format!(
        "{}{}", &database_dir_path, "directories.db");

    // Open connection with the database
    let conn = Connection::open(database_file_path)?;

    // Clear table command option
    if args.len() > 1 && args[1] == "--clear" {
        // write(z_file, "clear#", "".to_string());
        drop_directories_table(&conn)?;
        show_exit_message("Cleared database");
    }

    create_dirs_table_if_not_exist(&conn)?;

    write("empty", "".to_string());

    // If there is a dir argument, cd to the dir
    if args.len() > 1 {

        // Directory argument
        let mut dir_str = args[1].as_str();

        // If it is a dir AND exists in the FS
        if Path::new(dir_str).exists()
            && metadata(dir_str).unwrap().is_dir()
        {
            // If dir name ends with '/', remove it, in order to avoid
            //   having duplicated dirs (with and without '/' versions)
            if dir_str.len() > 1
                && dir_str.chars().last().unwrap() == '/'
            {
                dir_str = &dir_str[..dir_str.len() - 1];
            }

            // Replace multiple contiguous slashes by a single slash
            let re = Regex::new(r"/(/)+").unwrap();
            let result = re.replace_all(dir_str, "/");

            dir_str = result.borrow();

            // Check if dir is in the table
            let dir = get_dir(&conn, dir_str);

            // If the dir is not in the table and it does exists in the
            //   FS, add it
            if let Err(_err) = dir {
                // Do not store '..' or '.' dirs
                if !(dir_str == "." || dir_str == "..") {
                    insert_dir(&conn, dir_str)?;
                }
                // println!("{}", args[1]);
                write("direct_cd", dir_str.to_string());


            } else { // if it is already present in the table, update its
                     // counter
                update_dir_counter(&conn, String::from(dir_str))?;

                write("direct_cd", dir?);
            }
        } else { // if arguments are substrings

            let valid_dirs = get_valid_dirs(
                &conn, Vec::from(&args[1..])).unwrap();

            // if these is only one result, access it directly
            if valid_dirs.len() == 1 {
                let dir = &valid_dirs[0].name;
                update_dir_counter(&conn, dir.to_string())?;
                write("direct_cd", dir.to_string());
            } else {
                let dir_name = select_valid_dir(
                    &conn, valid_dirs).unwrap();
                write("direct_cd", dir_name);
            }
        }

        Ok(())

    } else { // if there is no argument, list frequent dirs

        let valid_dirs = get_valid_dirs(
            &conn, Vec::new()).unwrap();

        let dir_name = select_valid_dir(
            &conn, valid_dirs).unwrap();

        write("direct_cd", dir_name);

        Ok(())
    }
}
