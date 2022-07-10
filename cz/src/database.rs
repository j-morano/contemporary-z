
use crate::data::Directory;

use std::path::Path;

use rusqlite::{params, Connection, Result};
use crate::utils::canonicalize_dir_str;



pub(crate) fn insert_dir(conn: &Connection, dir_str: &str, current_seconds: i64) -> Result<usize> {
    // Ensure that a canonical dir is inserted
    let canonical_dir = canonicalize_dir_str(dir_str);
    let canonical_dir_str = canonical_dir.as_str();
    // Execute query
    return conn.execute(
        "INSERT INTO directories (name, counter, last_access, alias)
        VALUES (?1, 1, ?2, '')",
        params![canonical_dir_str, current_seconds],
    );
}


pub(crate) fn insert_dir_alias(
    conn: &Connection,
    dir_str: &str,
    current_seconds: i64,
    alias: &str
) -> Result<usize> {
    return conn.execute(
        "INSERT INTO directories (name, counter, last_access, alias)
        VALUES (?1, 1, ?2, ?3)",
        params![dir_str, current_seconds, alias],
    );
}


pub(crate) fn add_alias_to_directory(
    conn: &Connection,
    dir_str: &str,
    alias: &str
) -> Result<usize> {
    // Update dir accesses counter
    return conn.execute(
        "UPDATE directories SET
             alias = ?1
             where name = ?2",
        params![alias, dir_str],
    );
}


pub(crate) fn drop_directories_table(conn: &Connection) -> Result<usize> {
    return conn.execute("drop table if exists directories", []);
}

pub(crate) fn drop_current_dir_table(conn: &Connection) -> Result<usize> {
    return conn.execute("drop table if exists current_directory", []);
}

pub(crate) fn remove_dir(conn: &Connection, dir_name: String) -> Result<usize> {
    return conn.execute(
        "DELETE FROM directories WHERE name = ?", params![dir_name]
    );
}


pub(crate) fn get_valid_dirs(
    conn: &Connection,
    patterns: Vec<String>,
    current_seconds: i64,
    max_results: usize
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

    // 'Frecency' formula: https://github.com/rupa/z/blob/master/z.sh

    // Database pagination
    while valid_dirs.len() != max_results {
        // println!("{}", pages);
        pages += 1;
        let mut sql = format!(
            "SELECT
              name,
              counter,
              last_access,
              (10000.0
               * CAST(counter as REAL)
               * (3.75 / ((0.0001 * ({} - CAST(last_access as REAL)) + 1.0) + 0.25))
              ) as score,
              alias
            FROM directories
              --where
            ORDER BY score DESC
            LIMIT {}
            ;", current_seconds as f64, max_results);

        if pages > 1 {
            sql = sql.replace(
                "--where",
                format!(
                    "WHERE
                       (name NOT IN ( SELECT name FROM directories
                       ORDER BY counter DESC LIMIT {} ))
                       --pattern",
                    (pages-1)*max_results
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
                last_access: row.get(2)?,
                score: row.get(3)?,
                alias: row.get(4)?
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
            if valid_dirs.len() == max_results {
                break;
            }
        }

        // Exit loop if this was the last page or if there are enough results.
        if num_dirs < max_results || valid_dirs.len() == max_results {
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



pub(crate) fn get_dir_by_alias(conn: &Connection, alias: &str) -> Result<String> {
    conn.query_row(
        "SELECT name FROM directories WHERE alias = ?",
        params![alias],
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
             name varchar(256) not null, */
             name primary key,
             counter integer not null,
             last_access integer not null,
             alias varchar(64)
         )",
        [],
    );
}

pub(crate) fn update_current_dir(conn: &Connection, dir_name: String) -> Result<usize> {
    // Update dir accesses counter
    return conn.execute(
        "INSERT OR REPLACE INTO current_directory (id, name)
        VALUES ('current_dir', ?1)",
        params![dir_name],
    );
}

pub(crate) fn update_target_dir(conn: &Connection, dir_name: String) -> Result<usize> {
    // Update dir accesses counter
    return conn.execute(
        "INSERT OR REPLACE INTO current_directory (id, name)
        VALUES ('target_dir', ?1)",
        params![dir_name],
    );
}

pub(crate) fn obt_current_dir(conn: &Connection) -> Result<String> {
    conn.query_row(
        "SELECT name FROM current_directory WHERE id = 'current_dir'",
        [],
        |row| row.get(0),
    )
}

pub(crate) fn obt_target_dir(conn: &Connection) -> Result<String> {
    conn.query_row(
        "SELECT name FROM current_directory WHERE id = 'target_dir'",
        [],
        |row| row.get(0),
    )
}

pub(crate) fn create_current_dir_table_if_not_exist(conn: &Connection) -> Result<usize>{
    // Create dirs table if it does not exist
    return conn.execute(
        "create table if not exists current_directory (
             id varchar(256) primary key,
             name varchar(256) not null
         )",
        [],
    );
}

