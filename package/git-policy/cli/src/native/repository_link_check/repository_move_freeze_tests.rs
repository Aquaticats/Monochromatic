//! Controls for the freeze record:
//! a repository under a frozen subvolume is covered,
//! one beside it is not,
//! and a manifest the thaw renamed aside covers nothing.

use super::*;
use crate::test_support::{fixture, remove};

/// A state folder holding a manifest of two frozen subvolumes (the runner's rows: UUID, tab, path).
fn state_with_manifest(name: &str) -> PathBuf {
    let state: PathBuf = fixture(name);
    std::fs::write(
        state.join(FREEZE_MANIFEST_NAME),
        "0a1b\t/work/Monochromatic\n2c3d\t/work/Monochromatic/.claude/worktrees/a\n",
    )
    .expect("manifest");
    return state;
}

/// A repository at or under a frozen subvolume is covered,
/// and a neighbour with a longer name is not.
#[test]
fn a_repository_under_a_frozen_subvolume_is_covered_and_a_neighbour_is_not() {
    let state: PathBuf = state_with_manifest("freeze-covers");
    for covered in ["/work/Monochromatic", "/work/Monochromatic/.claude/worktrees/a", "/work/Monochromatic/package/x"] {
        assert!(freeze_covers(state.as_path(), Path::new(covered)), "{covered} is covered");
    }
    for outside in ["/work/Monochromatic2", "/work", "/elsewhere/Monochromatic"] {
        assert!(!freeze_covers(state.as_path(), Path::new(outside)), "{outside} is not covered");
    }
    remove(state.as_path());
}

/// No manifest, an unreadable one and one the thaw renamed aside cover nothing.
#[test]
fn without_a_manifest_or_after_the_thaw_nothing_is_covered() {
    let state: PathBuf = fixture("freeze-none");
    assert!(!freeze_covers(state.as_path(), Path::new("/work/Monochromatic")), "no manifest");
    std::fs::write(state.join(FREEZE_MANIFEST_NAME), b"\xff\xfe not a table").expect("garbage");
    assert!(!freeze_covers(state.as_path(), Path::new("/work/Monochromatic")), "a manifest that is not a table");
    std::fs::rename(state.join(FREEZE_MANIFEST_NAME), state.join("freeze-manifest.tsv.thawed-1")).expect("renamed aside");
    assert!(!freeze_covers(state.as_path(), Path::new("/work/Monochromatic")), "renamed aside by the thaw");
    remove(state.as_path());
}
