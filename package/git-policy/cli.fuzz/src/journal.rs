//! What: Generated transaction journal records, capture-order records and sequence files, with
//!       the invariants of their parsers.
//! Why: Recovery reads these files after a crash and acts on them: a misread landing record
//!      installs the wrong index, a misread capture record prunes a record a replay needs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const { text, expected } of generatedRecords(data)) expect(parse(text)).toEqual(expected);
//! ```

/// The generator cursor shared with the owner records.
use super::owners::Cursor;
/// The digest a landing record keeps of each index snapshot.
use git_policy_cli::content_digest::ContentDigest;
/// Capture-order records under test.
use git_policy_cli::capture_records::{
    CapturedRecord, LandedCaptureRecord, encode_captured_record, encode_landed_capture_record,
    parse_captured_record, parse_landed_capture_record,
};
/// The sequence file reader under test.
use git_policy_cli::capture_store::parse_sequence;
/// Journal record types.
use git_policy_cli::transaction_journal::{
    AddedPath, Base, Conclusion, FileIdentity, IndexLockRecord, LandingOperation, LandingRecord,
    LockIdentity, PreparingRecord, RefFormat, SymbolicHead, TransactionMode,
};
/// Journal encoders.
use git_policy_cli::transaction_journal_encode::{
    encode_index_lock, encode_landing, encode_preparing, encode_ref_updated,
};
/// Journal parsers.
use git_policy_cli::transaction_journal_parse::{
    parse_index_lock, parse_landing, parse_preparing, parse_ref_updated,
};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What: Text edits that make every record kind invalid, applied once to the encoded text.
/// Why:  Wrong schema, wrong state, a renamed required key, a mistyped number, a non-object.
const REFUSING_EDITS: [(&str, &str); 6] = [
    ("\"schemaVersion\":2", "\"schemaVersion\":1"),
    ("\"schemaVersion\":2", "\"schemaVersion\":\"2\""),
    ("\"state\":\"", "\"state\":\"x"),
    ("{\"schemaVersion\"", "[{\"schemaVersion\""),
    ("\"fsId\":", "\"fsId\":7,\"x\":"),
    ("\"attempt\":", "\"attempt\":0,\"x\":"),
];

/// A file identity from the cursor.
fn identity(cursor: &mut Cursor<'_>) -> FileIdentity {
    return FileIdentity {
        device: cursor.text(false),
        inode: cursor.text(false),
    };
}

/// A lock identity from the cursor.
fn lock(cursor: &mut Cursor<'_>) -> LockIdentity {
    return LockIdentity {
        file: identity(cursor),
        fs_id: cursor.text(false),
    };
}

/// Added-path records from the cursor.
fn added(cursor: &mut Cursor<'_>) -> Vec<AddedPath> {
    let mut records: Vec<AddedPath> = Vec::new();
    for _ in 0..cursor.byte() % 3 {
        records.push(AddedPath {
            path: cursor.text(false).into_bytes(),
            git_mode: String::from(if cursor.flip() { "100644" } else { "100755" }),
            original_oid: cursor.text(false),
            intended_oid: cursor.text(false),
        });
    }
    return records;
}

/// What: An optional content digest from the cursor:
///       absent, or a length and a hash that use the whole range a record can store.
/// Why:  Records written before digests existed have none, and a stored hash may be any 16 hexadecimal digits.
fn digest(cursor: &mut Cursor<'_>) -> Option<ContentDigest> {
    if cursor.flip() {
        return None;
    }
    let mut hash: u64 = 0;
    for _ in 0..8 {
        hash = (hash << 8) | u64::from(cursor.byte());
    }
    // `positive()` is a safe integer, so the length survives the signed JSON number it is stored as.
    return Some(ContentDigest {
        length: u64::try_from(cursor.positive()).unwrap_or(0),
        fnv1a64: hash,
    });
}

/// A base from the cursor.
fn base(cursor: &mut Cursor<'_>) -> Base {
    if cursor.flip() {
        return Base::Unborn;
    }
    return Base::Commit(cursor.text(false));
}

/// A preparing record from the cursor.
pub fn generated_preparing(cursor: &mut Cursor<'_>) -> PreparingRecord {
    let mut pathspecs: Vec<Vec<u8>> = Vec::new();
    for _ in 0..cursor.byte() % 3 {
        pathspecs.push(cursor.text(false).into_bytes());
    }
    return PreparingRecord {
        transaction_id: cursor.text(false),
        mode: if cursor.flip() {
            TransactionMode::ExplicitPath
        } else {
            TransactionMode::Index
        },
        base: base(cursor),
        symbolic_head: if cursor.flip() {
            SymbolicHead::Detached
        } else {
            SymbolicHead::Branch(cursor.text(false))
        },
        target_ref: cursor.text(false),
        conclusion: [
            Conclusion::None,
            Conclusion::Amend,
            Conclusion::Merge,
            Conclusion::CherryPick,
            Conclusion::Revert,
        ][usize::from(cursor.byte()) % 5],
        repository_root: cursor.text(false),
        git_dir: cursor.text(false),
        common_dir: cursor.text(false),
        real_index_path: cursor.text(false),
        object_directory: cursor.text(false),
        ref_format: if cursor.flip() {
            RefFormat::Files
        } else {
            RefFormat::Reftable
        },
        empty_tree_oid: cursor.text(false),
        shadow_path: cursor.text(false),
        selected_pathspecs: pathspecs,
        invoked_at: cursor.text(false),
    };
}

/// A landing record from the cursor.
pub fn generated_landing(cursor: &mut Cursor<'_>) -> LandingRecord {
    let commit: bool = cursor.flip();
    return LandingRecord {
        attempt: cursor.positive(),
        operation: if commit {
            LandingOperation::Commit
        } else {
            LandingOperation::NormalizeOnly
        },
        expected_old: base(cursor),
        new_oid: if commit {
            Some(cursor.text(false))
        } else {
            None
        },
        landed_tree_oid: cursor.text(false),
        pre_landing_index: identity(cursor),
        post_index: identity(cursor),
        lock: lock(cursor),
        pack_name: if cursor.flip() {
            None
        } else {
            Some(cursor.text(false))
        },
        added_paths: added(cursor),
        selected_worktree_paths: added(cursor),
        pre_landing_digest: digest(cursor),
        post_digest: digest(cursor),
    };
}

/// What: Generated records of every journal kind round-trip exactly, and each refusing edit
///       that applies to a kind makes its parser refuse.
/// Why:  The encoders write what the incumbent writes; the parsers must read it back and nothing
///       malformed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGeneratedJournal(data: Uint8Array): void;
/// ```
pub fn check_generated_journal(data: &[u8]) {
    let mut cursor: Cursor<'_> = Cursor::new(data);
    let preparing: PreparingRecord = generated_preparing(&mut cursor);
    let landing: LandingRecord = generated_landing(&mut cursor);
    let index_lock: IndexLockRecord = IndexLockRecord {
        attempt: cursor.positive(),
        lock: lock(&mut cursor),
    };
    let landed: String = cursor.text(false);
    let encoded: [String; 4] = [
        encode_preparing(&preparing),
        encode_landing(&landing),
        encode_index_lock(&index_lock),
        encode_ref_updated(landed.as_str()),
    ];
    assert_eq!(parse_preparing(encoded[0].as_bytes()), Ok(preparing));
    assert_eq!(parse_landing(encoded[1].as_bytes()), Ok(landing));
    assert_eq!(parse_index_lock(encoded[2].as_bytes()), Ok(index_lock));
    assert_eq!(parse_ref_updated(encoded[3].as_bytes()), Ok(landed));
    let (from, to) = REFUSING_EDITS[usize::from(cursor.byte()) % REFUSING_EDITS.len()];
    for (kind, text) in encoded.iter().enumerate() {
        if !text.contains(from) {
            continue;
        }
        let mutated: String = text.replacen(from, to, 1);
        let accepted: bool = match kind {
            0 => parse_preparing(mutated.as_bytes()).is_ok(),
            1 => parse_landing(mutated.as_bytes()).is_ok(),
            2 => parse_index_lock(mutated.as_bytes()).is_ok(),
            _ => parse_ref_updated(mutated.as_bytes()).is_ok(),
        };
        assert!(!accepted, "{mutated}");
    }
    let cut: usize = usize::from(cursor.byte()) % encoded[1].len().max(1);
    if cut + 1 < encoded[1].len() && encoded[1].is_char_boundary(cut) {
        assert!(parse_landing(&encoded[1].as_bytes()[..cut]).is_err());
    }
}

/// What: The journal invariants on any bytes: an accepted record restates to bytes that parse
///       to the same record, and restating is stable.
/// Why:  A record recovery accepts must mean one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkJournalBytes(data: Uint8Array): void;
/// ```
pub fn check_journal_bytes(data: &[u8]) {
    if let Ok(record) = parse_preparing(data) {
        let restated: String = encode_preparing(&record);
        assert_eq!(parse_preparing(restated.as_bytes()), Ok(record));
    }
    if let Ok(record) = parse_landing(data) {
        assert_eq!(
            record.operation == LandingOperation::Commit,
            record.new_oid.is_some(),
            "{record:?}"
        );
        let restated: String = encode_landing(&record);
        assert_eq!(parse_landing(restated.as_bytes()), Ok(record));
    }
    if let Ok(record) = parse_index_lock(data) {
        assert!(record.attempt >= 1);
        assert_eq!(
            parse_index_lock(encode_index_lock(&record).as_bytes()),
            Ok(record)
        );
    }
    if let Ok(landed) = parse_ref_updated(data) {
        assert_eq!(
            parse_ref_updated(encode_ref_updated(landed.as_str()).as_bytes()),
            Ok(landed)
        );
    }
}

/// What: Generated capture-order records round-trip, their raw-text invariants hold, and the
///       sequence reader accepts exactly canonical decimals.
/// Why:  Pruning and replay decide from these numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkCaptureRecords(data: Uint8Array): void;
/// ```
pub fn check_capture_records(data: &[u8]) {
    let mut cursor: Cursor<'_> = Cursor::new(data);
    let mut paths: Vec<String> = Vec::new();
    for _ in 0..cursor.byte() % 3 {
        paths.push(cursor.text(true));
    }
    let captured: CapturedRecord = CapturedRecord {
        worktree_id: cursor.text(true),
        sequence: cursor.positive(),
        next_sequence_before_base: cursor.positive(),
        worktree_paths: paths.clone(),
    };
    let encoded: String = encode_captured_record(&captured);
    assert_eq!(
        parse_captured_record(encoded.as_str()),
        Ok(captured.clone())
    );
    assert!(
        parse_captured_record(
            encoded
                .replacen("\"sequence\":", "\"sequence\":0,\"x\":", 1)
                .as_str()
        )
        .is_err()
    );
    let landed: LandedCaptureRecord = LandedCaptureRecord {
        commit: cursor.text(true),
        transaction_id: cursor.text(true),
        worktree_id: captured.worktree_id,
        sequence: captured.sequence,
        next_sequence_after_landing: cursor.positive(),
        worktree_paths: paths,
    };
    let landed_text: String = encode_landed_capture_record(&landed);
    assert_eq!(
        parse_landed_capture_record(landed_text.as_str(), "n"),
        Ok(landed)
    );
    let raw: String = String::from_utf8_lossy(data).into_owned();
    if let Ok(record) = parse_captured_record(raw.as_str()) {
        assert!(
            record.sequence >= 1 && record.next_sequence_before_base >= 1,
            "{record:?}"
        );
        assert_eq!(
            parse_captured_record(encode_captured_record(&record).as_str()),
            Ok(record)
        );
    }
    if let Ok(record) = parse_landed_capture_record(raw.as_str(), "n") {
        assert_eq!(
            parse_landed_capture_record(encode_landed_capture_record(&record).as_str(), "n"),
            Ok(record)
        );
    }
    let path: &Path = Path::new("/s");
    if let Ok(value) = parse_sequence(data, path) {
        assert_eq!(data, format!("{value}\n").as_bytes());
    }
    let value: i64 = cursor.positive() - 1;
    assert_eq!(
        parse_sequence(format!("{value}\n").as_bytes(), path),
        Ok(value)
    );
    assert!(parse_sequence(format!("0{value}\n").as_bytes(), path).is_err());
}

/// Generator and invariant controls.
#[cfg(test)]
#[path = "journal_tests.rs"]
mod tests;
