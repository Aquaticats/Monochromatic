//! Controls for the recipe that clears an incomplete registration:
//!  every name it knew is gone and the slot's pointer with them,
//!  the lock is removed last,
//!  a pointer naming another registration refuses and removes nothing,
//!  and a second run treats every missing name as done.

use crate::worktree_clone_binding_record::lock_words::lock_file_bytes;
use crate::worktree_clone_binding_record::registration_recipe::{
    NoSeams,
    RecipeRefusal,
    RecipeSeams,
    ToClear,
    clear_registration,
};
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

/// What the seams saw,
/// in the order they ran.
#[derive(Default)]
struct Seen {
    pointer_before: bool,
    locked_alone_before_the_lock: bool,
}

/// The seams of the fixture,
/// which read the tree at the two moments the design names.
struct ControlSeams<'a> {
    slot: &'a Path,
    registration: &'a Path,
    seen: &'a mut Seen,
}

impl RecipeSeams for ControlSeams<'_> {
    fn before_first_removal(&mut self) -> std::io::Result<()> {
        self.seen.pointer_before = self.slot.join(".git").exists();
        return Ok(());
    }

    fn before_the_lock(&mut self) -> std::io::Result<()> {
        let names: Vec<String> = std::fs::read_dir(self.registration)
            .map(|folder| {
                return folder
                    .flatten()
                    .map(|entry| return entry.file_name().to_string_lossy().into_owned())
                    .collect();
            })
            .unwrap_or_default();
        self.seen.locked_alone_before_the_lock = names == vec![String::from("locked")];
        return Ok(());
    }
}

/// A registration Git's add leaves complete,
/// its slot pointer and every name the recipe removes.
fn planted(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root: PathBuf = crate::test_support::fixture(name);
    let common: PathBuf = root.join(".git");
    let registration: PathBuf = common.join("worktrees/add-cache-layer");
    std::fs::create_dir_all(registration.join("refs/heads")).expect("the ref store is made");
    std::fs::create_dir_all(registration.join("logs")).expect("the log folder is made");
    std::fs::write(
        registration.join("gitdir"),
        b"/repo/.claude/worktrees/add-cache-layer/.git\n",
    )
    .expect("gitdir");
    std::fs::write(registration.join("commondir"), b"../..\n").expect("commondir");
    std::fs::write(registration.join("HEAD"), b"ref: refs/heads/main\n").expect("HEAD");
    std::fs::write(registration.join("index"), b"an index").expect("index");
    std::fs::write(registration.join("locked"), lock_file_bytes(&nonce())).expect("locked");
    std::fs::write(registration.join("refs/heads/main"), b"a commit").expect("a ref");
    std::fs::write(registration.join("logs/HEAD"), b"a log").expect("the log");
    let slot: PathBuf = root.join(".claude/worktrees/add-cache-layer");
    std::fs::create_dir_all(slot.as_path()).expect("the slot is made");
    std::fs::write(
        slot.join(".git"),
        format!("gitdir: {}\n", registration.display()),
    )
    .expect("the pointer");
    return (root, registration, slot);
}

/// The fixture's nonce.
fn nonce() -> RequestNonce {
    return RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f")
        .expect("the fixture nonce reads");
}

/// The recipe removes the pointer,
/// every name,
/// the lock last and the directory,
/// and leaves the slot empty.
#[test]
fn the_recipe_empties_the_registration_and_the_slot() {
    let (root, registration, slot) = planted("recipe-clear");
    let worktrees: PathBuf = registration
        .parent()
        .expect("the registration has a parent")
        .to_path_buf();
    let slot_parent: std::fs::File =
        std::fs::File::open(root.join(".claude/worktrees")).expect("the slot parent opens");
    let worktrees_parent: std::fs::File =
        std::fs::File::open(worktrees.as_path()).expect("the worktrees parent opens");
    let to: ToClear = ToClear {
        slot_parent: &slot_parent,
        slot_pointer_name: OsStr::new(".git"),
        slot: slot.as_path(),
        worktrees_parent: &worktrees_parent,
        registration_name: OsStr::new("add-cache-layer"),
        registration: registration.as_path(),
    };
    let mut seen: Seen = Seen::default();
    let outcome = clear_registration(
        &to,
        &mut ControlSeams {
            slot: slot.as_path(),
            registration: registration.as_path(),
            seen: &mut seen,
        },
    );
    assert!(
        outcome.is_ok(),
        "{}",
        outcome.expect_err("the recipe refused").message()
    );
    assert!(seen.pointer_before);
    assert!(seen.locked_alone_before_the_lock);
    assert!(!registration.exists());
    assert!(!slot.join(".git").exists());
    let left: Vec<String> = std::fs::read_dir(slot.as_path())
        .expect("the slot lists")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(left.is_empty(), "the slot still holds {left:?}");
    let again = clear_registration(&to, &mut NoSeams);
    assert!(
        again.is_ok(),
        "{}",
        again.expect_err("the second run refused").message()
    );
    crate::test_support::remove(root.as_path());
}

/// A pointer that names another registration refuses,
/// and nothing is removed.
#[test]
fn a_pointer_naming_another_registration_refuses() {
    let (root, registration, slot) = planted("recipe-foreign-pointer");
    std::fs::write(slot.join(".git"), b"gitdir: /repo/.git/worktrees/other\n")
        .expect("the foreign pointer");
    let worktrees: PathBuf = registration
        .parent()
        .expect("the registration has a parent")
        .to_path_buf();
    let slot_parent: std::fs::File =
        std::fs::File::open(root.join(".claude/worktrees")).expect("the slot parent opens");
    let worktrees_parent: std::fs::File =
        std::fs::File::open(worktrees.as_path()).expect("the worktrees parent opens");
    let to: ToClear = ToClear {
        slot_parent: &slot_parent,
        slot_pointer_name: OsStr::new(".git"),
        slot: slot.as_path(),
        worktrees_parent: &worktrees_parent,
        registration_name: OsStr::new("add-cache-layer"),
        registration: registration.as_path(),
    };
    let refusal: RecipeRefusal =
        clear_registration(&to, &mut NoSeams).expect_err("the foreign pointer refused");
    assert!(
        matches!(refusal, RecipeRefusal::PointerNamesAnother { .. }),
        "{}",
        refusal.message()
    );
    assert!(registration.exists());
    assert!(registration.join("locked").exists());
    assert!(slot.join(".git").exists());
    crate::test_support::remove(root.as_path());
}
