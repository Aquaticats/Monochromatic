//! What:
//!  Controls of the wake a released held push gives the service:
//!  who is asked,
//!  and that a caller with no service asks nobody.
//! Why:
//!  The step reaches a socket and a manager in production,
//! so the seam is what a control can check:
//! the line of a held
//!  record must read the same whether or not a service answers,
//! and nothing may reach a real service from a control.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const woken: string[] = []; heldPushNotices(gitDir, record, { wake: (dir) => woken.push(dir) });
//! ```

use super::{NothingWoken, WakeTheService};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// What:
///  The wake a caller with no service gives asks nothing,
/// which is what a command with no reachable service leaves undone.
/// Why:
///   A released record stays in `pending` in that case,
/// and the next service to look makes it,
/// so the silence loses
///  nothing.
#[test]
fn a_caller_with_no_service_asks_nobody() {
    let woken: RefCell<Vec<PathBuf>> = RefCell::new(Vec::new());
    struct Recorder<'a>(&'a RefCell<Vec<PathBuf>>);
    impl WakeTheService for Recorder<'_> {
        fn wake(&self, git_dir: &Path) {
            self.0.borrow_mut().push(git_dir.to_path_buf());
        }

        fn forget_programs(&self, _git_dir: &Path, _records: &[String]) {
            panic!("this control has no record that can no longer run");
        }
    }
    NothingWoken.wake(Path::new("/repository/.git"));
    assert!(woken.borrow().is_empty(), "nothing was recorded, because nothing was asked");
    Recorder(&woken).wake(Path::new("/repository/.git"));
    assert_eq!(woken.borrow().len(), 1, "the seam asks whatever it holds");
}

/// What:
///  A caller that gives up a push record's values asks the service to forget them by that record's id,
///  and a caller with no service asks nobody.
/// Why:
///   The values live in the service's memory alone,
/// and a record a command removes here leaves nothing it could learn the ids from
///  (`service_push_programs_held_in_memory`).
#[test]
fn a_caller_tells_the_service_which_records_can_no_longer_run() {
    let forgotten: RefCell<Vec<(PathBuf, Vec<String>)>> = RefCell::new(Vec::new());
    struct Recorder<'a>(&'a RefCell<Vec<(PathBuf, Vec<String>)>>);
    impl WakeTheService for Recorder<'_> {
        fn wake(&self, _git_dir: &Path) {}

        fn forget_programs(&self, git_dir: &Path, records: &[String]) {
            self.0.borrow_mut().push((git_dir.to_path_buf(), records.to_vec()));
        }
    }
    let records: Vec<String> = vec![String::from("push-0123456789abcdef0123456789abcdef01234567-aaaa")];
    NothingWoken.forget_programs(Path::new("/repository/.git"), records.as_slice());
    assert!(forgotten.borrow().is_empty(), "nothing was recorded, because nothing was asked");
    Recorder(&forgotten).forget_programs(Path::new("/repository/.git"), records.as_slice());
    assert_eq!(forgotten.borrow().len(), 1, "the seam asks whatever it holds");
    assert_eq!(forgotten.borrow()[0].1, records, "the record that can no longer run is the one named");
}
