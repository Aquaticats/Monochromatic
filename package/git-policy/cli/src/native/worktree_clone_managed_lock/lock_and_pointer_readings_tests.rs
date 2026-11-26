//! What:
//!  Controls for the two named reads:
//!  a registration's `locked` and a slot's `.git`,
//!  each read at its own path with its own bound.
//! Why:
//!  The two names are Git's own,
//!  and a reader that looked for either somewhere else would classify nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readTheLock(registration)).toEqual({ kind: 'bytes', bytes });
//! expect(readTheSlotPointer(slot)).toEqual({ kind: 'absent' });
//! ```

use super::{
    LOCK_FILE_NAME,
    SLOT_POINTER_NAME,
    lock_file_path,
    read_the_lock,
    read_the_slot_pointer,
    slot_pointer_path,
};
use crate::worktree_clone_binding_record::lock_words::LOCK_WORDS_MAXIMUM_BYTES;
use crate::worktree_clone_managed_lock::Reading;
use crate::worktree_clone_managed_lock::test_fixtures::{
    Fixture,
    RecordSpec,
};

/// The fixture's nonce,
/// which every control recomposes its sentence from.
const NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// The two names are Git's own,
/// and each read names the file below the folder it is given.
#[test]
fn the_two_names_are_gits_own() {
    let fixture: Fixture = Fixture::new("readings-paths");
    let registration: std::path::PathBuf = fixture.worktrees().join("entry");
    assert_eq!(
        lock_file_path(registration.as_path()),
        registration.join(LOCK_FILE_NAME)
    );
    assert_eq!(
        slot_pointer_path(fixture.slot_path.as_path()),
        fixture.slot_path.join(SLOT_POINTER_NAME)
    );
    assert_eq!(LOCK_FILE_NAME, "locked");
    assert_eq!(SLOT_POINTER_NAME, ".git");
}

/// A lock the add wrote reads as the sentence and its newline,
/// and its length is the bound this route reads locks with.
#[test]
fn a_written_lock_reads_as_its_sentence() {
    let fixture: Fixture = Fixture::new("readings-lock");
    let registration: std::path::PathBuf = fixture.write_registration("entry");
    fixture.write_lock(
        registration.as_path(),
        fixture.the_sentence(NONCE).as_slice(),
    );
    assert_eq!(
        read_the_lock(registration.as_path()),
        Reading::Bytes(fixture.the_sentence(NONCE))
    );
    assert_eq!(
        fixture.the_sentence(NONCE).len() as u64,
        LOCK_WORDS_MAXIMUM_BYTES
    );
}

/// A lock wider than the sentence is refused rather than read,
/// which is the state S3 and not a longer sentence.
#[test]
fn a_wider_lock_is_refused() {
    let fixture: Fixture = Fixture::new("readings-wide-lock");
    let registration: std::path::PathBuf = fixture.write_registration("entry");
    let mut wider: Vec<u8> = fixture.the_sentence(NONCE);
    wider.push(b'x');
    fixture.write_lock(registration.as_path(), wider.as_slice());
    let wide: Reading = read_the_lock(registration.as_path());
    assert!(
        matches!(wide, Reading::Refused(_)),
        "a lock wider than the sentence is refused, and it read as {wide:?}"
    );
}

/// A slot with no `.git` reads as absence,
/// which is the state S5 and not a refusal.
#[test]
fn a_slot_with_no_pointer_reads_as_absence() {
    let fixture: Fixture = Fixture::new("readings-pointer-absent");
    assert_eq!(
        read_the_slot_pointer(fixture.slot_path.as_path()),
        Reading::Absent
    );
    let spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    fixture.write_record(&spec);
    let registration: std::path::PathBuf = fixture.write_registration("entry");
    fixture.write_pointer(registration.as_path());
    assert_eq!(
        read_the_slot_pointer(fixture.slot_path.as_path()),
        Reading::Bytes(format!("gitdir: {}\n", registration.display()).into_bytes())
    );
}
