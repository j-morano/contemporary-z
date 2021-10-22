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


fn get_valid_dirs(
    conn: &Connection,
    patterns: Vec<String>
) -> Result<Vec<Folder>> {
    // Filter invalid dirs from the current path
    let mut valid_dirs: Vec<Folder> = Vec::new();

    // Sub-string coincidences
    let mut pattern = String::new();
    if !patterns.is_empty() {
        pattern = patterns.join("*");
        pattern = format!("*{}*", pattern);
    }

    // Results pages
    let mut pages = 0;

    // Database pagination
    while valid_dirs.len() != MAX_RESULTS {
        // println!("{}", pages);
        pages += 1;
        let mut sql = format!("SELECT name, counter
            FROM dirs
            --where
            ORDER BY counter DESC
            LIMIT {}
            ;", MAX_RESULTS);

        if pages > 1 {
            sql = sql.replace(
                "--where",
                format!(
                    "WHERE
                            (name NOT IN ( SELECT name FROM dirs
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

        // Return most common dirs ordered by counter (descending)
        let mut stmt = conn.prepare(sql.as_str(),)?;

        let dirs = stmt.query_map([], |row| {
            Ok(Folder {
                name: row.get(0)?,
                counter: row.get(1)?
            })
        })?;

        let dirs_collection: Vec<_> = dirs.collect();

        // Number of dirs collected
        let num_dirs = dirs_collection.len();

        // Add collected dirs to valid dirs, if appropriate
        for dir in dirs_collection {
            let dir_info = dir.as_ref().expect("Error");
            if Path::new(&dir_info.name).exists() {
                valid_dirs.push(dir?);
            }
            // If there are enough results, do not add more
            if valid_dirs.len() == MAX_RESULTS {
                break;
            }
        }

        // Exit loop if this was the last page or if there are enough results.
        if num_dirs < MAX_RESULTS || valid_dirs.len() == MAX_RESULTS {
            break;
        }
    }

    return Ok(valid_dirs);
}


fn get_dir(conn: &Connection, name: &str) -> Result<String> {
    conn.query_row(
        "SELECT name FROM dirs WHERE name = ?",
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

fn select_dir() -> String {
    let mut line = String::new();
    print!("Number: ");
    io::stdout().flush().expect("Could not flush output");
    std::io::stdin().read_line(&mut line).unwrap();
    return line.replace('\n', "");
}


fn select_valid_dir(
    conn: &Connection,
    valid_dirs: Vec<Folder>
) -> Result<String> {
    // If there are no dirs, exit
    if valid_dirs.len() == 0 {
        println!("No dirs");
        exit(0);
    }

    // Show valid dirs
    for (i, dir) in valid_dirs.iter().enumerate() {
        println!("{}) {} [{}]", i+1, dir.name, dir.counter);
    }
    println!();

    // Select dir by number
    let selected_dir = match select_dir().parse::<usize>() {
        Ok(number)  => number,
        Err(e) => {
            write("error", "".to_string());
            println!("No dir selected: {}", e);
            exit(1);
        },
    };

    // Check if the introduced number is valid
    if selected_dir > valid_dirs.len() || selected_dir < 1{
        write("error", "".to_string());
        println!("Invalid number: {} > {}", selected_dir, valid_dirs.len());
        exit(1);
    }

    // Get name of the selected dir
    let dir_name =
        format!("{}", valid_dirs[selected_dir-1].name);

    // Update dir accesses counter
    conn.execute(
        "UPDATE dirs SET counter = counter + 1 where name = ?1",
        params![dir_name],
    )?;

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
        println!("Cleared database");
        // write(z_file, "clear#", "".to_string());
        conn.execute("drop table if exists dirs", [])?;
        exit(0);
    }

    // Create dirs table if it does not exist
    conn.execute(
        "create table if not exists dirs (
             /* id integer primary key,
             name text not null, */
             name primary key,
             counter integer not null
         )",
        [],
    )?;

    write("empty", "".to_string());

    // If there is a dir argument, cd to the dir
    if args.len() > 1 {

        // Folder argument
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
                    conn.execute(
                        "INSERT INTO dirs (name, counter) values (?1, 1)",
                        params![dir_str],
                    )?;
                }
                // println!("{}", args[1]);
                write("direct_cd", dir_str.to_string());


            } else { // if it is already present in the table, update its
                     // counter
                conn.execute(
                    "UPDATE dirs SET counter = counter + 1 where name = ?1",
                    params![dir_str],
                )?;

                write("direct_cd", dir?);
            }
        } else { // if arguments are substrings

            let valid_dirs = get_valid_dirs(
                &conn, Vec::from(&args[1..])).unwrap();

            // if these is only one result, access it directly
            if valid_dirs.len() == 1 {
                let dir = &valid_dirs[0].name;
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
