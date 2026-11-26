use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::config::Mode;
use crate::utils::get_current_seconds;


#[derive(Clone, Debug)]
pub(crate) struct Directory {
    pub name: String,
    /// Visit count. Fractional after aging (see `Database::age`).
    pub counter: f64,
    /// Seconds since the Unix epoch.
    pub last_access: i64,
    /// Frecency, computed when the database is loaded.
    pub score: f64,
}


/// Higher for directories visited often and recently.
fn frecency(counter: f64, last_access: i64, now: i64) -> f64 {
    let age = (now - last_access) as f64;
    10000.0 * counter * (3.75 / ((0.0001 * age + 1.0) + 0.25))
}


/// The directories database, stored as a text file with one block per
/// directory:
///
/// ```text
/// name
/// counter
/// last_access
/// ---
/// ```
pub(crate) struct Database {
    dirs: Vec<Directory>,
}


impl Database {
    /// Load the database; a missing file is an empty database.
    pub(crate) fn load(path: &str) -> Database {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == ErrorKind::NotFound => String::new(),
            Err(e) => panic!("Error reading {}: {}", path, e),
        };
        Database::parse(&text, get_current_seconds())
    }


    /// Write the database to a temporary file and then rename it, so that
    /// the file is never left half-written. If several `cz` save at the same
    /// time (from several shells), the last one wins: there is no
    /// locking, so a visit may occasionally be lost.
    pub(crate) fn save(&self, path: &str) {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).unwrap_or_else(
                |e| panic!("Error creating dir: {}", e)
            );
        }
        // One temporary file per process, so that concurrent saves do not
        // write into the same file
        let tmp_path = format!("{}.{}.tmp", path, std::process::id());
        fs::write(&tmp_path, self.serialize())
            .and_then(|_| fs::rename(&tmp_path, path))
            .unwrap_or_else(|e| panic!("Error writing file: {}", e));
    }


    /// Malformed blocks are skipped instead of aborting the program.
    fn parse(text: &str, now: i64) -> Database {
        let mut dirs = Vec::new();
        let mut block: Vec<&str> = Vec::new();
        for line in text.lines() {
            if line == "---" {
                dirs.extend(parse_block(&block, now));
                block.clear();
            } else {
                block.push(line);
            }
        }
        // In case the last block has no separator
        dirs.extend(parse_block(&block, now));
        Database { dirs }
    }


    fn serialize(&self) -> String {
        let mut text = String::new();
        for dir in &self.dirs {
            // Aging produces long fractions: keep 4 decimals (whole counters
            // are written as integers)
            let counter = (dir.counter * 10_000.0).round() / 10_000.0;
            text.push_str(&format!("{}\n{}\n{}\n---\n", dir.name, counter, dir.last_access));
        }
        text
    }


    /// Keep the database bounded (same idea as zoxide's aging): when the sum
    /// of all counters exceeds `max_age`, scale them all down so that the sum
    /// is 90% of `max_age`, and drop the directories whose counter falls below
    /// 1.
    pub(crate) fn age(&mut self, max_age: f64) {
        let total: f64 = self.dirs.iter().map(|dir| dir.counter).sum();
        if total <= max_age {
            return;
        }
        let factor = 0.9 * max_age / total;
        for dir in self.dirs.iter_mut() {
            dir.counter *= factor;
        }
        self.dirs.retain(|dir| dir.counter >= 1.0);
    }


    pub(crate) fn clear(&mut self) {
        self.dirs.clear();
    }


    /// Remove the directories for which `keep` is false.
    pub(crate) fn retain(&mut self, keep: impl FnMut(&Directory) -> bool) {
        self.dirs.retain(keep);
    }


    /// Register a visit to `name` (adding it if needed). Names that do not
    /// fit in one line of the database file are ignored.
    pub(crate) fn insert(&mut self, name: &str) {
        self.insert_at(name, get_current_seconds());
    }


    fn insert_at(&mut self, name: &str, now: i64) {
        if name.contains('\n') {
            return;
        }
        let mut dir = match self.dirs.iter().position(|dir| dir.name == name) {
            Some(i) => self.dirs.remove(i),
            None => Directory {
                name: name.to_string(),
                counter: 0.0,
                last_access: now,
                score: 0.0,
            },
        };
        dir.counter += 1.0;
        dir.last_access = now;
        // Visited directories go to the end, so the list is in visit order:
        // that orders visits that happen within the same second
        self.dirs.push(dir);
    }


    pub(crate) fn remove(&mut self, name: &str) {
        self.dirs.retain(|dir| dir.name != name);
    }


    /// Directories (best score first) that contain all the `patterns`.
    /// Existence is not checked.
    pub(crate) fn matching(&self, patterns: &[String]) -> Vec<Directory> {
        let mut dirs: Vec<Directory> = self
            .dirs
            .iter()
            .filter(|dir| patterns.iter().all(|p| dir.name.contains(p.as_str())))
            .cloned()
            .collect();
        dirs.sort_by(|a, b| b.score.total_cmp(&a.score));
        dirs
    }


    /// Directory names, most recently accessed first. Visits within the same
    /// second are ordered by their position in the list (see `insert_at`).
    pub(crate) fn by_recency(&self) -> Vec<String> {
        let mut dirs: Vec<(usize, &Directory)> = self.dirs.iter().enumerate().collect();
        dirs.sort_by_key(|(position, dir)| std::cmp::Reverse((dir.last_access, *position)));
        dirs.into_iter().map(|(_, dir)| dir.name.clone()).collect()
    }
}


/// The directory described by the lines of one block of the database file,
/// or `None` if the block is malformed. A fourth line (the alias, in versions
/// before 0.8.0) is ignored.
fn parse_block(lines: &[&str], now: i64) -> Option<Directory> {
    if lines.len() < 3 {
        return None;
    }
    let counter: f64 = lines[1].parse().ok()?;
    let last_access: i64 = lines[2].parse().ok()?;
    Some(Directory {
        name: lines[0].to_string(),
        counter,
        last_access,
        score: frecency(counter, last_access, now),
    })
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

    fn dir(name: &str, counter: f64, last_access: i64) -> Directory {
        Directory {
            name: name.to_string(),
            counter,
            last_access,
            score: frecency(counter, last_access, NOW),
        }
    }

    fn db(dirs: Vec<Directory>) -> Database {
        Database { dirs }
    }

    fn nth_recent(d: &Database, n: usize) -> Option<String> {
        d.by_recency().get(n).cloned()
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
            dir("/home/u/a", 3.0, NOW - 10),
            dir("/home/u/b", 1.0, NOW - 20),
        ]);
        let parsed = Database::parse(&original.serialize(), NOW);
        assert_eq!(parsed.dirs.len(), 2);
        for (p, d) in parsed.dirs.iter().zip(original.dirs.iter()) {
            assert_eq!(p.name, d.name);
            assert_eq!(p.counter, d.counter);
            assert_eq!(p.last_access, d.last_access);
        }
        assert!(parsed.dirs[0].score > 0.0);
    }

    #[test]
    fn names_containing_separator() {
        let original = db(vec![dir("/home/u/my---proj", 2.0, NOW)]);
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
        let parsed = Database::parse(&db(vec![dir("/old", 1.0, old)]).serialize(), NOW);
        assert_eq!(parsed.dirs.len(), 1);
        assert!(parsed.dirs[0].score > 0.0);
    }

    #[test]
    fn fractional_counters() {
        let text = db(vec![dir("/a", 2.5, NOW), dir("/b", 3.0, NOW)]).serialize();
        // Whole counters are written as before, so older versions can read them
        assert!(text.contains("/b\n3\n"), "{}", text);
        let parsed = Database::parse(&text, NOW);
        assert_eq!(parsed.dirs[0].counter, 2.5);
        assert_eq!(parsed.dirs[1].counter, 3.0);
    }

    #[test]
    fn rounds_long_fractions() {
        let text = db(vec![dir("/a", 1.0 / 3.0, NOW)]).serialize();
        assert!(text.contains("/a\n0.3333\n"), "{}", text);
    }

    #[test]
    fn ignores_aliases_written_by_older_versions() {
        let parsed = Database::parse(&format!("/a\n2\n{}\nmy_alias\n---\n/b\n1\n{}\n---\n", NOW, NOW), NOW);
        assert_eq!(names(&parsed.dirs), vec!["/a", "/b"]);
        assert_eq!(parsed.serialize(), format!("/a\n2\n{}\n---\n/b\n1\n{}\n---\n", NOW, NOW));
    }

    #[test]
    fn reads_counters_written_by_older_versions() {
        let parsed = Database::parse(&format!("/a\n7\n{}\n---\n", NOW), NOW);
        assert_eq!(parsed.dirs[0].counter, 7.0);
    }

    // ---- aging

    #[test]
    fn age_does_nothing_under_the_limit() {
        let mut d = db(vec![dir("/a", 60.0, NOW), dir("/b", 40.0, NOW)]);
        d.age(100.0);
        assert_eq!(d.dirs.len(), 2);
        assert_eq!(d.dirs.iter().map(|x| x.counter).sum::<f64>(), 100.0);
    }

    #[test]
    fn age_scales_down_to_ninety_percent() {
        let mut d = db(vec![dir("/a", 150.0, NOW), dir("/b", 50.0, NOW)]);
        d.age(100.0);
        assert!((d.dirs.iter().map(|x| x.counter).sum::<f64>() - 90.0).abs() < 1e-9);
        // Relative order is preserved
        assert!(d.dirs[0].counter > d.dirs[1].counter);
    }

    #[test]
    fn age_drops_small_counters() {
        let mut d = db(vec![dir("/big", 1000.0, NOW), dir("/small", 1.0, NOW)]);
        d.age(100.0);
        assert_eq!(names(&d.dirs), vec!["/big"]);
    }

    // ---- queries and updates

    #[test]
    fn insert_adds_then_counts_visits() {
        let mut d = db(vec![]);
        d.insert_at("/a", 100);
        d.insert_at("/a", 200);
        d.insert_at("/b", 300);
        assert_eq!(names(&d.dirs), vec!["/a", "/b"]);
        assert_eq!(d.dirs[0].counter, 2.0);
        assert_eq!(d.dirs[0].last_access, 200);
    }

    #[test]
    fn visits_within_the_same_second_keep_their_order() {
        let mut d = db(vec![]);
        for name in ["/a", "/b", "/a", "/c"] {
            d.insert_at(name, 100);
        }
        assert_eq!(nth_recent(&d, 0).as_deref(), Some("/c"));
        assert_eq!(nth_recent(&d, 1).as_deref(), Some("/a"));
        assert_eq!(nth_recent(&d, 2).as_deref(), Some("/b"));
        assert_eq!(d.dirs.iter().find(|x| x.name == "/a").unwrap().counter, 2.0);
    }

    #[test]
    fn names_that_would_break_the_file_are_rejected() {
        let mut d = db(vec![]);
        d.insert("/a\nb");
        assert!(d.dirs.is_empty());
    }

    #[test]
    fn matching_filters_and_sorts_by_score() {
        let d = db(vec![
            dir("/proj/old", 1.0, NOW - 100_000),
            dir("/proj/new", 5.0, NOW),
            dir("/other", 9.0, NOW),
        ]);
        assert_eq!(names(&d.matching(&[])), vec!["/other", "/proj/new", "/proj/old"]);
        assert_eq!(names(&d.matching(&strings(&["proj"]))), vec!["/proj/new", "/proj/old"]);
        assert_eq!(names(&d.matching(&strings(&["proj", "old"]))), vec!["/proj/old"]);
    }

    #[test]
    fn nth_recent_orders_by_last_access() {
        let d = db(vec![dir("/a", 1.0, 10), dir("/b", 1.0, 30), dir("/c", 1.0, 20)]);
        assert_eq!(nth_recent(&d, 0).as_deref(), Some("/b"));
        assert_eq!(nth_recent(&d, 1).as_deref(), Some("/c"));
        assert_eq!(nth_recent(&d, 3), None);
    }

    #[test]
    fn remove_and_retain() {
        let mut d = db(vec![dir("/a", 1.0, NOW), dir("/b", 1.0, NOW)]);
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
            dir("/very/long/path/proj", 9.0, NOW),
            dir("/p/other_proj", 5.0, NOW),
            dir("/proj", 1.0, NOW),
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
        let dirs = vec![dir("/a/b", 1.0, NOW)];
        for mode in [Mode::Score, Mode::Shortest, Mode::Basename, Mode::Select] {
            assert!(pick(&dirs, &strings(&["a"]), mode).is_some());
        }
        assert!(pick(&[], &[], Mode::Score).is_none());
    }
}
