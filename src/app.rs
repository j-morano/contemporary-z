//! The commands: they combine the configuration, the database and the UI.

use std::path::Path;

use crate::cli::AliasArgs;
use crate::config::{Config, Mode};
use crate::db::{pick, valid_alias, Database, Directory};
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
    fn existing(&self, patterns: &[String], alias_only: bool) -> Vec<Directory> {
        let mut dirs = self.db.matching(patterns, alias_only);
        dirs.retain(|dir| Path::new(&dir.name).exists());
        dirs
    }


    /// Tell the shell to cd to `dir_name` and register the visit.
    fn direct_cd(&mut self, dir_name: String) {
        self.db.insert(&dir_name, None);
        write_dir(dir_name);
    }


    /// `patterns[0]` is tried as an alias, then as a directory, and finally all
    /// the patterns are used as substrings. `forced` overrides the configured
    /// substring mode.
    pub(crate) fn do_cd(&mut self, patterns: &[String], forced: Option<Mode>) {
        let first = match patterns.first() {
            Some(first) => first.as_str(),
            None => self.ui.show_error("No directory or substring provided", ""),
        };

        // If string is an alias, then cd to the directory, if exists
        if let Some(dir) = self.db.by_alias(first) {
            let name = dir.name.clone();
            if !Path::new(&name).is_dir() {
                self.ui.show_error("Aliased directory does not exist", &name);
            }
            self.direct_cd(name);
            return;
        }
        // If it is a dir AND exists in the FS, cd to it (adding it to the table)
        if let Some(dir) = canonical_dir(first) {
            self.direct_cd(dir);
            return;
        }
        let mode = forced.unwrap_or(self.config.substring);
        self.cd_by_substrings(patterns, mode);
    }


    fn cd_by_substrings(&mut self, patterns: &[String], mode: Mode) {
        let dirs = self.existing(patterns, false);
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
        let dirs = self.existing(patterns, false);
        if let Some(dir_name) = self.ui.select_dir(&dirs) {
            self.direct_cd(dir_name);
        }
    }


    /// No arguments: choose among all the directories.
    pub(crate) fn interactive_cd(&mut self) {
        self.list_matching_dirs(&[]);
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
            let dirs = self.existing(&[], true);
            if let Some(dir_name) = self.ui.select_dir(&dirs) {
                self.direct_cd(dir_name);
            }
            return;
        }

        if let Some(dir_str) = canonical_dir(dir_str) {
            if !remove && !valid_alias(alias) {
                self.ui.show_error("Invalid alias", alias);
            }
            let details = format!("{}->{}", alias, dir_str);
            if !self.db.contains(&dir_str) {
                self.db.insert(&dir_str, Some(alias));
                self.ui.show_detailed_message(
                    if remove { "Added dir (no alias)" } else { "Added dir alias" },
                    &details,
                );
            } else if remove {
                self.db.remove_alias(&dir_str);
                self.ui.show_detailed_message("Removed dir alias", &details);
            } else {
                self.db.set_alias(&dir_str, alias);
                self.ui.show_detailed_message("Added dir alias", &details);
            }
        } else {
            // Not a directory: use the argument as the alias of a dir that is
            // selected from the list
            if !valid_alias(dir_str) {
                self.ui.show_error("Invalid alias", dir_str);
            }
            println!("Select directory to alias");
            let dirs = self.existing(&[], false);
            if let Some(dir_name) = self.ui.select_dir(&dirs) {
                self.db.set_alias(&dir_name, dir_str);
                let details = format!("{}->{}", dir_str, dir_name);
                self.ui.show_detailed_message("Added dir alias", &details);
            }
        }
    }


    pub(crate) fn remove_alias_interactive(&mut self) {
        let dirs = self.existing(&[], true);
        if let Some(dir_name) = self.ui.select_dir(&dirs) {
            self.db.remove_alias(&dir_name);
            let details = format!("->{}", dir_name);
            self.ui.show_detailed_message("Removed dir alias", &details);
        }
    }


    pub(crate) fn remove_dirs(&mut self, patterns: &[String]) {
        let dirs = self.existing(patterns, false);
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
        self.ui.list_dirs(&self.existing(&[], false), max_num.unwrap_or(0));
    }


    pub(crate) fn list_all(&self) {
        self.ui.list_dirs(&self.db.matching(&[], false), 0);
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
