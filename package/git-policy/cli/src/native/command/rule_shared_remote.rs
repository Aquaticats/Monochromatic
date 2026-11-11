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
///       the refusal names the one it is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SharedRemoteWrite = { kind: 'remote-subcommand'; name: string }
///   | { kind: 'key'; key: string } | { kind: 'section'; section: string };
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
///  The `git config` write a region performs on the shared file,
///  when it is one.
///  `&ConfigRegion` borrows the region's reading.
/// Why:
///   Only a write to the repository's own file is shared by every worktree,
///  and only a key or
///       section under `remote.` changes a remote setting;
///  a read,
///  an editor,
///  a per-worktree
///       file and a file the command names are none of this rule's business.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configWrite = (region: string[], parsed: ConfigRegion): SharedRemoteWrite | undefined => { /* ... */ };
/// ```
fn config_write(region: &[OsString], parsed: &ConfigRegion) -> Option<SharedRemoteWrite> {
    if parsed.file != ConfigFile::Shared {
        return None;
    }
    // The tokens Git hands the command as arguments, in order.
    let mut arguments = parsed.arguments.iter();
    // `next()` takes the first argument, or answers nothing when there is none.
    let first: &OsString = region.get(*arguments.next()?)?;
    return match parsed.write {
        ConfigWrite::Nothing | ConfigWrite::Edit => None,
        ConfigWrite::Key => {
            if names_remote_section(first.as_encoded_bytes()) {
                return Some(SharedRemoteWrite::Key(token_text(first)));
            }
            return None;
        }
        ConfigWrite::RemoveSection => {
            if names_remote_section(first.as_encoded_bytes()) {
                return Some(SharedRemoteWrite::Section(token_text(first)));
            }
            return None;
        }
        ConfigWrite::RenameSection => {
            // Renaming into the `remote` section creates remote settings just as renaming out of it removes them.
            if names_remote_section(first.as_encoded_bytes()) {
                return Some(SharedRemoteWrite::Section(token_text(first)));
            }
            let second: &OsString = region.get(*arguments.next()?)?;
            if names_remote_section(second.as_encoded_bytes()) {
                return Some(SharedRemoteWrite::Section(token_text(second)));
            }
            return None;
        }
    };
}

/// What:
///  Decide from the command word and the tokens after it whether this command writes the remote
///  settings every worktree shares.
///  `&[u8]` borrows the word,
///  `&[OsString]` the region after it.
/// Why:
///   A region Git refuses writes nothing,
///  so its refusal is the answer and no measure is made;
///       every other command word is none of this rule's business.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sharedRemoteWrite(word: string, region: string[]): SharedRemoteWrite | undefined;
/// ```
pub fn shared_remote_write(word: &[u8], region: &[OsString]) -> Option<SharedRemoteWrite> {
    if word == b"remote" {
        // A region Git refuses runs nothing, and `?` answers nothing for it.
        let parsed: RemoteRegion = parse_remote_region(region).ok()?;
        // `?` unwraps the subcommand word, or answers nothing for a listing.
        let name: &'static str = writing_subcommand(parsed.subcommand?.as_encoded_bytes())?;
        return Some(SharedRemoteWrite::RemoteSubcommand(name));
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
