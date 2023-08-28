use std::borrow::Borrow;
use std::path::PathBuf;
use std::fs;
use regex::Regex;
use std::io::Write;



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


pub(crate) fn write_dir(path: String) {
    // Open file in read mode
    let mut z_file = match fs::OpenOptions::new()
        .read(true)
        .write(false)
        .open("/tmp/cz_path") {
            Err(_) => {
                // Open file in write mode
                fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open("/tmp/cz_path")
                    .expect("Could not open file")
            },
            Ok(file) => {
                // Set writeable
                let mut permissions = file.metadata().expect(
                    "Could not get metadata"
                    ).permissions();
                permissions.set_readonly(false);
                file.set_permissions(permissions.clone()).expect(
                    "Could not set permissions."
                    );
                // Open file in write mode
                fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .truncate(true)
                    .open("/tmp/cz_path")
                    .expect("Could not open file")
            }
        };
    // Write action
    z_file.write_all(
        format!("{}", path).as_bytes()
        ).expect("Could not write to file");
    // Set read-only again
    let mut permissions = z_file.metadata().expect(
        "Could not get metadata"
        ).permissions();
    permissions.set_readonly(false);
    z_file.set_permissions(permissions.clone()).expect(
        "Could not set permissions."
        );
    permissions.set_readonly(true);
    z_file.set_permissions(permissions).expect("Could not set permissions.");
}
