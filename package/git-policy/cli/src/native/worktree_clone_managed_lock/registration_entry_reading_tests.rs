//! What:
//!  Controls for reading one `worktrees/<name>` entry:
//!  its lock,
//!  its `gitdir` with a relative spelling,
//!  whether `commondir` and `HEAD` stand,
//!  the slot's pointer,
//!  and how many entries the slot holds besides `.git`.
//! Why:
//!  The finder decides from these seven facts,
//!  so a reader that missed one would put an entry in the wrong row.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readOneEntry(worktrees, 'slot', slot).locked).toEqual(bytes);
//! ```

use super::read_one_entry;
use crate::worktree_clone_binding_record::registration_finder::EntryFacts;
use crate::worktree_clone_managed_lock::test_fixtures::{
    Fixture,
    SLOT,
};

/// The fixture's nonce.
const NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// A complete entry reads with all seven facts,
/// and the slot's other entries are counted.
#[test]
fn a_complete_entry_reads_with_every_fact() {
    let fixture: Fixture = Fixture::new("entry-complete");
    let registration: std::path::PathBuf = fixture.write_registration("entry");
    fixture.write_pointer(registration.as_path());
    fixture.write_lock(
        registration.as_path(),
        fixture.the_sentence(NONCE).as_slice(),
    );
    std::fs::write(fixture.slot_path.join("checked-out-file"), b"x")
        .expect("a file stands in the slot");
    let facts: EntryFacts = read_one_entry(
        fixture.worktrees().as_path(),
        "entry",
        fixture.slot_path.as_path(),
    );
    assert_eq!(facts.name, "entry");
    assert_eq!(facts.locked, Some(fixture.the_sentence(NONCE)));
    assert_eq!(facts.gitdir, Some(fixture.slot_path.join(".git")));
    assert!(facts.commondir);
    assert!(facts.head);
    assert_eq!(
        facts.slot_git_pointer,
        Some(format!("gitdir: {}\n", registration.display()).into_bytes())
    );
    assert_eq!(facts.slot_other_entries, 1);
}

/// An unlocked entry holds no lock,
/// and a slot with no pointer answers no pointer.
#[test]
fn an_unlocked_entry_and_an_empty_slot_read_as_nothing() {
    let fixture: Fixture = Fixture::new("entry-unlocked");
    let registration: std::path::PathBuf = fixture.write_registration("entry");
    std::fs::remove_file(registration.join("HEAD"))
        .expect("the head goes, which an unfinished entry never wrote");
    let facts: EntryFacts = read_one_entry(
        fixture.worktrees().as_path(),
        "entry",
        fixture.slot_path.as_path(),
    );
    assert_eq!(facts.locked, None);
    assert!(!facts.head);
    assert_eq!(facts.slot_git_pointer, None);
    assert_eq!(facts.slot_other_entries, 0);
}

/// A lock the read refuses attributes nothing,
/// while the entry's own names still read,
/// because `file_exists` keeps the registration from a prune whatever the file is.
#[test]
fn a_lock_the_read_refuses_attributes_nothing() {
    let fixture: Fixture = Fixture::new("entry-refused-lock");
    let registration: std::path::PathBuf = fixture.write_registration("entry");
    std::fs::create_dir_all(registration.join("locked"))
        .expect("a directory takes the lock's name");
    let facts: EntryFacts = read_one_entry(
        fixture.worktrees().as_path(),
        "entry",
        fixture.slot_path.as_path(),
    );
    assert_eq!(facts.locked, Some(Vec::new()));
    assert!(facts.commondir);
}

/// A `gitdir` written relative to the registration resolves to this slot's `.git`.
#[test]
fn a_relative_gitdir_resolves() {
    let fixture: Fixture = Fixture::new("entry-relative");
    let registration: std::path::PathBuf = fixture.write_registration("entry");
    let depth: usize = registration
        .strip_prefix(fixture.root.as_path())
        .expect("the registration is inside the fixture")
        .components()
        .count();
    let up: String = (0..depth).map(|_step| return "../").collect::<String>();
    std::fs::write(
        registration.join("gitdir"),
        format!("{up}repository/.claude/worktrees/{SLOT}/.git\n").as_bytes(),
    )
    .expect("the back-link is written relative");
    let facts: EntryFacts = read_one_entry(
        fixture.worktrees().as_path(),
        "entry",
        fixture.slot_path.as_path(),
    );
    assert_eq!(facts.gitdir, Some(fixture.slot_path.join(".git")));
}
