//! The owner of a copy transaction's items:
//! its token,
//! and the proof of whether it still runs.

use super::*;
use std::path::PathBuf;

/// A process id no running process has,
/// large enough that the kernel's default limit never reaches it.
const UNUSED_PID: i64 = 4_000_000_000;

/// A token names its owner exactly,
/// holds no dot or other character that would split a journal's name,
/// and stays short.
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

/// Text that is no token names nobody:
/// a name an older release wrote,
/// a stranger's file,
/// or a damaged token.
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

/// The running process is alive;
/// a killed and reaped process is dead,
/// and so is a live pid with another birth.
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
    assert!(!token_owner_is_dead(Some(here.token().as_str()), anchor).expect("judged"));
    assert!(token_owner_is_dead(Some(crate::test_support::dead_copy_owner().token().as_str()), anchor).expect("judged"));
    assert!(token_owner_is_dead(None, anchor).expect("judged"));
    assert!(token_owner_is_dead(Some("json"), anchor).expect("judged"));
}

/// An owner in another process-id namespace cannot be judged by its id:
/// it stays alive while its item is recent.
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

/// An owner of another namespace whose item is older than the patience is neither alive nor dead (owner,
/// 2026-11-04,
/// "Refuse and
/// name it (Recommended)"):
/// judging it is an error that names the item,
/// the owner,
/// its namespace,
/// the age and the `rm`;
/// the callers
/// stop,
/// and the item is not touched.
#[test]
fn an_owner_of_another_namespace_with_an_old_item_is_refused_and_named() {
    let root: PathBuf = crate::test_support::fixture("owner-foreign-expired");
    let item: PathBuf = root.join("item");
    std::fs::write(item.as_path(), b"x").expect("item");
    std::fs::File::options().write(true).open(item.as_path()).expect("open").set_modified(std::time::SystemTime::now() - crate::pid_namespace::FOREIGN_OWNER_PATIENCE - std::time::Duration::from_secs(300)).expect("age");
    let foreign: CopyOwner = CopyOwner { pid: UNUSED_PID, birth_identity: String::from("linux:1"), pid_namespace: Some(String::from("pid:[1]")) };
    let judged: Result<bool, WorktreeCopyError> = foreign.is_alive(item.as_path());
    let refused: WorktreeCopyError = judged.expect_err("an expired foreign item is refused");
    let text: String = refused.0;
    assert!(text.contains(item.to_string_lossy().as_ref()) && text.contains("pid:[1]") && text.contains(UNUSED_PID.to_string().as_str()), "{text}");
    assert!(text.contains("minutes old") && text.contains("another container") && text.contains("rm -- ") && text.contains("left running"), "{text}");
    assert!(item.exists(), "the item is untouched");
    crate::test_support::remove(root.as_path());
}

/// One decision is made from one reading of the item's age (sol-1 round 3,
/// P1 C):
/// an owner that is young at the first look cannot be
/// found expired at a second one,
/// and the refusal is never read as "not dead".
#[test]
fn an_owner_of_another_namespace_is_judged_from_one_reading_of_its_item() {
    let item: PathBuf = crate::test_support::fixture("copy-owner-one-reading").join("item");
    std::fs::write(item.as_path(), b"x").expect("item");
    let foreign = CopyOwner { pid: 31337, birth_identity: String::from("linux:1"), pid_namespace: Some(String::from("pid:[1]")) };
    crate::pid_namespace::STANDING_LOOKUPS.with(|count| { count.set(0); });
    let dead: bool = token_owner_is_dead(Some(foreign.token().as_str()), item.as_path()).expect("a young foreign item");
    assert!(!dead, "a young item of another namespace is alive");
    assert_eq!(crate::pid_namespace::STANDING_LOOKUPS.with(std::cell::Cell::get), 1, "the age was read once for the decision");
}

/// What:
///  A record of a live owner whose identity is spelled another way is never a dead owner's,
///  and the item it names is kept.
/// Why:
///  The wrapper writes one spelling of its own identity,
///  so a second spelling in a record was written by something that is not this release:
///  an older release,
///  another tool,
///  or a damaged file.
///  Read as another process,
///  it takes what a running command holds:
///  its journal,
///  its stage claim,
///  its owed copy,
///  and the private index a prediction reads.
///  A spelling a reader cannot compare is no proof of a dead owner,
///  so the owner counts as one that may still run and the item stays.
///  The arms cover the parts a record of an owner carries:
///  its process number,
///  its birth identity,
///  and its PID namespace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const respelled of spellings(owner)) expect(tokenOwnerIsDead(respelled, item)).toBe(false);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_live_owners_record_in_another_spelling_is_never_a_dead_owner() {
    let root: PathBuf = crate::test_support::fixture("copy-owner-spelled-another-way");
    // The item a record is judged by: an owner of another namespace is judged by this file's own age, so it is written
    // now.
    let item: PathBuf = root.join("item");
    std::fs::write(item.as_path(), b"x").expect("item");
    let here: CopyOwner = current_copy_owner().expect("this process has an identity");
    let namespace: &str = here.pid_namespace.as_deref().expect("this host reports the PID namespace of a process");
    let spellings: [(&str, String); 5] = [
        (
            "a leading zero on the start tick",
            CopyOwner {
                birth_identity: crate::process_identity_spellings::birth_with_a_leading_zero(here.birth_identity.as_str()),
                ..here.clone()
            }
            .token(),
        ),
        (
            "the platform's tag in capitals",
            CopyOwner {
                birth_identity: crate::process_identity_spellings::birth_with_its_platform_tag_in_capitals(here.birth_identity.as_str()),
                ..here.clone()
            }
            .token(),
        ),
        (
            "a space before the birth",
            CopyOwner {
                birth_identity: crate::process_identity_spellings::birth_behind_a_space(here.birth_identity.as_str()),
                ..here.clone()
            }
            .token(),
        ),
        // The process number is the first field of a token, so a padded one is the same token with a zero in front.
        ("a leading zero on the process number", format!("0{}", here.token())),
        (
            "a leading zero on the PID namespace",
            CopyOwner {
                pid_namespace: Some(crate::process_identity_spellings::namespace_with_a_leading_zero(namespace)),
                ..here.clone()
            }
            .token(),
        ),
    ];
    // Every arm is asked and every wrong answer is kept, so one run names each spelling that was read as another process
    // instead of stopping at the first.
    let mut wrong: Vec<String> = Vec::new();
    for (what, token) in spellings {
        let Some(read) = CopyOwner::from_token(token.as_str()) else {
            wrong.push(format!("{what}: the record {token} is not a token any reader parses"));
            continue;
        };
        if read.pid != here.pid {
            wrong.push(format!("{what}: the record {token} names process {} and one process's record is one process", read.pid));
        }
        if token_owner_is_dead(Some(token.as_str()), item.as_path()).expect("a young item of this namespace is no refusal") {
            wrong.push(format!("{what}: the reader took the record {token} for a dead owner's, and process {} runs", here.pid));
        }
        if !read.is_alive(item.as_path()).expect("a young item of this namespace is no refusal") {
            wrong.push(format!("{what}: `is_alive` answered that {token} does not run, and process {} does", here.pid));
        }
    }
    // The other answer of the same reader, so an arm that passes above is not a reader that never answers "dead": a
    // process that ended is a dead owner of the same shape of record.
    let ended: CopyOwner = crate::test_support::dead_copy_owner();
    assert!(
        token_owner_is_dead(Some(ended.token().as_str()), item.as_path()).expect("a young item"),
        "a process that ended is a dead owner"
    );
    assert!(!ended.is_alive(item.as_path()).expect("a young item"), "and it does not run");
    crate::test_support::remove(root.as_path());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
