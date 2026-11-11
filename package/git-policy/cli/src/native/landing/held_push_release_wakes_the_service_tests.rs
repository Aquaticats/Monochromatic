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
    }
    NothingWoken.wake(Path::new("/repository/.git"));
    assert!(woken.borrow().is_empty(), "nothing was recorded, because nothing was asked");
    Recorder(&woken).wake(Path::new("/repository/.git"));
    assert_eq!(woken.borrow().len(), 1, "the seam asks whatever it holds");
}
