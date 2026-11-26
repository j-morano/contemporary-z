use crate::cli::AliasArgs;
use crate::config::{Config, Mode, ShowFiles, Theme};
use crate::data::Directory;
use crate::utils::canonicalize_dir_str;
use crate::utils::write_dir;

use std::process::exit;
use std::env;
use std::io::prelude::*;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::colors::{color_code, sgr_code};
use std::path::Path;
use std::fs;



pub(crate) fn get_current_seconds() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

pub(crate) fn get_home_dir() -> String {
    env::var("HOME").unwrap_or_default()
}

pub(crate) fn get_user() -> String {
    env::var("USER").unwrap_or_default()
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

pub(crate) struct App {
    pub(crate) config: Config,
    pub(crate) dirs: Vec<Directory>,
}

impl App {
    pub(crate) fn format(&self, sgr: &str, color: &str, text: String) -> String {
        let mut full_color ;
        if self.config.theme == Theme::Bright {
            full_color = format!("bright_{}_fg", color);
            // If a black fg is forced, replace it by white
            full_color = full_color.replace("black", "white");
        } else {
            full_color = format!("{}_fg", color);
        }
        if color.is_empty() {
            format!(
                "\x1b[{}m{}\x1b[0m",
                sgr_code(sgr), text
            )
        } else {
            format!(
                "\x1b[{};{}m{}\x1b[0m",
                sgr_code(sgr), color_code(full_color.as_str()), text
            )
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


    /// Like `show_error`, but does not exit.
    fn show_warning(&self, text: &str, error: &str) {
        println!("{}: {}", self.format("bold", "magenta", text.to_string()), error);
    }


    pub(crate) fn show_exit_message(&self, text: &str) {
        self.printf("bold", "green", String::from(text));
    }


    pub(crate) fn show_exit_detailed_message(&self, message: &str, details: &str) {
        println!(
            "{}: {}",
            self.format("bold", "green", message.to_string()),
            details,
        );
    }


    pub(crate) fn select_dir(&self) -> String {
        let mut line = String::new();
        print!("Number: ");
        io::stdout().flush().expect("Could not flush output");
        std::io::stdin().read_line(&mut line).unwrap();
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
            self.show_exit_message("No dirs");
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
    fn select_entry(&self, dirs: &[String], files: &[String]) -> Option<String> {
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

        let number = self.select_dir().parse::<usize>().ok()?;
        dirs.get(number.checked_sub(start)?).cloned()
    }


    /// Ask for several numbers (separated by whitespace). Exits with an error
    /// message on invalid input.
    pub(crate) fn select_valid_dirs(&self, valid_dirs: Vec<Directory>, max_num: usize) -> Vec<String> {
        self.list_dirs(&valid_dirs, max_num, 1);
        println!();

        self.select_dir()
            .split_whitespace()
            .map(|num| {
                let n = self.parse_and_validate_dir_number(num, valid_dirs.len());
                valid_dirs[n - 1].name.clone()
            })
            .collect()
    }


    /// Parse a 1-based selection. Exits with an error message if it is not a
    /// number in `1..=max_num`.
    fn parse_and_validate_dir_number(&self, selected_dir: &str, max_num: usize) -> usize {
        let number = match selected_dir.parse::<usize>() {
            Ok(number) => number,
            Err(error) => {
                self.show_error("No dir selected", error.to_string().as_str());
                unreachable!("show_error exits");
            },
        };
        if number > max_num || number < 1 {
            self.show_error("Invalid number", &format!("{} > {}", number, max_num));
        }
        number
    }


    /// Show the directories, paginated, and ask for one of them (`e` shows the
    /// next page). Returns `None` if there are no directories.
    pub(crate) fn select_valid_dir(&self, valid_dirs: Vec<Directory>, max_num: usize) -> Option<String> {
        if valid_dirs.is_empty() {
            self.show_exit_message("No dirs");
            return None;
        }
        let page_size = self.config.max_results;
        let number_of_pages = valid_dirs.len().div_ceil(page_size);
        let mut page = 0;

        loop {
            let start = page * page_size;
            let dirs_to_show = &valid_dirs[start..];
            println!("[{}/{}]", page + 1, number_of_pages);
            self.list_dirs(dirs_to_show, max_num, 1);
            println!();

            let selected = self.select_dir();
            if selected != "e" {
                let n = self.parse_and_validate_dir_number(
                    &selected,
                    dirs_to_show.len().min(page_size),
                );
                return Some(valid_dirs[start + n - 1].name.clone());
            }
            page += 1;
            if page >= number_of_pages {
                page = 0;
            }
        }
    }


    pub(crate) fn direct_cd(&mut self, dir_name: String) {
        self.insert(dir_name.as_str());
        write_dir(dir_name.clone());
    }


    /// `patterns[0]` is tried as an alias, then as a directory, and finally all
    /// the patterns are used as substrings. `forced` overrides the configured
    /// substring mode.
    pub(crate) fn do_cd(&mut self, patterns: &[String], forced: Option<Mode>) {
        let first = match patterns.first() {
            Some(first) => first.as_str(),
            None => {
                self.show_error("No directory or substring provided", "");
                return;
            }
        };

        // If string is an alias, then cd to the directory, if exists
        if let Some(dir) = self.get_by_alias(first) {
            if Path::new(&dir.name).is_dir() {
                self.direct_cd(dir.name);
            }
            return;
        }
        // If it is a dir AND exists in the FS, cd to it (adding it to the table)
        if Path::new(first).is_dir() {
            self.direct_cd(canonicalize_dir_str(first));
            return;
        }
        let mode = forced.unwrap_or(self.config.substring);
        self.cd_by_substrings(patterns, mode);
    }


    fn cd_by_substrings(&mut self, patterns: &[String], mode: Mode) {
        let valid_dirs = self.get_valid(patterns, false);
        let selected = match mode {
            _ if valid_dirs.is_empty() => {
                self.show_exit_message("No dirs");
                None
            },
            // With only one result, always cd to it
            _ if valid_dirs.len() == 1 => Some(valid_dirs[0].name.clone()),
            // Sorted by score
            Mode::Score => Some(valid_dirs[0].name.clone()),
            Mode::Shortest => valid_dirs
                .iter()
                .min_by_key(|dir| dir.name.len())
                .map(|dir| dir.name.clone()),
            Mode::Basename => {
                let found = valid_dirs
                    .iter()
                    .find(|dir| basename_matches(&dir.name, patterns))
                    .map(|dir| dir.name.clone());
                if found.is_none() {
                    self.show_exit_message("No dirs");
                }
                found
            },
            Mode::Select => self.select_valid_dir(valid_dirs, 0),
        };
        if let Some(dir) = selected {
            self.direct_cd(dir);
        }
    }


    pub(crate) fn list_matching_dirs(&mut self, patterns: &[String]) {
        let valid_dirs = self.get_valid(patterns, false);
        if let Some(dir_name) = self.select_valid_dir(valid_dirs, 0) {
            self.direct_cd(dir_name);
        }
    }


    pub(crate) fn replace_alias(&mut self, dir_str: &str, alias: &str,) {
        for dir in self.dirs.iter_mut() {
            if dir.alias == alias {
                dir.alias = "".to_string();
            }
            if dir.name == dir_str {
                dir.alias = alias.to_string();
            }
        }
    }


    pub(crate) fn add_alias(&mut self, args: AliasArgs) {
        // `-a DIR` removes the alias of DIR; `-a ALIAS DIR` sets it
        let (alias, dir_str, remove) = match &args {
            AliasArgs::Select => ("", "", false),
            AliasArgs::Remove(dir) => ("", dir.as_str(), true),
            AliasArgs::Set { alias, dir } => (alias.as_str(), dir.as_str(), false),
        };
        if args == AliasArgs::Select {
            println!("Aliased dirs");
            let valid_dirs = self.get_valid(&[], true);
            if let Some(dir_name) = self.select_valid_dir(valid_dirs, 0) {
                self.direct_cd(dir_name);
            }
            return;
        }

        if Path::new(dir_str).is_dir() {
            let dir_str = canonicalize_dir_str(dir_str);
            let details = format!("{}->{}", alias, dir_str);
            if self.get(&dir_str).is_none() {
                self.insert_with_alias(&dir_str, Some(alias));
                self.show_exit_detailed_message(
                    if remove { "Added dir (no alias)" } else { "Added dir alias" },
                    &details,
                );
            } else if remove {
                self.remove_alias(&dir_str);
                self.show_exit_detailed_message("Removed dir alias", &details);
            } else {
                self.replace_alias(&dir_str, alias);
                self.show_exit_detailed_message("Added dir alias", &details);
            }
        } else {
            // Not a directory: use the argument as the alias of a dir that is
            // selected from the list
            println!("Select directory to alias");
            let valid_dirs = self.get_valid(&[], false);
            if let Some(dir_name) = self.select_valid_dir(valid_dirs, 0) {
                self.replace_alias(&dir_name, dir_str);
                let details = format!("{}->{}", dir_str, dir_name);
                self.show_exit_detailed_message("Added dir alias", &details);
            }
        }
    }


    pub(crate) fn insert(&mut self, dir: &str) {
        self.insert_with_alias(dir, None);
    }


    pub(crate) fn insert_with_alias(&mut self, dir: &str, alias: Option<&str>) {
        // println!("inserting dir: {}", dir);
        // Check if dir is already in dirs
        let mut found = false;
        for d in self.dirs.iter_mut() {
            if d.name == dir {
                d.counter += 1.0;
                d.last_access = get_current_seconds();
                if let Some(alias) = alias {
                    d.alias = alias.to_string();
                }
                found = true;
                break;
            }
        }
        // If not, add it
        if !found {
            let alias = alias.unwrap_or_default();
            let dir = Directory {
                name: dir.to_string(),
                counter: 1.0,
                last_access: get_current_seconds(),
                score: 0.0,
                alias: alias.to_string(),
            };
            self.dirs.push(dir);
        }
    }


    pub(crate) fn get_by_alias(&self, alias: &str) -> Option<Directory> {
        if alias.is_empty() {
            return None;
        }
        self.dirs.iter().find(|dir| dir.alias == alias).cloned()
    }


    pub(crate) fn get(&self, name: &str) -> Option<Directory> {
        self.dirs.iter().find(|dir| dir.name == name).cloned()
    }


    /// Existing directories (best score first) that contain all the
    /// `patterns`; if `alias_only`, only those with an alias.
    pub(crate) fn get_valid(&mut self, patterns: &[String], alias_only: bool) -> Vec<Directory> {
        self.dirs.sort_by(|a, b| b.score.total_cmp(&a.score));
        self.dirs
            .iter()
            .filter(|dir| patterns.iter().all(|p| dir.name.contains(p.as_str())))
            .filter(|dir| !alias_only || !dir.alias.is_empty())
            .filter(|dir| Path::new(&dir.name).exists())
            .cloned()
            .collect()
    }


    pub(crate) fn remove_alias(&mut self, dir_str: &str) {
        for dir in self.dirs.iter_mut() {
            if dir.name == dir_str {
                dir.alias = "".to_string();
                break;
            }
        }
    }


    fn get_all_dirs(&mut self, existent_only: bool) -> Vec<Directory> {
        let mut all_dirs: Vec<Directory> = Vec::new();
        for dir in self.dirs.iter() {
            if existent_only && !Path::new(&dir.name).exists() {
                continue;
            }
            all_dirs.push(dir.clone());
        }
        all_dirs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        all_dirs
    }


    pub(crate) fn list_existent(&mut self, max_num: Option<usize>) {
        let all_dirs = self.get_all_dirs(true);
        self.list_dirs(&all_dirs, max_num.unwrap_or(0), 1);
    }


    pub(crate) fn list_all(&mut self) {
        let all_dirs = self.get_all_dirs(false);
        self.list_dirs(&all_dirs, 0, 1);
    }


    pub(crate) fn sync_dirs(&mut self) {
        self.dirs.retain(|dir| Path::new(&dir.name).exists());
    }


    pub(crate) fn remove(&mut self, dir_str: &str) {
        if let Some(i) = self.dirs.iter().position(|dir| dir.name == dir_str) {
            self.dirs.remove(i);
        }
    }


    pub(crate) fn go_to_last(&mut self) {
        // Sort by last access
        self.dirs.sort_by_key(|dir| std::cmp::Reverse(dir.last_access));
        // Get the first dir
        if self.dirs.is_empty() {
            self.show_exit_message("No dirs");
            return;
        }
        let dir = self.dirs[0].name.clone();
        self.direct_cd(dir);
    }


    pub(crate) fn go_to_previous(&mut self) {
        // Sort by last access
        self.dirs.sort_by_key(|dir| std::cmp::Reverse(dir.last_access));
        // Get the second dir
        if self.dirs.len() > 1 {
            let dir = self.dirs[1].name.clone();
            self.direct_cd(dir);
        }
    }


    pub(crate) fn clear_database(&mut self) {
        self.dirs.clear();
    }


    pub(crate) fn remove_alias_interactive(&mut self) {
        let valid_dirs = self.get_valid(&[], true);
        if let Some(dir_name) = self.select_valid_dir(valid_dirs, 0) {
            self.remove_alias(&dir_name);
            let details = format!("->{}", dir_name);
            self.show_exit_detailed_message("Removed dir alias", &details);
        }
    }


    pub(crate) fn interactive_cd(&mut self) {
        let valid_dirs = self.get_valid(&[], false);
        if let Some(dir_name) = self.select_valid_dir(valid_dirs, 0) {
            self.direct_cd(dir_name);
        }
    }


    pub(crate) fn remove_dirs(&mut self, patterns: &[String]) {
        let valid_dirs = self.get_valid(patterns, false);
        if valid_dirs.is_empty() {
            self.show_exit_message("No dirs");
            return;
        }
        for dir_name in self.select_valid_dirs(valid_dirs, 0) {
            self.remove(&dir_name);
        }
        self.show_exit_message("Removed directories");
    }


    /// Browse the subdirectories of the current directory, one level at a
    /// time, and cd to the last one that was selected. Only that final
    /// directory is recorded in the database.
    pub(crate) fn interactive_navigation(&mut self, hidden: bool, dirs_only: bool) {
        let mut selected_dir: Option<String> = None;
        loop {
            let base = selected_dir.as_deref().unwrap_or(".");
            let (mut dirs, files) = match read_entries(Path::new(base), hidden, dirs_only) {
                Ok(entries) => entries,
                Err(error) => {
                    self.show_warning("Cannot read directory", &error.to_string());
                    break;
                }
            };
            dirs.push("..".to_string());
            dirs.sort();

            let name = match self.select_entry(&dirs, &files) {
                Some(name) => name,
                None => break,
            };
            println!();

            let mut path = Path::new(base).join(name);
            if self.config.abs_paths {
                path = match path.canonicalize() {
                    Ok(path) => path,
                    Err(error) => {
                        self.show_warning("Cannot access directory", &error.to_string());
                        break;
                    }
                };
            }
            let path = match path.to_str() {
                Some(path) => path.to_string(),
                None => break,
            };
            println!("{}", self.format("bold", "", path.clone()));
            selected_dir = Some(path);
        }
        if let Some(dir) = selected_dir {
            self.direct_cd(dir);
        }
    }
}


/// Subdirectory names and file names of `dir` (names that are not valid
/// UTF-8 are skipped). Hidden entries are only included if `hidden`; files are
/// not included if `dirs_only`.
fn read_entries(dir: &Path, hidden: bool, dirs_only: bool) -> io::Result<(Vec<String>, Vec<String>)> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };
        if !hidden && name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            dirs.push(name);
        } else if !dirs_only && path.exists() {
            files.push(name);
        }
    }
    Ok((dirs, files))
}


/// Whether the basename of `path` contains all the `patterns`.
fn basename_matches(path: &str, patterns: &[String]) -> bool {
    match Path::new(path).file_name() {
        Some(basename) => {
            let basename = basename.to_string_lossy();
            patterns.iter().all(|p| basename.contains(p.as_str()))
        }
        None => false,
    }
}
