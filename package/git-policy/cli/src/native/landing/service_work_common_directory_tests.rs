//! Controls for the common Git directory a service drain looks at:
//! the directory a linked worktree's `commondir` file names,
//! the `core.bare` answer its `config` gives,
//! and both files holding a named pipe instead,
//! which must answer at once rather than hold every drain.
//! Why:
//!  A drain that waits on a pipe is a service that runs no record at all,
//!  and the files it waits on are ones any process that may write the Git directory can put a pipe at
//!  (`crate::plain_file`; issue 6415 for the same shape at the wrapper's own records).

/// The rules under test.
use super::{
    common_directory_of,
    is_bare_common_directory,
};
/// Fixture folders, their removal and a bound on the wait.
use crate::test_support::{
    finished_within,
    fixture,
    remove,
};
use std::path::PathBuf;

/// A `commondir` that climbs out of the file system root names no directory,
/// and one that is relative to the Git directory names the one above it.
#[test]
fn a_commondir_is_resolved_by_its_text() {
    let root: PathBuf = fixture("service-common-dir-text");
    let common: PathBuf = root.join("main/.git");
    let own: PathBuf = common.join("worktrees/w");
    std::fs::create_dir_all(own.as_path()).expect("the linked worktree's Git directory");
    std::fs::write(own.join("commondir"), "../..\n").expect("the pointer");
    assert_eq!(common_directory_of(own.as_path()), Some(common.clone()));
    // More `..` than the fixture path has parts, so the resolution climbs past the root whatever depth the fixture sits at.
    std::fs::write(own.join("commondir"), format!("{}\n", "../".repeat(64)))
        .expect("a pointer past the root");
    assert_eq!(
        common_directory_of(own.as_path()),
        None,
        "a path that climbs past the root names nothing"
    );
    remove(root.as_path());
}

/// A repository whose `core.bare` is true is bare,
/// one whose configuration says otherwise is not,
/// and the worktree's own configuration file decides when the extension turns it on.
#[test]
fn core_bare_decides_whether_a_common_directory_is_bare() {
    let root: PathBuf = fixture("service-common-dir-bare");
    std::fs::write(root.join("config"), "[core]\n\tbare = true\n").expect("the configuration");
    assert!(is_bare_common_directory(root.as_path()));
    std::fs::write(root.join("config"), "[core]\n\tbare = false\n").expect("the configuration");
    assert!(!is_bare_common_directory(root.as_path()));
    std::fs::write(root.join("config"), "[core]\n\tbare = false\n[extensions]\n\tworktreeConfig = true\n[core]\n\trepositoryformatversion = 1\n")
        .expect("the configuration");
    std::fs::write(root.join("config.worktree"), "[core]\n\tbare = true\n")
        .expect("the worktree's configuration");
    assert!(
        is_bare_common_directory(root.as_path()),
        "the worktree's own file decides when the extension is on"
    );
    remove(root.as_path());
}

/// A `commondir` that is a named pipe names no directory,
/// and the drain's question answers at once:
/// opening a named pipe for reading waits for a writer that may never come.
#[test]
fn a_commondir_that_is_a_named_pipe_names_no_directory_at_once() {
    let root: PathBuf = fixture("service-common-dir-named-pipe");
    crate::subvolume_sync_attrs::make_fifo(root.join("commondir").as_path(), 0o600)
        .expect("a named pipe where a commondir goes");
    let inside: PathBuf = root.clone();
    let asked = finished_within(move || return common_directory_of(inside.as_path()));
    let Some(named) = asked else {
        panic!("a `commondir` that is a named pipe held the drain")
    };
    assert_eq!(named, None, "a pipe names no common directory");
    // The read this module made before, on the same fixture under the same bound, so the bound above is shown to see a wait.
    let pointer: PathBuf = root.join("commondir");
    let waited = finished_within(move || return std::fs::read_to_string(pointer).ok());
    assert!(
        waited.is_none(),
        "a plain read of the same name waits, so the bound above can see a wait"
    );
    remove(root.as_path());
}

/// A `config` that is a named pipe is a repository this command cannot read,
/// which answers as not bare at once,
/// so the records stay for the main worktree's instance,
/// the safe side of a wrong answer.
#[test]
fn a_config_that_is_a_named_pipe_is_not_bare_at_once() {
    let root: PathBuf = fixture("service-common-dir-config-named-pipe");
    crate::subvolume_sync_attrs::make_fifo(root.join("config").as_path(), 0o600)
        .expect("a named pipe where a configuration goes");
    let inside: PathBuf = root.clone();
    let asked = finished_within(move || return is_bare_common_directory(inside.as_path()));
    let Some(bare) = asked else {
        panic!("a `config` that is a named pipe held the drain")
    };
    assert!(!bare, "a pipe is no configuration this command reads");
    // The read this module made before, on the same fixture under the same bound, so the bound above is shown to see a wait.
    let configuration: PathBuf = root.join("config");
    let waited = finished_within(move || return std::fs::read(configuration).ok());
    assert!(
        waited.is_none(),
        "a plain read of the same name waits, so the bound above can see a wait"
    );
    remove(root.as_path());
}
