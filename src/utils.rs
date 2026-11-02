use std::path::PathBuf;
use std::fs;
use std::env;



pub(crate) fn canonicalize_dir_str(dir_str_name: &str) -> String {
    let mut dir_str = dir_str_name;

    // Canonicalize path
    let dir_pathbuf = PathBuf::from(dir_str).canonicalize().unwrap();
    dir_str = dir_pathbuf.to_str().unwrap();

    // If dir name ends with '/', remove it, in order to avoid
    //   having duplicated dirs (with and without '/' versions)
    if dir_str.len() > 1
        && dir_str.chars().last().unwrap() == '/'
    {
        dir_str = &dir_str[..dir_str.len() - 1];
    }

    // Do the same with a loop
    let mut dir_str = String::from(dir_str);
    while dir_str.contains("//") {
        dir_str = dir_str.replace("//", "/");
    }

    dir_str
}


/// Path of the file through which `cz` tells the shell function where to `cd`.
/// Must match the logic in `z.sh`.
pub(crate) fn path_file() -> String {
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
