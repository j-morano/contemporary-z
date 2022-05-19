use std::borrow::Borrow;
use std::path::PathBuf;
use regex::Regex;



pub(crate) fn canonicalize_dir_str(dir_str_name: &str) -> String {

    let mut dir_str = dir_str_name;

    // Canonicalize path
    let dir_pathbuf;
    dir_pathbuf = PathBuf::from(dir_str).canonicalize().unwrap();
    dir_str = dir_pathbuf.to_str().unwrap();

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
    String::from(dir_str)
}
