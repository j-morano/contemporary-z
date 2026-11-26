//! The commands: they combine the configuration, the database and the UI.

use std::path::Path;

use crate::config::{Config, Mode};
use crate::db::{pick, Database, Directory};
use crate::ui::Ui;
use crate::utils::{canonical_dir, write_dir};


pub(crate) struct App {
    config: Config,
    ui: Ui,
    db: Database,
}


impl App {
    pub(crate) fn new(config: Config, db: Database) -> App {
        App { ui: Ui::new(config.clone()), config, db }
    }


    /// Age the database and write it to disk.
    pub(crate) fn save(&mut self) {
        self.db.age(self.config.max_age as f64);
        self.db.save(&self.config.database_path);
    }


    pub(crate) fn show_database_path(&self) {
        println!("{}", self.config.database_path);
    }


    /// Existing directories that match, best score first.
    fn existing(&self, patterns: &[String]) -> Vec<Directory> {
        let mut dirs = self.db.matching(patterns);
        dirs.retain(|dir| Path::new(&dir.name).exists());
        dirs
    }


    /// Tell the shell to cd to `dir_name` and register the visit.
    fn direct_cd(&mut self, dir_name: String) {
        self.db.insert(&dir_name);
        write_dir(dir_name);
    }


    /// `patterns[0]` is tried as a directory, and otherwise all the patterns
    /// are used as substrings. `forced` overrides the configured
    /// substring mode.
    pub(crate) fn do_cd(&mut self, patterns: &[String], forced: Option<Mode>) {
        let first = match patterns.first() {
            Some(first) => first.as_str(),
            None => self.ui.show_error("No directory or substring provided", ""),
        };

        // If it is a dir AND exists in the FS, cd to it (adding it to the table)
        if let Some(dir) = canonical_dir(first) {
            self.direct_cd(dir);
            return;
        }
        let mode = forced.unwrap_or(self.config.substring);
        self.cd_by_substrings(patterns, mode);
    }


    fn cd_by_substrings(&mut self, patterns: &[String], mode: Mode) {
        let dirs = self.existing(patterns);
        let selected = if mode == Mode::Select && dirs.len() > 1 {
            // Interactively select among all the dirs that match
            self.ui.select_dir(&dirs)
        } else {
            pick(&dirs, patterns, mode).map(|dir| dir.name.clone())
        };
        match selected {
            Some(dir) => self.direct_cd(dir),
            None => self.ui.show_message("No dirs"),
        }
    }


    /// `-f`: always list the matching directories.
    pub(crate) fn list_matching_dirs(&mut self, patterns: &[String]) {
        let dirs = self.existing(patterns);
        if let Some(dir_name) = self.ui.select_dir(&dirs) {
            self.direct_cd(dir_name);
        }
    }


    /// No arguments: choose among all the directories.
    pub(crate) fn interactive_cd(&mut self) {
        self.list_matching_dirs(&[]);
    }


    pub(crate) fn remove_dirs(&mut self, patterns: &[String]) {
        let dirs = self.existing(patterns);
        if dirs.is_empty() {
            self.ui.show_message("No dirs");
            return;
        }
        for dir_name in self.ui.select_dirs(&dirs) {
            self.db.remove(&dir_name);
        }
        self.ui.show_message("Removed directories");
    }


    /// `-l [NUMBER]`
    pub(crate) fn list_existent(&self, max_num: Option<usize>) {
        self.ui.list_dirs(&self.existing(&[]), max_num.unwrap_or(0));
    }


    pub(crate) fn list_all(&self) {
        self.ui.list_dirs(&self.db.matching(&[]), 0);
    }


    /// Remove the directories that do not exist.
    pub(crate) fn sync_dirs(&mut self) {
        self.db.retain(|dir| Path::new(&dir.name).exists());
    }


    pub(crate) fn clear_database(&mut self) {
        self.db.clear();
    }


    /// The `n`-th most recently accessed directory that still exists.
    fn nth_recent(&self, n: usize) -> Option<String> {
        self.db.by_recency().into_iter().filter(|dir| Path::new(dir).is_dir()).nth(n)
    }


    /// `=`: the most recently accessed directory.
    pub(crate) fn go_to_last(&mut self) {
        match self.nth_recent(0) {
            Some(dir) => self.direct_cd(dir),
            None => self.ui.show_message("No dirs"),
        }
    }


    /// `-`: the directory accessed before the last one.
    pub(crate) fn go_to_previous(&mut self) {
        match self.nth_recent(1) {
            Some(dir) => self.direct_cd(dir),
            None => self.ui.show_message("No dirs"),
        }
    }
}
