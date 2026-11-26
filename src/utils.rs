use std::fs;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};



/// The canonical (absolute, without symlinks) form of `path`, if it is an
/// existing directory whose path is valid UTF-8.
pub(crate) fn canonical_dir(path: &str) -> Option<String> {
    let path = fs::canonicalize(path).ok()?;
    if !path.is_dir() {
        return None;
    }
    path.into_os_string().into_string().ok()
}


pub(crate) fn get_current_seconds() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}


pub(crate) fn get_home_dir() -> String {
    env::var("HOME").unwrap_or_default()
}


pub(crate) fn get_user() -> String {
    env::var("USER").unwrap_or_default()
}


/// Path of the file through which `cz` tells the shell function where to `cd`.
/// The shell functions (`z.sh`, `z.fish`) pass a private temporary file in
/// `CZ_PATH_FILE`; the fallback is for functions from older versions.
pub(crate) fn path_file() -> String {
    if let Ok(file) = env::var("CZ_PATH_FILE") {
        if !file.is_empty() {
            return file;
        }
    }
    match env::var("XDG_RUNTIME_DIR") {
        Ok(dir) if !dir.is_empty() => format!("{}/cz_path", dir),
        _ => format!("/tmp/cz_path_{}", env::var("USER").unwrap_or_default()),
    }
}


pub(crate) fn write_dir(path: String) {
    fs::write(path_file(), path).unwrap_or_else(
        |e| panic!("Could not write to {}: {}", path_file(), e)
    );
}
