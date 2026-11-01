//! What: Generated owner records and `/proc/<pid>/stat` lines, with the invariants of their
//!       parsers.
//! Why: Owner lock records and transaction owner records are files any process can write, and a
//!      stat line carries a command name the process chose; a misread record or start time
//!      would retire a live owner's lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { text, expected } = generatedOwnerLock(data); expect(parseOwnerLockRecord(text)).toEqual(expected);
//! ```

/// The owner lock record codec under test.
use git_policy_cli::owner_lock_record::{
    OwnerLockRecord, encode_owner_lock_record, parse_owner_lock_record,
};
/// The `/proc/<pid>/stat` reader under test.
use git_policy_cli::process_identity::linux_identity_from_stat;
/// The transaction owner record codec under test.
use git_policy_cli::transaction_owner::{
    TransactionOwner, encode_transaction_owner, parse_transaction_owner,
};

/// The largest integer JavaScript represents exactly.
const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

/// Text pieces generated strings draw from: quotes, escapes, controls, non-ASCII and empty.
const PIECES: [&str; 10] = [
    "a",
    "\"",
    "\\",
    "\n",
    "\u{0}",
    "\u{e9}",
    "\u{1f600}",
    " ",
    "-",
    "",
];

/// A byte cursor over fuzz input that yields zero once exhausted.
pub struct Cursor<'a> {
    /// The remaining bytes.
    rest: &'a [u8],
}

/// What: `impl Cursor` attaches the reads.
/// Why:  Generators draw small choices from the input in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Cursor { next(): number; text(): string }
/// ```
impl<'a> Cursor<'a> {
    /// A cursor over `data`.
    pub fn new(data: &'a [u8]) -> Self {
        return Self { rest: data };
    }

    /// The next byte, or zero after the input ends.
    pub fn byte(&mut self) -> u8 {
        match self.rest.split_first() {
            Some((first, rest)) => {
                self.rest = rest;
                return *first;
            }
            None => return 0,
        }
    }

    /// One choice between two cases, from the low bit of the next byte.
    pub fn flip(&mut self) -> bool {
        return self.byte() & 1 == 0;
    }

    /// Text of up to seven pieces; `nonempty` adds a leading letter.
    pub fn text(&mut self, nonempty: bool) -> String {
        let mut text: String = String::from(if nonempty { "t" } else { "" });
        for _ in 0..self.byte() % 8 {
            text.push_str(PIECES[usize::from(self.byte()) % PIECES.len()]);
        }
        return text;
    }

    /// A positive safe integer, small or at the edge of the safe range.
    pub fn positive(&mut self) -> i64 {
        let choice: u8 = self.byte();
        if choice & 3 == 0 {
            return MAX_SAFE_INTEGER - i64::from(self.byte() % 3);
        }
        return 1 + i64::from(choice) * i64::from(self.byte());
    }
}

/// What: Field mutations of a valid owner lock record: the text replaced once, its replacement,
///       and whether the record must still be accepted.
/// Why:  Each names one rule of the parser.
const LOCK_MUTATIONS: [(&str, &str, bool); 12] = [
    ("\"ownerPid\":", "\"ownerPid\":0,\"x\":", false),
    ("\"ownerPid\":", "\"ownerPid\":-1,\"x\":", false),
    ("\"ownerPid\":", "\"ownerPid\":1.5,\"x\":", false),
    (
        "\"ownerPid\":",
        "\"ownerPid\":9007199254740992,\"x\":",
        false,
    ),
    ("\"ownerPid\":", "\"ownerPid\":\"7\",\"x\":", false),
    ("\"token\":", "\"token\":\"\",\"x\":", false),
    ("\"token\":", "\"tokem\":", false),
    (
        "\"ownerBirthIdentity\":",
        "\"ownerBirthIdentity\":\"\",\"x\":",
        false,
    ),
    ("\"schemaVersion\":1", "\"schemaVersion\":2", false),
    ("\"schemaVersion\":1", "\"schemaVersion\":1.0", true),
    ("{", "{\"transactionId\":7,", false),
    ("{", "{\"extra\":[1,{}],", true),
];

/// What: A generated owner lock record's text, the record it must parse to (or none), and the
///       record it was built from.
/// Why:  The expected outcome is decided by the mutation, never by the parser under test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedOwnerLock(data: Uint8Array): { text: string; expected?: OwnerLockRecord };
/// ```
pub fn generated_owner_lock(data: &[u8]) -> (String, Option<OwnerLockRecord>) {
    let mut cursor: Cursor<'_> = Cursor::new(data);
    let record: OwnerLockRecord = OwnerLockRecord {
        token: cursor.text(true),
        owner_pid: cursor.positive(),
        owner_birth_identity: cursor.text(true),
        transaction_id: if cursor.flip() {
            None
        } else {
            Some(cursor.text(true))
        },
        owner_pid_namespace: if cursor.flip() {
            None
        } else {
            Some(cursor.text(true))
        },
    };
    let text: String = encode_owner_lock_record(&record);
    let choice: usize = usize::from(cursor.byte()) % (LOCK_MUTATIONS.len() + 1);
    let Some((from, to, accepted)) = LOCK_MUTATIONS.get(choice) else {
        return (text, Some(record));
    };
    let mutated: String = text.replacen(from, to, 1);
    // A `transactionId` added before an existing one is overwritten by the later key, as
    // `JSON.parse` keeps the last of duplicate keys.
    let overridden: bool = to.contains("transactionId") && record.transaction_id.is_some();
    if !accepted && !overridden {
        return (mutated, None);
    }
    return (mutated, Some(record));
}

/// What: The owner lock record invariants on any text: an accepted record has a positive safe
///       PID and non-empty token and identity, and survives being restated.
/// Why:  A parser accepting what the incumbent refuses would let a malformed lock look owned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkOwnerLockText(text: string): void;
/// ```
pub fn check_owner_lock_text(text: &str) {
    let Some(record) = parse_owner_lock_record(text) else {
        return;
    };
    assert!(
        (1..=MAX_SAFE_INTEGER).contains(&record.owner_pid),
        "{record:?}"
    );
    assert!(
        !record.token.is_empty() && !record.owner_birth_identity.is_empty(),
        "{record:?}"
    );
    assert_ne!(record.transaction_id.as_deref(), Some(""), "{record:?}");
    let restated: String = encode_owner_lock_record(&record);
    assert_eq!(parse_owner_lock_record(restated.as_str()), Some(record));
}

/// What: The generated owner lock record parses to exactly its expected outcome.
/// Why:  The fuzz target and its controls run the same check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGeneratedOwnerLock(data: Uint8Array): void;
/// ```
pub fn check_generated_owner_lock(data: &[u8]) {
    let (text, expected) = generated_owner_lock(data);
    assert_eq!(parse_owner_lock_record(text.as_str()), expected, "{text}");
    check_owner_lock_text(text.as_str());
}

/// What: The transaction owner invariants on any bytes, and the round trip of a generated owner.
/// Why:  A published directory's owner decides whether recovery may touch it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkTransactionOwner(data: Uint8Array): void;
/// ```
pub fn check_transaction_owner(data: &[u8]) {
    if let Some(owner) = parse_transaction_owner(data) {
        assert!(
            (1..=MAX_SAFE_INTEGER).contains(&owner.owner_pid),
            "{owner:?}"
        );
        assert!(!owner.owner_identity.is_empty(), "{owner:?}");
        assert_eq!(
            parse_transaction_owner(encode_transaction_owner(&owner).as_bytes()),
            Some(owner)
        );
    }
    let mut cursor: Cursor<'_> = Cursor::new(data);
    let generated: TransactionOwner = TransactionOwner {
        transaction_id: cursor.text(false),
        owner_pid: cursor.positive(),
        owner_identity: cursor.text(true),
        created_at: cursor.text(false),
        owner_pid_namespace: if cursor.flip() {
            None
        } else {
            Some(cursor.text(true))
        },
    };
    let encoded: String = encode_transaction_owner(&generated);
    assert_eq!(parse_transaction_owner(encoded.as_bytes()), Some(generated));
}

/// What: A generated stat line and the identity it must give: `Ok(Some)` for a running process,
///       `Ok(None)` for a zombie or dead one, `Err(())` for a line without the start time.
/// Why:  The command name may hold spaces and parentheses; only the last `)` ends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedStat(data: Uint8Array): { line: string; expected: string | null | 'malformed' };
/// ```
pub fn generated_stat(data: &[u8]) -> (String, Result<Option<String>, ()>) {
    let mut cursor: Cursor<'_> = Cursor::new(data);
    let mut command: String = String::new();
    for _ in 0..cursor.byte() % 6 {
        command.push_str(["a", " ", ")", "(", ") 1 2", "\u{e9}"][usize::from(cursor.byte()) % 6]);
    }
    let state: &str = ["R", "S", "D", "T", "Z", "X", "I"][usize::from(cursor.byte()) % 7];
    let tick: String = cursor.positive().to_string();
    let mut fields: Vec<String> = vec![String::from(state)];
    for index in 1..19 {
        fields.push((index * 3).to_string());
    }
    fields.push(tick.clone());
    for _ in 0..cursor.byte() % 4 {
        fields.push(String::from("0"));
    }
    let shape: u8 = cursor.byte() % 4;
    if shape == 1 {
        // The line ends before the start time.
        fields.truncate(usize::from(cursor.byte()) % 19 + 1);
    }
    let line: String = format!("{} ({command}) {}\n", cursor.positive(), fields.join(" "));
    let exited: bool = state == "Z" || state == "X";
    if shape == 2 {
        return (line.replace(')', "]"), Err(()));
    }
    if exited {
        return (line, Ok(None));
    }
    if shape == 1 {
        return (line, Err(()));
    }
    return (line, Ok(Some(format!("linux:{tick}"))));
}

/// What: The stat reader gives exactly the generated outcome, and any text either fails or names
///       a non-empty `linux:` start time.
/// Why:  The fuzz target and its controls run the same check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkStat(data: Uint8Array): void;
/// ```
pub fn check_stat(data: &[u8]) {
    let (line, expected) = generated_stat(data);
    let found: Result<Option<String>, ()> = match linux_identity_from_stat(line.as_str()) {
        Ok(identity) => Ok(identity),
        Err(_) => Err(()),
    };
    assert_eq!(found, expected, "{line:?}");
    let raw: String = String::from_utf8_lossy(data).into_owned();
    if let Ok(Some(identity)) = linux_identity_from_stat(raw.as_str()) {
        assert!(
            identity.len() > "linux:".len() && identity.starts_with("linux:"),
            "{identity}"
        );
        assert!(!identity.contains(' '), "{identity}");
    }
}

/// Generator and invariant controls.
#[cfg(test)]
#[path = "owners_tests.rs"]
pub(crate) mod tests;
