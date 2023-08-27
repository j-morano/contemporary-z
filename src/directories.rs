use crate::app::current_seconds;
use crate::data::Directory;



pub(crate) fn remove_old(dirs: &mut Vec<Directory>) {
    let current_seconds = current_seconds();
    let seconds_in_a_month = 60 * 60 * 24 * 30;
    let limit = current_seconds - seconds_in_a_month;
    // Remove old dirs, i.e. dirs that have not been accessed in a month
    dirs.retain(|dir| dir.last_access > limit);
}


// directories::insert(dirs, dir_str, current_seconds);
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
