#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct Directory {
    pub name: String,
    /// Visit count. Fractional after aging (see `age`).
    pub counter: f64,
    pub last_access: i64,
    pub score: f64,
    pub alias: String,
}


/// Keep the database bounded (same idea as zoxide's aging): when the sum of
/// all counters exceeds `max_age`, scale them all down so that the sum is
/// 90% of `max_age`, and drop the directories whose counter falls below 1.
/// Aliased directories are never dropped.
pub(crate) fn age(dirs: &mut Vec<Directory>, max_age: f64) {
    let total: f64 = dirs.iter().map(|dir| dir.counter).sum();
    if total <= max_age {
        return;
    }
    let factor = 0.9 * max_age / total;
    for dir in dirs.iter_mut() {
        dir.counter *= factor;
    }
    dirs.retain(|dir| dir.counter >= 1.0 || !dir.alias.is_empty());
}


#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str, counter: f64, alias: &str) -> Directory {
        Directory {
            name: name.to_string(),
            counter,
            last_access: 0,
            score: 0.0,
            alias: alias.to_string(),
        }
    }

    fn total(dirs: &[Directory]) -> f64 {
        dirs.iter().map(|d| d.counter).sum()
    }

    #[test]
    fn age_does_nothing_under_the_limit() {
        let mut dirs = vec![dir("/a", 60.0, ""), dir("/b", 40.0, "")];
        age(&mut dirs, 100.0);
        assert_eq!(dirs.len(), 2);
        assert_eq!(total(&dirs), 100.0);
    }

    #[test]
    fn age_scales_down_to_ninety_percent() {
        let mut dirs = vec![dir("/a", 150.0, ""), dir("/b", 50.0, "")];
        age(&mut dirs, 100.0);
        assert!((total(&dirs) - 90.0).abs() < 1e-9);
        // Relative order is preserved
        assert!(dirs[0].counter > dirs[1].counter);
    }

    #[test]
    fn age_drops_small_counters_but_keeps_aliased() {
        let mut dirs = vec![
            dir("/big", 1000.0, ""),
            dir("/small", 1.0, ""),
            dir("/small_alias", 1.0, "s"),
        ];
        age(&mut dirs, 100.0);
        let names: Vec<&str> = dirs.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["/big", "/small_alias"]);
    }
}
