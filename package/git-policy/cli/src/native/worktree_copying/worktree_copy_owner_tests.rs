//! The owner of a copy transaction's items: its token, and the proof of whether it still runs.

use super::*;
use std::path::PathBuf;

/// A process id no running process has, large enough that the kernel's default limit never reaches it.
const UNUSED_PID: i64 = 4_000_000_000;

/// A token names its owner exactly, holds no dot or other character that would split a journal's name, and stays short.
#[test]
fn a_token_round_trips_and_is_safe_in_a_file_name() {
    let owner: CopyOwner = CopyOwner { pid: 77, birth_identity: String::from("linux:123"), pid_namespace: Some(String::from("pid:[4026531836]")) };
    let token: String = owner.token();
    assert!(token.bytes().all(|byte| { return byte.is_ascii_alphanumeric() || b":-[]~%".contains(&byte); }), "{token}");
    assert_eq!(CopyOwner::from_token(token.as_str()), Some(owner));
    let without_namespace: CopyOwner = CopyOwner { pid: 78, birth_identity: String::from("linux:9"), pid_namespace: None };
    assert_eq!(CopyOwner::from_token(without_namespace.token().as_str()), Some(without_namespace));
    // An identity with spaces, dots, slashes and the separator itself survives, as the identity of another platform may hold them.
    let awkward: CopyOwner = CopyOwner { pid: 79, birth_identity: String::from("darwin:Mon Nov  3 02:00:00.5 2026/~%"), pid_namespace: None };
    let awkward_token: String = awkward.token();
    assert!(!awkward_token.contains(['.', '/', ' ']), "{awkward_token}");
    assert_eq!(CopyOwner::from_token(awkward_token.as_str()), Some(awkward));
}

/// Text that is no token names nobody: a name an older release wrote, a stranger's file, or a damaged token.
#[test]
fn text_that_is_not_a_token_names_nobody() {
    assert_eq!(CopyOwner::from_token("json"), None);
    assert_eq!(CopyOwner::from_token(""), None);
    assert_eq!(CopyOwner::from_token("0~linux:1~"), None, "a pid below one");
    assert_eq!(CopyOwner::from_token("5~~"), None, "an empty birth identity");
    assert_eq!(CopyOwner::from_token("5~linux:1~ns~extra"), None, "a fourth field");
    assert_eq!(CopyOwner::from_token("5~linux:%zz~"), None, "an escape that is not hexadecimal");
    assert_eq!(CopyOwner::from_token("5~linux:%4~"), None, "an escape cut short");
}

/// The running process is alive; a killed and reaped process is dead, and so is a live pid with another birth.
#[test]
fn the_running_process_is_alive_and_a_killed_one_is_dead() {
    let here: CopyOwner = current_copy_owner().expect("current owner");
    let anchor: &Path = Path::new("/");
    assert!(here.is_alive(anchor).expect("answer"));
    assert!(!crate::test_support::dead_copy_owner().is_alive(anchor).expect("answer"));
    let reused: CopyOwner = CopyOwner { birth_identity: String::from("linux:not-this-process"), ..here };
    assert!(!reused.is_alive(anchor).expect("answer"), "a reused process id is not the owner");
}

/// An id no process has is dead.
#[test]
fn a_process_id_nobody_has_is_dead() {
    let owner: CopyOwner = CopyOwner { pid: UNUSED_PID, birth_identity: String::from("linux:1"), pid_namespace: current_pid_namespace() };
    assert!(!owner.is_alive(Path::new("/")).expect("answer"));
}

/// The deadness check keeps what it cannot judge and takes only a proven dead owner or a nameless one.
#[test]
fn only_a_proven_dead_or_nameless_owner_is_dead() {
    let here: CopyOwner = current_copy_owner().expect("current owner");
    let anchor: &Path = Path::new("/");
    assert!(!token_owner_is_dead(Some(here.token().as_str()), anchor));
    assert!(token_owner_is_dead(Some(crate::test_support::dead_copy_owner().token().as_str()), anchor));
    assert!(token_owner_is_dead(None, anchor));
    assert!(token_owner_is_dead(Some("json"), anchor));
}

/// An owner in another process-id namespace cannot be judged by its id: it stays alive while its item is recent.
#[test]
fn an_owner_of_another_namespace_stays_while_its_item_is_recent() {
    let root: PathBuf = crate::test_support::fixture("owner-foreign");
    let item: PathBuf = root.join("item");
    std::fs::write(item.as_path(), b"x").expect("item");
    let foreign: CopyOwner = CopyOwner { pid: UNUSED_PID, birth_identity: String::from("linux:1"), pid_namespace: Some(String::from("pid:[1]")) };
    let judged: Result<bool, WorktreeCopyError> = foreign.is_alive(item.as_path());
    crate::test_support::remove(root.as_path());
    assert!(judged.expect("answer"));
}
