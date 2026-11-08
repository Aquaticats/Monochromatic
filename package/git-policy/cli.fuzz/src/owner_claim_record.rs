//! What:
//!  Generated text of a single-owner claim file,
//! and the invariants of the decoder that decides what it records.
//! Why:
//!  A claim file is text any process can write,
//! and it decides whether a command may work on one item.
//! A claim read as released,
//! or as another owner's,
//! lets a second command work on the item another one holds;
//! a claim read as nobody's lets recovery take a live command's item.
//! The decoder lives in
//!  `package/git-policy/cli/src/native/lock/single_owner_claim.rs`
//! and is reached here through its own `decode` and `ClaimRecord`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { text, expected } = generatedClaimText(data); expect(decodeClaimText(text)).toEqual(expected);
//! ```

/// The generator cursor shared with the owner and index-lock records.
use super::owners::Cursor;
/// The decoder under test and the record it returns.
use git_policy_cli::single_owner_claim::{ClaimRecord, decode};
/// The folder names a claim file's first line is spelled with,
/// the current release's first.
use git_policy_cli::wrapper_names_old_and_new::DATA_FOLDER;

/// The rest of a claim file's first line after the folder name.
/// Restated here instead of read from the subject,
/// so one spelling drifting from the other fails the controls instead of passing silently.
const HEADER_SUFFIX: &str = " owner-claim v1";

/// The line a released claim holds instead of an owner.
const RELEASED_LINE: &str = "released";

/// Lines a generated claim may carry after the last field it reads.
const TRAILING_LINES: [&str; 4] = ["pid=7", "released", "birth=x", "garbage"];

/// Spellings of a process id field that name no process:
/// zero,
/// a negative id,
/// a fraction,
/// nothing,
/// text and a value past the integer range.
const NO_PROCESS_PIDS: [&str; 6] = ["0", "-1", "1.5", "", "seven", "9223372036854775808"];

/// What:
///  The first line of a claim file written under one folder name.
/// Why:
///   A claim of an earlier release is read in its own spelling,
/// so the current and the earlier name are the two first lines the decoder accepts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const headerOf = (folder: string) => `${folder}${HEADER_SUFFIX}`;
/// ```
fn header_of(folder: &str) -> String {
    return format!("{folder}{HEADER_SUFFIX}");
}

/// What:
///  A generated birth identity's or namespace's text.
/// Why:
///   The shared cursor's pieces hold a line feed,
/// and a claim file's field is exactly one line:
/// a line feed inside a field would end it and make the claim another one,
/// so the field is written without one and the expected record holds the text the file holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldText = (cursor: Cursor, nonempty: boolean) => cursor.text(nonempty).replaceAll('\n', ' ');
/// ```
fn field_text(cursor: &mut Cursor<'_>, nonempty: bool) -> String {
    return cursor.text(nonempty).replace('\n', " ");
}

/// What:
///  A generated owner:
/// a process id of at least one,
/// a birth identity and a namespace that is absent,
/// empty or text.
/// Why:
///  Those are the owners a running process can have,
/// and the claim must carry every one of them,
/// including identities holding quotes,
/// controls,
/// spaces and non-ASCII text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownedRecord(cursor: Cursor): ClaimRecord;
/// ```
fn owned_record(cursor: &mut Cursor<'_>) -> ClaimRecord {
    let pid: i64 = cursor.positive();
    let birth: String = field_text(cursor, true);
    let namespace: Option<String> = match cursor.byte() % 3 {
        0 => None,
        1 => Some(String::new()),
        _ => {
            // The file writes an absent namespace as `-`, so a text of exactly that marker reads back as none.
            let text: String = field_text(cursor, false);
            if text == "-" { None } else { Some(text) }
        }
    };
    return ClaimRecord::Owned { pid, birth, namespace };
}

/// What:
///  The namespace line's value for an owner,
/// the released marker when the owner's namespace is absent.
/// Why:
///   The file writes an unknown namespace as `-`,
/// which reads back as no namespace at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namespaceLine = (namespace?: string) => namespace ?? '-';
/// ```
fn namespace_text(namespace: Option<&str>) -> &str {
    return namespace.unwrap_or("-");
}

/// What:
///  A whole claim file's text for one record,
/// written under one first line.
/// Why:
///   A generated whole claim must hold exactly the record it was built from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function claimText(header: string, record: ClaimRecord): string;
/// ```
fn claim_text(header: &str, record: &ClaimRecord) -> String {
    return match record {
        ClaimRecord::Released => format!("{header}\n{RELEASED_LINE}\n"),
        ClaimRecord::Owned { pid, birth, namespace } => format!(
            "{header}\npid={pid}\nbirth={birth}\nnamespace={}\n",
            namespace_text(namespace.as_deref())
        ),
    };
}

/// What:
///  A generated claim file's text and the record it must hold,
/// or nothing when the text is not a claim of this command.
/// Why:
///  The expectation is decided by the chosen shape,
/// never by the decoder under test,
/// and every shape is built so its outcome cannot depend on which byte
///  happened to hold which choice.
/// The shapes are a released claim,
/// the same followed by a line no reader acts on,
/// a whole owned claim,
/// the same followed by such a line,
/// an owned claim one field edit leaves naming no process,
/// and a released line a trailing space makes a near miss.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedClaimText(data: Uint8Array): [string, ClaimRecord | undefined];
/// ```
pub fn generated_claim_text(data: &[u8]) -> (String, Option<ClaimRecord>) {
    let mut cursor: Cursor<'_> = Cursor::new(data);
    // Three first lines are the current or the earlier folder's name; the capitalised and the folder-less one are what a hand edit looks like.
    let header_edit: u8 = cursor.byte() % 5;
    let header: String = match header_edit {
        0 | 1 => header_of(DATA_FOLDER.current),
        2 => header_of(DATA_FOLDER.earlier),
        3 => header_of(DATA_FOLDER.current.to_uppercase().as_str()),
        _ => String::from(HEADER_SUFFIX.trim_start()),
    };
    let header_accepted: bool = header_edit < 3;
    let shape: u8 = cursor.byte() % 6;
    if shape == 5 {
        return (format!("{header}\n{RELEASED_LINE} \n"), None);
    }
    if shape == 4 {
        return (broken_owned_claim(header.as_str(), &mut cursor), None);
    }
    let released: bool = shape < 2;
    let record: ClaimRecord = if released { ClaimRecord::Released } else { owned_record(&mut cursor) };
    let mut text: String = claim_text(header.as_str(), &record);
    if shape == 1 || shape == 3 {
        text.push_str(TRAILING_LINES[usize::from(cursor.byte()) % TRAILING_LINES.len()]);
        text.push('\n');
    }
    return (text, header_accepted.then_some(record));
}

/// What:
///  An owned claim whose process id,
/// birth identity or namespace line is edited so that it names no process.
/// Why:
///  Each edit is one refusal rule of the decoder,
/// and none of them may still read as an owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function brokenOwnedClaim(header: string, cursor: Cursor): string;
/// ```
fn broken_owned_claim(header: &str, cursor: &mut Cursor<'_>) -> String {
    let owner: ClaimRecord = owned_record(cursor);
    let ClaimRecord::Owned { pid, birth, namespace } = &owner else {
        panic!("a generated owner is an owned claim: {owner:?}");
    };
    let namespace_line: &str = namespace_text(namespace.as_deref());
    return match cursor.byte() % 3 {
        0 => {
            let no_process: &str = NO_PROCESS_PIDS[usize::from(cursor.byte()) % NO_PROCESS_PIDS.len()];
            format!("{header}\npid={no_process}\nbirth={birth}\nnamespace={namespace_line}\n")
        }
        1 => format!("{header}\npid={pid}\nbirth=\nnamespace={namespace_line}\n"),
        _ => format!("{header}\npid={pid}\nbirth={birth}\n"),
    };
}

/// What:
///  Whether a record holds a carriage return in one of its text fields.
/// Why:
///   Line splitting removes one trailing carriage return from every line,
/// so a hand-written file can give a birth identity or namespace the text restated from it does not give back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsCarriageReturn = (record: ClaimRecord) => /[\r]/u.test(JSON.stringify(record));
/// ```
fn holds_carriage_return(record: &ClaimRecord) -> bool {
    return match record {
        ClaimRecord::Released => false,
        ClaimRecord::Owned { birth, namespace, .. } => {
            birth.contains('\r') || namespace.as_deref().is_some_and(|text| return text.contains('\r'))
        }
    };
}

/// What:
///  The claim invariants on any text:
/// an accepted claim names either a released claim or a live process,
/// and a record its own text restates is the record that text gives back.
/// Why:
///  A decoder that accepted a claim naming no owner,
/// or a claim whose own text it reads differently,
/// would decide liveness from what no command wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkClaimText(text: string): void;
/// ```
pub fn check_claim_text(text: &str) {
    let Some(record) = decode(text) else { return };
    let header: &str = text.lines().next().unwrap_or_default();
    assert!(
        header == header_of(DATA_FOLDER.current) || header == header_of(DATA_FOLDER.earlier),
        "{header:?}"
    );
    match &record {
        ClaimRecord::Released => {}
        ClaimRecord::Owned { pid, birth, namespace } => {
            assert!(*pid >= 1, "{record:?}");
            assert!(!birth.is_empty(), "{record:?}");
            assert_ne!(namespace.as_deref(), Some("-"), "{record:?}");
        }
    }
    if holds_carriage_return(&record) {
        return;
    }
    assert_eq!(
        decode(claim_text(header_of(DATA_FOLDER.current).as_str(), &record).as_str()),
        Some(record),
        "{text:?}"
    );
}

/// What:
///  A generated claim file holds exactly its expected record,
/// and every expected record holds the claim invariants.
/// Why:
///  The fuzz target and its controls run the same check,
/// so the spellings the controls count are the spellings the target explores.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGeneratedClaimText(data: Uint8Array): void;
/// ```
pub fn check_generated_claim_text(data: &[u8]) {
    let (text, expected): (String, Option<ClaimRecord>) = generated_claim_text(data);
    assert_eq!(decode(text.as_str()), expected, "{text:?}");
    check_claim_text(text.as_str());
}

/// Generator and invariant controls.
#[cfg(test)]
#[path = "owner_claim_record_tests.rs"]
pub(crate) mod tests;
