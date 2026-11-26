//! Everything that talks to the terminal: colours, messages, numbered lists
//! and the prompt.

use std::io;
use std::io::prelude::*;
use std::process::exit;

use crate::config::{Config, Theme};
use crate::db::Directory;
use crate::utils::{get_home_dir, get_user, write_dir};


// ANSI colour codes
const GREEN: u8 = 32;
const BLUE: u8 = 34;
const MAGENTA: u8 = 35;


fn bold(text: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", text)
}


pub(crate) struct Ui {
    config: Config,
}


/// Replace `prefix` by `replacement` only if `path` is `prefix` itself or
/// lies inside it (so `/home/jo` does not match `/home/john`).
fn replace_path_prefix(path: &str, prefix: &str, replacement: &str) -> Option<String> {
    if prefix.is_empty() {
        return None;
    }
    let rest = path.strip_prefix(prefix)?;
    if rest.is_empty() || rest.starts_with('/') {
        Some(format!("{}{}", replacement, rest))
    } else {
        None
    }
}


impl Ui {
    pub(crate) fn new(config: Config) -> Ui {
        Ui { config }
    }


    /// `text` in bold and in `color` (one of the ANSI codes above; the bright
    /// theme uses their bright variants).
    fn colored(&self, color: u8, text: &str) -> String {
        let color = if self.config.theme == Theme::Bright { color + 60 } else { color };
        format!("\x1b[1;{}m{}\x1b[0m", color, text)
    }


    /// Print an error and exit with status 1, telling the shell not to cd.
    pub(crate) fn show_error(&self, text: &str, error: &str) -> ! {
        write_dir("".to_string());
        let joint = if error.is_empty() { "" } else { ":" };
        println!("{}{} {}", self.colored(MAGENTA, text), joint, error);
        exit(1);
    }


    pub(crate) fn show_message(&self, text: &str) {
        println!("{}", self.colored(GREEN, text));
    }


    pub(crate) fn show_detailed_message(&self, message: &str, details: &str) {
        println!("{}: {}", self.colored(GREEN, message), details);
    }


    fn prompt(&self) -> String {
        let mut line = String::new();
        print!("Number: ");
        io::stdout().flush().expect("Could not flush output");
        io::stdin().read_line(&mut line).unwrap();
        line.replace('\n', "")
    }


    /// Print a numbered list (from 1) of `dirs`: at most `max_num` of them, or
    /// `max_results` if `max_num` is 0.
    pub(crate) fn list_dirs(&self, dirs: &[Directory], max_num: usize) {
        if dirs.is_empty() {
            self.show_message("No dirs");
            return;
        }
        let max_results = if max_num == 0 { self.config.max_results } else { max_num };

        // Replace /home/<user> with '~' and (/run)/media/<user> with '>'
        let mut prefixes: Vec<(String, &str)> = Vec::new();
        if self.config.compact_paths {
            let user = get_user();
            prefixes.push((get_home_dir(), "~"));
            prefixes.push((format!("/run/media/{}", user), ">"));
            prefixes.push((format!("/media/{}", user), ">"));
        }

        for (i, dir) in dirs.iter().take(max_results).enumerate() {
            let name = prefixes
                .iter()
                .find_map(|(prefix, replacement)| replace_path_prefix(&dir.name, prefix, replacement))
                .unwrap_or_else(|| dir.name.clone());
            let alias = if dir.alias.is_empty() { String::new() } else { format!("{}:", dir.alias) };
            println!("{}) {}{}", bold(&(i + 1).to_string()), alias, self.colored(BLUE, &name));
        }
    }


    /// Ask for several numbers (separated by whitespace). Exits with an error
    /// message on invalid input.
    pub(crate) fn select_dirs(&self, dirs: &[Directory]) -> Vec<String> {
        self.list_dirs(dirs, 0);
        println!();

        self.prompt()
            .split_whitespace()
            .map(|num| {
                let n = self.parse_number(num, dirs.len());
                dirs[n - 1].name.clone()
            })
            .collect()
    }


    /// Parse a 1-based selection. Exits with an error message if it is not a
    /// number in `1..=max_num`.
    fn parse_number(&self, selected: &str, max_num: usize) -> usize {
        let number = match selected.parse::<usize>() {
            Ok(number) => number,
            Err(error) => self.show_error("No dir selected", &error.to_string()),
        };
        if number > max_num || number < 1 {
            self.show_error("Invalid number", &format!("{} > {}", number, max_num));
        }
        number
    }


    /// Show the directories, paginated, and ask for one of them (`e` shows the
    /// next page). Returns `None` if there are no directories.
    pub(crate) fn select_dir(&self, dirs: &[Directory]) -> Option<String> {
        if dirs.is_empty() {
            self.show_message("No dirs");
            return None;
        }
        let page_size = self.config.max_results;
        let number_of_pages = dirs.len().div_ceil(page_size);
        let mut page = 0;

        loop {
            let start = page * page_size;
            let dirs_to_show = &dirs[start..];
            println!("[{}/{}]", page + 1, number_of_pages);
            self.list_dirs(dirs_to_show, 0);
            println!();

            let selected = self.prompt();
            if selected != "e" {
                let n = self.parse_number(&selected, dirs_to_show.len().min(page_size));
                return Some(dirs[start + n - 1].name.clone());
            }
            page += 1;
            if page >= number_of_pages {
                page = 0;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_prefix_respects_component_boundaries() {
        assert_eq!(replace_path_prefix("/home/jo/x", "/home/jo", "~"), Some("~/x".to_string()));
        assert_eq!(replace_path_prefix("/home/jo", "/home/jo", "~"), Some("~".to_string()));
        assert_eq!(replace_path_prefix("/home/john/x", "/home/jo", "~"), None);
        assert_eq!(replace_path_prefix("/x", "", "~"), None);
    }
}
