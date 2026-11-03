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
use git_policy_cli::commit_identity::{Bound, IdentityError, bind, binding_path, create_dir_all_durable, directory_sync_count};
use git_policy_cli::commit_integrity::{crc32, sha256_hex};
use git_policy_cli::commit_ledger::{LedgerEntry, LedgerError, append, encode_line, lane_ledger_path, lookup, parse_line, repair_tail};
use git_policy_cli::commit_request::{Change, Published, Request, RequestError, content_hash, decode, encode, publish};
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
        branch: "main".to_string(),
        base: "b".repeat(40),
        message: message.to_string(),
        changes: vec![
            Change {
                path: "a/gone.txt".to_string(),
                mode: "000000".to_string(),
                sha256: "-".to_string(),
                old_mode: "100644".to_string(),
                old_sha256: sha256_hex(b"old gone"),
            },
            Change {
                path: "b/new.txt".to_string(),
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
    append(&ledger, &entry('1', "published")).expect("append");
    append(&ledger, &entry('2', "saved")).expect("append");
    append(&ledger, &entry('1', "landed")).expect("append");
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
        append(&ledger, &entry('3', "landed")).expect("append after repair");
        assert!(lookup(&ledger, &identity('1')).expect("lookup").is_some());
        assert!(lookup(&ledger, &identity('3')).expect("lookup").is_some());
    }
}

/// A damaged line that is not the last is reported and nothing is truncated: the lines behind it are good data.
#[test]
fn ledger_damaged_middle_is_reported_and_kept() {
    let ledger: PathBuf = scratch("ledger-middle").join("ledger");
    for digit in ['1', '2', '3'] {
        append(&ledger, &entry(digit, "published")).expect("append");
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
    for number in 0..8 {
        let worker_root: PathBuf = root.clone();
        let worker_id: String = id.clone();
        workers.push(std::thread::spawn(move || {
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
        original.branch = "feature/with space\nand newline".to_string();
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
        bad.changes[1].path = path.to_string();
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
    cases.push(Request { message: "  ".to_string(), ..request('a', "m") });
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
    assert_eq!(publish(&inbox, &first).expect("publish"), Published::Created);
    assert_eq!(publish(&inbox, &first).expect("publish"), Published::AlreadyPresent);
    assert!(matches!(publish(&inbox, &second), Err(RequestError::Conflict)));
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
    for number in 0..6 {
        let worker_inbox: PathBuf = inbox.clone();
        workers.push(std::thread::spawn(move || { return publish(&worker_inbox, &request('d', &format!("message {}", number))); }));
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

/// Review P1 3: every newly created ancestor is reported created, so the helper has synced the parent of each.
#[test]
fn durable_directory_creation_reports_every_created_ancestor() {
    let root: PathBuf = scratch("durable-dirs");
    let created: Vec<PathBuf> = create_dir_all_durable(&root.join("a/b/c")).expect("create");
    assert_eq!(created, vec![root.join("a"), root.join("a/b"), root.join("a/b/c")]);
    assert!(create_dir_all_durable(&root.join("a/b/c")).expect("again").is_empty());
}

/// Review P1 4: a same-content resend, which finds the name already linked, syncs the namespace itself: visibility is not
/// durability, and the creator may have died between its link and its directory sync.
#[test]
fn a_resend_that_finds_the_name_syncs_the_directory_itself() {
    let root: PathBuf = scratch("resend-sync");
    let id: String = identity('1');
    let hash: String = sha256_hex(b"x");
    assert_eq!(bind(&root, &id, &hash).expect("bind"), Bound::Created);
    let before: usize = directory_sync_count();
    assert_eq!(bind(&root, &id, &hash).expect("resend"), Bound::SameContent);
    assert!(directory_sync_count() > before, "the binding resend must sync");
    let inbox: PathBuf = root.join("inbox");
    let one: Request = request('2', "m");
    publish(&inbox, &one).expect("publish");
    let before: usize = directory_sync_count();
    assert_eq!(publish(&inbox, &one).expect("resend"), Published::AlreadyPresent);
    assert!(directory_sync_count() > before, "the record resend must sync");
    let ledger: PathBuf = root.join("ledger");
    append(&ledger, &entry('3', "published")).expect("append");
    let before: usize = directory_sync_count();
    append(&ledger, &entry('4', "published")).expect("append");
    assert!(directory_sync_count() > before, "an append to an existing ledger syncs its directory");
}

/// Review P1 2 (revision 9, R8-2): each lane has its own ledger path, and a reader of a ledger never repairs it: a torn tail
/// is not yet written, whatever it is.
#[test]
fn a_reader_never_repairs_another_lanes_ledger() {
    let common: PathBuf = scratch("lane-ledger");
    let ledger: PathBuf = lane_ledger_path(&common, &identity('a')).expect("path");
    assert_eq!(ledger, common.join("ledgers").join(identity('a')).join("ledger"));
    assert_ne!(ledger, lane_ledger_path(&common, &identity('b')).expect("path"));
    append(&ledger, &entry('1', "published")).expect("append");
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    bytes.extend_from_slice(b"deadbeef partial line without a newline");
    std::fs::write(&ledger, &bytes).expect("torn tail of an append in progress");
    assert!(lookup(&ledger, &identity('1')).expect("lookup").is_some());
    assert_eq!(lookup(&ledger, &identity('2')).expect("lookup"), None);
    assert_eq!(std::fs::read(&ledger).expect("read"), bytes, "a reader changes nothing");
}

/// Review P2: the bytes cut from a torn tail are in a directory entry that is durable before the ledger is truncated.
#[test]
fn a_repair_saves_the_cut_bytes_before_it_truncates() {
    let directory: PathBuf = scratch("torn-first");
    let ledger: PathBuf = directory.join("ledger");
    append(&ledger, &entry('1', "published")).expect("append");
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    bytes.extend_from_slice(b"abc");
    std::fs::write(&ledger, &bytes).expect("torn");
    let before: usize = directory_sync_count();
    repair_tail(&ledger).expect("repair");
    assert!(directory_sync_count() > before, "the .torn entry is synced before the truncation");
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
        path: "kept.txt".to_string(),
        mode: "100644".to_string(),
        sha256: sha256_hex(b"new bytes"),
        old_mode: "100644".to_string(),
        old_sha256: sha256_hex(&base_bytes),
    }];
    write_spool(&repository, &original.id, &[b"new bytes".to_vec()], &[base_bytes.clone()]).expect("write");
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
    let bytes: Vec<u8> = encode(&empty).expect("encode");
    assert_eq!(decode(&bytes).expect("decode"), empty);
}
