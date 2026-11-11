//! What:
//!  Pure decision of the shared-remote-settings refusal:
//!  whether a command changes remote settings every worktree of the repository shares,
//! and which settings it would change.
//! Why:
//!  One `.git/config` is shared by the main checkout and every linked worktree,
//!  so a `remote set-url` run in a
//!       worktree changes the push address of every other checkout;
//!  once it made every push of the main checkout
//!       fail and nothing recorded which command did it
//!       (`doc/handover/questions-for-the-owner.md`,
//!  "A push URL changed from a
//!       worktree").
//!  This file decides from the arguments alone which spelling of `git remote` and
//!  `git config` writes that file;
//!  it starts no process and reads no file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sharedRemoteWrite('remote', ['set-url', 'origin', url]) === { kind: 'remote-subcommand', name: 'set-url' }
//! ```

/// What:
///  Bring the two command readers and their refusals into this file.
/// Why:
///   Which spelling is an option,
/// a scope or a subcommand is Git's grammar,
///  and each command's module
///       already holds that answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseConfigRegion, ConfigFile, ConfigWrite } from './command_config.ts';
/// import { parseRemoteRegion } from './command_remote.ts';
/// ```
use crate::command_config::{ConfigFile, ConfigRegion, ConfigWrite, parse_config_region};
use crate::command_remote::{RemoteRegion, parse_remote_region};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path of the same bytes.
use std::path::PathBuf;

/// What:
///  The finding code of a shared-remote-settings refusal.
///  `&str` is borrowed text baked into the program.
/// Why:
///   Callers and transcripts identify the finding by this stable code,
///  which `event_hints.rs` lists as
///       carrying its own hint.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHARED_REMOTE_SETTINGS_CODE = 'shared-remote-settings';
/// ```
pub const SHARED_REMOTE_SETTINGS_CODE: &str = "shared-remote-settings";

/// What:
///  The section of the configuration file every remote setting belongs to.
///  `&[u8]` borrows its bytes.
/// Why:
///   `remote.origin.url` and `remote "origin"` are two spellings of the same section,
///  and Git
///       compares section names without case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REMOTE_SECTION = Buffer.from('remote');
/// ```
const REMOTE_SECTION: &[u8] = b"remote";

/// What:
///  Every `git remote` subcommand that writes the shared configuration.
///  `&[(&[u8], &str)]` is a list of the spelling Git dispatches to and the word a
///       diagnostic prints.
/// Why:
///   The other subcommands read (`get-url`,
///  `show`) or update remote-tracking refs (`prune`,
///       `update`),
///  and a refusal must never cover a read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WRITING_SUBCOMMANDS: readonly [Buffer, string][] = [[Buffer.from('set-url'), 'set-url'], /* ... */];
/// ```
const WRITING_SUBCOMMANDS: &[(&[u8], &str)] = &[
    (b"set-url", "set-url"),
    (b"add", "add"),
    (b"remove", "remove"),
    (b"rm", "rm"),
    (b"rename", "rename"),
    (b"set-branches", "set-branches"),
    (b"set-head", "set-head"),
];

/// What:
///  The setting a refused form would change,
///  named the way a diagnostic names it.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A remote subcommand,
///  one configuration key and a whole section are different things to say,
///  and
///       the refusal names the one it is about;
///  an editor session names none of them and can change
///       any of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SharedRemoteWrite = { kind: 'remote-subcommand'; name: string }
///   | { kind: 'key'; key: string } | { kind: 'section'; section: string } | { kind: 'edit' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SharedRemoteWrite {
    /// A `git remote` subcommand that writes the configuration,
    ///  as Git spells it.
    RemoteSubcommand(&'static str),
    /// A `git config` write to one key under the `remote` section.
    Key(String),
    /// A `git config` write to a whole section under `remote.`.
    Section(String),
    /// `git config --edit`,
    ///  which names no key and can change every one of them.
    Edit,
}

/// What:
///  The write a command would make to the remote settings every worktree of the repository
///  shares,
/// and the file it names for it.
///  `Option<PathBuf>` is the path the command named,
///  when it named one.
/// Why:
///   One `.git/config` is shared by the main checkout and every linked worktree,
///  while a
///       `git config --file` may name that file or a file of its own;
///  only a caller that knows the
///       working directory can resolve the path,
///  so the decision reports it and the caller compares it with
///       `$GIT_COMMON_DIR/config`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SharedRemoteChange = { setting: SharedRemoteWrite; namedFile?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedRemoteChange {
    /// The setting the command would change.
    pub setting: SharedRemoteWrite,
    /// The file the command names with `--file`,
    ///  or nothing when it writes the repository's own file.
    pub named_file: Option<PathBuf>,
}

/// What:
///  The word a diagnostic prints for a subcommand Git would dispatch to,
///  or nothing when the subcommand writes nothing.
/// Why:
///   Git matches the subcommand words exactly,
///  without abbreviation,
///  so one byte comparison per
///       word is the whole reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writingSubcommand = (word: Buffer): string | undefined => WRITING_SUBCOMMANDS.find(([spelling]) => spelling.equals(word))?.[1];
/// ```
fn writing_subcommand(word: &[u8]) -> Option<&'static str> {
    for (spelling, name) in WRITING_SUBCOMMANDS {
        if *spelling == word {
            // `Some(x)` is the "present" case of `Option`.
            return Some(name);
        }
    }
    // `None` is the "absent" case of `Option`.
    return None;
}

/// What:
///  Whether a configuration key or section name belongs to the `remote` section.
///  `&[u8]` borrows the token Git reads as a name.
/// Why:
///   A key is written `section.subsection.name` and a section `section.subsection`,
///  so the section
///       is the bytes before the first dot;
///  Git compares section names without case and keeps the subsection's
///       case,
///  so `Remote.Origin.URL` writes the same section as `remote.origin.url`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesRemote = (token: Buffer): boolean => token.toString().split('.')[0].toLowerCase() === 'remote';
/// ```
fn names_remote_section(token: &[u8]) -> bool {
    // `.split(...)` yields the part before the first dot; `.next()` takes it.
    let section: &[u8] = match token.split(is_dot).next() {
        Some(found) => found,
        None => token,
    };
    return section.eq_ignore_ascii_case(REMOTE_SECTION);
}

/// What:
///  Whether one byte is the dot that separates a key's section,
///  its subsection and its name.
/// Why:
///   `split` takes the predicate itself,
///  so the test is a named function instead of a closure that
///       captures nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isDot = (byte: number): boolean => byte === 0x2e;
/// ```
fn is_dot(byte: &u8) -> bool {
    return *byte == b'.';
}

/// What:
///  The text of one region token,
///  for a diagnostic that names it.
/// Why:
///   A key or section name is normally text;
///  `String::from_utf8_lossy` keeps a name that
///       is not text readable instead of hiding the refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tokenText = (token: OsString): string => token.toString();
/// ```
fn token_text(token: &OsString) -> String {
    return token.to_string_lossy().into_owned();
}

/// What:
///  The next token Git hands the command as an argument,
///  when it names a section under `remote.`.
///  `&[OsString]` borrows the region,
///  `&mut` borrows the cursor that walks it.
/// Why:
///   A key is written `section.name` and a section `section.name`,
///  so the section is what both
///       readings compare,
///  and a token that names another section is not this rule's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remoteToken = (region, arguments) => { const token = region[arguments.next()]; return token && namesRemoteSection(token) ? token : undefined; };
/// ```
fn remote_token<'region>(
    region: &'region [OsString],
    arguments: &mut std::slice::Iter<'region, usize>,
) -> Option<&'region OsString> {
    // `?` takes the next argument, or answers nothing when the region names none.
    let token: &OsString = region.get(*arguments.next()?)?;
    if !names_remote_section(token.as_encoded_bytes()) {
        return None;
    }
    return Some(token);
}

/// What:
///  The setting a region writes,
///  when it writes a remote setting,
///  and the file it names for it.
///  `&ConfigRegion` borrows the region's reading.
/// Why:
///   Only a write to the repository's own file,
///  or to the one `--file` names,
///  is shared by
///       every worktree,
///  and only a key,
///  a section and an edit change a remote setting;
///  a read,
///  a per-worktree file,
///  the per-user file and a file the command names are none of this rule's
///       business.
///  An edit names no setting and can change a remote one,
///  so it is refused as one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configWrite = (region: string[], parsed: ConfigRegion): SharedRemoteChange | undefined => { /* ... */ };
/// ```
fn config_write(region: &[OsString], parsed: &ConfigRegion) -> Option<SharedRemoteChange> {
    // Which file: the repository's own, or the one the command names for the caller to resolve.
    let named_file: Option<PathBuf> = match parsed.file {
        ConfigFile::Shared => None,
        // `?` answers nothing for a `--blob`, which names no path, and for a path no platform can read.
        ConfigFile::Named => Some(parsed.named_file.clone()?),
        ConfigFile::Worktree | ConfigFile::Global | ConfigFile::System | ConfigFile::Contradictory => {
            return None;
        }
    };
    // The tokens Git hands the command as arguments, in order.
    let mut arguments = parsed.arguments.iter();
    let setting: SharedRemoteWrite = match parsed.write {
        ConfigWrite::Nothing => return None,
        ConfigWrite::Edit => SharedRemoteWrite::Edit,
        ConfigWrite::Key => {
            SharedRemoteWrite::Key(token_text(remote_token(region, &mut arguments)?))
        }
        ConfigWrite::RemoveSection => {
            SharedRemoteWrite::Section(token_text(remote_token(region, &mut arguments)?))
        }
        ConfigWrite::RenameSection => {
            // Renaming into the `remote` section creates remote settings just as renaming out of it removes them.
            let first: Option<&OsString> = remote_token(region, &mut arguments);
            if let Some(named) = first {
                SharedRemoteWrite::Section(token_text(named))
            } else {
                // The first section is not remote, so only a rename into `remote.` is this rule's.
                SharedRemoteWrite::Section(token_text(remote_token(region, &mut arguments)?))
            }
        }
    };
    return Some(SharedRemoteChange {
        setting,
        named_file,
    });
}

/// What:
///  Decide from the command word and the tokens after it whether this command writes the remote
///  settings every worktree shares,
///  and the file it names for that write.
///  `&[u8]` borrows the word,
///  `&[OsString]` the region after it.
/// Why:
///   A region Git refuses writes nothing,
///  so its refusal is the answer and no measure is made;
///       every other command word is none of this rule's business.
///  A write to a file the command names is decided
///       by the caller,
///  which alone knows where the command runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sharedRemoteWrite(word: string, region: string[]): SharedRemoteChange | undefined;
/// ```
pub fn shared_remote_write(word: &[u8], region: &[OsString]) -> Option<SharedRemoteChange> {
    if word == b"remote" {
        // A region Git refuses runs nothing, and `?` answers nothing for it.
        let parsed: RemoteRegion = parse_remote_region(region).ok()?;
        // `?` unwraps the subcommand word, or answers nothing for a listing.
        let name: &'static str = writing_subcommand(parsed.subcommand?.as_encoded_bytes())?;
        return Some(SharedRemoteChange {
            setting: SharedRemoteWrite::RemoteSubcommand(name),
            // `git remote` writes the repository's own file and names no other.
            named_file: None,
        });
    }
    if word != b"config" {
        return None;
    }
    // `&[]` is an empty flag list: wrapper controls were removed before this point.
    let parsed: ConfigRegion = parse_config_region(region, &[]).ok()?;
    return config_write(region, &parsed);
}

/// The decision and the spellings Git accepts,
///  with real Git 2.56.0 controls.
#[cfg(test)]
#[path = "rule_shared_remote_tests.rs"]
mod tests;
