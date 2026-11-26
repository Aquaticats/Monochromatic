//! What:
//!  Controls for the put-back:
//!  the exclusive write,
//!  the sentence it holds,
//!  the line it returns,
//!  a lock that already stands,
//!  and a link at the name.
//! Why:
//!  The project decided a lost lock is written again and reported,
//!  and an install that overwrote a standing lock would undo a person's `git worktree lock` silently.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(writeTheLostLock(registration, slot, nonce)).toEqual({ kind: 'wrote', line });
//! ```

use super::{
    InstallAnswer,
    line_of_an_installed_lock,
    write_the_lost_lock,
};
use crate::worktree_clone_binding_record::lock_words::{
    LOCK_WORDS_MAXIMUM_BYTES,
    lock_file_bytes,
};
use crate::worktree_clone_managed_lock::test_fixtures::{
    Fixture,
    SLOT,
};
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

/// The fixture's nonce.
const NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// The parser's reading of it.
fn nonce() -> RequestNonce {
    return RequestNonce::parse(NONCE).expect("the fixture nonce reads");
}

/// A lost lock is written with the sentence and Git's newline,
/// at the mode Git's own write leaves,
/// and the command owes the one line.
#[test]
fn a_lost_lock_is_written_and_reported() {
    let fixture: Fixture = Fixture::new("install-written");
    let registration: PathBuf = fixture.write_registration("locked-slot");
    let answer: InstallAnswer = write_the_lost_lock(registration.as_path(), SLOT, &nonce());
    assert_eq!(
        answer,
        InstallAnswer::Wrote {
            line: line_of_an_installed_lock(SLOT, &nonce())
        }
    );
    let bytes: Vec<u8> = std::fs::read(registration.join("locked")).expect("the lock reads");
    assert_eq!(bytes, lock_file_bytes(&nonce()));
    assert_eq!(bytes.len() as u64, LOCK_WORDS_MAXIMUM_BYTES);
    let mode: u32 = std::fs::metadata(registration.join("locked"))
        .expect("the lock's metadata reads")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o644);
    let line: String = line_of_an_installed_lock(SLOT, &nonce());
    assert_eq!(
        line,
        "the clone slot locked-slot had lost its managed Git lock, so this command wrote it again for add nonce 9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f"
    );
}

/// A lock that stands is never overwritten,
/// and the second install prints nothing.
#[test]
fn a_standing_lock_is_never_overwritten() {
    let fixture: Fixture = Fixture::new("install-standing");
    let registration: PathBuf = fixture.write_registration("locked-slot");
    fixture.write_lock(registration.as_path(), b"initializing\n");
    assert_eq!(
        write_the_lost_lock(registration.as_path(), SLOT, &nonce()),
        InstallAnswer::ALockStands
    );
    let bytes: Vec<u8> = std::fs::read(registration.join("locked")).expect("the lock reads");
    assert_eq!(bytes, b"initializing\n");
}

/// Two installs at once leave one lock:
/// the first writes and the second answers that one stands.
#[test]
fn the_second_install_leaves_the_first_lock() {
    let fixture: Fixture = Fixture::new("install-twice");
    let registration: PathBuf = fixture.write_registration("locked-slot");
    let first: InstallAnswer = write_the_lost_lock(registration.as_path(), SLOT, &nonce());
    let second: InstallAnswer = write_the_lost_lock(registration.as_path(), SLOT, &nonce());
    assert!(matches!(first, InstallAnswer::Wrote { .. }));
    assert_eq!(second, InstallAnswer::ALockStands);
    assert_eq!(
        std::fs::read(registration.join("locked")).expect("the lock reads"),
        lock_file_bytes(&nonce())
    );
}

/// A link at the lock's name is never written through:
/// `O_CREAT | O_EXCL` refuses any name that stands,
/// a symbolic link among them,
/// so the write answers that something is there and the target keeps its bytes.
#[test]
fn a_link_at_the_lock_name_is_never_written_through() {
    let fixture: Fixture = Fixture::new("install-link");
    let registration: PathBuf = fixture.write_registration("locked-slot");
    let elsewhere: PathBuf = fixture.root.join("elsewhere");
    std::fs::write(elsewhere.as_path(), b"").expect("the target is written");
    std::os::unix::fs::symlink(elsewhere.as_path(), registration.join("locked"))
        .expect("the link is made");
    assert_eq!(
        write_the_lost_lock(registration.as_path(), SLOT, &nonce()),
        InstallAnswer::ALockStands
    );
    assert_eq!(
        std::fs::read(elsewhere.as_path()).expect("the target reads"),
        b""
    );
    assert!(
        std::fs::symlink_metadata(registration.join("locked"))
            .expect("the name's metadata reads")
            .file_type()
            .is_symlink(),
        "the link is left as it stood"
    );
}

/// A registration folder that is gone refuses the write,
/// with the reason named.
#[test]
fn a_missing_registration_refuses_the_write() {
    let fixture: Fixture = Fixture::new("install-missing");
    assert!(matches!(
        write_the_lost_lock(fixture.worktrees().join("gone").as_path(), SLOT, &nonce()),
        InstallAnswer::Refused { .. }
    ));
}
