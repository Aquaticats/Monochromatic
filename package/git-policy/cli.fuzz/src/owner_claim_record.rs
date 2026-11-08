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

/// Text pieces a generated birth identity or namespace draws from:
/// quotes,
/// escapes,
/// a space,
/// the separator character,
/// text that is not ASCII and empty.
/// No line feed: a generated claim keeps the line structure it is built with.
const PIECES: [&str; 9] = ["a", "\"", "\\", "\u{0}", "\u{e9}", "\u{1f600}", " ", "-", ""];

/// Lines a generated claim may carry after the last field it reads.
const TRAILING_LINES: [&str; 4] = ["pid=7", "released", "birth=x", "garbage"];

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
///  The process id field of a generated claim and the id it must read as,
/// or nothing when the field names no process.
/// Why:
///  Zero,
/// a negative id,
/// a fraction,
/// a value past the integer range,
/// a leading-zero spelling and text are the cases the reader splits between accepting and refusing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedPid(cursor: Cursor): [string, number | undefined];
/// ```
fn generated_pid(cursor: &mut Cursor<'_>) -> (String, Option<i64>) {
    return match cursor.byte() % 7 {
        0 => {
            let value: i64 = cursor.positive();
            (value.to_string(), Some(value))
        }
        1 => (String::from("0"), None),
        2 => (String::from("-1"), None),
        3 => (String::from("1.5"), None),
        4 => (String::from("9223372036854775808"), None),
        5 => (String::from("007"), Some(7)),
        _ => (String::from("seven"), None),
    };
}

/// What:
///  A generated claim file's text and the record it must hold,
/// or nothing when the text is not a claim of this command.
/// Why:
///  The expectation is decided by the chosen pieces,
/// never by the decoder under test.
/// The shapes are a whole claim,
/// one whose last field line is missing,
/// one holding only the process id,
/// and a whole claim with lines no reader of it may act on after its last field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedClaimText(data: Uint8Array): [string, ClaimRecord | undefined];
/// ```
pub fn generated_claim_text(data: &[u8]) -> (String, Option<ClaimRecord>) {
    let mut cursor: Cursor<'_> = Cursor::new(data);
    let header_choice: u8 = cursor.byte() % 4;
    let header: String = match header_choice {
        0 => header_of(DATA_FOLDER.current),
        1 => header_of(DATA_FOLDER.earlier),
        2 => format!("{}{HEADER_SUFFIX}", DATA_FOLDER.current.to_uppercase()),
        _ => String::from(HEADER_SUFFIX.trim_start()),
    };
    // Only the two folder names are a header; the capitalised and the folder-less spelling are what a hand edit looks like.
    let header_accepted: bool = header_choice < 2;
    let second_choice: u8 = cursor.byte() % 4;
    if second_choice == 2 || second_choice == 3 {
        let second: &str = if second_choice == 2 { "released " } else { "pid" };
        return (format!("{header}\n{second}\n"), None);
    }
    if second_choice == 0 {
        let mut text: String = format!("{header}\n{RELEASED_LINE}\n");
        if cursor.flip() {
            text.push_str(TRAILING_LINES[usize::from(cursor.byte()) % TRAILING_LINES.len()]);
            text.push('\n');
        }
        return (text, header_accepted.then_some(ClaimRecord::Released));
    }
    let (digits, pid): (String, Option<i64>) = generated_pid(&mut cursor);
    let empty_birth: bool = cursor.flip();
    let birth: String = if empty_birth { String::new() } else { cursor.text(true) };
    let namespace_choice: u8 = cursor.byte() % 3;
    let namespace: Option<String> = match namespace_choice {
        0 => None,
        1 => Some(String::new()),
        _ => Some(cursor.text(false)),
    };
    let shape: u8 = cursor.byte() % 4;
    let mut text: String = format!("{header}\npid={digits}\n");
    if shape == 3 {
        text.push_str(&format!("birth={birth}\nnamespace={}\n", namespace.as_deref().unwrap_or("-")));
        text.push_str(TRAILING_LINES[usize::from(cursor.byte()) % TRAILING_LINES.len()]);
        text.push('\n');
    } else if shape == 1 {
        text.push_str(&format!("birth={birth}\n"));
    } else if shape == 0 {
        text.push_str(&format!("birth={birth}\nnamespace={}\n", namespace.as_deref().unwrap_or("-")));
    }
    let complete: bool = (shape == 0 || shape == 3) && !empty_birth && namespace.is_some();
    let record: Option<ClaimRecord> = match (header_accepted, complete, pid) {
        (true, true, Some(value)) => Some(ClaimRecord::Owned { pid: value, birth, namespace }),
        _ => None,
    };
    return (text, record);
}

/// What:
///  The text a claim file for one record holds,
/// built here rather than by the subject's encoder.
/// Why:
///   A record an accepted file gave must be the record that text gives back,
/// and reading a file nobody edited writes that same text again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restatedText(record: ClaimRecord): string;
/// ```
fn restated_text(record: &ClaimRecord) -> String {
    let header: String = header_of(DATA_FOLDER.current);
    return match record {
        ClaimRecord::Released => format!("{header}\n{RELEASED_LINE}\n"),
        ClaimRecord::Owned { pid, birth, namespace } => format!(
            "{header}\npid={pid}\nbirth={birth}\nnamespace={}\n",
            namespace.as_deref().unwrap_or("-")
        ),
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
            birth.contains('\r') || namespace.as_deref().is_some_and(|text| text.contains('\r'))
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
    let header: String = text.lines().next().unwrap_or_default().to_string();
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
        decode(restated_text(&record).as_str()),
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
