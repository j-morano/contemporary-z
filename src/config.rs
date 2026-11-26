use std::fs;
use std::path::Path;
use std::str::FromStr;
use crate::app::{get_current_seconds, get_home_dir, App};
use crate::data::Directory;



fn init_dir_file(database_fn: String, dirs: &mut Vec<Directory>) {
    // println!("database_fn: {}", database_fn);
    // Create database_fn if it does not exist
    if !Path::new(database_fn.as_str()).exists() {
        let database_fn_parent = Path::new(database_fn.as_str()).parent().unwrap();
        fs::create_dir_all(database_fn_parent).unwrap_or_else(
            |e| panic!("Error creating dir: {}", e)
            );
        fs::write(database_fn.as_str(), "").unwrap_or_else(
            |e| panic!("Error creating file: {}", e)
            );
    }

    let db_string = fs::read_to_string(database_fn).unwrap();
    dirs.extend(parse_database(&db_string, get_current_seconds()));
}


/// Serialize directories to the database text format.
pub(crate) fn serialize_database(dirs: &[Directory]) -> String {
    let mut db_string = String::new();
    for dir in dirs {
        // Aging produces long fractions: keep a few decimals (whole counters
        // are written as integers)
        let counter = (dir.counter * 10_000.0).round() / 10_000.0;
        db_string.push_str(&format!("{}\n{}\n{}\n", dir.name, counter, dir.last_access));
        if !dir.alias.is_empty() {
            db_string.push_str(&format!("{}\n", dir.alias));
        }
        db_string.push_str("---\n");
    }
    db_string
}


/// Parse the database text format (see `serialize_database`).
fn parse_database(db_string: &str, current_seconds: i64) -> Vec<Directory> {
    let mut dirs = Vec::new();
    // Blocks are terminated by a line that is exactly "---". Malformed blocks
    // are skipped instead of aborting the program.
    let mut block: Vec<&str> = Vec::new();
    for line in db_string.lines().chain(std::iter::once("---")) {
        if line != "---" {
            block.push(line);
            continue;
        }
        let fields = std::mem::take(&mut block);
        if fields.len() < 3 {
            continue;
        }
        let (counter, last_access) = match (
            fields[1].parse::<f64>(),
            fields[2].parse::<i64>(),
        ) {
            (Ok(c), Ok(l)) => (c, l),
            _ => continue,
        };
        let alias = fields.get(3).map(|a| a.to_string()).unwrap_or_default();
        let score = 10000.0 * counter * (3.75 / ((0.0001 * (current_seconds - last_access) as f64 + 1.0) + 0.25));
        dirs.push(Directory {
            name: fields[0].to_string(),
            counter,
            last_access,
            score,
            alias,
        });
    }
    dirs
}

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


pub(crate) fn app_from_config() -> App {
    let path = format!("{}/.config/contemporary-z/cz.conf", get_home_dir());
    let config = match fs::read_to_string(path) {
        Ok(contents) => Config::parse(&contents),
        Err(_) => Config::default(),
    };

    // Create application user-specific data dir if it does not exist
    if let Some(parent) = Path::new(&config.database_path).parent() {
        fs::create_dir_all(parent).unwrap_or_else(
            |e| panic!("Error creating dir: {}", e)
        );
    }
    let mut dirs = Vec::new();
    init_dir_file(config.database_path.clone(), &mut dirs);
    App { config, dirs }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::strings::DEFAULT_CONFIG;

    const NOW: i64 = 1_800_000_000;

    fn dir(name: &str, counter: f64, last_access: i64, alias: &str) -> Directory {
        Directory {
            name: name.to_string(),
            counter,
            last_access,
            score: 0.0,
            alias: alias.to_string(),
        }
    }

    #[test]
    fn database_round_trip() {
        let dirs = vec![
            dir("/home/u/a", 3.0, NOW - 10, ""),
            dir("/home/u/b", 1.0, NOW - 20, "bee"),
        ];
        let parsed = parse_database(&serialize_database(&dirs), NOW);
        assert_eq!(parsed.len(), 2);
        for (p, d) in parsed.iter().zip(dirs.iter()) {
            assert_eq!(p.name, d.name);
            assert_eq!(p.counter, d.counter);
            assert_eq!(p.last_access, d.last_access);
            assert_eq!(p.alias, d.alias);
        }
        assert!(parsed[0].score > 0.0);
    }

    #[test]
    fn database_names_containing_separator() {
        let dirs = vec![dir("/home/u/my---proj", 2.0, NOW, "a-b")];
        let parsed = parse_database(&serialize_database(&dirs), NOW);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "/home/u/my---proj");
    }

    #[test]
    fn database_skips_malformed_blocks() {
        let db = format!("/a\nnot_a_number\n5\n---\n/b\n---\n/c\n1\n{}\n---\n", NOW);
        let parsed = parse_database(&db, NOW);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "/c");
        assert!(parse_database("", NOW).is_empty());
    }

    #[test]
    fn database_keeps_old_entries() {
        // Bounding the database is the job of `data::age`, not of loading
        let old = NOW - 60 * 60 * 24 * 365 * 3;
        let dirs = vec![dir("/old", 1.0, old, "")];
        let parsed = parse_database(&serialize_database(&dirs), NOW);
        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].score > 0.0);
    }

    #[test]
    fn database_fractional_counters() {
        let dirs = vec![dir("/a", 2.5, NOW, ""), dir("/b", 3.0, NOW, "")];
        let text = serialize_database(&dirs);
        // Whole counters are written as before, so older versions can read them
        assert!(text.contains("/b\n3\n"), "{}", text);
        let parsed = parse_database(&text, NOW);
        assert_eq!(parsed[0].counter, 2.5);
        assert_eq!(parsed[1].counter, 3.0);
    }

    #[test]
    fn database_rounds_long_fractions() {
        let text = serialize_database(&[dir("/a", 1.0 / 3.0, NOW, "")]);
        assert!(text.contains("/a\n0.3333\n"), "{}", text);
    }

    #[test]
    fn database_reads_counters_written_by_older_versions() {
        let parsed = parse_database(&format!("/a\n7\n{}\n---\n", NOW), NOW);
        assert_eq!(parsed[0].counter, 7.0);
    }

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
