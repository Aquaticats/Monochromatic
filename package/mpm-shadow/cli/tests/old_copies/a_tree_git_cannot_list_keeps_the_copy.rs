//! What:
//! The end-to-end run for a tree `git` cannot list:
//! the link that could be seen moves to the managed copy,
//! and the old copy stays,
//! with the report saying why.
//! Why:
//! A second checkout's link may be hiding where git would not tell,
//! so removing the copy is the one thing a run must not do with a place left unsearched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // old_copies/a_tree_git_cannot_list_keeps_the_copy.test.ts
//! test('a tree git cannot list keeps the copy', () => { const fixture = Fixture.new(); ... });
//! ```

use std::fs;

use crate::{
    Fixture,
    make_link,
    text,
};

#[test]
fn a_tree_git_cannot_list_keeps_the_old_copy_and_says_why() {
    let fixture = Fixture::new();
    let local_git = fixture.home().join(".local/bin/git");
    make_link(&local_git, &fixture.old_copy());
    fs::remove_dir_all(fixture.work().join(".git")).expect("make the tree not a repository");
    let installed = fixture.run(&["install", "--frozen-lockfile"]);
    assert_eq!(
        installed.status.code(),
        Some(0),
        "{}{}",
        text(&installed.stdout),
        text(&installed.stderr)
    );
    assert!(
        fixture.old_copy().exists(),
        "a checkout's link could be hiding where git would not tell"
    );
    assert!(
        text(&installed.stdout).contains("is kept"),
        "{}",
        text(&installed.stdout)
    );
    assert_eq!(
        fs::read_link(&local_git).expect("a link"),
        fixture.managed_root().join("current/bin/cli-git-native"),
        "the link that was seen moved"
    );
}
