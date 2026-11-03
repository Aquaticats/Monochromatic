//! What:
//!  Controls of the immutable commit components (ledger, identity binding, request record, spool) on disposable
//!  directories: serialization, validation, integrity, the id-to-content binding with no overwrite, torn-tail repair
//!  at every byte, and a real two-thread race on the file system's no-replace primitives.
//! Why:
//!  Step 1 builds these components with no reclamation. A crash is modeled by the on-disk states it can leave (a
//!  prefix of an append, a temporary file, a missing manifest), enumerated at every cut point.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('torn tail at every byte is repaired', ...);
//! ```

use crate::model_fixture::scratch;
use git_policy_cli::commit_identity::{Bound, IdentityError, bind, binding_path, create_dir_all_durable, start_sync_trace, take_sync_trace};
use git_policy_cli::commit_integrity::{crc32, sha256_hex};
use git_policy_cli::commit_ledger::{LedgerEntry, LedgerError, append, encode_line, lane_ledger_path, lookup, parse_line, repair_tail};
use git_policy_cli::commit_request::{Change, Operation, Published, Request, RequestError, content_hash, decode, encode, publish};
use git_policy_cli::commit_spool::{SpoolError, read_closure, verify_against_request, verify_spool, write_spool};
use std::path::{Path, PathBuf};

/// A 32-character identity made of one repeated digit.
fn identity(digit: char) -> String {
    return digit.to_string().repeat(32);
}

/// A ledger entry for identity digit `digit`.
fn entry(digit: char, outcome: &str) -> LedgerEntry {
    return LedgerEntry { id: identity(digit), outcome: outcome.to_string(), commit: "a".repeat(40), time: "2026-11-03T12:00:00-05:00".to_string() };
}

/// A valid request for identity digit `digit` with one set and one deletion.
fn request(digit: char, message: &str) -> Request {
    let content: Vec<u8> = b"new bytes".to_vec();
    return Request {
        id: identity(digit),
        branch: b"main".to_vec(),
        base: "b".repeat(40),
        message: message.as_bytes().to_vec(),
        operation: Operation::Commit,
        changes: vec![
            Change {
                path: b"a/gone.txt".to_vec(),
                mode: "000000".to_string(),
                sha256: "-".to_string(),
                old_mode: "100644".to_string(),
                old_sha256: sha256_hex(b"old gone"),
            },
            Change {
                path: b"b/new.txt".to_vec(),
                mode: "100644".to_string(),
                sha256: sha256_hex(&content),
                old_mode: "000000".to_string(),
                old_sha256: "-".to_string(),
            },
        ],
    };
}

/// The CRC-32 of the standard check string is the published value.
#[test]
fn crc32_matches_the_published_check_value() {
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
}

/// A line encodes, parses back, and refuses fields it cannot hold.
#[test]
fn ledger_lines_round_trip_and_refuse_bad_fields() {
    let good: LedgerEntry = entry('1', "published");
    let line: Vec<u8> = encode_line(&good).expect("encode");
    assert_eq!(parse_line(&line[..line.len() - 1]), Some(good.clone()));
    for bad in [
        LedgerEntry { id: "short".to_string(), ..good.clone() },
        LedgerEntry { outcome: "has space".to_string(), ..good.clone() },
        LedgerEntry { outcome: "UPPER".to_string(), ..good.clone() },
        LedgerEntry { commit: "xyz".to_string(), ..good.clone() },
        LedgerEntry { time: "two words".to_string(), ..good.clone() },
        LedgerEntry { time: "".to_string(), ..good.clone() },
    ] {
        assert!(matches!(encode_line(&bad), Err(LedgerError::InvalidField(_))));
    }
}

/// The first line for an identity is the answer, however many lines follow.
#[test]
fn ledger_lookup_answers_from_the_first_line() {
    let ledger: PathBuf = scratch("ledger-first").join("ledger");
    append(ledger.parent().expect("parent"), &ledger, &entry('1', "published")).expect("append");
    append(ledger.parent().expect("parent"), &ledger, &entry('2', "saved")).expect("append");
    append(ledger.parent().expect("parent"), &ledger, &entry('1', "landed")).expect("append");
    assert_eq!(lookup(&ledger, &identity('1')).expect("lookup").expect("entry").outcome, "published");
    assert_eq!(lookup(&ledger, &identity('3')).expect("lookup"), None);
}

/// A crash can cut the last append at any byte: every cut point is repaired, the cut bytes are kept beside the ledger,
/// and the next append lands on a clean line.
#[test]
fn ledger_torn_tail_is_repaired_at_every_byte() {
    let first: Vec<u8> = encode_line(&entry('1', "published")).expect("encode");
    let second: Vec<u8> = encode_line(&entry('2', "saved")).expect("encode");
    for cut in 0..second.len() {
        let directory: PathBuf = scratch("ledger-torn");
        let ledger: PathBuf = directory.join("ledger");
        let mut bytes: Vec<u8> = first.clone();
        bytes.extend_from_slice(&second[..cut]);
        std::fs::write(&ledger, &bytes).expect("write torn ledger");
        let repair = repair_tail(&ledger).expect("repair");
        assert_eq!(repair.kept as usize, first.len(), "cut {}", cut);
        assert_eq!(repair.cut as usize, cut, "cut {}", cut);
        assert_eq!(std::fs::read(&ledger).expect("read"), first);
        if cut > 0 {
            assert_eq!(std::fs::read(directory.join("ledger.torn")).expect("saved tail"), second[..cut].to_vec());
        }
        append(ledger.parent().expect("parent"), &ledger, &entry('3', "landed")).expect("append after repair");
        assert!(lookup(&ledger, &identity('1')).expect("lookup").is_some());
        assert!(lookup(&ledger, &identity('3')).expect("lookup").is_some());
    }
}

/// A damaged line that is not the last is reported and nothing is truncated: the lines behind it are good data.
#[test]
fn ledger_damaged_middle_is_reported_and_kept() {
    let ledger: PathBuf = scratch("ledger-middle").join("ledger");
    for digit in ['1', '2', '3'] {
        append(ledger.parent().expect("parent"), &ledger, &entry(digit, "published")).expect("append");
    }
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    let line: usize = encode_line(&entry('1', "published")).expect("encode").len();
    bytes[line + 12] ^= 1;
    std::fs::write(&ledger, &bytes).expect("damage");
    assert!(matches!(repair_tail(&ledger), Err(LedgerError::DamagedLine(1))));
    assert!(matches!(lookup(&ledger, &identity('3')), Err(LedgerError::DamagedLine(1))));
    assert_eq!(std::fs::read(&ledger).expect("read"), bytes);
}

/// The binding is created whole, answers a resend, refuses another content, and never changes.
#[test]
fn identity_binding_creates_once_and_never_changes() {
    let root: PathBuf = scratch("identity-once");
    let id: String = identity('4');
    let one: String = sha256_hex(b"one");
    let two: String = sha256_hex(b"two");
    assert_eq!(bind(&root, &id, &one).expect("bind"), Bound::Created);
    assert_eq!(bind(&root, &id, &one).expect("bind"), Bound::SameContent);
    assert_eq!(bind(&root, &id, &two).expect("bind"), Bound::DifferentContent(one.clone()));
    let path: PathBuf = binding_path(&root, &id).expect("path");
    assert_eq!(std::fs::read_to_string(&path).expect("read"), format!("{}\n", one));
    assert!(matches!(bind(&root, "../etc", &one), Err(IdentityError::InvalidIdentity)));
    assert!(matches!(bind(&root, &id, "short"), Err(IdentityError::InvalidHash)));
    let leftovers: Vec<_> = std::fs::read_dir(path.parent().expect("parent")).expect("dir").collect();
    assert_eq!(leftovers.len(), 1, "no temporary file stays");
}

/// An empty or partial binding (as an `O_EXCL` creator that died would leave) is reported and left alone.
#[test]
fn identity_binding_refuses_a_partial_file() {
    let root: PathBuf = scratch("identity-partial");
    let id: String = identity('5');
    let path: PathBuf = binding_path(&root, &id).expect("path");
    std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
    for partial in [&b""[..], &b"abc"[..], &b"zz"[..]] {
        std::fs::write(&path, partial).expect("partial");
        assert!(matches!(bind(&root, &id, &sha256_hex(b"x")), Err(IdentityError::UnreadableBinding(_))));
        assert_eq!(std::fs::read(&path).expect("read"), partial.to_vec());
    }
}

/// Many threads race to bind one identity to different contents: exactly one creates it and every other is refused,
/// on the real file system's link primitive.
#[test]
fn identity_binding_has_one_winner_under_a_real_race() {
    let root: PathBuf = scratch("identity-race");
    let id: String = identity('6');
    let mut workers = Vec::new();
    let start: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(8));
    for number in 0..8 {
        let worker_root: PathBuf = root.clone();
        let worker_id: String = id.clone();
        let gate: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::clone(&start);
        workers.push(std::thread::spawn(move || {
            gate.wait();
            return bind(&worker_root, &worker_id, &sha256_hex(format!("content {}", number).as_bytes())).expect("bind");
        }));
    }
    let results: Vec<Bound> = workers.into_iter().map(|worker| { return worker.join().expect("join"); }).collect();
    assert_eq!(results.iter().filter(|result| { return **result == Bound::Created; }).count(), 1);
    assert_eq!(results.iter().filter(|result| { return matches!(result, Bound::DifferentContent(_)); }).count(), 7);
}

/// A request encodes, decodes to itself and has a stable content hash that equals its end line.
#[test]
fn request_round_trips_with_a_stable_hash() {
    let original: Request = request('7', "fix the parser");
    let bytes: Vec<u8> = encode(&original).expect("encode");
    assert_eq!(decode(&bytes).expect("decode"), original);
    let hash: String = content_hash(&original).expect("hash");
    assert!(String::from_utf8(bytes).expect("utf-8").ends_with(&format!("end {}\n", hash)));
}

/// Hostile text in the free fields cannot break the grammar (delimiters, newlines, quotes, an `end ` line, controls).
#[test]
fn request_fields_survive_hostile_text() {
    for message in ["end deadbeef\nchange 100644 x y", "line one\nline two\n", "quote \" and ' and \\ and \t tab", "unicode \u{1F600} \u{00E9}"] {
        let mut original: Request = request('8', message);
        original.branch = b"feature/with space\nand \xff newline".to_vec();
        let bytes: Vec<u8> = encode(&original).expect("encode");
        assert_eq!(decode(&bytes).expect("decode"), original);
    }
}

/// Flipping any bit of any byte, or cutting the record at any length, is detected.
#[test]
fn request_damage_is_detected_at_every_byte() {
    let bytes: Vec<u8> = encode(&request('9', "damage me")).expect("encode");
    for at in 0..bytes.len() {
        let mut flipped: Vec<u8> = bytes.clone();
        flipped[at] ^= 1;
        assert!(decode(&flipped).is_err(), "flip at {}", at);
        assert!(decode(&bytes[..at]).is_err(), "cut at {}", at);
    }
}

/// Fields that could escape or confuse the repository are refused before anything is written.
#[test]
fn request_validation_refuses_bad_fields() {
    let mut cases: Vec<Request> = Vec::new();
    for path in ["../escape", "/absolute", "a//b", "a/./b", ".git/config", "a/.git/x", "", "trailing/", "nul\0byte"] {
        let mut bad: Request = request('a', "m");
        bad.changes[1].path = path.as_bytes().to_vec();
        cases.push(bad);
    }
    let mut unordered: Request = request('a', "m");
    unordered.changes.reverse();
    cases.push(unordered);
    let mut mode: Request = request('a', "m");
    mode.changes[1].mode = "100664".to_string();
    cases.push(mode);
    let mut hash: Request = request('a', "m");
    hash.changes[1].sha256 = "UPPER".to_string();
    cases.push(hash);
    let mut deletion: Request = request('a', "m");
    deletion.changes[0].sha256 = "a".repeat(64);
    cases.push(deletion);
    cases.push(Request { id: "short".to_string(), ..request('a', "m") });
    cases.push(Request { message: b"  ".to_vec(), ..request('a', "m") });
    cases.push(Request { base: "xyz".to_string(), ..request('a', "m") });
    for bad in cases {
        assert!(matches!(encode(&bad), Err(RequestError::Invalid(_))), "{:?}", bad);
    }
}

/// A record of another version is not read, and the file stays where it is, byte for byte, for a release that reads it.
#[test]
fn request_of_an_unknown_version_is_left_alone() {
    let bytes: Vec<u8> = encode(&request('b', "m")).expect("encode");
    let text: String = String::from_utf8(bytes).expect("utf-8").replacen("cli-git-request 2", "cli-git-request 3", 1);
    let inbox: PathBuf = scratch("request-version").join("inbox");
    std::fs::create_dir_all(&inbox).expect("inbox");
    let file: PathBuf = inbox.join(identity('b'));
    std::fs::write(&file, text.as_bytes()).expect("write");
    assert!(matches!(decode(&std::fs::read(&file).expect("read")), Err(RequestError::Damaged("version"))));
    assert_eq!(std::fs::read(&file).expect("read"), text.as_bytes());
}

/// Publication links the record in without replacing: the same bytes are a resend, other content is a conflict and the
/// acknowledged record is unchanged.
#[test]
fn request_publication_never_replaces_an_acknowledged_record() {
    let inbox: PathBuf = scratch("request-publish").join("inbox");
    let first: Request = request('c', "first");
    let second: Request = request('c', "second");
    assert_eq!(publish(inbox.parent().expect("parent"), &inbox, &first).expect("publish"), Published::Created);
    assert_eq!(publish(inbox.parent().expect("parent"), &inbox, &first).expect("publish"), Published::AlreadyPresent);
    assert!(matches!(publish(inbox.parent().expect("parent"), &inbox, &second), Err(RequestError::Conflict)));
    let kept: Vec<u8> = std::fs::read(inbox.join(identity('c'))).expect("read");
    assert_eq!(decode(&kept).expect("decode"), first);
    assert_eq!(std::fs::read_dir(&inbox).expect("dir").count(), 1, "no temporary file stays");
}

/// The review's F7 schedule on the real file system: two threads publish different content under one identity; one
/// wins, the other is refused, and the inbox holds exactly the winner.
#[test]
fn request_publication_has_one_winner_under_a_real_race() {
    let inbox: PathBuf = scratch("request-race").join("inbox");
    let mut workers = Vec::new();
    let start: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(6));
    for number in 0..6 {
        let worker_inbox: PathBuf = inbox.clone();
        let gate: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::clone(&start);
        workers.push(std::thread::spawn(move || { gate.wait(); return publish(worker_inbox.parent().expect("parent"), &worker_inbox, &request('d', &format!("message {}", number))); }));
    }
    let results: Vec<Result<Published, RequestError>> = workers.into_iter().map(|worker| { return worker.join().expect("join"); }).collect();
    assert_eq!(results.iter().filter(|result| { return matches!(result, Ok(Published::Created)); }).count(), 1);
    assert_eq!(results.iter().filter(|result| { return matches!(result, Err(RequestError::Conflict)); }).count(), 5);
    assert_eq!(std::fs::read_dir(&inbox).expect("dir").count(), 1);
}

/// The spool writes complete files and a manifest last, verifies against the manifest and the request, and detects
/// damage.
#[test]
fn spool_verifies_against_its_manifest_and_its_request() {
    let root: PathBuf = scratch("spool-verify");
    let original: Request = request('e', "m");
    let files: Vec<Vec<u8>> = vec![Vec::new(), b"new bytes".to_vec()];
    let old_files: Vec<Vec<u8>> = vec![b"old gone".to_vec(), Vec::new()];
    write_spool(&root, &original.id, &files, &old_files).expect("write");
    assert_eq!(verify_spool(&root, &original.id).expect("verify").len(), 4);
    verify_against_request(&root, &original).expect("matches the request");
    write_spool(&root, &original.id, &files, &old_files).expect("a repeated write is a no-op");
    assert!(matches!(write_spool(&root, &original.id, &[Vec::new(), b"other".to_vec()], &old_files), Err(SpoolError::Conflict)));
    let victim: PathBuf = root.join("spool").join(&original.id).join("1");
    std::fs::write(&victim, b"damaged!!").expect("damage");
    assert!(matches!(verify_spool(&root, &original.id), Err(SpoolError::Mismatch(_))));
}

/// A crash before the manifest leaves an incomplete spool; the same client's resend completes it, and nothing it wrote
/// before the crash is overwritten.
#[test]
fn spool_crash_before_the_manifest_is_completed_by_a_resend() {
    let root: PathBuf = scratch("spool-crash");
    let original: Request = request('f', "m");
    let files: Vec<Vec<u8>> = vec![Vec::new(), b"new bytes".to_vec()];
    let old_files: Vec<Vec<u8>> = vec![b"old gone".to_vec(), Vec::new()];
    write_spool(&root, &original.id, &files, &old_files).expect("write");
    let directory: PathBuf = root.join("spool").join(&original.id);
    std::fs::remove_file(directory.join("manifest")).expect("simulate the crash");
    std::fs::remove_file(directory.join("1")).expect("simulate the crash");
    std::fs::write(directory.join(".tmp-0123456789abcdef"), b"half").expect("leftover temporary file");
    assert!(matches!(verify_spool(&root, &original.id), Err(SpoolError::Incomplete)));
    write_spool(&root, &original.id, &files, &old_files).expect("resend completes");
    verify_against_request(&root, &original).expect("complete");
}

/// A helper so the directory type is used for its documented purpose in this test target.
#[test]
fn scratch_directories_are_new_and_inside_the_build_tree() {
    let first: PathBuf = scratch("distinct");
    let second: PathBuf = scratch("distinct");
    assert_ne!(first, second);
    assert!(first.starts_with(Path::new(env!("CARGO_TARGET_TMPDIR"))));
    assert!(!first.starts_with("/tmp"));
}

/// Review P1 3 and A: every created directory is reported, and the chain through the anchor is synced on every call, the
/// second included, which creates nothing.
#[test]
fn durable_directory_creation_syncs_the_chain_through_the_anchor_every_time() {
    let root: PathBuf = scratch("durable-dirs");
    start_sync_trace();
    let created: Vec<PathBuf> = create_dir_all_durable(&root, &root.join("a/b/c")).expect("create");
    let first: Vec<PathBuf> = take_sync_trace();
    assert_eq!(created, vec![root.join("a"), root.join("a/b"), root.join("a/b/c")]);
    assert_eq!(first, vec![root.join("a/b/c"), root.join("a/b"), root.join("a"), root.clone()]);
    start_sync_trace();
    assert!(create_dir_all_durable(&root, &root.join("a/b/c")).expect("again").is_empty());
    assert_eq!(take_sync_trace(), first, "a chain that merely exists is synced again");
}

/// Re-review A: an ancestor another creator made and never synced is synced by the next successful bind, through the anchor.
#[test]
fn a_bind_syncs_an_ancestor_that_only_exists() {
    let root: PathBuf = scratch("anchor-sync");
    std::fs::create_dir_all(root.join("ids")).expect("an ancestor made by a creator that died before syncing the root");
    let id: String = identity('d');
    start_sync_trace();
    assert_eq!(bind(&root, &id, &sha256_hex(b"x")).expect("bind"), Bound::Created);
    let synced: Vec<PathBuf> = take_sync_trace();
    assert!(synced.contains(&root.join("ids")), "{:?}", synced);
    assert_eq!(synced.last(), Some(&root), "the chain ends at the anchor, the last directory synced");
}

/// Re-review D: a relative path works: the empty parent of a bare name is the current directory, never `create_dir("")`.
#[test]
fn durable_directory_creation_accepts_a_relative_path() {
    let root: PathBuf = scratch("relative");
    let previous: PathBuf = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(&root).expect("chdir");
    let created = create_dir_all_durable(Path::new("."), Path::new("new/a"));
    std::env::set_current_dir(previous).expect("chdir back");
    assert_eq!(created.expect("create").len(), 2);
    assert!(root.join("new/a").is_dir());
}

/// Review P1 4: a same-content resend, which finds the name already linked, syncs the namespace itself, in order, through the
/// anchor: the directory of the name, then each directory above it.
#[test]
fn a_resend_that_finds_the_name_syncs_the_directory_chain_itself() {
    let root: PathBuf = scratch("resend-sync");
    let id: String = identity('1');
    let hash: String = sha256_hex(b"x");
    assert_eq!(bind(&root, &id, &hash).expect("bind"), Bound::Created);
    let fan_out: PathBuf = binding_path(&root, &id).expect("path").parent().expect("parent").to_path_buf();
    start_sync_trace();
    assert_eq!(bind(&root, &id, &hash).expect("resend"), Bound::SameContent);
    let synced: Vec<PathBuf> = take_sync_trace();
    assert_eq!(synced.first(), Some(&fan_out));
    assert_eq!(synced.last(), Some(&root));
    let inbox: PathBuf = root.join("inbox");
    let one: Request = request('2', "m");
    publish(&root, &inbox, &one).expect("publish");
    start_sync_trace();
    assert_eq!(publish(&root, &inbox, &one).expect("resend"), Published::AlreadyPresent);
    let synced_record: Vec<PathBuf> = take_sync_trace();
    assert_eq!(synced_record[synced_record.len() - 2..], [inbox.clone(), root.clone()], "the inbox, then the anchor, last");
    let ledger: PathBuf = root.join("ledger");
    append(&root, &ledger, &entry('3', "published")).expect("append");
    start_sync_trace();
    append(&root, &ledger, &entry('4', "published")).expect("append");
    assert_eq!(take_sync_trace().last(), Some(&root));
}

/// Review P1 2 (revision 9, R8-2): each lane has its own ledger path, and a reader of a ledger never repairs it: a torn tail
/// is not yet written, whatever it is.
#[test]
fn a_reader_never_repairs_another_lanes_ledger() {
    let common: PathBuf = scratch("lane-ledger");
    let ledger: PathBuf = lane_ledger_path(&common, &identity('a')).expect("path");
    assert_eq!(ledger, common.join("ledgers").join(identity('a')).join("ledger"));
    assert_ne!(ledger, lane_ledger_path(&common, &identity('b')).expect("path"));
    append(&common, &ledger, &entry('1', "published")).expect("append");
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    bytes.extend_from_slice(b"deadbeef partial line without a newline");
    std::fs::write(&ledger, &bytes).expect("torn tail of an append in progress");
    assert!(lookup(&ledger, &identity('1')).expect("lookup").is_some());
    assert_eq!(lookup(&ledger, &identity('2')).expect("lookup"), None);
    assert_eq!(std::fs::read(&ledger).expect("read"), bytes, "a reader changes nothing");
}

/// Review P2: the bytes cut from a torn tail are saved, and their directory is synced before the ledger is truncated.
#[test]
fn a_repair_saves_the_cut_bytes_before_it_truncates() {
    let directory: PathBuf = scratch("torn-first");
    let ledger: PathBuf = directory.join("ledger");
    append(&directory, &ledger, &entry('1', "published")).expect("append");
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    bytes.extend_from_slice(b"abc");
    std::fs::write(&ledger, &bytes).expect("torn");
    start_sync_trace();
    repair_tail(&ledger).expect("repair");
    assert_eq!(take_sync_trace(), vec![directory.clone()], "the .torn entry's directory is synced");
    assert_eq!(std::fs::read(directory.join("ledger.torn")).expect("saved"), b"abc".to_vec());
}

/// Review P1 5: the spool alone carries both sides of every change. A repository is built, the base objects are captured into
/// the spool, every object of the repository is then removed (as a branch rewrite and a prune would), and the closure still
/// reads back the base bytes, which a replay needs to detect a competing change.
#[test]
fn spool_closure_survives_the_removal_of_the_base_objects() {
    let repository: PathBuf = scratch("spool-closure");
    crate::model_fixture::git(&repository, &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repository.join("kept.txt"), "base bytes").expect("write");
    crate::model_fixture::git(&repository, &["add", "--all"]);
    crate::model_fixture::git(&repository, &["commit", "--quiet", "--message=base"]);
    let base_bytes: Vec<u8> = crate::model_fixture::git(&repository, &["show", "HEAD:kept.txt"]);
    let mut original: Request = request('9', "m");
    original.changes = vec![Change {
        path: b"kept.txt".to_vec(),
        mode: "100644".to_string(),
        sha256: sha256_hex(b"new bytes"),
        old_mode: "100644".to_string(),
        old_sha256: sha256_hex(&base_bytes),
    }];
    write_spool(&repository, &original.id, &[b"new bytes".to_vec()], std::slice::from_ref(&base_bytes)).expect("write");
    for entry in std::fs::read_dir(repository.join(".git/objects")).expect("objects") {
        let path: PathBuf = entry.expect("entry").path();
        std::fs::remove_dir_all(&path).expect("remove the object store");
    }
    let closure = read_closure(&repository, &original).expect("closure");
    assert_eq!(closure[0].old, Some(("100644".to_string(), base_bytes)));
    assert_eq!(closure[0].new, Some(("100644".to_string(), b"new bytes".to_vec())));
}

/// Review P2: a request with no changes (an empty commit, a message-only amend) is valid.
#[test]
fn a_request_with_no_changes_is_valid() {
    let mut empty: Request = request('8', "message only");
    empty.changes = Vec::new();
    empty.operation = Operation::EmptyCommit;
    let bytes: Vec<u8> = encode(&empty).expect("encode");
    assert_eq!(decode(&bytes).expect("decode"), empty);
}

/// Re-review E: an empty commit and a message-only amend are different requests with different hashes, and an ordinary commit
/// with no change, an empty commit with changes and an amend of a non-object are refused.
#[test]
fn the_operation_kind_and_amend_target_are_part_of_the_request() {
    let mut empty: Request = request('7', "checkpoint");
    empty.changes = Vec::new();
    empty.operation = Operation::EmptyCommit;
    let mut amend: Request = empty.clone();
    amend.operation = Operation::Amend("c".repeat(40));
    assert_ne!(content_hash(&empty).expect("hash"), content_hash(&amend).expect("hash"));
    assert_eq!(decode(&encode(&amend).expect("encode")).expect("decode"), amend);
    let mut bare: Request = empty.clone();
    bare.operation = Operation::Commit;
    assert!(matches!(encode(&bare), Err(RequestError::Invalid("changes"))));
    let mut full: Request = request('7', "m");
    full.operation = Operation::EmptyCommit;
    assert!(matches!(encode(&full), Err(RequestError::Invalid("changes"))));
    amend.operation = Operation::Amend("zz".to_string());
    assert!(matches!(encode(&amend), Err(RequestError::Invalid("amend target"))));
}

/// Re-review: paths and branch names are bytes (a tracked path need not be UTF-8).
#[test]
fn paths_and_branch_names_are_bytes() {
    let mut original: Request = request('6', "m");
    original.changes[1].path = b"b/\xff\xfe.txt".to_vec();
    original.branch = b"topic/\xff".to_vec();
    assert_eq!(decode(&encode(&original).expect("encode")).expect("decode"), original);
}

/// Re-review B: a manifest whose records are reordered, renumbered or point outside the spool is refused, even with a correct
/// end hash, and `read_closure` returns only bytes that match the request.
#[test]
fn spool_refuses_a_manifest_that_swaps_the_new_and_old_records() {
    let root: PathBuf = scratch("spool-manifest");
    let original: Request = request('5', "m");
    let new_files: Vec<Vec<u8>> = vec![Vec::new(), b"new bytes".to_vec()];
    let old_files: Vec<Vec<u8>> = vec![b"old gone".to_vec(), Vec::new()];
    write_spool(&root, &original.id, &new_files, &old_files).expect("write");
    let manifest: PathBuf = root.join("spool").join(&original.id).join("manifest");
    let good: String = std::fs::read_to_string(&manifest).expect("manifest");
    let body: &str = &good[..good.rfind("end ").expect("end")];
    let lines: Vec<&str> = body.lines().collect();
    let hostile: Vec<Vec<&str>> = vec![
        vec![lines[2], lines[3], lines[0], lines[1]],
        vec![lines[1], lines[0], lines[2], lines[3]],
        vec![lines[0], lines[0], lines[2], lines[3]],
        vec![lines[0], lines[1], lines[2]],
    ];
    for records in hostile {
        let mut forged: String = records.join("\n");
        forged.push('\n');
        let with_end: String = format!("{}end {}\n", forged, sha256_hex(forged.as_bytes()));
        std::fs::remove_file(&manifest).expect("remove");
        std::fs::write(&manifest, with_end).expect("forge");
        assert!(matches!(verify_spool(&root, &original.id), Err(SpoolError::Mismatch(_))), "{:?}", records);
    }
    let escaping: String = format!("file ../x 0 {}\nold 0 0 {}\n", sha256_hex(b""), sha256_hex(b""));
    std::fs::remove_file(&manifest).expect("remove");
    std::fs::write(&manifest, format!("{}end {}\n", escaping, sha256_hex(escaping.as_bytes()))).expect("forge");
    assert!(matches!(verify_spool(&root, &original.id), Err(SpoolError::Mismatch(_))));
}

/// The barriers of the fault-point tests. Each test has its own pair, because the statics are shared by the whole test process.
static BIND_ARRIVED: std::sync::Barrier = std::sync::Barrier::new(2);
/// Releases the writer paused at the bind fault point.
static BIND_RELEASE: std::sync::Barrier = std::sync::Barrier::new(2);
/// Marks that the writer paused at the publish fault point.
static PUBLISH_ARRIVED: std::sync::Barrier = std::sync::Barrier::new(2);
/// Releases the writer paused at the publish fault point.
static PUBLISH_RELEASE: std::sync::Barrier = std::sync::Barrier::new(2);

/// Pauses the calling writer at the bind decision point until the test releases it.
fn pause_at_bind(point: &str) {
    if point == "bind:before-link" {
        BIND_ARRIVED.wait();
        BIND_RELEASE.wait();
    }
}

/// Pauses the calling writer at the publish decision point until the test releases it.
fn pause_at_publish(point: &str) {
    if point == "publish:before-link" {
        PUBLISH_ARRIVED.wait();
        PUBLISH_RELEASE.wait();
    }
}

/// Re-review P3, deterministic: writer A has written and synced its temporary binding and stops just before the link; writer B
/// binds the same identity to other content and finishes; A then resumes. The unsafe schedule always runs, and A is refused.
#[test]
fn a_writer_paused_before_the_link_loses_to_one_that_finished_first() {
    let root: PathBuf = scratch("fault-bind");
    let id: String = identity('7');
    let (a_root, a_id) = (root.clone(), id.clone());
    let paused = std::thread::spawn(move || {
        git_policy_cli::commit_identity::set_fault_point(Some(pause_at_bind));
        return bind(&a_root, &a_id, &sha256_hex(b"first")).expect("bind");
    });
    BIND_ARRIVED.wait();
    assert_eq!(bind(&root, &id, &sha256_hex(b"second")).expect("bind"), Bound::Created);
    BIND_RELEASE.wait();
    assert_eq!(paused.join().expect("join"), Bound::DifferentContent(sha256_hex(b"second")));
    assert_eq!(std::fs::read_to_string(binding_path(&root, &id).expect("path")).expect("read"), format!("{}\n", sha256_hex(b"second")));
}

/// The same for the record: writer A stops before linking `inbox/<id>`, writer B publishes other content and finishes, A resumes
/// and finds a conflict; the acknowledged record is B's and unchanged.
#[test]
fn a_publisher_paused_before_the_link_finds_the_other_record_in_place() {
    let root: PathBuf = scratch("fault-publish");
    let inbox: PathBuf = root.join("inbox");
    let (a_root, a_inbox) = (root.clone(), inbox.clone());
    let paused = std::thread::spawn(move || {
        git_policy_cli::commit_identity::set_fault_point(Some(pause_at_publish));
        return publish(&a_root, &a_inbox, &request('c', "first"));
    });
    PUBLISH_ARRIVED.wait();
    assert_eq!(publish(&root, &inbox, &request('c', "second")).expect("publish"), Published::Created);
    PUBLISH_RELEASE.wait();
    assert!(matches!(paused.join().expect("join"), Err(RequestError::Conflict)));
    assert_eq!(decode(&std::fs::read(inbox.join(identity('c'))).expect("read")).expect("decode"), request('c', "second"));
}
