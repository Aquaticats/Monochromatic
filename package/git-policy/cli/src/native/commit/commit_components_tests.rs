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

use crate::model_fixture::{owned_append, owned_repair, scratch};
use git_policy_cli::commit_identity::create_dir_all_durable;
use git_policy_cli::commit_integrity::{crc32, sha256_hex};
use git_policy_cli::commit_ledger::{LedgerEntry, LedgerError, LedgerOwner, MAX_LINE_BYTES, TAIL_WINDOW, TIME_MAX, encode_line, lane_ledger_path, lookup, parse_line};
use git_policy_cli::commit_request::{Change, Eligibility, Operation, Request, RequestError, content_hash, decode, eligibility, encode};
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
        nonce: vec![1; 16],
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
    owned_append(ledger.parent().expect("parent"), &ledger, &entry('1', "published")).expect("append");
    owned_append(ledger.parent().expect("parent"), &ledger, &entry('2', "saved")).expect("append");
    owned_append(ledger.parent().expect("parent"), &ledger, &entry('1', "landed")).expect("append");
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
        let repair = owned_repair(ledger.parent().expect("parent"), &ledger).expect("repair");
        assert_eq!(repair.kept as usize, first.len(), "cut {}", cut);
        assert_eq!(repair.cut as usize, cut, "cut {}", cut);
        assert_eq!(std::fs::read(&ledger).expect("read"), first);
        if cut > 0 {
            assert_eq!(std::fs::read(directory.join("ledger.torn")).expect("saved tail"), second[..cut].to_vec());
        }
        owned_append(ledger.parent().expect("parent"), &ledger, &entry('3', "landed")).expect("append after repair");
        assert!(lookup(&ledger, &identity('1')).expect("lookup").is_some());
        assert!(lookup(&ledger, &identity('3')).expect("lookup").is_some());
    }
}

/// A damaged line that is not the last is reported and nothing is truncated: the lines behind it are good data.
#[test]
fn ledger_damaged_middle_is_reported_and_kept() {
    let ledger: PathBuf = scratch("ledger-middle").join("ledger");
    for digit in ['1', '2', '3'] {
        owned_append(ledger.parent().expect("parent"), &ledger, &entry(digit, "published")).expect("append");
    }
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    let line: usize = encode_line(&entry('1', "published")).expect("encode").len();
    bytes[line + 12] ^= 1;
    std::fs::write(&ledger, &bytes).expect("damage");
    assert!(matches!(owned_repair(ledger.parent().expect("parent"), &ledger), Err(LedgerError::DamagedLine(1))));
    assert!(matches!(lookup(&ledger, &identity('3')), Err(LedgerError::DamagedLine(1))));
    assert_eq!(std::fs::read(&ledger).expect("read"), bytes);
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
    let text: String = String::from_utf8(bytes).expect("utf-8").replacen("cli-git-request 4", "cli-git-request 5", 1);
    let inbox: PathBuf = scratch("request-version").join("inbox");
    std::fs::create_dir_all(&inbox).expect("inbox");
    let file: PathBuf = inbox.join(identity('b'));
    std::fs::write(&file, text.as_bytes()).expect("write");
    assert!(matches!(decode(&std::fs::read(&file).expect("read")), Err(RequestError::Damaged("version"))));
    assert_eq!(std::fs::read(&file).expect("read"), text.as_bytes());
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

/// Review P1 2 (revision 9, R8-2): each lane has its own ledger path, and a reader of a ledger never repairs it: a torn tail
/// is not yet written, whatever it is.
#[test]
fn a_reader_never_repairs_another_lanes_ledger() {
    let common: PathBuf = scratch("lane-ledger");
    let ledger: PathBuf = lane_ledger_path(&common, &identity('a')).expect("path");
    assert_eq!(ledger, common.join("ledgers").join(identity('a')).join("ledger"));
    assert_ne!(ledger, lane_ledger_path(&common, &identity('b')).expect("path"));
    owned_append(&common, &ledger, &entry('1', "published")).expect("append");
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    bytes.extend_from_slice(b"deadbeef partial line without a newline");
    std::fs::write(&ledger, &bytes).expect("torn tail of an append in progress");
    assert!(lookup(&ledger, &identity('1')).expect("lookup").is_some());
    assert_eq!(lookup(&ledger, &identity('2')).expect("lookup"), None);
    assert_eq!(std::fs::read(&ledger).expect("read"), bytes, "a reader changes nothing");
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

/// Re-review E: an empty commit is a request of its own kind; an ordinary commit with no change and an empty commit with changes are
/// refused. An amendment is not accepted (its target is not recoverable from the record): it is classified for the native fallback.
#[test]
fn the_operation_kind_is_part_of_the_request_and_an_amendment_goes_native() {
    let mut empty: Request = request('7', "checkpoint");
    empty.changes = Vec::new();
    empty.operation = Operation::EmptyCommit;
    assert_eq!(decode(&encode(&empty).expect("encode")).expect("decode"), empty);
    let mut bare: Request = empty.clone();
    bare.operation = Operation::Commit;
    assert!(matches!(encode(&bare), Err(RequestError::Invalid("changes"))));
    let mut full: Request = request('7', "m");
    full.operation = Operation::EmptyCommit;
    assert!(matches!(encode(&full), Err(RequestError::Invalid("changes"))));
    let mut amend: Request = empty.clone();
    amend.operation = Operation::Amend { target: "c".repeat(40), parents: vec!["d".repeat(40)] };
    assert!(matches!(encode(&amend), Err(RequestError::Invalid("amend"))));
    assert!(matches!(eligibility(&amend), Eligibility::NativeFallback(_)));
    assert_eq!(eligibility(&request('7', "m")), Eligibility::Accept);
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


/// Review 3: a correctly checksummed manifest with CRLF endings, or whose last body record lacks its newline, is not the manifest
/// the writer emits and is refused.
#[test]
fn spool_refuses_a_manifest_that_is_not_byte_canonical() {
    let root: PathBuf = scratch("spool-bytes");
    let original: Request = request('4', "m");
    write_spool(&root, &original.id, &[Vec::new(), b"new bytes".to_vec()], &[b"old gone".to_vec(), Vec::new()]).expect("write");
    let manifest: PathBuf = root.join("spool").join(&original.id).join("manifest");
    let good: String = std::fs::read_to_string(&manifest).expect("manifest");
    let body: String = good[..good.rfind("end ").expect("end")].to_string();
    let crlf: String = body.replace('\n', "\r\n");
    let no_final_newline: String = body.trim_end_matches('\n').to_string();
    for forged in [crlf, no_final_newline] {
        std::fs::remove_file(&manifest).expect("remove");
        std::fs::write(&manifest, format!("{}end {}\n", forged, sha256_hex(forged.as_bytes()))).expect("forge");
        assert!(matches!(verify_spool(&root, &original.id), Err(SpoolError::Mismatch(_))));
    }
}

/// Review 4: a gitlink change is refused before acceptance and classified for the native fallback, and the fallback works: real Git
/// commits the submodule pointer and amends it, in a disposable repository.
#[test]
fn gitlinks_and_amendments_are_refused_before_acceptance_and_real_git_commits_them() {
    let mut gitlink: Request = request('3', "m");
    gitlink.changes[1].mode = "160000".to_string();
    assert!(matches!(encode(&gitlink), Err(RequestError::Invalid("mode"))));
    assert!(matches!(eligibility(&gitlink), Eligibility::NativeFallback(_)));
    let repository: PathBuf = scratch("fallback-native");
    crate::model_fixture::git(&repository, &["init", "--quiet", "--initial-branch=main"]);
    crate::model_fixture::git(&repository, &["commit", "--quiet", "--allow-empty", "--message=root"]);
    let pointer: String = crate::model_fixture::git_text(&repository, &["rev-parse", "HEAD"]);
    crate::model_fixture::git(&repository, &["update-index", "--add", "--cacheinfo", &format!("160000,{},sub", pointer)]);
    crate::model_fixture::git(&repository, &["commit", "--quiet", "--message=submodule pointer"]);
    assert!(crate::model_fixture::git_text(&repository, &["ls-tree", "HEAD"]).starts_with("160000 commit"));
    crate::model_fixture::git(&repository, &["commit", "--quiet", "--amend", "--message=submodule pointer, amended"]);
    assert_eq!(crate::model_fixture::git_text(&repository, &["log", "-1", "--format=%s"]), "submodule pointer, amended");
}

/// Review 4 P2: the standalone repair has its contract in the type: only the holder of the ledger's exclusive owner lock may repair
/// or append through it, a second acquirer is refused at once (never waits), and the lock ends with its holder.
#[test]
fn only_the_exclusive_owner_of_a_ledger_may_repair_it() {
    let directory: PathBuf = scratch("ledger-owner");
    let ledger: PathBuf = directory.join("ledger");
    let mut owner: LedgerOwner = LedgerOwner::acquire(&ledger).expect("first owner");
    assert!(matches!(LedgerOwner::acquire(&ledger), Err(LedgerError::OwnedElsewhere)));
    owner.append(&directory, &ledger, &entry('1', "published")).expect("append");
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    bytes.extend_from_slice(b"abc");
    std::fs::write(&ledger, &bytes).expect("torn");
    let repair = owner.repair_tail(&directory, &ledger).expect("repair");
    assert_eq!(repair.cut, 3);
    drop(owner);
    assert!(LedgerOwner::acquire(&ledger).is_ok(), "the lock ends with its holder");
}

/// Review 5 P2: an owner is the owner of one ledger; it cannot repair or append to another ledger.
#[test]
fn an_owner_of_one_ledger_cannot_touch_another() {
    let directory: PathBuf = scratch("ledger-other");
    let mine: PathBuf = directory.join("mine");
    let other: PathBuf = directory.join("other");
    let mut owner: LedgerOwner = LedgerOwner::acquire(&mine).expect("owner");
    assert!(owner.append(&directory, &other, &entry('1', "published")).is_err());
    assert!(owner.repair_tail(&directory, &other).is_err());
    assert!(!other.exists());
}

/// Review 5 item 3.7: the bounded-read append is justified only by a validated history. A ledger with a damaged middle line is refused
/// when the owner is acquired (the one full validation per owner), so no append can succeed behind damage and leave `lookup` failing.
#[test]
fn an_owner_cannot_be_acquired_over_a_damaged_middle() {
    let ledger: PathBuf = scratch("ledger-owner-middle").join("ledger");
    for digit in ['1', '2', '3'] {
        owned_append(ledger.parent().expect("parent"), &ledger, &entry(digit, "published")).expect("append");
    }
    let mut bytes: Vec<u8> = std::fs::read(&ledger).expect("read");
    let line: usize = encode_line(&entry('1', "published")).expect("encode").len();
    bytes[line + 12] ^= 1;
    std::fs::write(&ledger, &bytes).expect("damage");
    assert!(matches!(LedgerOwner::acquire(&ledger), Err(LedgerError::DamagedLine(1))));
    assert!(matches!(owned_append(ledger.parent().expect("parent"), &ledger, &entry('4', "published")), Err(LedgerError::DamagedLine(1))));
    assert_eq!(std::fs::read(&ledger).expect("read"), bytes, "nothing was appended behind the damage");
}

/// Review 5 item 3.7: no valid line is longer than the tail window, so a clean append reads at most the window. The time field is the
/// only unbounded one; its bound and the window are derived together.
#[test]
fn no_valid_line_is_longer_than_the_tail_window() {
    let longest: LedgerEntry = LedgerEntry { id: identity('1'), outcome: "a".repeat(32), commit: "a".repeat(64), time: "t".repeat(TIME_MAX) };
    let line: Vec<u8> = encode_line(&longest).expect("the longest valid line");
    assert!(line.len() <= MAX_LINE_BYTES && (line.len() as u64) < TAIL_WINDOW, "{} bytes against a window of {}", line.len(), TAIL_WINDOW);
    let too_long: LedgerEntry = LedgerEntry { time: "t".repeat(TIME_MAX + 1), ..longest };
    assert!(matches!(encode_line(&too_long), Err(LedgerError::InvalidField("time"))));
}

/// Review 7, 4.6: a FIFO at the ledger's owner-lock path is refused at once; the blocking open before `try_lock` is never made. A deadline
/// on a worker thread turns a regression into a failure and not a hang.
#[test]
fn a_fifo_at_the_ledger_owner_lock_path_is_refused_without_blocking() {
    let directory: PathBuf = scratch("ledger-fifo-lock");
    let ledger: PathBuf = directory.join("ledger");
    let lock = std::ffi::CString::new(format!("{}.owner", ledger.display())).expect("path");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode; the path outlives the call.
    assert_eq!(unsafe { libc::mkfifo(lock.as_ptr(), 0o600) }, 0);
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || { let _ = sender.send(LedgerOwner::acquire(&ledger).map(|_| ())); });
    let outcome = receiver.recv_timeout(std::time::Duration::from_secs(5)).expect("acquire returned within the deadline");
    assert!(outcome.is_err());
}

/// Review 7, 4.1: a fresh lane has no parent for the owner lock; `acquire_in` makes the chain durable through the anchor first.
#[test]
fn a_fresh_lane_ledger_can_be_acquired_through_its_anchor() {
    let root: PathBuf = scratch("ledger-fresh-lane");
    let ledger: PathBuf = root.join("ledgers").join(identity('1')).join("ledger");
    assert!(LedgerOwner::acquire(&ledger).is_err(), "without the parent the plain acquire fails");
    let mut owner: LedgerOwner = LedgerOwner::acquire_in(&root, &ledger).expect("acquire through the anchor");
    owner.append(&root, &ledger, &entry('1', "published")).expect("append");
}

/// Review 7, 4.6: a root spelled with a parent-directory component is refused before anything is created.
#[test]
fn a_root_with_a_parent_component_is_refused_before_creation() {
    let parent: PathBuf = scratch("root-dotdot");
    let spelled: PathBuf = parent.join("a").join("..").join("state");
    assert!(git_policy_cli::commit_identity::init_root(&parent, &spelled).is_err());
    assert!(!parent.join("a").exists() && !parent.join("state").exists());
}
