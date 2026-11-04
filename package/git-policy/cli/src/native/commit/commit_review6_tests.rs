//! What:
//!  Controls for the reads and writes of the ledger, the spool and the inbox: a file of the lane is read or appended to only when the name
//!  leads to a regular file, without following a link and without waiting on a FIFO, and a read stops at the length the file had when it
//!  was opened.
//! Why:
//!  The owner's rule: "nothing should ever wait forever or indefinitely". The state log's reads were made so in the fifth review of step 3;
//!  that a review did not name the other components does not make them safe: a FIFO named like a ledger, a spool file or a request blocks
//!  the open for ever, and a link at the ledger's name makes an append write to a file outside the lane.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a FIFO named like the ledger blocks no ledger operation', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_inbox::{Accepted, Departure, Disposition, InboxRepairer, accept, init_lane_directories, with_identity};
use git_policy_cli::commit_integrity::sha256_hex;
use git_policy_cli::commit_ledger::{LedgerEntry, LedgerOwner, lookup};
use git_policy_cli::commit_request::{Change, Operation, Request};
use git_policy_cli::commit_spool::{verify_spool, write_spool};
use git_policy_cli::commit_state_log::StateWriter;
use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};

/// The lane instance of the tests.
const INSTANCE: &str = "0123456789abcdef0123456789abcdef";

/// Makes a FIFO at `path`.
fn make_fifo(path: &Path) {
    let name: CString = CString::new(path.as_os_str().as_encoded_bytes()).expect("a path");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode and touches no other memory.
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0, "mkfifo");
}

/// A one-path request with its identity, and the bytes of its file.
fn one_path_request() -> (Request, Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let request: Request = with_identity(
        INSTANCE,
        Request {
            id: "0".repeat(32),
            branch: b"main".to_vec(),
            base: "b".repeat(40),
            message: b"m".to_vec(),
            operation: Operation::Commit,
            nonce: vec![1; 16],
            changes: vec![Change { path: b"f.txt".to_vec(), mode: "100644".to_string(), sha256: sha256_hex(b"new bytes"), old_mode: "000000".to_string(), old_sha256: "-".to_string() }],
        },
    )
    .expect("identity");
    return (request, vec![b"new bytes".to_vec()], vec![Vec::new()]);
}

/// A ledger entry.
fn entry(id: &str) -> LedgerEntry {
    return LedgerEntry { id: id.to_string(), outcome: "conflicted".to_string(), commit: "-".to_string(), time: "2026-01-01T00:00:00+00:00".to_string() };
}

//region the ledger

/// A FIFO named like the ledger: a lookup and the owner's first validation end with an error at once, and an owner that already holds a
/// regular ledger whose name is then taken by a FIFO is refused at its next append (each in a single-threaded copy with the harness' limit).
#[test]
fn a_fifo_named_like_the_ledger_blocks_no_ledger_operation() {
    let lane: PathBuf = scratch("rev6-ledger-fifo");
    let ledger: PathBuf = lane.join("ledger");
    make_fifo(&ledger);
    let looked_lane: PathBuf = ledger.clone();
    let looked: Option<i32> = in_single_threaded_process(move || { return if lookup(&looked_lane, &"a".repeat(32)).is_err() { 0 } else { 1 }; });
    assert_eq!(looked, Some(0), "a lookup waited on the FIFO (None) or read it as a ledger (1)");
    let acquired_ledger: PathBuf = ledger.clone();
    let acquired: Option<i32> = in_single_threaded_process(move || { return if LedgerOwner::acquire(&acquired_ledger).is_err() { 0 } else { 1 }; });
    assert_eq!(acquired, Some(0), "the owner's validation waited on the FIFO (None) or accepted it (1)");
    fs::remove_file(&ledger).expect("remove the FIFO");
    fs::write(&ledger, b"").expect("a regular ledger");
    let swapped_lane: PathBuf = lane.clone();
    let swapped: Option<i32> = in_single_threaded_process(move || {
        let ledger: PathBuf = swapped_lane.join("ledger");
        let Ok(mut owner) = LedgerOwner::acquire(&ledger) else { return 2 };
        fs::remove_file(&ledger).expect("remove the ledger");
        let name: CString = CString::new(ledger.as_os_str().as_encoded_bytes()).expect("a path");
        // SAFETY: as in `make_fifo`.
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0, "mkfifo");
        return if owner.append(&swapped_lane, &ledger, &entry(&"b".repeat(32))).is_err() { 0 } else { 1 };
    });
    assert_eq!(swapped, Some(0), "an append waited on a FIFO that took the ledger's name (None), was accepted (1) or the owner was not acquired (2)");
}

/// A link at the ledger's name: an append writes to the file it leads to, outside the lane. The owner refuses the ledger when it is a link,
/// and an owner that holds a regular ledger whose name becomes a link is refused at its next append; the file outside stays as it was.
#[test]
fn a_ledger_that_is_a_link_is_never_appended_to() {
    let lane: PathBuf = scratch("rev6-ledger-link");
    let outside: PathBuf = scratch("rev6-ledger-outside").join("valuable");
    fs::write(&outside, b"keep these bytes\n").expect("a file outside");
    let ledger: PathBuf = lane.join("ledger");
    std::os::unix::fs::symlink(&outside, &ledger).expect("a link at the ledger's name");
    let appended = LedgerOwner::acquire(&ledger).and_then(|mut owner| { return owner.append(&lane, &ledger, &entry(&"a".repeat(32))); });
    assert!(appended.is_err(), "an append went through a link");
    assert_eq!(fs::read(&outside).expect("the file"), b"keep these bytes\n", "the file outside the lane was appended to");
    fs::remove_file(&ledger).expect("remove the link");
    fs::write(&ledger, b"").expect("a regular ledger");
    let mut owner: LedgerOwner = LedgerOwner::acquire(&ledger).expect("the owner");
    fs::remove_file(&ledger).expect("remove the ledger");
    std::os::unix::fs::symlink(&outside, &ledger).expect("a link in its place");
    assert!(owner.append(&lane, &ledger, &entry(&"b".repeat(32))).is_err(), "an append went through a link that took the ledger's name");
    assert_eq!(fs::read(&outside).expect("the file"), b"keep these bytes\n", "the file outside the lane was appended to after the swap");
}

//endregion

//region the spool

/// A spool whose manifest is a FIFO is refused at once; a spool file that is a link to a file with the same bytes is not followed; and a
/// resend whose target is a FIFO does not wait.
#[test]
fn a_fifo_or_a_link_in_a_spool_is_refused_and_never_waited_for() {
    let root: PathBuf = scratch("rev6-spool");
    let id: String = "c".repeat(32);
    write_spool(&root, &id, &[b"new bytes".to_vec()], &[Vec::new()]).expect("a spool");
    verify_spool(&root, &id).expect("it verifies");
    let directory: PathBuf = git_policy_cli::commit_spool::spool_directory(&root, &id).expect("the directory");
    let outside: PathBuf = scratch("rev6-spool-outside").join("copy");
    fs::copy(directory.join("0"), &outside).expect("a copy with the same bytes");
    fs::remove_file(directory.join("0")).expect("remove the file");
    std::os::unix::fs::symlink(&outside, directory.join("0")).expect("a link in its place");
    assert!(verify_spool(&root, &id).is_err(), "a spool file that is a link was followed");
    fs::remove_file(directory.join("0")).expect("remove the link");
    let manifest: Vec<u8> = fs::read(directory.join("manifest")).expect("the manifest");
    fs::remove_file(directory.join("manifest")).expect("remove the manifest");
    make_fifo(&directory.join("manifest"));
    let (verify_root, verify_id) = (root.clone(), id.clone());
    let verified: Option<i32> = in_single_threaded_process(move || { return if verify_spool(&verify_root, &verify_id).is_err() { 0 } else { 1 }; });
    assert_eq!(verified, Some(0), "the verification waited on a FIFO manifest (None) or accepted it (1)");
    fs::remove_file(directory.join("manifest")).expect("remove the FIFO");
    fs::write(directory.join("manifest"), manifest).expect("the manifest again");
    fs::write(directory.join("0"), b"new bytes").expect("the file again");
    make_fifo(&directory.join("1"));
    let (resend_root, resend_id) = (root.clone(), id.clone());
    let resent: Option<i32> = in_single_threaded_process(move || {
        // A resend of a spool whose second file's name is held by a FIFO: the target is read first and must not wait.
        return if write_spool(&resend_root, &resend_id, &[b"new bytes".to_vec(), b"more".to_vec()], &[Vec::new(), Vec::new()]).is_err() { 0 } else { 1 };
    });
    assert_eq!(resent, Some(0), "a resend waited on a FIFO at a spool file's name (None) or took it for the file (1)");
}

//endregion

//region the inbox

/// A request name held by a FIFO or a link: acceptance does not wait and does not follow, a repairer's inspection does not wait, and a
/// departure does not wait on the order file or on a request file that is a FIFO.
#[test]
fn a_fifo_or_a_link_at_a_request_or_an_order_name_blocks_no_inbox_operation() {
    let root: PathBuf = scratch("rev6-inbox");
    let lane: PathBuf = init_lane_directories(&root, &root.join("lane")).expect("init");
    let (request, new_files, old_files) = one_path_request();
    let target: PathBuf = lane.join("inbox").join(&request.id);
    make_fifo(&target);
    let (accept_lane, accept_request, accept_new, accept_old) = (lane.clone(), request.clone(), new_files.clone(), old_files.clone());
    let accepted: Option<i32> = in_single_threaded_process(move || { return if accept(&accept_lane.join("inbox"), INSTANCE, &accept_request, &accept_new, &accept_old).is_err() { 0 } else { 1 }; });
    assert_eq!(accepted, Some(0), "an acceptance waited on a FIFO at the request's name (None) or took it (1)");
    let repair_lane: PathBuf = lane.clone();
    let repair_id: String = request.id.clone();
    let repaired: Option<i32> = in_single_threaded_process(move || {
        let Ok(mut repairer) = InboxRepairer::acquire(&repair_lane.join("inbox")) else { return 2 };
        return if repairer.quarantine(&repair_id).is_err() { 0 } else { 1 };
    });
    assert_eq!(repaired, Some(0), "a repairer's inspection waited on a FIFO (None), moved it (1) or could not start (2)");
    fs::remove_file(&target).expect("remove the FIFO");
    // A link at the name to a file with the very bytes of the request: the request is not accepted by following it.
    accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files).expect("accept");
    let outside: PathBuf = scratch("rev6-inbox-outside").join("copy");
    fs::copy(&target, &outside).expect("a copy");
    fs::remove_file(&target).expect("remove the request");
    std::os::unix::fs::symlink(&outside, &target).expect("a link in its place");
    let followed = accept(&lane.join("inbox"), INSTANCE, &request, &new_files, &old_files);
    assert!(!matches!(followed, Ok(Accepted::AlreadyPresent)), "an acceptance followed a link and found its request present");
    fs::remove_file(&target).expect("remove the link");
    // The order file and the request file of a departure.
    StateWriter::acquire(&lane.join("state")).expect("writer").append(1, b"adopted").expect("state");
    let order: PathBuf = lane.join("order");
    make_fifo(&order);
    let (order_lane, order_id) = (lane.clone(), request.id.clone());
    let ordered: Option<i32> = in_single_threaded_process(move || {
        let ids: Vec<String> = vec![order_id];
        let (inbox, state): (PathBuf, PathBuf) = (order_lane.join("inbox"), order_lane.join("state"));
        let departure = Departure::new(Disposition::Handover, &inbox, &ids, &state).with_order(&order);
        return if departure.run().is_err() { 0 } else { 1 };
    });
    assert_eq!(ordered, Some(0), "a departure waited on a FIFO order file (None) or accepted it (1)");
    make_fifo(&target);
    let (force_lane, force_id) = (lane.clone(), request.id.clone());
    let forced: Option<i32> = in_single_threaded_process(move || {
        let ids: Vec<String> = vec![force_id];
        let (inbox, state): (PathBuf, PathBuf) = (force_lane.join("inbox"), force_lane.join("state"));
        return if Departure::new(Disposition::Saved, &inbox, &ids, &state).run().is_err() { 0 } else { 1 };
    });
    assert_eq!(forced, Some(0), "a departure waited on a FIFO that took a request file's name (None) or accepted it (1)");
}

//endregion
