use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;

use crate::config::Mode;
use crate::lock::Lock;
use crate::utils::get_current_seconds;


#[derive(Clone, Debug)]
pub(crate) struct Directory {
    pub name: String,
    /// Visit count. Fractional after aging (see `Database::age`).
    pub counter: f64,
    pub last_access: i64,
    /// Frecency, computed when the database is loaded.
    pub score: f64,
    pub alias: String,
}


fn frecency(counter: f64, last_access: i64, now: i64) -> f64 {
    10000.0 * counter * (3.75 / ((0.0001 * (now - last_access) as f64 + 1.0) + 0.25))
}


/// Whether `alias` can be stored: it must fit in one line of the database
/// file and not be mistaken for the block separator.
pub(crate) fn valid_alias(alias: &str) -> bool {
    !alias.is_empty() && !alias.contains('\n') && alias != "---"
}


/// A change to the database. Changes are applied to the in-memory database
/// immediately and also recorded, so that `commit` can re-apply them to the
/// latest version on disk (another `cz` may have saved it in the meantime).
#[derive(Clone, Debug)]
enum Change {
    Visit { name: String, alias: Option<String>, now: i64 },
    SetAlias { dir: String, alias: String },
    RemoveAlias(String),
    Remove(String),
    Clear,
}


/// The directories database. The text format is:
///
/// ```text
/// name1
/// counter1
/// last_access1
/// alias1        (optional)
/// ---
/// name2
/// ...
/// ```
pub(crate) struct Database {
    dirs: Vec<Directory>,
    changes: Vec<Change>,
}


impl Database {
    /// Load the database; a missing file is an empty database.
    pub(crate) fn load(path: &str) -> Database {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).unwrap_or_else(
                |e| panic!("Error creating dir: {}", e)
            );
        }
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == ErrorKind::NotFound => String::new(),
            Err(e) => panic!("Error reading {}: {}", path, e),
        };
        Database::parse(&text, get_current_seconds())
    }


    /// Save the changes merged with the latest version on disk: under the
    /// lock, reload the database, re-apply the changes, age it and write it.
    /// The lock is only held for that, never while waiting for the user.
    /// Nothing is written if there are no changes. `Err` if the lock could not
    /// be acquired within `timeout`.
    pub(crate) fn commit(&self, path: &str, max_age: f64, timeout: Duration) -> Result<(), ()> {
        if self.changes.is_empty() {
            return Ok(());
        }
        let _lock = Lock::acquire(path, timeout).ok_or(())?;
        let mut latest = Database::load(path);
        for change in &self.changes {
            latest.apply(change);
        }
        latest.age(max_age);
        latest.write(path);
        Ok(())
    }


    /// Write atomically: a crash mid-write must not truncate the database.
    fn write(&self, path: &str) {
        let tmp_path = format!("{}.tmp", path);
        fs::write(&tmp_path, self.serialize())
            .and_then(|_| fs::rename(&tmp_path, path))
            .unwrap_or_else(|e| panic!("Error writing file: {}", e));
    }


    /// Malformed blocks are skipped instead of aborting the program.
    fn parse(text: &str, now: i64) -> Database {
        let mut dirs = Vec::new();
        let mut block: Vec<&str> = Vec::new();
        // Blocks are terminated by a line that is exactly "---"
        for line in text.lines().chain(std::iter::once("---")) {
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
            dirs.push(Directory {
                name: fields[0].to_string(),
                counter,
                last_access,
                score: frecency(counter, last_access, now),
                alias: fields.get(3).map(|a| a.to_string()).unwrap_or_default(),
            });
        }
        Database { dirs, changes: Vec::new() }
    }


    fn serialize(&self) -> String {
        let mut text = String::new();
        for dir in &self.dirs {
            // Aging produces long fractions: keep a few decimals (whole
            // counters are written as integers)
            let counter = (dir.counter * 10_000.0).round() / 10_000.0;
            text.push_str(&format!("{}\n{}\n{}\n", dir.name, counter, dir.last_access));
            if !dir.alias.is_empty() {
                text.push_str(&format!("{}\n", dir.alias));
            }
            text.push_str("---\n");
        }
        text
    }


    /// Keep the database bounded (same idea as zoxide's aging): when the sum
    /// of all counters exceeds `max_age`, scale them all down so that the sum
    /// is 90% of `max_age`, and drop the directories whose counter falls below
    /// 1. Aliased directories are never dropped.
    fn age(&mut self, max_age: f64) {
        let total: f64 = self.dirs.iter().map(|dir| dir.counter).sum();
        if total <= max_age {
            return;
        }
        let factor = 0.9 * max_age / total;
        for dir in self.dirs.iter_mut() {
            dir.counter *= factor;
        }
        self.dirs.retain(|dir| dir.counter >= 1.0 || !dir.alias.is_empty());
    }


    fn record(&mut self, change: Change) {
        self.apply(&change);
        self.changes.push(change);
    }


    fn apply(&mut self, change: &Change) {
        match change {
            Change::Visit { name, alias, now } => self.insert_at(name, alias.as_deref(), *now),
            Change::SetAlias { dir, alias } => {
                for d in self.dirs.iter_mut() {
                    if d.alias == *alias {
                        d.alias.clear();
                    }
                    if d.name == *dir {
                        d.alias = alias.clone();
                    }
                }
            }
            Change::RemoveAlias(dir) => {
                if let Some(d) = self.dirs.iter_mut().find(|d| d.name == *dir) {
                    d.alias.clear();
                }
            }
            Change::Remove(name) => {
                if let Some(i) = self.dirs.iter().position(|dir| dir.name == *name) {
                    self.dirs.remove(i);
                }
            }
            Change::Clear => self.dirs.clear(),
        }
    }


    pub(crate) fn clear(&mut self) {
        self.record(Change::Clear);
    }


    /// Remove the directories for which `keep` is false.
    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&Directory) -> bool) {
        let removed: Vec<String> = self
            .dirs
            .iter()
            .filter(|dir| !keep(dir))
            .map(|dir| dir.name.clone())
            .collect();
        for name in removed {
            self.remove(&name);
        }
    }


    pub(crate) fn contains(&self, name: &str) -> bool {
        self.dirs.iter().any(|dir| dir.name == name)
    }


    pub(crate) fn by_alias(&self, alias: &str) -> Option<&Directory> {
        if alias.is_empty() {
            return None;
        }
        self.dirs.iter().find(|dir| dir.alias == alias)
    }


    /// Register a visit to `name` (adding it if needed). Names that do not
    /// fit in one line of the database file are ignored.
    pub(crate) fn insert(&mut self, name: &str, alias: Option<&str>) {
        if name.contains('\n') {
            return;
        }
        self.record(Change::Visit {
            name: name.to_string(),
            alias: alias.map(String::from),
            now: get_current_seconds(),
        });
    }


    fn insert_at(&mut self, name: &str, alias: Option<&str>, now: i64) {
        let mut dir = match self.dirs.iter().position(|dir| dir.name == name) {
            Some(i) => {
                let mut dir = self.dirs.remove(i);
                dir.counter += 1.0;
                dir.last_access = now;
                dir
            }
            None => Directory {
                name: name.to_string(),
                counter: 1.0,
                last_access: now,
                score: 0.0,
                alias: String::new(),
            },
        };
        if let Some(alias) = alias {
            dir.alias = alias.to_string();
        }
        // Most recent last: the order breaks ties between visits that happen
        // within the same second (access times have a resolution of 1 s)
        self.dirs.push(dir);
    }


    /// Give `dir` the alias `alias`; aliases are unique, so any other
    /// directory that had it loses it.
    pub(crate) fn set_alias(&mut self, dir: &str, alias: &str) {
        self.record(Change::SetAlias { dir: dir.to_string(), alias: alias.to_string() });
    }


    pub(crate) fn remove_alias(&mut self, dir: &str) {
        self.record(Change::RemoveAlias(dir.to_string()));
    }


    pub(crate) fn remove(&mut self, name: &str) {
        self.record(Change::Remove(name.to_string()));
    }


    /// Directories (best score first) that contain all the `patterns`; if
    /// `alias_only`, only those with an alias. Existence is not checked.
    pub(crate) fn matching(&self, patterns: &[String], alias_only: bool) -> Vec<Directory> {
        let mut dirs: Vec<Directory> = self
            .dirs
            .iter()
            .filter(|dir| patterns.iter().all(|p| dir.name.contains(p.as_str())))
            .filter(|dir| !alias_only || !dir.alias.is_empty())
            .cloned()
            .collect();
        dirs.sort_by(|a, b| b.score.total_cmp(&a.score));
        dirs
    }


    /// Directory names, most recently accessed first.
    pub(crate) fn by_recency(&self) -> Vec<String> {
        // The stable sort keeps the order of insert for equal times
        let mut dirs: Vec<&Directory> = self.dirs.iter().rev().collect();
        dirs.sort_by_key(|dir| std::cmp::Reverse(dir.last_access));
        dirs.into_iter().map(|dir| dir.name.clone()).collect()
    }
}


/// Choose among directories that are sorted by score (and that all match
/// `patterns`) according to `mode`. With a single directory, it is always
/// chosen. `None` if there is no choice to make automatically: no
/// directories, `Mode::Select`, or a basename that does not match.
pub(crate) fn pick<'a>(dirs: &'a [Directory], patterns: &[String], mode: Mode) -> Option<&'a Directory> {
    if dirs.len() == 1 {
        return dirs.first();
    }
    match mode {
        Mode::Score => dirs.first(),
        Mode::Shortest => dirs.iter().min_by_key(|dir| dir.name.len()),
        Mode::Basename => dirs.iter().find(|dir| basename_matches(&dir.name, patterns)),
        Mode::Select => None,
    }
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


#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn dir(name: &str, counter: f64, last_access: i64, alias: &str) -> Directory {
        Directory {
            name: name.to_string(),
            counter,
            last_access,
            score: frecency(counter, last_access, NOW),
            alias: alias.to_string(),
        }
    }

    fn db(dirs: Vec<Directory>) -> Database {
        Database { dirs, changes: Vec::new() }
    }

    fn nth_recent(d: &Database, n: usize) -> Option<String> {
        d.by_recency().get(n).cloned()
    }

    /// A database file in a fresh temporary directory.
    fn temp_db_path(test: &str) -> (std::path::PathBuf, String) {
        let dir = std::env::temp_dir().join(format!("cz-db-test-{}-{}", test, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("db.dir").to_str().unwrap().to_string();
        (dir, path)
    }

    fn names(dirs: &[Directory]) -> Vec<&str> {
        dirs.iter().map(|d| d.name.as_str()).collect()
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    // ---- text format

    #[test]
    fn round_trip() {
        let original = db(vec![
            dir("/home/u/a", 3.0, NOW - 10, ""),
            dir("/home/u/b", 1.0, NOW - 20, "bee"),
        ]);
        let parsed = Database::parse(&original.serialize(), NOW);
        assert_eq!(parsed.dirs.len(), 2);
        for (p, d) in parsed.dirs.iter().zip(original.dirs.iter()) {
            assert_eq!(p.name, d.name);
            assert_eq!(p.counter, d.counter);
            assert_eq!(p.last_access, d.last_access);
            assert_eq!(p.alias, d.alias);
        }
        assert!(parsed.dirs[0].score > 0.0);
    }

    #[test]
    fn names_containing_separator() {
        let original = db(vec![dir("/home/u/my---proj", 2.0, NOW, "a-b")]);
        let parsed = Database::parse(&original.serialize(), NOW);
        assert_eq!(parsed.dirs.len(), 1);
        assert_eq!(parsed.dirs[0].name, "/home/u/my---proj");
    }

    #[test]
    fn skips_malformed_blocks() {
        let text = format!("/a\nnot_a_number\n5\n---\n/b\n---\n/c\n1\n{}\n---\n", NOW);
        let parsed = Database::parse(&text, NOW);
        assert_eq!(names(&parsed.dirs), vec!["/c"]);
        assert!(Database::parse("", NOW).dirs.is_empty());
    }

    #[test]
    fn keeps_old_entries() {
        // Bounding the database is the job of `age`, not of loading
        let old = NOW - 60 * 60 * 24 * 365 * 3;
        let parsed = Database::parse(&db(vec![dir("/old", 1.0, old, "")]).serialize(), NOW);
        assert_eq!(parsed.dirs.len(), 1);
        assert!(parsed.dirs[0].score > 0.0);
    }

    #[test]
    fn fractional_counters() {
        let text = db(vec![dir("/a", 2.5, NOW, ""), dir("/b", 3.0, NOW, "")]).serialize();
        // Whole counters are written as before, so older versions can read them
        assert!(text.contains("/b\n3\n"), "{}", text);
        let parsed = Database::parse(&text, NOW);
        assert_eq!(parsed.dirs[0].counter, 2.5);
        assert_eq!(parsed.dirs[1].counter, 3.0);
    }

    #[test]
    fn rounds_long_fractions() {
        let text = db(vec![dir("/a", 1.0 / 3.0, NOW, "")]).serialize();
        assert!(text.contains("/a\n0.3333\n"), "{}", text);
    }

    #[test]
    fn reads_counters_written_by_older_versions() {
        let parsed = Database::parse(&format!("/a\n7\n{}\n---\n", NOW), NOW);
        assert_eq!(parsed.dirs[0].counter, 7.0);
    }

    // ---- aging

    #[test]
    fn age_does_nothing_under_the_limit() {
        let mut d = db(vec![dir("/a", 60.0, NOW, ""), dir("/b", 40.0, NOW, "")]);
        d.age(100.0);
        assert_eq!(d.dirs.len(), 2);
        assert_eq!(d.dirs.iter().map(|x| x.counter).sum::<f64>(), 100.0);
    }

    #[test]
    fn age_scales_down_to_ninety_percent() {
        let mut d = db(vec![dir("/a", 150.0, NOW, ""), dir("/b", 50.0, NOW, "")]);
        d.age(100.0);
        assert!((d.dirs.iter().map(|x| x.counter).sum::<f64>() - 90.0).abs() < 1e-9);
        // Relative order is preserved
        assert!(d.dirs[0].counter > d.dirs[1].counter);
    }

    #[test]
    fn age_drops_small_counters_but_keeps_aliased() {
        let mut d = db(vec![
            dir("/big", 1000.0, NOW, ""),
            dir("/small", 1.0, NOW, ""),
            dir("/small_alias", 1.0, NOW, "s"),
        ]);
        d.age(100.0);
        assert_eq!(names(&d.dirs), vec!["/big", "/small_alias"]);
    }

    // ---- queries and updates

    #[test]
    fn insert_adds_then_counts_visits() {
        let mut d = db(vec![]);
        d.insert_at("/a", None, 100);
        d.insert_at("/a", None, 200);
        d.insert_at("/b", Some("bee"), 300);
        assert_eq!(d.dirs[0].counter, 2.0);
        assert_eq!(d.dirs[0].last_access, 200);
        assert_eq!(d.dirs[1].alias, "bee");
        assert!(d.contains("/a") && !d.contains("/c"));
    }

    #[test]
    fn visits_within_the_same_second_keep_their_order() {
        let mut d = db(vec![]);
        for name in ["/a", "/b", "/a", "/c"] {
            d.insert_at(name, None, 100);
        }
        assert_eq!(nth_recent(&d, 0).as_deref(), Some("/c"));
        assert_eq!(nth_recent(&d, 1).as_deref(), Some("/a"));
        assert_eq!(nth_recent(&d, 2).as_deref(), Some("/b"));
        assert_eq!(d.dirs.iter().find(|x| x.name == "/a").unwrap().counter, 2.0);
    }

    #[test]
    fn names_and_aliases_that_would_break_the_file_are_rejected() {
        let mut d = db(vec![]);
        d.insert("/a\nb", None);
        assert!(d.dirs.is_empty());
        assert!(valid_alias("x") && valid_alias("my alias"));
        assert!(!valid_alias("") && !valid_alias("a\nb") && !valid_alias("---"));
    }

    // ---- saving

    #[test]
    fn commit_merges_with_changes_saved_meanwhile() {
        let (dir, path) = temp_db_path("merge");
        let long = Duration::from_secs(5);
        let mut setup = Database::load(&path);
        setup.insert("/shared", None);
        setup.insert("/gone", None);
        setup.commit(&path, 1e9, long).unwrap();

        // Two processes load the same version...
        let mut first = Database::load(&path);
        let mut second = Database::load(&path);
        first.insert("/first", None);
        first.insert("/shared", None);
        second.insert("/second", Some("s"));
        second.remove("/gone");
        // ...and save one after the other: no change is lost
        first.commit(&path, 1e9, long).unwrap();
        second.commit(&path, 1e9, long).unwrap();

        let result = Database::load(&path);
        let mut result_names: Vec<&str> = result.dirs.iter().map(|d| d.name.as_str()).collect();
        result_names.sort();
        assert_eq!(result_names, vec!["/first", "/second", "/shared"]);
        assert_eq!(result.dirs.iter().find(|d| d.name == "/shared").unwrap().counter, 2.0);
        assert_eq!(result.by_alias("s").unwrap().name, "/second");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn commit_without_changes_writes_nothing() {
        let (dir, path) = temp_db_path("nochange");
        Database::load(&path).commit(&path, 1e9, Duration::from_secs(1)).unwrap();
        assert!(!Path::new(&path).exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn aliases_are_unique() {
        let mut d = db(vec![dir("/a", 1.0, NOW, "x"), dir("/b", 1.0, NOW, "")]);
        assert_eq!(d.by_alias("x").unwrap().name, "/a");
        d.set_alias("/b", "x");
        assert_eq!(d.by_alias("x").unwrap().name, "/b");
        assert_eq!(d.dirs[0].alias, "");
        d.remove_alias("/b");
        assert!(d.by_alias("x").is_none());
        // An empty alias never matches
        assert!(d.by_alias("").is_none());
    }

    #[test]
    fn matching_filters_and_sorts_by_score() {
        let d = db(vec![
            dir("/proj/old", 1.0, NOW - 100_000, ""),
            dir("/proj/new", 5.0, NOW, "n"),
            dir("/other", 9.0, NOW, ""),
        ]);
        assert_eq!(names(&d.matching(&[], false)), vec!["/other", "/proj/new", "/proj/old"]);
        assert_eq!(names(&d.matching(&strings(&["proj"]), false)), vec!["/proj/new", "/proj/old"]);
        assert_eq!(names(&d.matching(&strings(&["proj", "old"]), false)), vec!["/proj/old"]);
        assert_eq!(names(&d.matching(&[], true)), vec!["/proj/new"]);
    }

    #[test]
    fn nth_recent_orders_by_last_access() {
        let d = db(vec![dir("/a", 1.0, 10, ""), dir("/b", 1.0, 30, ""), dir("/c", 1.0, 20, "")]);
        assert_eq!(nth_recent(&d, 0).as_deref(), Some("/b"));
        assert_eq!(nth_recent(&d, 1).as_deref(), Some("/c"));
        assert_eq!(nth_recent(&d, 3), None);
    }

    #[test]
    fn remove_and_retain() {
        let mut d = db(vec![dir("/a", 1.0, NOW, ""), dir("/b", 1.0, NOW, "")]);
        d.remove("/a");
        d.remove("/missing");
        assert_eq!(names(&d.dirs), vec!["/b"]);
        d.retain(|_| false);
        assert!(d.dirs.is_empty());
    }

    // ---- picking

    #[test]
    fn pick_by_mode() {
        let dirs = vec![
            dir("/very/long/path/proj", 9.0, NOW, ""),
            dir("/p/other_proj", 5.0, NOW, ""),
            dir("/proj", 1.0, NOW, ""),
        ];
        let patterns = strings(&["proj"]);
        let name = |mode| pick(&dirs, &patterns, mode).map(|d| d.name.as_str());
        assert_eq!(name(Mode::Score), Some("/very/long/path/proj"));
        assert_eq!(name(Mode::Shortest), Some("/proj"));
        assert_eq!(name(Mode::Basename), Some("/very/long/path/proj"));
        assert_eq!(name(Mode::Select), None);
        // Basename: patterns must match the basename, not the whole path
        assert_eq!(pick(&dirs, &strings(&["path"]), Mode::Basename).map(|d| &d.name[..]), None);
    }

    #[test]
    fn pick_single_result_is_always_chosen() {
        let dirs = vec![dir("/a/b", 1.0, NOW, "")];
        for mode in [Mode::Score, Mode::Shortest, Mode::Basename, Mode::Select] {
            assert!(pick(&dirs, &strings(&["a"]), mode).is_some());
        }
        assert!(pick(&[], &[], Mode::Score).is_none());
    }
}
