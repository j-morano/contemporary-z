use rusqlite::{params, Connection, Result};
use std::env;
use std::process;
use std::path::Path;
use std::process::exit;
use std::fs;
use home::home_dir;

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

fn main() -> Result<()> {
    // Collect command-line arguments 
    let args: Vec<_> = env::args().collect();

    let home_dir_o = home_dir().unwrap();
    let home_dir_d = home_dir_o.display();

    // print!("{}", home_dir_d);

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
        println!("clear#");
        conn.execute("drop table if exists folders", [])?;
        process::exit(0);
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
    
    
    if args.len() > 1 {

        // Print argument
        // println!("The first argument is {}", args[1]);

        let folder = get_folder(&conn, &args[1]);
        
        // If the folder is not in the table and exists, add it
        if let Err(_err) = folder {
            // If the folder exists, add it
            if Path::new(&args[1]).exists() {
                // println!("New folder {}", args[1]);
                conn.execute(
                    "INSERT INTO folders (name, counter) values (?1, 1)",
                    params![args[1]],
                )?;
                println!("direct_cd#{}", args[1]);
            } else {
                println!("Invalid path {}", args[1]);
                exit(1);
            }

        // If it is already present in the table, update its counter
        } else {
            conn.execute(
                "UPDATE folders SET counter = counter + 1 where name = ?1",
                params![args[1]],
            )?;

            println!("direct_cd#{}", folder?);
        }

        Ok(())

    // If there is no argument, go to the most frequent folder
    } else {
        print!("folder_selection#");

        // Return most common folders ordered by counter (descending) 
        let mut stmt = conn.prepare(
            "SELECT name, counter 
            FROM folders
            ORDER BY counter DESC
            ;",
        )?;

        let folders = stmt.query_map([], |row| {
            Ok(Folder {
                name: row.get(0)?,
                counter: row.get(1)?
            })
        })?;

        let folders_collection: Vec<_> = folders.collect();

        if folders_collection.len() == 0 {
            exit(1);
        }

        // Number of items
        print!("{}#", folders_collection.len());

        let max_results = 9;
        for (i, folder) in folders_collection.iter().enumerate() {
            let folder_info = folder.as_ref().expect("Error");
            print!("{}) {} [{}]\\n", i+1, folder_info.name, folder_info.counter);
            // Only print the first folder
            if i > max_results+1 { break; }
        }

        Ok(())
    }
}
