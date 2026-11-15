//! What:
//!  Controls of the answer that decides whether a standing lock is named to the command that met it:
//!  the worktree the command runs in and the shared common directory,
//!  and never another linked worktree's own administrative directory.
//! Why:
//!  The gate on `0.6.1-alpha.34` failed an add that printed a standing-lock line about another add's `index.lock`,
//!  which the pass had found by walking the common directory into every linked worktree.
//!  The proof that clears or keeps a lock is unchanged everywhere;
//!  only the words stay where the person is,
//!  so these controls hold the line between the two.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(standsInTheWay('/r/.git/worktrees/held/index.lock', '/r/.git', '/r/.git')).toBe(false);
//! ```

use super::stands_in_the_way;
use std::path::Path;

/// The Git directory of a repository whose worktrees are below it,
/// as every path of a control is written.
const COMMON: &str = "/r/.git";

/// What:
///  A command of the main checkout meets the lock of the shared index and of a shared reference,
///  and not the lock of a linked worktree's own administrative directory.
/// Why:
///  The shared index and the references are the files this command's own Git writes;
///  the worktree below `worktrees/` is another worktree's state,
///  which its own commands meet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(standsInTheWay('/r/.git/index.lock', '/r/.git', '/r/.git')).toBe(true);
/// ```
#[test]
fn a_main_checkouts_command_meets_the_shared_locks_and_no_other_worktrees() {
    let common: &Path = Path::new(COMMON);
    assert!(stands_in_the_way(Path::new("/r/.git/index.lock"), common, common), "the shared index is this command's");
    assert!(stands_in_the_way(Path::new("/r/.git/refs/heads/main.lock"), common, common), "a shared reference is this command's");
    assert!(!stands_in_the_way(Path::new("/r/.git/worktrees/held/index.lock"), common, common), "another worktree's index lock is not");
}

/// What:
///  A command of a linked worktree meets its own administrative directory's lock,
///  the shared locks,
///  and not a sibling worktree's lock.
/// Why:
///  The index of a linked worktree lives in its own administrative directory,
///  so that lock is the one this command's Git writes;
///  a sibling's is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(standsInTheWay('/r/.git/worktrees/held/index.lock', '/r/.git/worktrees/held', '/r/.git')).toBe(true);
/// ```
#[test]
fn a_linked_worktrees_command_meets_its_own_administrative_directory_and_the_shared_locks() {
    let common: &Path = Path::new(COMMON);
    let own: &Path = Path::new("/r/.git/worktrees/held");
    assert!(stands_in_the_way(Path::new("/r/.git/worktrees/held/index.lock"), own, common), "its own index lock is this command's");
    assert!(stands_in_the_way(Path::new("/r/.git/index.lock"), own, common), "the shared index is this command's");
    assert!(!stands_in_the_way(Path::new("/r/.git/worktrees/other/index.lock"), own, common), "a sibling's index lock is not");
}

/// What:
///  A lock the pass cannot place below the common directory is named.
/// Why:
///  A path this cannot place is the state every lock got before this answer existed,
///  so a lock is named unless the pass knows it is another worktree's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(standsInTheWay('/elsewhere/index.lock', '/r/.git', '/r/.git')).toBe(true);
/// ```
#[test]
fn a_lock_no_worktree_explains_is_named() {
    assert!(stands_in_the_way(Path::new("/elsewhere/index.lock"), Path::new(COMMON), Path::new(COMMON)), "a lock outside the Git directories is named rather than hidden");
}

/// What:
///  A lock of a reference whose namespace is named `objects` or `logs` is found by the pass,
///  as one in any other namespace is.
/// Why:
///  Git accepts `refs/heads/objects/topic` and `refs/heads/logs/topic`,
///  and a lock taken for one stands in the folder that name gives.
///  A walk that skipped those names at every depth never found such a lock,
///  so a record of it was a licence nothing acted on,
///  and the command that met the lock could not clear it
///  (a review of `6a506fc03f`,
/// finding 5).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const ns of ['ordinary', 'objects', 'logs']) write(refs/heads/ns/topic.lock)
/// expect(walked([root]).locks).toEqual(all three)
/// ```
#[test]
fn a_lock_in_a_reference_namespace_named_objects_is_found() {
    let root: std::path::PathBuf = crate::test_support::fixture("abandoned-locks-objects-namespace");
    let mut expected: Vec<std::path::PathBuf> = Vec::new();
    for namespace in ["ordinary", "objects", "logs"] {
        let folder: std::path::PathBuf = root.join("refs/heads").join(namespace);
        std::fs::create_dir_all(folder.as_path()).expect("the reference namespace");
        let lock: std::path::PathBuf = folder.join("topic.lock");
        std::fs::write(lock.as_path(), b"a reference update\n").expect("the reference lock");
        expected.push(lock);
    }
    let found: Vec<std::path::PathBuf> = super::walked(std::slice::from_ref(&root)).locks;
    crate::test_support::remove(root.as_path());
    for lock in expected {
        assert!(found.contains(&lock), "the pass finds the lock of {}: {found:?}", lock.display());
    }
}
