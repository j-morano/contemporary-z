//! Advisory lock on the database, so that several `cz` processes (for
//! example, the hooks of several shells) do not overwrite each other's
//! changes. It is only held while saving (see `Database::commit`).

use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};


/// The lock is released when this is dropped (or when the process exits, even
/// if it crashes, so there are never stale locks).
pub(crate) struct Lock {
    _file: Option<File>,
}


impl Lock {
    /// Wait up to `timeout` for the lock of the database at `db_path`. `None`
    /// if another process still holds it after that. If locking is not
    /// possible at all (for example, on a file system without lock support),
    /// proceed without it instead of failing.
    pub(crate) fn acquire(db_path: &str, timeout: Duration) -> Option<Lock> {
        // A separate file: saving replaces the database file itself
        let path = format!("{}.lock", db_path);
        if let Some(parent) = Path::new(&path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        let file = match OpenOptions::new().create(true).write(true).truncate(false).open(&path) {
            Ok(file) => file,
            Err(_) => return Some(Lock { _file: None }),
        };
        let start = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Some(Lock { _file: Some(file) }),
                Err(TryLockError::WouldBlock) => {
                    if start.elapsed() >= timeout {
                        return None;
                    }
                    sleep(Duration::from_millis(10));
                }
                Err(TryLockError::Error(_)) => return Some(Lock { _file: None }),
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_holder_waits_and_gives_up_until_released() {
        let dir = std::env::temp_dir().join(format!("cz-lock-test-{}", std::process::id()));
        let db_path = dir.join("db.dir").to_str().unwrap().to_string();
        let short = Duration::from_millis(30);

        let first = Lock::acquire(&db_path, short).expect("free lock");
        assert!(Lock::acquire(&db_path, short).is_none(), "held lock");
        drop(first);
        assert!(Lock::acquire(&db_path, short).is_some(), "released lock");

        let _ = fs::remove_dir_all(dir);
    }
}
