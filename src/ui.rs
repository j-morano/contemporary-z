//! Everything that talks to the terminal: colours, messages, numbered lists
//! and the prompt.

use std::io;
use std::io::prelude::*;
use std::process::exit;

use crate::colors::{color_code, sgr_code};
use crate::config::{Config, ShowFiles, Theme};
use crate::db::Directory;
use crate::utils::{get_home_dir, get_user, write_dir};


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


    fn format(&self, sgr: &str, color: &str, text: String) -> String {
        if color.is_empty() {
            return format!("\x1b[{}m{}\x1b[0m", sgr_code(sgr), text);
        }
        let full_color = if self.config.theme == Theme::Bright {
            // If a black fg is forced, replace it by white
            format!("bright_{}_fg", color).replace("black", "white")
        } else {
            format!("{}_fg", color)
        };
        format!("\x1b[{};{}m{}\x1b[0m", sgr_code(sgr), color_code(full_color.as_str()), text)
    }


    /// Print an error and exit with status 1, telling the shell not to cd.
    pub(crate) fn show_error(&self, text: &str, error: &str) -> ! {
        write_dir("".to_string());
        let joint = if error.is_empty() { "" } else { ":" };
        println!("{}{} {}", self.format("bold", "magenta", text.to_string()), joint, error);
        exit(1);
    }


    /// Like `show_error`, but does not exit.
    pub(crate) fn show_warning(&self, text: &str, error: &str) {
        println!("{}: {}", self.format("bold", "magenta", text.to_string()), error);
    }


    pub(crate) fn show_message(&self, text: &str) {
        println!("{}", self.format("bold", "green", text.to_string()));
    }


    pub(crate) fn show_detailed_message(&self, message: &str, details: &str) {
        println!("{}: {}", self.format("bold", "green", message.to_string()), details);
    }


    pub(crate) fn show_bold(&self, text: &str) {
        println!("{}", self.format("bold", "", text.to_string()));
    }


    fn prompt(&self) -> String {
        let mut line = String::new();
        print!("Number: ");
        io::stdout().flush().expect("Could not flush output");
        io::stdin().read_line(&mut line).unwrap();
        line.replace('\n', "")
    }


    pub(crate) fn list_dirs(&self, dirs: &[Directory], max_num: usize, start: usize) {
        self.list_entries(
            dirs.iter().map(|dir| (dir.alias.as_str(), dir.name.as_str())),
            dirs.len(),
            max_num,
            start,
        );
    }


    /// Print a numbered list of `(alias, name)` entries; at most `max_num`
    /// (or `max_results` if it is 0) of the `count` entries.
    fn list_entries<'a>(
        &self,
        entries: impl Iterator<Item = (&'a str, &'a str)>,
        count: usize,
        max_num: usize,
        start: usize,
    ) {
        if count == 0 {
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

        for (i, (alias, name)) in entries.take(max_results).enumerate() {
            let name = prefixes
                .iter()
                .find_map(|(prefix, replacement)| replace_path_prefix(name, prefix, replacement))
                .unwrap_or_else(|| name.to_string());
            let alias = if alias.is_empty() { String::new() } else { format!("{}:", alias) };
            println!(
                "{}) {}{}",
                self.format("bold", "", (i + start).to_string()),
                alias,
                self.format("bold", "blue", name),
            );
        }
    }


    /// Show the subdirectories (and the files, depending on `show_files`) and
    /// ask for one of the subdirectories. Returns `None` on invalid input
    /// instead of exiting.
    pub(crate) fn select_entry(&self, dirs: &[String], files: &[String]) -> Option<String> {
        let start = self.config.nav_start_number;
        if self.config.show_files == ShowFiles::Top {
            files.iter().for_each(|file| println!("{}", file));
        }
        self.list_entries(
            dirs.iter().map(|dir| ("", dir.as_str())),
            dirs.len(),
            usize::MAX,
            start,
        );
        if self.config.show_files == ShowFiles::Bottom {
            files.iter().for_each(|file| println!("{}", file));
        }
        println!();

        let number = self.prompt().parse::<usize>().ok()?;
        dirs.get(number.checked_sub(start)?).cloned()
    }


    /// Ask for several numbers (separated by whitespace). Exits with an error
    /// message on invalid input.
    pub(crate) fn select_dirs(&self, dirs: &[Directory]) -> Vec<String> {
        self.list_dirs(dirs, 0, 1);
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
            self.list_dirs(dirs_to_show, 0, 1);
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
