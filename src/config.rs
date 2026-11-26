use std::fs;
use std::str::FromStr;

use crate::utils::get_home_dir;


#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Theme {
    Dark,
    Bright,
}

impl FromStr for Theme {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "dark" => Ok(Theme::Dark),
            "bright" => Ok(Theme::Bright),
            _ => Err(()),
        }
    }
}


/// How to choose among the directories that match the given substring(s).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Mode {
    /// Directory with the shortest path.
    Shortest,
    /// Directory with the highest score (most 'frecent').
    Score,
    /// Best-scored directory whose basename matches.
    Basename,
    /// Show a selection list (config value: `none`).
    Select,
}

impl FromStr for Mode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "shortest" => Ok(Mode::Shortest),
            "score" => Ok(Mode::Score),
            "basename" => Ok(Mode::Basename),
            "none" => Ok(Mode::Select),
            _ => Err(()),
        }
    }
}


/// Where to show non-directory files in interactive navigation.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum ShowFiles {
    Top,
    Bottom,
    /// Config value: `none`.
    Off,
}

impl FromStr for ShowFiles {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "top" => Ok(ShowFiles::Top),
            "bottom" => Ok(ShowFiles::Bottom),
            "none" => Ok(ShowFiles::Off),
            _ => Err(()),
        }
    }
}


#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Config {
    pub(crate) theme: Theme,
    pub(crate) abs_paths: bool,
    pub(crate) compact_paths: bool,
    pub(crate) max_results: usize,
    pub(crate) database_path: String,
    pub(crate) substring: Mode,
    pub(crate) show_files: ShowFiles,
    pub(crate) nav_start_number: usize,
    /// Limit of the sum of all the visit counters (see `data::age`).
    pub(crate) max_age: usize,
}

fn expand_home(path: &str) -> String {
    path.replace("$HOME", &get_home_dir())
}

impl Default for Config {
    /// Must match `DEFAULT_CONFIG` (checked by a test).
    fn default() -> Config {
        Config {
            theme: Theme::Dark,
            abs_paths: true,
            compact_paths: true,
            max_results: 9,
            database_path: expand_home("$HOME/.local/share/contemporary-z/directories.dir"),
            substring: Mode::Score,
            show_files: ShowFiles::Off,
            nav_start_number: 1,
            max_age: 10000,
        }
    }
}

impl Config {
    /// Set one option. Unknown options are ignored (forward compatibility);
    /// an invalid value is an error and leaves the option unchanged.
    fn set(&mut self, option: &str, value: &str) -> Result<(), ()> {
        match option {
            "theme" => self.theme = value.parse()?,
            "abs_paths" => self.abs_paths = value.parse().map_err(|_| ())?,
            "compact_paths" => self.compact_paths = value.parse().map_err(|_| ())?,
            "max_results" => {
                // Zero would break pagination
                let n: usize = value.parse().map_err(|_| ())?;
                if n == 0 {
                    return Err(());
                }
                self.max_results = n;
            }
            "nav_start_number" => self.nav_start_number = value.parse().map_err(|_| ())?,
            "max_age" => {
                let n: usize = value.parse().map_err(|_| ())?;
                if n == 0 {
                    return Err(());
                }
                self.max_age = n;
            }
            "database_path" => self.database_path = expand_home(value),
            "substring" => self.substring = value.parse()?,
            "show_files" => self.show_files = value.parse()?,
            _ => {}
        }
        Ok(())
    }

    /// The user's configuration file, or the defaults if there is none.
    pub(crate) fn load() -> Config {
        let path = format!("{}/.config/contemporary-z/cz.conf", get_home_dir());
        match fs::read_to_string(path) {
            Ok(contents) => Config::parse(&contents),
            Err(_) => Config::default(),
        }
    }

    /// Defaults overridden by the options in `config_string`. Invalid values
    /// are reported and replaced by the default.
    fn parse(config_string: &str) -> Config {
        let mut config = Config::default();
        for (option, value) in parse_config(config_string) {
            if config.set(&option, &value).is_err() {
                eprintln!("cz: invalid value for config option '{}', using default", option);
            }
        }
        config
    }
}


fn parse_config(config_string: &str) -> Vec<(String, String)> {
    /* File format (see DEFAULT_CONFIG):
     * option = value
     * Lines starting with '#' and malformed lines are ignored.
     */
    let mut option_values = Vec::new();
    for config_option in config_string.lines() {
        let config_option = config_option.trim();
        if config_option.is_empty() || config_option.starts_with('#') {
            continue;
        }
        if let Some((option, value)) = config_option.split_once('=') {
            option_values.push((option.trim().to_string(), value.trim().to_string()));
        }
    }
    option_values
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::strings::DEFAULT_CONFIG;

    #[test]
    fn config_skips_comments_and_bad_lines() {
        let opts = parse_config("# comment\n\ntheme = bright\nnonsense\nmax_results=5\n");
        assert_eq!(
            opts,
            vec![
                ("theme".to_string(), "bright".to_string()),
                ("max_results".to_string(), "5".to_string()),
            ]
        );
    }

    #[test]
    fn config_default_matches_default_config_text() {
        assert_eq!(Config::parse(DEFAULT_CONFIG), Config::default());
    }

    #[test]
    fn config_user_values_override_defaults() {
        let c = Config::parse("theme = bright\nmax_results = 4\nsubstring = none\nshow_files = top");
        assert_eq!(c.theme, Theme::Bright);
        assert_eq!(c.max_results, 4);
        assert_eq!(c.max_age, 10000);
        assert_eq!(c.substring, Mode::Select);
        assert_eq!(c.show_files, ShowFiles::Top);
        // Not set by the user: default
        assert!(c.abs_paths);
    }

    #[test]
    fn config_invalid_values_fall_back_to_default() {
        let c = Config::parse(
            "max_results = abc\nabs_paths = maybe\nnav_start_number = -3\nsubstring = nope\nmax_results = 0\nmax_age = 0"
        );
        assert_eq!(c, Config::default());
    }

    #[test]
    fn config_expands_home_in_database_path() {
        let c = Config::parse("database_path = $HOME/db/x.dir");
        assert_eq!(c.database_path, format!("{}/db/x.dir", get_home_dir()));
    }

    #[test]
    fn config_ignores_unknown_options() {
        assert_eq!(Config::parse("future_option = 1"), Config::default());
    }
}
