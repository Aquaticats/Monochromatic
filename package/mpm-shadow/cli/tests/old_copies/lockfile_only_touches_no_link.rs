//! What:
//! The end-to-end run of the two `install --lockfile-only` modes:
//! neither re-points a link nor removes an old copy.
//! Why:
//! A lockfile-only run installs nothing,
//! so no managed copy is made or checked,
//! and a run that moved a link would move it to a copy it never installed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // old_copies/lockfile_only_touches_no_link.test.ts
//! test('lockfile only touches no link', () => { const fixture = Fixture.new(); ... });
//! ```

use std::fs;

use crate::{
    Fixture,
    make_link,
    text,
};

#[test]
fn install_lockfile_only_changes_no_link_and_removes_no_copy() {
    let fixture = Fixture::new();
    let local_git = fixture.home().join(".local/bin/git");
    make_link(&local_git, &fixture.old_copy());
    let locked = fixture.run(&["install", "--lockfile-only", "--frozen-lockfile"]);
    assert_eq!(locked.status.code(), Some(0), "{}", text(&locked.stderr));
    let offline = fixture.run(&["install", "--lockfile-only", "--offline"]);
    assert_eq!(offline.status.code(), Some(0), "{}", text(&offline.stderr));
    assert!(fixture.old_copy().exists());
    assert_eq!(
        fs::read_link(&local_git).expect("a link"),
        fixture.old_copy()
    );
}
