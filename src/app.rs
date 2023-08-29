use crate::data::Directory;
use crate::utils::canonicalize_dir_str;
use crate::utils::write_dir;

use std::fs::metadata;
use std::process::exit;
use std::env;
use std::io::prelude::*;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::colors::{color_code, sgr_code};
use std::path::Path;
use std::path::PathBuf;
use std::fs;



pub(crate) fn current_seconds() -> i64 {
    return SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
}

pub(crate) fn get_home_dir() -> String {
    let current_home_dir = env::var("HOME").unwrap();
    return current_home_dir;
}

pub(crate) fn get_user() -> String {
    let current_user = env::var("USER").unwrap();
    return current_user;
}

#[derive(Debug, Clone)]
pub struct SelectionError;


#[allow(dead_code)]
pub(crate) struct App <'a> {
    pub(crate) theme: String,
    pub(crate) abs_paths: bool,
    pub(crate) compact_paths: bool,
    pub(crate) max_results: usize,
    pub(crate) database_path: String,
    pub(crate) substring: String,
    pub(crate) show_files: String,
    pub(crate) nav_start_number: usize,
    pub(crate) dirs: &'a mut Vec<Directory>,
}

impl App <'_> {
    pub(crate) fn format(&self, sgr: &str, color: &str, text: String) -> String {
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

    pub(crate) fn show_exit_detailed_message(&self, message: &str, details: &str) {
        println!(
            "{}: {}",
            self.format("bold", "green", message.to_string()),
            details,
        );
        exit(1);
    }

    pub(crate) fn select_dir(&self) -> String {
        let mut line = String::new();
        print!("Number: ");
        io::stdout().flush().expect("Could not flush output");
        std::io::stdin().read_line(&mut line).unwrap();
        return line.replace('\n', "");
    }


    pub(crate) fn list_dirs(&self, valid_dirs: &Vec<Directory>, max_num: usize, start: usize) {
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
                    dir_name = dir_name.replace(current_home_dir.as_str(), "~");

                    // Replace (/run)/media/<user> with '>'
                    let user = get_user();
                    let media_user = format!("/media/{}", user);
                    let run_media_user = format!("/run/media/{}", user);
                    dir_name = dir_name.replace(run_media_user.as_str(), ">");
                    dir_name = dir_name.replace(media_user.as_str(), ">");
                }

                let mut alias = String::new();
                if !dir.alias.is_empty() {
                    alias = format!("{}:", dir.alias);
                }

                println!(
                    "{}) {}{} {}",
                    self.format("bold", "", (i+start).to_string()),
                    alias,
                    self.format("bold", "blue", dir_name),
                    (i+start),
                    // dir.score
                );
                if i == (max_results - start) {
                    break;
                }
            }
        }
    }


    fn print_files(files: Vec<String>) {
        // check if files is empty
        if files.len() > 0 {
            for file in files {
                println!("{}", file.to_string());
            }
        }
    }


    pub(crate) fn select_valid_dir_no_exit(
        &self,
        valid_dirs: Vec<Directory>,
        max_num: usize,
        start: usize,
        files: Vec<String>,
    ) -> Result<String, SelectionError>
    {
        if self.show_files == "top" {
            App::print_files(files.clone());
        }
        self.list_dirs(&valid_dirs, max_num, start);
        if self.show_files == "bottom" {
            App::print_files(files.clone());
        }
        println!();

        // Select dir by number
        let selected_dir = match self.select_dir().parse::<usize>() {
            Ok(number)  => number,
            Err(_) => {
                return Err(SelectionError);
            },
        };

        // Check if the introduced number is valid
        if selected_dir > valid_dirs.len() + start - 1 || selected_dir < start {
            return Err(SelectionError);
        }

        // Get name of the selected dir
        let dir_name = format!("{}", valid_dirs[selected_dir-start].name);

        return Ok(dir_name);
    }

    pub(crate) fn select_valid_dirs(&self, valid_dirs: Vec<Directory>, max_num: usize) -> Result<Vec<String>, String> {

        self.list_dirs(&valid_dirs, max_num, 1);
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

    pub(crate) fn select_valid_dir(&self, valid_dirs: Vec<Directory>, max_num: usize) -> Result<String, String> {
        let mut i = 0;
        let mut selected_dir: String;
        let mut dirs_to_show = &valid_dirs[0..];
        let mut starting_index = 0;
        let number_of_pages = (valid_dirs.len() + self.max_results - 1) / self.max_results;

        loop {
            if number_of_pages > 0 {
                println!("[{}/{}]", i+1, number_of_pages);
            }
            self.list_dirs(&dirs_to_show.to_vec(), max_num, 1);
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


    pub(crate) fn direct_cd(&mut self, dir_name: String) {
        self.insert(dir_name.as_str());
        write_dir(dir_name.clone());
    }

    pub(crate) fn do_cd(
        &mut self,
        args: &[String],
        forced_substring: &str,
    ) {
        // Directory argument
        let mut starting_index = 1;
        if forced_substring != "none" {
            starting_index = 2;
        }
        let mut dir_str = args[starting_index].as_str();

        // If string is an alias, then cd to the directory, if exists
        match self.get_by_alias(dir_str) {
            Ok(dir) => {
                let dir_str = dir.name.as_str();
                if Path::new(dir_str).exists()
                    && metadata(dir_str).unwrap().is_dir()
                {
                    self.direct_cd(dir.name);
                }
            },
            Err(_) => {
                // If it is a dir AND exists in the FS
                if Path::new(dir_str).exists()
                    && metadata(dir_str).unwrap().is_dir()
                {
                    let canonical_dir = canonicalize_dir_str(dir_str);
                    dir_str = canonical_dir.as_str();

                    // Check if dir is in the table
                    match self.get(dir_str) {
                        Ok(dir) => {
                            // If the dir is not in the table and it does exists in the
                            //   FS, add it
                            self.direct_cd(dir.name);
                        },
                        Err(_) => {
                            self.direct_cd(dir_str.to_string());
                        }
                    }
                } else { // if arguments are substrings, go to the parent folder of the
                         // top results that matches the substrings
                    // Get shortest directory
                    let valid_dirs = self.get_valid(
                        Vec::from(&args[starting_index..]), false
                    ).unwrap();

                    if valid_dirs.is_empty() {
                        self.show_exit_message("No dirs");
                    } else {
                        // If there is only one result, cd to it
                        if
                            valid_dirs.len() == 1
                            || (self.substring == "score" && forced_substring != "shortest")
                        {
                            // Access the substring with the highest score
                            let selected_dir = valid_dirs[0].name.clone();
                            // app.direct_cd(&conn, selected_dir.clone());
                            self.direct_cd(selected_dir);
                        } else {
                            // Access the uppermost dir that matches the substring(s)
                            if self.substring == "shortest" || forced_substring == "shortest" {
                                let mut selected_dir = valid_dirs[0].name.as_str();
                                for dir in valid_dirs.iter() {
                                    if dir.name.len() < selected_dir.len() {
                                        selected_dir = dir.name.as_str();
                                    }
                                }
                                // app.direct_cd(&conn, selected_dir.to_string());
                                self.direct_cd(selected_dir.to_string());
                            } else {
                                // Interactively select dir among all the dirs that
                                // match the substring(s)
                                let dir_name = self.select_valid_dir(valid_dirs, 0).unwrap();
                                // app.direct_cd(&conn, dir_name.clone());
                                self.direct_cd(dir_name);
                            }
                        }
                    }
                }
            },
        };
    }


    pub(crate) fn list_matching_dirs(&mut self, args: &[String]) {
        if args.len() < 3 {
            self.show_error("No substring provided", "");
        } else {
            let valid_dirs = self.get_valid(Vec::from(&args[2..]), false).unwrap();
            if valid_dirs.is_empty() {
                self.show_exit_message("No dirs");
            } else {
                // Interactively select dir among all the dirs that
                // match the substring(s)
                let dir_name = self.select_valid_dir(valid_dirs, 0).unwrap();
                self.direct_cd(dir_name.clone());
            }
        }
    }

    
    pub(crate) fn replace_alias(&mut self, dir_str: &str, alias: &str,) {
        for dir in self.dirs.iter_mut() {
            if dir.alias == alias {
                dir.alias = alias.to_string();
            }
            if dir.name == dir_str {
                dir.alias = alias.to_string();
            }
        }
    }



    pub(crate) fn add_alias(&mut self, args: &[String]) {
        if args.len() < 3 {
            println!("Aliased dirs");
            let valid_dirs = self.get_valid(
                Vec::new(), true
            ).unwrap();

            // Always list dirs
            let dir_name = self.select_valid_dir(valid_dirs, 0).unwrap();
            self.direct_cd(dir_name.clone());
        } else {
            let mut alias = &String::from("");
            let mut dir_str;
            if args.len() < 4 {
                // Remove alias
                dir_str = args[2].as_str();
            } else {
                alias = &args[2];
                dir_str = args[3].as_str();
            }

            if Path::new(dir_str).exists()
                && metadata(dir_str).unwrap().is_dir()
            {
                let canonical_dir = canonicalize_dir_str(dir_str);
                dir_str = canonical_dir.as_str();

                // Check if dir is in the table
                let dir = self.get(dir_str);

                // If the dir is not in the table and it does exists in the
                //   FS, add it
                if let Err(_err) = dir {
                    // Do not store '..' or '.' dirs
                    if !(dir_str == "." || dir_str == "..") {
                        self.insert_with_alias(dir_str, alias.as_str());
                        let details = format!("{}->{}", alias, dir_str);
                        self.show_exit_detailed_message("Removed dir alias", details.as_str());
                    }
                } else {
                    if args.len() < 4 {
                        self.remove_alias(dir_str);
                        let details = format!("{}->{}", alias, dir_str);
                        self.show_exit_detailed_message("Removed dir alias", details.as_str());
                    } else {
                        // add_alias_to_directory_unique(&conn, dir_str, alias.as_str()).unwrap();
                        self.replace_alias(dir_str, alias.as_str());
                        let details = format!("{}->{}", alias, dir_str);
                        self.show_exit_detailed_message("Added dir alias", details.as_str());
                    }
                }
            } else {
                println!("Select directory to alias");
                // app.show_error("The provided directory does not exist", "");
                let valid_dirs = self.get_valid(
                    Vec::new(), false
                ).unwrap();

                // Always list dirs
                let dir_name = self.select_valid_dir(valid_dirs, 0).unwrap();
                // add_alias_to_directory_unique(&conn, &dir_name, dir_str).unwrap();
                self.replace_alias(&dir_name, dir_str);
                let details = format!("{}->{}", dir_str, dir_name);
                self.show_exit_detailed_message("Added dir alias", details.as_str());
            }
        }
    }



    pub(crate) fn remove_old(&mut self) {
        let current_seconds = current_seconds();
        let seconds_in_a_month = 60 * 60 * 24 * 30;
        let limit = current_seconds - seconds_in_a_month;
        // Remove old dirs, i.e. dirs that have not been accessed in a month
        self.dirs.retain(|dir| dir.last_access > limit);
    }


    pub(crate) fn insert(&mut self, dir: &str) {
        self.insert_with_alias(dir, "");
    }



    pub(crate) fn insert_with_alias(&mut self, dir: &str, alias: &str) {
        // println!("inserting dir: {}", dir);
        // Check if dir is already in dirs
        let mut found = false;
        for d in self.dirs.iter_mut() {
            if d.name == dir {
                d.counter += 1;
                d.last_access = current_seconds();
                d.alias = alias.to_string();
                found = true;
                break;
            }
        }
        // If not, add it
        if !found {
            let dir = Directory {
                name: dir.to_string(),
                counter: 1,
                last_access: current_seconds(),
                score: 0.0,
                alias: alias.to_string(),
            };
            self.dirs.push(dir);
        }
    }


    pub(crate) fn get_by_alias(&mut self, alias: &str) -> Result<Directory, String> {
        for dir in self.dirs.iter() {
            if dir.alias == alias {
                return Ok(dir.clone());
            }
        }
        Err("Alias not found".to_string())
    }


    pub(crate) fn get(&mut self, name: &str) -> Result<Directory, String> {
        for d in self.dirs.iter() {
            if d.name == name {
                return Ok(d.clone());
            }
        }
        Err("Directory not found".to_string())
    }


    pub(crate) fn compute_score(&mut self, current_seconds: i64) {
        // 'Frecency' formula: https://github.com/rupa/z/blob/master/z.sh
        for dir in self.dirs.iter_mut() {
            dir.score = 10000.0 * dir.counter as f64 * (3.75 / ((0.0001 * (current_seconds - dir.last_access) as f64 + 1.0) + 0.25));
        }
    }


    pub(crate) fn get_valid( 
        &mut self,
        patterns: Vec<String>,
        alias_only: bool,
    ) -> Result<Vec<Directory>, String> {
        // Filter invalid dirs from the current path
        let mut valid_dirs: Vec<Directory> = Vec::new();

        // Sub-string coincidences
        let mut pattern = String::new();
        if !patterns.is_empty() {
            pattern = patterns.join("*");
            pattern = format!("*{}*", pattern);
        }

        // Sort by score
        self.dirs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Filter by pattern
        let mut filtered_dirs: Vec<Directory> = Vec::new();
        for dir in self.dirs.iter() {
            if pattern.is_empty() || dir.name.contains(&pattern) {
                filtered_dirs.push(dir.clone());
            }
        }

        // Filter by alias
        if alias_only {
            let mut alias_dirs: Vec<Directory> = Vec::new();
            for dir in filtered_dirs.iter() {
                if dir.alias != "" {
                    alias_dirs.push(dir.clone());
                }
            }
            filtered_dirs = alias_dirs;
        }
        
        // Filter by existence
        for dir in filtered_dirs.iter() {
            if Path::new(&dir.name).exists() {
                valid_dirs.push(dir.clone());
            }
        }

        Ok(valid_dirs)
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
        return all_dirs;
    }


    pub(crate) fn list_existent(&mut self) {
        let all_dirs = self.get_all_dirs(true);
        self.list_dirs(&all_dirs, 0, 1);
    }


    pub(crate) fn list_all(&mut self) {
        let all_dirs = self.get_all_dirs(false);
        self.list_dirs(&all_dirs, 0, 1);
    }

    pub(crate) fn sync_dirs(&mut self) {
        let mut dirs_to_remove: Vec<String> = Vec::new();
        for dir in self.dirs.iter() {
            if !Path::new(&dir.name).exists() {
                dirs_to_remove.push(dir.name.clone());
            }
        }
        for dir in dirs_to_remove.iter() {
            self.remove(dir);
        }
    }

    pub(crate) fn remove(&mut self, dir_str: &str) {
        let mut i = 0;
        for dir in self.dirs.iter() {
            if dir.name == dir_str {
                self.dirs.remove(i);
                break;
            }
            i += 1;
        }
    }


    pub(crate) fn go_to_last(&mut self) {
        // Sort by last access
        self.dirs.sort_by(|a, b| b.last_access.partial_cmp(&a.last_access).unwrap());
        // Get the first dir
        let dir = self.dirs[0].name.clone();
        self.direct_cd(dir);
    }


    pub(crate) fn go_to_previous(&mut self) {
        // Sort by last access
        self.dirs.sort_by(|a, b| b.last_access.partial_cmp(&a.last_access).unwrap());
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
        let valid_dirs = self.get_valid(
            Vec::new(), true
        ).unwrap();

        // Always list dirs
        let dir_name = self.select_valid_dir(valid_dirs, 0).unwrap();
        self.remove_alias(&dir_name);
        let details = format!("{}->{}", "", dir_name);
        self.show_exit_detailed_message("Removed dir alias", details.as_str());
    }

    pub(crate) fn interactive_cd(&mut self, args: &[String]) {
        let valid_dirs = self.get_valid(
            Vec::from(&args[1..]), false
        ).unwrap();

        // Always list dirs
        let dir_name = self.select_valid_dir(valid_dirs, 0).unwrap();
        self.direct_cd(dir_name.clone());
    }


    pub(crate) fn remove_dirs(&mut self, args: &[String]) {
        let valid_dirs = self.get_valid(
            Vec::from(&args[2..]), false
        ).unwrap();

        let dir_names = self.select_valid_dirs(valid_dirs, 0).unwrap();

        let all_dirs_removed = true;
        for dir_name in dir_names {
            self.remove(&dir_name);
        }
        if all_dirs_removed {
            self.show_exit_message("Removed directories");
        } else {
            self.show_error("Could not remove directories", "");
        }
    }


    pub(crate) fn interactive_navigation(
        &mut self,
        hidden: bool,
        force_dir_only: bool,
    ) {
        let mut dir_to_read = String::from(".");
        loop {
            let paths = fs::read_dir(dir_to_read.as_str()).unwrap();
            let mut valid_dirs: Vec<Directory> = Vec::new();
            let mut files: Vec<String> = Vec::new();

            for result_path in paths {
                let dir_path = result_path.unwrap().path();
                if dir_path.exists()
                {
                    if dir_path.is_dir() {
                        let filename = String::from(
                            dir_path.file_name().unwrap().to_str().unwrap()
                        );
                        if (hidden && filename != ".")
                            || (!hidden  && !filename.starts_with("."))
                        {
                            let directory = Directory{
                                name: filename.clone(),
                                counter: 0,
                                last_access: 0,
                                score: 0.0,
                                alias: String::new()
                            };
                            valid_dirs.push(directory);
                        }
                    } else {
                        if !force_dir_only {
                            // Add to files
                            let filename = String::from(
                                dir_path.file_name().unwrap().to_str().unwrap()
                            );
                            files.push(filename);
                        }
                    }
                }
            }
            // Parent directory
            let directory = Directory{
                name: "..".to_string(),
                counter: 0,
                last_access: 0,
                score: 0.0,
                alias: String::new()
            };
            valid_dirs.push(directory);
            // Sort dirs by name
            valid_dirs.sort_by_key(|dir| dir.name.clone());

            if valid_dirs.is_empty() {
                break;
            }

            let dir_name: String; //= String::new();
            match self.select_valid_dir_no_exit(valid_dirs, usize::MAX, self.nav_start_number, files) {
                Ok(dir_string)  => {
                    dir_name = dir_string
                }
                Err(_error) => {
                    break;
                }
            };
            println!();

            let dir_name_str = dir_name.as_str();
            let base_dir_str = dir_to_read.as_str();
            let dir_path = Path::new(base_dir_str);
            let dir_path_buf = dir_path.join(dir_name_str);
            let mut dir_str = dir_path_buf.to_str().unwrap();

            let dir_pathbuf;
            if self.abs_paths {
                dir_pathbuf = PathBuf::from(dir_str).canonicalize().unwrap();
                dir_str = dir_pathbuf.to_str().unwrap();
            }

            // Check if dir is in the table
            let dir = self.get(dir_str);

            // If the dir is not in the table and it does exists in the
            //   FS, add it
            if let Err(_err) = dir {
                // Do not store '..' or '.' dirs
                if !(dir_str == "." || dir_str == "..") {
                    self.insert(dir_str);
                }
                dir_to_read = String::from(dir_str);

            } else { // if it is already present in the table, update its
                     // counter
                match dir {
                    Ok(dir_string)  => {
                        dir_to_read = dir_string.name.clone();
                    }
                    Err(error) => {
                        self.show_error("Directory does not exist", error.to_string().as_str());
                    }
                };
            }
            // print in bold dir_to_read
            println!("{}", self.format("bold", "", dir_to_read.to_string()));
            // println!("{}", dir_to_read);
        }
        self.direct_cd(dir_to_read);
    }
}
