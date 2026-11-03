//! What:
//!  Ordering and race controls of the durable-record components that need the test seams (the event log and the fault points).
//! Why:
//!  A directory trace cannot show that a saved file is synced before a ledger is truncated, and a barrier cannot force the unsafe
//!  interleaving; the events (file syncs, directory syncs, truncation, each recorded after it succeeded) and the fault points do.
//!  The seams exist only with the Cargo feature `test-seams`, so this module is compiled only then.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('the saved tail is synced before the ledger is truncated', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::model_fixture::scratch;
use git_policy_cli::commit_identity::{
    Bound, bind, binding_path, create_dir_all_durable, init_root, set_fault_point, start_events, take_events,
};
use git_policy_cli::commit_integrity::sha256_hex;
use git_policy_cli::commit_ledger::{LedgerEntry, append, repair_tail};
use git_policy_cli::commit_request::{Change, Operation, Published, Request, RequestError, decode, publish};
use std::path::{Path, PathBuf};

/// A 32-character identity made of one repeated digit.
fn identity(digit: char) -> String {
    return digit.to_string().repeat(32);
}

/// A ledger entry for identity digit `digit`.
fn entry(digit: char) -> LedgerEntry {
    return LedgerEntry { id: identity(digit), outcome: "published".to_string(), commit: "a".repeat(40), time: "2026-11-03T12:00:00-05:00".to_string() };
}

/// A valid request for identity digit `digit` with one added file.
fn request(digit: char, message: &str) -> Request {
    return Request {
        id: identity(digit),
        branch: b"main".to_vec(),
        base: "b".repeat(40),
        message: message.as_bytes().to_vec(),
        operation: Operation::Commit,
        changes: vec![Change {
            path: b"f.txt".to_vec(),
            mode: "100644".to_string(),
            sha256: sha256_hex(b"x"),
            old_mode: "000000".to_string(),
            old_sha256: "-".to_string(),
        }],
    };
}

/// The directories synced in `events`, in order.
fn directories(events: &[String]) -> Vec<PathBuf> {
    return events.iter().filter_map(|event| { return event.strip_prefix("dir:").map(PathBuf::from); }).collect();
}

/// Every created directory is reported, and the chain through the anchor is synced on every call, the second included.
#[test]
fn durable_directory_creation_syncs_the_chain_through_the_anchor_every_time() {
    let root: PathBuf = scratch("seam-chain");
    start_events();
    let created: Vec<PathBuf> = create_dir_all_durable(&root, &root.join("a/b/c")).expect("create");
    let first: Vec<PathBuf> = directories(&take_events());
    assert_eq!(created, vec![root.join("a"), root.join("a/b"), root.join("a/b/c")]);
    assert_eq!(first, vec![root.join("a/b/c"), root.join("a/b"), root.join("a"), root.clone()]);
    start_events();
    assert!(create_dir_all_durable(&root, &root.join("a/b/c")).expect("again").is_empty());
    assert_eq!(directories(&take_events()), first, "a chain that merely exists is synced again");
}

/// An ancestor that only exists is synced by the next successful bind, through the anchor, which is the last directory synced.
#[test]
fn a_bind_syncs_an_ancestor_that_only_exists() {
    let root: PathBuf = scratch("seam-ancestor");
    std::fs::create_dir_all(root.join("ids")).expect("an ancestor made by a creator that died before syncing the root");
    start_events();
    assert_eq!(bind(&root, &identity('d'), &sha256_hex(b"x")).expect("bind"), Bound::Created);
    let synced: Vec<PathBuf> = directories(&take_events());
    assert!(synced.contains(&root.join("ids")), "{:?}", synced);
    assert_eq!(synced.last(), Some(&root));
}

/// A resend that finds the name syncs the chain itself, in order, through the anchor.
#[test]
fn a_resend_that_finds_the_name_syncs_the_directory_chain_itself() {
    let root: PathBuf = scratch("seam-resend");
    let id: String = identity('1');
    let hash: String = sha256_hex(b"x");
    assert_eq!(bind(&root, &id, &hash).expect("bind"), Bound::Created);
    let fan_out: PathBuf = binding_path(&root, &id).expect("path").parent().expect("parent").to_path_buf();
    start_events();
    assert_eq!(bind(&root, &id, &hash).expect("resend"), Bound::SameContent);
    let synced: Vec<PathBuf> = directories(&take_events());
    assert_eq!(synced.first(), Some(&fan_out));
    assert_eq!(synced.last(), Some(&root));
    let inbox: PathBuf = root.join("inbox");
    let one: Request = request('2', "m");
    publish(&root, &inbox, &one).expect("publish");
    start_events();
    assert_eq!(publish(&root, &inbox, &one).expect("resend"), Published::AlreadyPresent);
    let synced_record: Vec<PathBuf> = directories(&take_events());
    assert_eq!(synced_record[synced_record.len() - 2..], [inbox.clone(), root.clone()]);
    let ledger: PathBuf = root.join("ledger");
    append(&root, &ledger, &entry('3')).expect("append");
    start_events();
    append(&root, &ledger, &entry('4')).expect("append");
    assert_eq!(directories(&take_events()).last(), Some(&root));
}

/// Review 3: the saved tail's file sync, then its directory sync, then the truncation, in that order and nothing destructive earlier.
#[test]
fn a_repair_syncs_the_saved_tail_and_its_directory_before_it_truncates() {
    let directory: PathBuf = scratch("seam-repair");
    let ledger: PathBuf = directory.join("ledger");
    append(&directory, &ledger, &entry('1')).expect("append");
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    bytes.extend_from_slice(b"abc");
    std::fs::write(&ledger, &bytes).expect("torn");
    start_events();
    repair_tail(&ledger).expect("repair");
    let events: Vec<String> = take_events();
    let torn: String = format!("file-sync:{}", directory.join("ledger.torn").display());
    let parent: String = format!("dir:{}", directory.display());
    let truncate: String = format!("truncate:{}", ledger.display());
    assert_eq!(events[..3], [torn, parent, truncate]);
}

/// Review 3: with a fault point at the truncation, the saved bytes are already on disk when the writer stops there.
#[test]
fn a_writer_stopped_before_the_truncation_has_already_saved_the_tail() {
    static AT_TRUNCATION: std::sync::Barrier = std::sync::Barrier::new(2);
    static RELEASED: std::sync::Barrier = std::sync::Barrier::new(2);
    fn pause(point: &str) {
        if point == "ledger:before-truncate" {
            AT_TRUNCATION.wait();
            RELEASED.wait();
        }
    }
    let directory: PathBuf = scratch("seam-truncate");
    let ledger: PathBuf = directory.join("ledger");
    append(&directory, &ledger, &entry('1')).expect("append");
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    bytes.extend_from_slice(b"abc");
    std::fs::write(&ledger, &bytes).expect("torn");
    let ledger_clone: PathBuf = ledger.clone();
    let writer = std::thread::spawn(move || {
        set_fault_point(Some(pause));
        return repair_tail(&ledger_clone).expect("repair");
    });
    AT_TRUNCATION.wait();
    assert_eq!(std::fs::read(directory.join("ledger.torn")).expect("saved"), b"abc".to_vec());
    assert_eq!(std::fs::read(&ledger).expect("untouched"), bytes);
    RELEASED.wait();
    assert_eq!(writer.join().expect("join").cut, 3);
}

/// Review 3: a bare relative ledger name repairs and appends (the empty parent is the current directory), and the chain is made
/// durable before the repair.
#[test]
fn a_bare_relative_ledger_name_repairs_and_appends() {
    let directory: PathBuf = scratch("seam-relative");
    let previous: PathBuf = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(&directory).expect("chdir");
    let outcome = (|| -> Result<(), String> {
        append(Path::new("."), Path::new("ledger"), &entry('1')).map_err(|e| { return format!("{:?}", e); })?;
        let mut bytes: Vec<u8> = std::fs::read("ledger").map_err(|e| { return e.to_string(); })?;
        bytes.extend_from_slice(b"abc");
        std::fs::write("ledger", &bytes).map_err(|e| { return e.to_string(); })?;
        append(Path::new("."), Path::new("ledger"), &entry('2')).map_err(|e| { return format!("{:?}", e); })?;
        return Ok(());
    })();
    std::env::set_current_dir(previous).expect("chdir back");
    outcome.expect("relative ledger");
    assert_eq!(std::fs::read(directory.join("ledger.torn")).expect("saved"), b"abc".to_vec());
}

/// Review 3: the anchor is never created silently, a path outside it or with `..` is refused before anything is created, and the
/// initialized-root constructor makes the root and then its known-durable parent durable.
#[test]
fn the_root_is_initialized_through_a_known_durable_parent_and_paths_are_contained() {
    let parent: PathBuf = scratch("seam-root");
    let root: PathBuf = parent.join("state");
    assert!(create_dir_all_durable(&root, &root.join("a")).is_err(), "a missing anchor is not created");
    assert!(!root.exists());
    start_events();
    let made: PathBuf = init_root(&parent, &root).expect("init");
    assert_eq!(directories(&take_events()), vec![made.clone(), parent.clone()]);
    assert_eq!(init_root(&parent, &root).expect("again"), made, "an interrupted creator's root is accepted and synced again");
    assert!(init_root(&parent, &parent.join("x/y")).is_err());
    assert!(create_dir_all_durable(&root, &parent.join("outside")).is_err());
    assert!(!parent.join("outside").exists(), "refused before creation");
    assert!(create_dir_all_durable(&root, &root.join("a/../../escape")).is_err());
    assert!(!parent.join("escape").exists());
}

/// A bind paused before the link loses to a writer that finished first, and a publisher paused likewise finds the other record.
#[test]
fn writers_paused_before_the_link_find_the_other_writers_result() {
    static BIND_ARRIVED: std::sync::Barrier = std::sync::Barrier::new(2);
    static BIND_RELEASE: std::sync::Barrier = std::sync::Barrier::new(2);
    static PUBLISH_ARRIVED: std::sync::Barrier = std::sync::Barrier::new(2);
    static PUBLISH_RELEASE: std::sync::Barrier = std::sync::Barrier::new(2);
    fn pause(point: &str) {
        if point == "bind:before-link" {
            BIND_ARRIVED.wait();
            BIND_RELEASE.wait();
        }
        if point == "publish:before-link" {
            PUBLISH_ARRIVED.wait();
            PUBLISH_RELEASE.wait();
        }
    }
    let root: PathBuf = scratch("seam-pause");
    let id: String = identity('7');
    let (a_root, a_id) = (root.clone(), id.clone());
    let paused = std::thread::spawn(move || {
        set_fault_point(Some(pause));
        return bind(&a_root, &a_id, &sha256_hex(b"first")).expect("bind");
    });
    BIND_ARRIVED.wait();
    assert_eq!(bind(&root, &id, &sha256_hex(b"second")).expect("bind"), Bound::Created);
    BIND_RELEASE.wait();
    assert_eq!(paused.join().expect("join"), Bound::DifferentContent(sha256_hex(b"second")));
    let inbox: PathBuf = root.join("inbox");
    let (b_root, b_inbox) = (root.clone(), inbox.clone());
    let publisher = std::thread::spawn(move || {
        set_fault_point(Some(pause));
        return publish(&b_root, &b_inbox, &request('c', "first"));
    });
    PUBLISH_ARRIVED.wait();
    assert_eq!(publish(&root, &inbox, &request('c', "second")).expect("publish"), Published::Created);
    PUBLISH_RELEASE.wait();
    assert!(matches!(publisher.join().expect("join"), Err(RequestError::Conflict)));
    assert_eq!(decode(&std::fs::read(inbox.join(identity('c'))).expect("read")).expect("decode"), request('c', "second"));
}
