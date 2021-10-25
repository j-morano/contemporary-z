use crate::database::{
    update_dir_counter,
    update_current_dir,
};
use crate::data::Directory;

use rusqlite::{Connection, Result};
use std::env::current_dir;
use std::process::exit;
use home::home_dir;
use std::fs::{File};
use std::io::prelude::*;
use std::io;
use regex::Regex;
use std::time::{SystemTime, UNIX_EPOCH};


pub(crate) fn write(action:&str, text: String) {
    // https://stackoverflow.com/questions/65782872/
    let mut z_file = File::create(
        "/tmp/cz_path"
    ).expect("Could not open file");
    z_file.write_all(
        format!("{}|{}", action, text).as_bytes()
    ).expect("Could not write to file");
    // println!("{}", format!("{}|{}", action, text));
}

pub(crate) fn current_seconds() -> i64 {
    return SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
}

pub(crate) fn get_home_dir() -> String {
    let current_home_dir = home_dir().unwrap();
    return current_home_dir.into_os_string().into_string().unwrap();
}


const COLORS: [(&str, &str); 32] = [
    ("black_fg", "30"),
    ("red_fg", "31"),
    ("green_fg", "32"),
    ("yellow_fg", "33"),
    ("blue_fg", "34"),
    ("magenta_fg", "35"),
    ("cyan_fg", "36"),
    ("white_fg", "37"),
    ("bright_black_fg", "90"),
    ("bright_red_fg", "91"),
    ("bright_green_fg", "92"),
    ("bright_yellow_fg", "93"),
    ("bright_blue_fg", "94"),
    ("bright_magenta_fg", "95"),
    ("bright_cyan_fg", "96"),
    ("bright_white_fg", "97"),
    ("black_bg", "40"),
    ("red_bg", "41"),
    ("green_bg", "42"),
    ("yellow_bg", "43"),
    ("blue_bg", "44"),
    ("magenta_bg", "45"),
    ("cyan_bg", "46"),
    ("white_bg", "47"),
    ("bright_black_bg", "100"),
    ("bright_red_bg", "101"),
    ("bright_green_bg", "102"),
    ("bright_yellow_bg", "103"),
    ("bright_blue_bg", "104"),
    ("bright_magenta_bg", "105"),
    ("bright_cyan_bg", "106"),
    ("bright_white_bg", "107"),
];

const SGR: [(&str, &str); 2] = [
    ("normal", "0"),
    ("bold", "1"),
];

fn color_code(color: &str) -> &str {
    for c in COLORS {
        if c.0 == color {
            return c.1;
        }
    }
    return "30";
}

fn sgr_code(sgr: &str) -> &str{
    for c in SGR {
        if c.0 == sgr {
            return c.1;
        }
    }
    return "0";
}

pub(crate) struct App {
    pub(crate) theme: String,
    pub(crate) abs_paths: bool,
    pub(crate) max_results: usize,
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
        print!("{}", self.format(sgr, color, text));
    }

    pub(crate) fn show_error(&self, text: &str, error: &str) {
        write("error", text.to_string());
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
        exit(0);
    }

    pub(crate) fn select_dir(&self) -> String {
        let mut line = String::new();
        print!("Number: ");
        io::stdout().flush().expect("Could not flush output");
        std::io::stdin().read_line(&mut line).unwrap();
        return line.replace('\n', "");
    }


    pub(crate) fn select_valid_dir(&self, valid_dirs: Vec<Directory>) -> Result<String> {
        // If there are no dirs, exit
        if valid_dirs.len() == 0 {
            self.show_exit_message("No dirs");
        }

        // Show valid dirs
        for (i, dir) in valid_dirs.iter().enumerate() {
            let mut dir_name = dir.name.clone();
            // Replace /home/<user> with '~'
            let current_home_dir = get_home_dir();
            let re_h = Regex::new(
                format!(r"^{}", current_home_dir.as_str()).as_str()
            ).unwrap();
            dir_name = re_h.replace(dir_name.as_str(), "~").parse().unwrap();

            // Replace /run/media/<user> with '>'
            let re_m = Regex::new(r"^/run/media/([^/]+)").unwrap();
            dir_name = re_m.replace(dir_name.as_str(), ">").parse().unwrap();

            println!(
                "{}) {} {}",
                self.format("bold", "", (i+1).to_string()),
                self.format("bold", "blue", dir_name),
                (i+1),
                // dir.score
            );
        }
        println!();

        // Select dir by number
        let selected_dir = match self.select_dir().parse::<usize>() {
            Ok(number)  => number,
            Err(error) => {
                self.show_error("No dir selected", error.to_string().as_str());
                1 as usize
            },
        };

        // Check if the introduced number is valid
        if selected_dir > valid_dirs.len() || selected_dir < 1{
            self.show_error(
                "Invalid number",
                format!(
                    "{} > {}",
                    selected_dir, valid_dirs.len()
                ).as_str()
            );
        }

        // Get name of the selected dir
        let dir_name =
            format!("{}", valid_dirs[selected_dir-1].name);

        // update_dir_counter(conn, dir_name.clone())?;
        // println!("{}", dir_name);

        return Ok(dir_name);
    }

    pub(crate) fn post_current_dir(&self, conn: &Connection) {
        let current_dir = current_dir().unwrap();
        let current_dir_string = current_dir.into_os_string().into_string().expect("Error");
        // println!("{}", current_dir_string);
        match update_current_dir(conn, current_dir_string) {
            Ok(_) => { }
            Err(error) => {
                self.show_error("Could not load current dir", error.to_string().as_str());
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
        write("command", format!("cd {}", dir_name.clone()));
    }


    pub(crate) fn run_in_background(c_args: &[String]) {
        // Build command string
        let command = c_args.join(" ");

        write(
            "command",
            format!("nohup {} </dev/null >/dev/null 2>&1 & disown", command)
        );
    }

}