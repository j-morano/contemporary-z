
use crate::data::Directory;

use std::path::Path;

use rusqlite::{params, Connection, Result};


const MAX_RESULTS: usize = 9;



pub(crate) fn insert_dir(conn: &Connection, dir_str: &str, current_seconds: i64) -> Result<usize> {
    return conn.execute(
        "INSERT INTO directories (name, counter, last_access) values (?1, 1, ?2)",
        params![dir_str, current_seconds], // TODO
    );
}

pub(crate) fn drop_directories_table(conn: &Connection) -> Result<usize> {
    return conn.execute("drop table if exists directories", []);
}

pub(crate) fn drop_current_dir_table(conn: &Connection) -> Result<usize> {
    return conn.execute("drop table if exists current_directory", []);
}


pub(crate) fn get_valid_dirs(
    conn: &Connection,
    patterns: Vec<String>,
    current_seconds: i64
) -> Result<Vec<Directory>> {
    // Filter invalid dirs from the current path
    let mut valid_dirs: Vec<Directory> = Vec::new();

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
        let mut sql = format!(
            "SELECT name, counter, last_access
            FROM directories
            --where
            ORDER BY (
                10000.0
                * CAST(counter as REAL)
                * (3.75 / ((0.0001 * ({} - CAST(last_access as REAL)) + 1.0) + 0.25))
            ) DESC
            LIMIT {}
            ;", current_seconds as f64, MAX_RESULTS);

        if pages > 1 {
            sql = sql.replace(
                "--where",
                format!(
                    "WHERE
                            (name NOT IN ( SELECT name FROM directories
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
            Ok(Directory {
                name: row.get(0)?,
                counter: row.get(1)?,
                last_access: row.get(2)?
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


pub(crate) fn get_dir(conn: &Connection, name: &str) -> Result<String> {
    conn.query_row(
        "SELECT name FROM directories WHERE name = ?",
        params![name],
        |row| row.get(0),
    )
}


pub(crate) fn update_dir_counter(conn: &Connection, dir_name: String, current_seconds: i64) -> Result<usize> {
    // Update dir accesses counter
    return conn.execute(
        "UPDATE directories SET
             counter = counter + 1,
             last_access = ?1
             where name = ?2",
        params![current_seconds, dir_name],
    );
}

pub(crate) fn create_dirs_table_if_not_exist(conn: &Connection) -> Result<usize>{
    // Create dirs table if it does not exist
    return conn.execute(
        "create table if not exists directories (
             /* id integer primary key,
             name text not null, */
             name primary key,
             counter integer not null,
             last_access integer not null
         )",
        [],
    );
}

pub(crate) fn update_current_dir(conn: &Connection, dir_name: String) -> Result<usize> {
    // Update dir accesses counter
    return conn.execute(
        "INSERT OR REPLACE INTO current_directory (id, name) VALUES (0, ?1)",
        params![dir_name],
    );
}

pub(crate) fn get_current_dir(conn: &Connection) -> Result<String> {
    conn.query_row(
        "SELECT name FROM current_directory",
        [],
        |row| row.get(0),
    )
}

pub(crate) fn create_current_dir_table_if_not_exist(conn: &Connection) -> Result<usize>{
    // Create dirs table if it does not exist
    return conn.execute(
        "create table if not exists current_directory (
             id integer primary key,
             name text not null
         )",
        [],
    );
}

