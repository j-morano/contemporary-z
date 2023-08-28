use crate::app::current_seconds;
use crate::data::Directory;

use std::path::Path;



pub(crate) fn remove_old(dirs: &mut Vec<Directory>) {
    let current_seconds = current_seconds();
    let seconds_in_a_month = 60 * 60 * 24 * 30;
    let limit = current_seconds - seconds_in_a_month;
    // Remove old dirs, i.e. dirs that have not been accessed in a month
    dirs.retain(|dir| dir.last_access > limit);
}


pub(crate) fn insert(dirs: &mut Vec<Directory>, dir: &str, current_seconds: i64) {
    println!("inserting dir: {}", dir);
    // Check if dir is already in dirs
    let mut found = false;
    for d in dirs.iter_mut() {
        if d.name == dir {
            d.counter += 1;
            d.last_access = current_seconds;
            found = true;
            break;
        }
    }
    // If not, add it
    if !found {
        let dir = Directory {
            name: dir.to_string(),
            counter: 1,
            last_access: current_seconds,
            score: 0.0,
            alias: "".to_string(),
        };
        dirs.push(dir);
    }
}


pub(crate) fn get_by_alias(dirs: &mut Vec<Directory>, alias: &str) -> Result<Directory, String> {
    for dir in dirs.iter() {
        if dir.alias == alias {
            return Ok(dir.clone());
        }
    }
    Err("Alias not found".to_string())
}


pub(crate) fn get(dirs: &mut Vec<Directory>, name: &str) -> Result<Directory, String> {
    for d in dirs.iter() {
        if d.name == name {
            return Ok(d.clone());
        }
    }
    Err("Directory not found".to_string())
}


pub(crate) fn compute_score(dirs: &mut Vec<Directory>, current_seconds: i64) {
    // 'Frecency' formula: https://github.com/rupa/z/blob/master/z.sh
    for dir in dirs.iter_mut() {
        dir.score = 10000.0 * dir.counter as f64 * (3.75 / ((0.0001 * (current_seconds - dir.last_access) as f64 + 1.0) + 0.25));
    }
}


pub(crate) fn get_valid( 
    dirs: &mut Vec<Directory>,
    patterns: Vec<String>,
    alias_only: bool,
) -> Result<Vec<Directory>, String> {
    // Filter invalid dirs from the current path
    let mut valid_dirs: Vec<Directory> = Vec::new();

    // Sub-string coincidences
    let mut pattern = String::new();
    if !patterns.is_empty() {
        pattern = patterns.join("*");
        pattern = format!("*{}*", pattern);
    }

    // Sort by score
    dirs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Filter by pattern
    let mut filtered_dirs: Vec<Directory> = Vec::new();
    for dir in dirs.iter() {
        if pattern.is_empty() || dir.name.contains(&pattern) {
            filtered_dirs.push(dir.clone());
        }
    }

    // Filter by alias
    if alias_only {
        let mut alias_dirs: Vec<Directory> = Vec::new();
        for dir in filtered_dirs.iter() {
            if dir.alias != "" {
                alias_dirs.push(dir.clone());
            }
        }
        filtered_dirs = alias_dirs;
    }
    
    // Filter by existence
    for dir in filtered_dirs.iter() {
        if Path::new(&dir.name).exists() {
            valid_dirs.push(dir.clone());
        }
    }

    Ok(valid_dirs)
}
