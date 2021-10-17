use rusqlite::{params, Connection, Result};
use std::env;
use std::path::Path;
use std::process::exit;
use std::fs;
use home::home_dir;
use std::fs::File;
use std::io::prelude::*;
use std::io;


#[derive(Debug)]
struct Folder {
    name: String,
    counter: i32,
}


fn get_folder(conn: &Connection, name: &str) -> Result<String> {
    conn.query_row(
        "SELECT name FROM folders WHERE name = ?",
        params![name],
        |row| row.get(0),
    )
}

fn write(action:&str, text: String) {
    // https://stackoverflow.com/questions/65782872/
    let mut z_file = File::create(
        "/tmp/z_path").expect("Could not open file");
    z_file.write_all(
        format!("{}#{}", action, text).as_bytes()
    ).expect("Could not write to file");
    // println!("{}", format!("{}#{}", action, text));
}

fn select_folder() -> String {
    let mut line = String::new();
    print!("Number: ");
    io::stdout().flush().expect("Could not flush output");
    std::io::stdin().read_line(&mut line).unwrap();
    return line.replace('\n', "");
}

fn main() -> Result<()> {
    // Collect command-line arguments 
    let args: Vec<_> = env::args().collect();

    // Get user home directory
    let home_dir_o = home_dir().unwrap();
    let home_dir_d = home_dir_o.display();

    let database_folder_path = format!(
        "{}{}", home_dir_d, "/.local/share/z/");

    // Create application user-specific data folder if it does not exist
    fs::create_dir_all(&database_folder_path).unwrap_or_else(
        |e| panic!("Error creating dir: {}", e));

    let database_file_path = format!(
        "{}{}", &database_folder_path, "folders.db");

    // Open connection with the database
    let conn = Connection::open(database_file_path)?;

    // Clear table command option
    if args.len() > 1 && args[1] == "--clear" {
        println!("Cleared database");
        // write(z_file, "clear#", "".to_string());
        conn.execute("drop table if exists folders", [])?;
        exit(0);
    }

    // Create folders table if it does not exist
    conn.execute(
        "create table if not exists folders (
             /* id integer primary key,
             name text not null, */
             name primary key,
             counter integer not null
         )",
        [],
    )?;

    write("empty", "".to_string());

    // If there is a folder argument, cd to the folder
    if args.len() > 1 {

        // Print argument
        // println!("The first argument is {}", args[1]);

        let folder = get_folder(&conn, &args[1]);
        
        // If the folder is not in the table and exists, add it
        if let Err(_err) = folder {
            // If the folder exists, add it
            if Path::new(&args[1]).exists() {
                // Do not store '..' or '.' folders
                if !(args[1] == "." || args[1] == "..") {
                    conn.execute(
                        "INSERT INTO folders (name, counter) values (?1, 1)",
                        params![args[1]],
                    )?;
                }
                // println!("{}", args[1]);
                write("direct_cd", args[1].clone());
            } else {
                println!("Invalid path '{}'", args[1]);
                exit(1);
            }

        // If it is already present in the table, update its counter
        } else {
            conn.execute(
                "UPDATE folders SET counter = counter + 1 where name = ?1",
                params![args[1]],
            )?;

            write("direct_cd", folder?);
        }

        Ok(())

    // If there is no argument, list frequent folders
    } else {

        // Return most common folders ordered by counter (descending) 
        let mut stmt = conn.prepare(
            "SELECT name, counter 
            FROM folders
            ORDER BY counter DESC
            LIMIT 9
            ;",
        )?;

        let folders = stmt.query_map([], |row| {
            Ok(Folder {
                name: row.get(0)?,
                counter: row.get(1)?
            })
        })?;

        let folders_collection: Vec<_> = folders.collect();

        // If there are no folders, exit
        if folders_collection.len() == 0 {
            println!("No folders");
            exit(0);
        }

        for (i, folder) in folders_collection.iter().enumerate() {
            let folder_info = folder.as_ref().expect("Error");
            println!("{}) {} [{}]", i+1, folder_info.name, folder_info.counter);
        }
        println!();

        // let selected_folder: usize = select_folder().parse().unwrap();

        let selected_folder = match select_folder().parse::<usize>() {
            Ok(number)  => number,
            Err(e) => {
                write("error", "".to_string());
                println!("No folder selected: {}", e);
                exit(1);
            },
        };

        let folder_name =
            format!("{}", folders_collection[selected_folder-1].as_ref().unwrap().name);

        conn.execute(
            "UPDATE folders SET counter = counter + 1 where name = ?1",
            params![folder_name],
        )?;

        write("direct_cd", folder_name);

        Ok(())
    }
}
