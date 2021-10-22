use std::borrow::Borrow;
use rusqlite::{params, Connection, Result};
use std::env;
use std::path::Path;
use std::process::exit;
use std::fs;
use home::home_dir;
use std::fs::{File, metadata};
use std::io::prelude::*;
use std::io;
use regex::Regex;


const MAX_RESULTS: usize = 9;


#[derive(Debug)]
struct Folder {
    name: String,
    counter: i32,
}


fn get_valid_folders(
    conn: &Connection,
    patterns: Vec<String>
) -> Result<Vec<Folder>> {
    // Filter invalid folders from the current path
    let mut valid_folders: Vec<Folder> = Vec::new();

    // Sub-string coincidences
    let mut pattern = String::new();
    if !patterns.is_empty() {
        pattern = patterns.join("*");
        pattern = format!("*{}*", pattern);
    }

    // Results pages
    let mut pages = 0;

    // Database pagination
    while valid_folders.len() != MAX_RESULTS {
        // println!("{}", pages);
        pages += 1;
        let mut sql = format!("SELECT name, counter
            FROM folders
            --where
            ORDER BY counter DESC
            LIMIT {}
            ;", MAX_RESULTS);

        if pages > 1 {
            sql = sql.replace(
                "--where",
                format!(
                    "WHERE
                            (name NOT IN ( SELECT name FROM folders
                            ORDER BY counter DESC LIMIT {} ))
                        --pattern",
                    (pages-1)*MAX_RESULTS
                ).as_str()
            );
            if !pattern.is_empty() {
                sql = sql.replace(
                    "--pattern",
                    format!("AND (name GLOB '{}')", pattern).as_str()
                );
            }
        } else {
            if !pattern.is_empty() {
                sql = sql.replace(
                    "--where",
                    format!("WHERE (name GLOB '{}')", pattern).as_str()
                );
            }
        }

        // println!("{}", sql);

        // Return most common folders ordered by counter (descending)
        let mut stmt = conn.prepare(sql.as_str(),)?;

        let folders = stmt.query_map([], |row| {
            Ok(Folder {
                name: row.get(0)?,
                counter: row.get(1)?
            })
        })?;

        let folders_collection: Vec<_> = folders.collect();

        // Number of folders collected
        let num_folders = folders_collection.len();

        // Add collected folders to valid folders, if appropriate
        for folder in folders_collection {
            let folder_info = folder.as_ref().expect("Error");
            if Path::new(&folder_info.name).exists() {
                valid_folders.push(folder?);
            }
            // If there are enough results, do not add more
            if valid_folders.len() == MAX_RESULTS {
                break;
            }
        }

        // Exit loop if this was the last page or if there are enough results.
        if num_folders < MAX_RESULTS || valid_folders.len() == MAX_RESULTS {
            break;
        }
    }

    return Ok(valid_folders);
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
        "/tmp/cz_path"
    ).expect("Could not open file");
    z_file.write_all(
        format!("{}|{}", action, text).as_bytes()
    ).expect("Could not write to file");
    // println!("{}", format!("{}|{}", action, text));
}

fn select_folder() -> String {
    let mut line = String::new();
    print!("Number: ");
    io::stdout().flush().expect("Could not flush output");
    std::io::stdin().read_line(&mut line).unwrap();
    return line.replace('\n', "");
}


fn select_valid_folder(
    conn: &Connection,
    valid_folders: Vec<Folder>
) -> Result<String> {
    // If there are no folders, exit
    if valid_folders.len() == 0 {
        println!("No folders");
        exit(0);
    }

    // Show valid folders
    for (i, folder) in valid_folders.iter().enumerate() {
        println!("{}) {} [{}]", i+1, folder.name, folder.counter);
    }
    println!();

    // Select folder by number
    let selected_folder = match select_folder().parse::<usize>() {
        Ok(number)  => number,
        Err(e) => {
            write("error", "".to_string());
            println!("No folder selected: {}", e);
            exit(1);
        },
    };

    // Check if the introduced number is valid
    if selected_folder > valid_folders.len() || selected_folder < 1{
        write("error", "".to_string());
        println!("Invalid number: {} > {}", selected_folder, valid_folders.len());
        exit(1);
    }

    // Get name of the selected folder
    let folder_name =
        format!("{}", valid_folders[selected_folder-1].name);

    // Update folder accesses counter
    conn.execute(
        "UPDATE folders SET counter = counter + 1 where name = ?1",
        params![folder_name],
    )?;

    // println!("{}", folder_name);

    return Ok(folder_name);
}


fn main() -> Result<()> {
    // Collect command-line arguments 
    let args: Vec<_> = env::args().collect();

    // Get user home directory
    let home_dir_o = home_dir().unwrap();
    let home_dir_d = home_dir_o.display();

    let database_folder_path = format!(
        "{}{}", home_dir_d, "/.local/share/cz/");

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

        // Folder argument
        let mut folder_str = args[1].as_str();

        // If it is a folder AND exists in the FS
        if Path::new(folder_str).exists()
            && metadata(folder_str).unwrap().is_dir()
        {
            // If folder name ends with '/', remove it, in order to avoid
            //   having duplicated folders (with and without '/' versions)
            if folder_str.len() > 1
                && folder_str.chars().last().unwrap() == '/'
            {
                folder_str = &folder_str[..folder_str.len() - 1];
            }

            // Replace multiple contiguous slashes by a single slash
            let re = Regex::new(r"/(/)+").unwrap();
            let result = re.replace_all(folder_str, "/");

            folder_str = result.borrow();

            // Check if folder is in the table
            let folder = get_folder(&conn, folder_str);

            // If the folder is not in the table and it does exists in the
            //   FS, add it
            if let Err(_err) = folder {
                // Do not store '..' or '.' folders
                if !(folder_str == "." || folder_str == "..") {
                    conn.execute(
                        "INSERT INTO folders (name, counter) values (?1, 1)",
                        params![folder_str],
                    )?;
                }
                // println!("{}", args[1]);
                write("direct_cd", folder_str.to_string());


            } else { // if it is already present in the table, update its
                     // counter
                conn.execute(
                    "UPDATE folders SET counter = counter + 1 where name = ?1",
                    params![folder_str],
                )?;

                write("direct_cd", folder?);
            }
        } else { // if arguments are substrings

            let valid_folders = get_valid_folders(
                &conn, Vec::from(&args[1..])).unwrap();

            // if these is only one result, access it directly
            if valid_folders.len() == 1 {
                let folder = &valid_folders[0].name;
                write("direct_cd", folder.to_string());
            } else {
                let folder_name = select_valid_folder(
                    &conn, valid_folders).unwrap();
                write("direct_cd", folder_name);
            }
        }

        Ok(())

    } else { // if there is no argument, list frequent folders

        let valid_folders = get_valid_folders(
            &conn, Vec::new()).unwrap();

        let folder_name = select_valid_folder(
            &conn, valid_folders).unwrap();

        write("direct_cd", folder_name);

        Ok(())
    }
}
