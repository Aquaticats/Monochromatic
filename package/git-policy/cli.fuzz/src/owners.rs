//! What:
//! Generated owner tokens,
//! the file names that carry them,
//! transaction owner records and `/proc/<pid>/stat` lines,
//! with the invariants of their readers.
//! Why:
//! A copy's journal,
//! stage claim and temporary file name their owner in the file name,
//! and the transaction owner record is a file any process can write;
//! a stat line carries a command name the process chose.
//! A misread name,
//! record or start time would retire a live owner's item.
//! The owner lock record these targets once covered was removed with the settlement lock in commit `ee30d244f6`,
//! and the owner token is what replaced it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { token, expected } = generatedCopyOwnerToken(data); expect(CopyOwner.fromToken(token)).toEqual(expected);
//! ```

/// The file name suffixes and the readers of the owner token in a journal name and a temporary file name.
use git_policy_cli::worktree_copy_journal::temporary_writer_token;
/// The journal suffix,
/// the temporary suffix and the journal name reader.
use git_policy_cli::worktree_copy_journal_encoding_and_validation::{
    JOURNAL_SUFFIX, TEMPORARY_SUFFIX, owner_token_of,
};
/// The copy owner and its token codec under test.
use git_policy_cli::worktree_copy_owner::CopyOwner;
/// The path type the journal name reader takes.
use std::path::Path;
/// The one borrowed file-name type a path's last component is read as.
use std::ffi::OsStr;
/// The `/proc/<pid>/stat` reader under test.
use git_policy_cli::process_identity::linux_identity_from_stat;
/// The transaction owner record codec under test.
use git_policy_cli::transaction_owner::{
    TransactionOwner, encode_transaction_owner, parse_transaction_owner,
};

/// The largest integer JavaScript represents exactly.
const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

/// Text pieces generated strings draw from:
/// quotes,
/// escapes,
/// controls,
/// non-ASCII and empty.
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

/// What:
/// `impl Cursor` attaches the reads.
/// Why:
///  Generators draw small choices from the input in order.
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

    /// The next byte,
    /// or zero after the input ends.
    pub fn byte(&mut self) -> u8 {
        match self.rest.split_first() {
            Some((first, rest)) => {
                self.rest = rest;
                return *first;
            }
            None => return 0,
        }
    }

    /// One choice between two cases,
    /// from the low bit of the next byte.
    pub fn flip(&mut self) -> bool {
        return self.byte() & 1 == 0;
    }

    /// Text of up to seven pieces;
    /// `nonempty` adds a leading letter.
    pub fn text(&mut self, nonempty: bool) -> String {
        let mut text: String = String::from(if nonempty { "t" } else { "" });
        for _ in 0..self.byte() % 8 {
            text.push_str(PIECES[usize::from(self.byte()) % PIECES.len()]);
        }
        return text;
    }

    /// A positive safe integer,
    /// small or at the edge of the safe range.
    pub fn positive(&mut self) -> i64 {
        let choice: u8 = self.byte();
        if choice & 3 == 0 {
            return MAX_SAFE_INTEGER - i64::from(self.byte() % 3);
        }
        return 1 + i64::from(choice) * i64::from(self.byte());
    }
}

/// What:
/// Field mutations of a valid owner token's three fields (process id,
/// escaped birth identity,
/// escaped namespace):
/// which field is edited,
/// the text appended or put in its place,
/// and whether the token must still be accepted.
/// Why:
///  Each names one rule of the token reader.
#[derive(Clone, Copy)]
enum TokenEdit {
    /// The process id field becomes this text.
    Pid(&'static str),
    /// This text follows the birth identity field.
    AfterBirth(&'static str),
    /// The birth identity field becomes this text.
    Birth(&'static str),
    /// The first character of the birth identity (a plain `t`) is spelled as the escape this text holds.
    RespelledBirthStart(&'static str),
    /// This text follows the namespace field.
    AfterNamespace(&'static str),
    /// The namespace field and its separator are removed.
    NoNamespaceField,
}

/// The edits one generated token is put through,
/// each with whether the token must still be accepted.
const TOKEN_EDITS: [(TokenEdit, bool); 13] = [
    (TokenEdit::Pid("0"), false),
    (TokenEdit::Pid("-1"), false),
    (TokenEdit::Pid("1.5"), false),
    (TokenEdit::Pid("9223372036854775808"), false),
    (TokenEdit::Pid(""), false),
    (TokenEdit::Pid("seven"), false),
    (TokenEdit::Birth(""), false),
    (TokenEdit::AfterBirth("%"), false),
    (TokenEdit::AfterBirth("%zz"), false),
    (TokenEdit::AfterBirth("%ff"), false),
    (TokenEdit::AfterNamespace("~x"), false),
    (TokenEdit::NoNamespaceField, false),
    (TokenEdit::RespelledBirthStart("%74"), true),
];

/// The characters an owner token may hold:
/// it sits between dots in a file name.
fn token_character_is_allowed(character: char) -> bool {
    return character.is_ascii_alphanumeric() || ":-[]~%".contains(character);
}

/// What:
/// An owner drawn from the input,
/// with a positive process id,
/// a non-empty birth identity and a namespace that is absent or non-empty.
/// Why:
///  Those are the owners a running process can have;
/// the token codec must carry every one of them,
/// including identities holding quotes,
/// controls,
/// separators and non-ASCII text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedCopyOwner(cursor: Cursor): CopyOwner;
/// ```
fn generated_copy_owner(cursor: &mut Cursor<'_>) -> CopyOwner {
    return CopyOwner {
        pid: cursor.positive(),
        birth_identity: cursor.text(true),
        pid_namespace: if cursor.flip() {
            None
        } else {
            Some(cursor.text(true))
        },
    };
}

/// What:
/// A generated owner,
/// the token text built from it and edited once,
/// and the owner that text must name (or none).
/// Why:
///  The expected outcome is decided by the edit,
/// never by the reader under test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedCopyOwnerToken(data: Uint8Array): { token: string; expected?: CopyOwner };
/// ```
pub fn generated_copy_owner_token(data: &[u8]) -> (String, Option<CopyOwner>) {
    let mut cursor: Cursor<'_> = Cursor::new(data);
    let owner: CopyOwner = generated_copy_owner(&mut cursor);
    let token: String = owner.token();
    let fields: Vec<&str> = token.split('~').collect();
    let [pid, birth, namespace] = fields.as_slice() else {
        panic!("an owner token has three fields: {token}");
    };
    let choice: usize = usize::from(cursor.byte()) % (TOKEN_EDITS.len() + 1);
    let Some((edit, accepted)) = TOKEN_EDITS.get(choice) else {
        return (token.clone(), Some(owner));
    };
    let edited: String = match edit {
        TokenEdit::Pid(text) => format!("{text}~{birth}~{namespace}"),
        TokenEdit::AfterBirth(text) => format!("{pid}~{birth}{text}~{namespace}"),
        TokenEdit::Birth(text) => format!("{pid}~{text}~{namespace}"),
        TokenEdit::RespelledBirthStart(text) => {
            // A generated birth identity starts with the plain letter `t`.
            let rest: &str = birth.strip_prefix('t').unwrap_or_default();
            format!("{pid}~{text}{rest}~{namespace}")
        }
        TokenEdit::AfterNamespace(text) => format!("{pid}~{birth}~{namespace}{text}"),
        TokenEdit::NoNamespaceField => format!("{pid}~{birth}"),
    };
    if *accepted {
        return (edited, Some(owner));
    }
    return (edited, None);
}

/// What:
/// The owner token invariants on any text:
/// an accepted token names a positive process id and a non-empty birth identity,
/// and survives being restated.
/// Why:
///  A reader accepting a name an owner never wrote could hide a live copy from the recovery that takes dead owners' items,
/// or take a live owner's item.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkCopyOwnerToken(text: string): void;
/// ```
pub fn check_copy_owner_token(text: &str) {
    let Some(owner) = CopyOwner::from_token(text) else {
        return;
    };
    assert!(owner.pid >= 1, "{owner:?}");
    assert!(!owner.birth_identity.is_empty(), "{owner:?}");
    assert_ne!(owner.pid_namespace.as_deref(), Some(""), "{owner:?}");
    let restated: String = owner.token();
    assert!(
        restated.chars().all(token_character_is_allowed),
        "{restated}"
    );
    assert_eq!(CopyOwner::from_token(restated.as_str()), Some(owner));
}

/// What:
/// The generated token names exactly its expected owner,
/// the owner's own token is found again by the file name readers in the names built around it,
/// and holds only characters a name part can carry.
/// Why:
///  The fuzz target and its controls run the same check.
///  A dot or a slash inside a token would move the owner out of its place in the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGeneratedCopyOwnerToken(data: Uint8Array): void;
/// ```
pub fn check_generated_copy_owner_token(data: &[u8]) {
    let (token, expected) = generated_copy_owner_token(data);
    assert_eq!(CopyOwner::from_token(token.as_str()), expected, "{token}");
    check_copy_owner_token(token.as_str());
    let mut cursor: Cursor<'_> = Cursor::new(data);
    let owner: CopyOwner = generated_copy_owner(&mut cursor);
    let built: String = owner.token();
    assert!(built.chars().all(token_character_is_allowed), "{built}");
    assert_eq!(CopyOwner::from_token(built.as_str()), Some(owner));
    // The names an owner is written into: `<id>.<token>.json` and `<item>.<token>.<unique>.tmp`.
    let journal: String = format!("{}.{built}{JOURNAL_SUFFIX}", cursor.text(false));
    assert_eq!(
        owner_token_of(Path::new(journal.as_str())),
        Some(built.clone()),
        "{journal}"
    );
    let unique: String = cursor.positive().to_string();
    let temporary: String = format!("{}.{built}.{unique}{TEMPORARY_SUFFIX}", cursor.text(true));
    assert_eq!(
        temporary_writer_token(temporary.as_str()),
        Some(built.as_str()),
        "{temporary}"
    );
}

/// What:
/// The file name readers' invariants on any text:
/// a journal name or a temporary name that gives a token ends in its suffix,
/// and the token is a dot-free part of the name.
/// Why:
///  A stranger's file name must name no owner by accident.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkOwnerFileNames(text: string): void;
/// ```
pub fn check_owner_file_names(name: &str) {
    // The journal reader takes a path, so the name it reads a token from is the last component: a trailing separator
    // names the component before it.
    let component: &str = Path::new(name).file_name().and_then(OsStr::to_str).unwrap_or(name);
    if let Some(token) = owner_token_of(Path::new(name)) {
        assert!(component.ends_with(JOURNAL_SUFFIX), "{name}");
        assert!(!token.is_empty(), "{name}");
        assert!(!token.contains('.') && !token.contains('/'), "{name}");
        assert!(name.contains(token.as_str()), "{name}");
    }
    if let Some(token) = temporary_writer_token(name) {
        assert!(name.ends_with(TEMPORARY_SUFFIX), "{name}");
        assert!(!token.is_empty(), "{name}");
        assert!(!token.contains('.'), "{name}");
        assert!(name.contains(token), "{name}");
    }
}

/// What:
/// The transaction owner invariants on any bytes,
/// and the round trip of a generated owner.
/// Why:
///  A published directory's owner decides whether recovery may touch it.
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

/// What:
/// A generated stat line and the identity it must give:
/// `Ok(Some)` for a running process,
///       `Ok(None)` for a zombie or dead one,
/// `Err(())` for a line without the start time.
/// Why:
///  The command name may hold spaces and parentheses;
/// only the last `)` ends it.
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

/// What:
/// The stat reader gives exactly the generated outcome,
/// and any text either fails or names
///       a non-empty `linux:` start time.
/// Why:
///  The fuzz target and its controls run the same check.
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
