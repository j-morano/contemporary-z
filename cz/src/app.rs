use crate::database::{update_dir_counter, update_current_dir, update_target_dir};
use crate::data::Directory;

use rusqlite::{Connection, Result};
use std::env::current_dir;
use std::process::exit;
use home::home_dir;
use std::fs;
use std::io::prelude::*;
use std::io;
use regex::Regex;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::colors::{color_code, sgr_code};


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

pub(crate) fn current_seconds() -> i64 {
    return SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
}

pub(crate) fn get_home_dir() -> String {
    let current_home_dir = home_dir().unwrap();
    return current_home_dir.into_os_string().into_string().unwrap();
}

#[derive(Debug, Clone)]
pub struct SelectionError;


#[allow(dead_code)]
pub(crate) struct App {
    pub(crate) theme: String,
    pub(crate) abs_paths: bool,
    pub(crate) compact_paths: bool,
    pub(crate) max_results: usize,
    pub(crate) database_path: String,
    pub(crate) substring: String,
}

impl App {
    fn format(&self, sgr: &str, color: &str, text: String) -> String {
        let mut full_color ;
        if self.theme == "bright" {
            full_color = format!("bright_{}_fg", color);
            // If a black fg is forced, replace it by white
            full_color = full_color.replace("black", "white");
        } else {
            full_color = format!("{}_fg", color);
        }
        if color.is_empty() {
            return format!(
                "\x1b[{}m{}\x1b[0m",
                sgr_code(sgr), text
            );
        } else {
            return format!(
                "\x1b[{};{}m{}\x1b[0m",
                sgr_code(sgr), color_code(full_color.as_str()), text
            );
        }
    }

    fn printf(&self, sgr: &str, color: &str, text: String) {
        println!("{}", self.format(sgr, color, text));
    }

    pub(crate) fn show_error(&self, text: &str, error: &str) {
        write_dir("".to_string());
        let mut joint = "";
        if !error.is_empty() {
            joint = ":";
        }
        println!(
            "{}{} {}",
            self.format("bold", "magenta", text.to_string()),
            joint,
            error,
        );
        exit(1);
    }

    pub(crate) fn show_exit_message(&self, text: &str) {
        self.printf("bold", "green", String::from(text));
    }

    pub(crate) fn select_dir(&self) -> String {
        let mut line = String::new();
        print!("Number: ");
        io::stdout().flush().expect("Could not flush output");
        std::io::stdin().read_line(&mut line).unwrap();
        return line.replace('\n', "");
    }


    pub(crate) fn list_dirs(&self, valid_dirs: &Vec<Directory>, max_num: usize) {
        let mut max_results = max_num;
        if max_num == 0 {
            max_results = self.max_results;
        }
        // If there are no dirs, exit
        if valid_dirs.len() == 0 {
            self.show_exit_message("No dirs");
            exit(0);
        } else {
            // Show valid dirs
            for (i, dir) in valid_dirs.iter().enumerate() {
                let mut dir_name = dir.name.clone();

                if self.compact_paths {
                    // Replace /home/<user> with '~'
                    let current_home_dir = get_home_dir();
                    let re_h = Regex::new(
                        format!(r"^{}", current_home_dir.as_str()).as_str()
                        ).unwrap();
                    dir_name = re_h.replace(dir_name.as_str(), "~").parse().unwrap();

                    // Replace (/run)/media/<user> with '>'
                    let re_m = Regex::new(r"^/(run/)?media/([^/]+)").unwrap();
                    dir_name = re_m.replace(dir_name.as_str(), ">").parse().unwrap();
                }

                let mut alias = String::new();
                if !dir.alias.is_empty() {
                    alias = format!("{}:", dir.alias);
                }

                println!(
                    "{}) {}{} {}",
                    self.format("bold", "", (i+1).to_string()),
                    alias,
                    self.format("bold", "blue", dir_name),
                    (i+1),
                    // dir.score
                );
                if i == (max_results - 1) {
                    break;
                }
            }
        }
    }

    pub(crate) fn select_valid_dir_no_exit(
        &self,
        valid_dirs: Vec<Directory>,
        max_num: usize,
    ) -> Result<String, SelectionError>
    {

        self.list_dirs(&valid_dirs, max_num);
        println!();

        // Select dir by number
        let selected_dir = match self.select_dir().parse::<usize>() {
            Ok(number)  => number,
            Err(_) => {
                return Err(SelectionError);
            },
        };

        // Check if the introduced number is valid
        if selected_dir > valid_dirs.len() || selected_dir < 1{
            return Err(SelectionError);
        }

        // Get name of the selected dir
        let dir_name = format!("{}", valid_dirs[selected_dir-1].name);

        return Ok(dir_name);
    }

    pub(crate) fn select_valid_dirs(&self, valid_dirs: Vec<Directory>, max_num: usize) -> Result<Vec<String>> {

        self.list_dirs(&valid_dirs, max_num);
        println!();

        // Select dirs by numbers
        let selected_dirs_string = self.select_dir();
        // parse list of numbers separated by spaces
        let selected_dirs_nums_str: Vec<&str> = selected_dirs_string.split(' ').collect();
        // selected dirs names strs
        let mut selected_dirs_strings: Vec<String> = Vec::new();

        for selected_dir_num_str in selected_dirs_nums_str {
             
            let selected_dir_num = self.parse_and_validate_dir_number(&selected_dir_num_str, valid_dirs.len()).unwrap();

            // Get name of the selected dir and add it to the list
            selected_dirs_strings.push(valid_dirs[selected_dir_num-1].name.clone());
            // let dir_name = format!("{}", valid_dirs[selected_dir_num-1].name);
        }

        return Ok(selected_dirs_strings);
    }

    fn parse_and_validate_dir_number(
        &self,
        selected_dir: &str,
        max_num: usize
    ) -> Result<usize, SelectionError> 
    {

        // Select dir by number
        let selected_dir = match selected_dir.parse::<usize>() {
            Ok(number)  => number,
            Err(error) => {
                self.show_error("No dir selected", error.to_string().as_str());
                1 as usize
            },
        };

        // Check if the introduced number is valid
        if selected_dir > max_num || selected_dir < 1 {
            self.show_error(
                "Invalid number",
                format!(
                    "{} > {}",
                    selected_dir, max_num
                ).as_str()
            );
            return Err(SelectionError);
        } else {
            return Ok(selected_dir);
        }
    }

    pub(crate) fn select_valid_dir(&self, valid_dirs: Vec<Directory>, max_num: usize) -> Result<String> {

        let mut i = 0;
        let mut selected_dir: String;
        let mut dirs_to_show = &valid_dirs[0..];
        let mut starting_index = 0;
        let number_of_pages = (valid_dirs.len() + self.max_results - 1) / self.max_results;

        loop {
            if number_of_pages > 0 {
                println!("[{}/{}]", i+1, number_of_pages);
            }
            self.list_dirs(&dirs_to_show.to_vec(), max_num);
            println!();

            selected_dir = self.select_dir();
            if selected_dir != "e" { break; }
            i = i + 1;
            starting_index = i * self.max_results;
            if starting_index >= valid_dirs.len() {
                starting_index = 0;
                i = 0;
            }
            dirs_to_show = &valid_dirs[starting_index..];
        }

        let selected_dir_num = self.parse_and_validate_dir_number(&selected_dir, dirs_to_show.len()).unwrap();

        // Get name of the selected dir
        let dir_name = format!("{}", valid_dirs[starting_index+selected_dir_num-1].name);

        return Ok(dir_name);
    }

    pub(crate) fn post_current_dir(&self, conn: &Connection) {
        let current_dir = match current_dir() {
            Ok(current_dir) => { current_dir }
            Err(_) => {
                // If the current dir has been deleted, do not update current
                //  dir
                return
            }
        };
        let current_dir_string = current_dir.into_os_string().into_string().expect("Error");
        match update_current_dir(conn, current_dir_string) {
            Ok(_) => { }
            Err(error) => {
                self.show_error("Could not update current dir", error.to_string().as_str());
            }
        };
    }

    pub(crate) fn post_target_dir(&self, conn: &Connection, dir_name: String) {
        match update_target_dir(conn, dir_name) {
            Ok(_) => { }
            Err(error) => {
                self.show_error("Could not update target dir", error.to_string().as_str());
            }
        };
    }

    pub(crate) fn direct_cd(&self, conn: &Connection, dir_name: String) {
        let current_seconds = current_seconds();
        match update_dir_counter(&conn, String::from(dir_name.clone()), current_seconds) {
            Ok(_) => {}
            Err(_) => {}
        };
        self.post_current_dir(&conn);
        self.post_target_dir(&conn, dir_name.clone());
        write_dir(dir_name.clone());
    }

}
