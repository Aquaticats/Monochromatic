//! What:
//!  The configuration native `git commit` reads that decides what it writes,
//!  the file in the commit's scratch directory that holds it,
//!  and the entries that file states.
//! Why:
//!  The shadow's own config includes the real configuration,
//!  but an `includeIf` condition that names the real Git directory no longer matches
//!  once the shadow directory is substituted for it.
//!  A value that decided this commit,
//!  whether it signs,
//!  which key and program sign it,
//!  which template or clean-up mode shapes its message,
//!  would then be missing,
//!  and the commit would succeed with the person's request dropped.
//!  The capture's snapshot holds the entries Git resolved for the invocation in precedence order,
//!  so carrying every entry of every such key in that order gives the shadow's Git the same answers,
//!  a key that is read once keeping its last entry and a key that may repeat keeping all of them.
//!  The entries travel in one configuration file the command line includes,
//!  because a command-line argument and an environment string both stop at the platform's
//!  single-argument limit,
//!  and a `key=value` argument cannot name a subsection that itself holds an equals sign.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const file = writeEntries(snapshot.entries.filter(e => isNativeCommitKey(e.key)), scratch)
//! // return ['-c', `include.path=${file}`]
//! ```

/// The configuration as the invocation read it,
/// and the object that the carried file is built from.
use crate::config_snapshot::ConfigSnapshot;
/// Owned argument text.
use std::ffi::OsString;
/// The bytes of a path,
/// for a value that is not text.
use crate::path_bytes::os_from_path_bytes;
/// The directory the carried file is written into,
/// and the path of that file.
use std::path::{Path, PathBuf};

/// The name the carried configuration takes in the commit's scratch directory.
pub const NATIVE_COMMIT_CONFIGURATION_NAME: &str = "native-commit.config";

/// What:
///  The configuration keys native `git commit` reads that change what it writes.
/// Why:
///  A commit takes the native route because one of these is set,
///  so each one's effective value must reach the shadow's Git:
///  signing,
///  its format and programs,
///  and the message-shaping keys.
///  `trailer.*` keys are of this kind too and are matched by prefix,
///  and so is every `gpg.*` key,
///  which is the whole set `git_gpg_config` in Git 2.56.0's `gpg-interface.c` reads:
///  `gpg.format`,
///  the program of each format (`gpg.program`,
///  the legacy name of `gpg.openpgp.program`,
///  `gpg.openpgp.program`,
///  `gpg.x509.program` and `gpg.ssh.program`),
///  `gpg.mintrustlevel`,
///  `gpg.ssh.defaultkeycommand`,
///  `gpg.ssh.allowedsignersfile` and
///  `gpg.ssh.revocationfile`.
///  A prefix is what carries a program a format needs,
///  rather than one key named now and the next one missed:
///  a format whose program was missing signed with the default program instead,
///  which is a commit real Git makes and the shadow does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NATIVE_COMMIT_KEYS = ['commit.gpgsign', 'user.signingkey', ...]; // plus 'gpg.*' and 'trailer.*'
/// ```
const NATIVE_COMMIT_KEYS: [&str; 9] = [
    "commit.gpgsign",
    "commit.signoff",
    "commit.template",
    "commit.cleanup",
    "commit.verbose",
    "user.signingkey",
    "i18n.commitencoding",
    "core.commentchar",
    "core.commentstring",
];

/// What:
///  Whether one configuration key changes what native `git commit` writes.
/// Why:
///  The route to native preparation and the values it carries both read this one list,
///  so a key cannot be counted in one place and lost in the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNativeCommitKey = (key: string) =>
///   NATIVE_COMMIT_KEYS.includes(key) || key.startsWith('trailer.') || key.startsWith('gpg.');
/// ```
pub fn is_native_commit_key(key: &str) -> bool {
    return NATIVE_COMMIT_KEYS.contains(&key) || key.starts_with("trailer.") || key.starts_with("gpg.");
}

/// What:
///  Writes every native-commit entry the invocation resolved into the commit's scratch directory,
///  and returns the `-c` settings that make the shadow's Git read them.
/// Why:
///  `-c include.path` is read after every configuration file and in the place it is given,
///  so these are the values the person's command used,
///  and a condition that no longer matches in the shadow cannot override them.
///  One file is one configuration source,
///  so the entries keep Git's precedence order inside it.
///  A file also carries what one argument cannot:
///  a subsection that itself holds an equals sign,
///  and a value longer than the platform's single-argument limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = join(scratch, NATIVE_COMMIT_CONFIGURATION_NAME);
/// await writeFile(file, text(snapshot.entries.filter((e) => isNativeCommitKey(e.key))));
/// return ['-c', `include.path=${file}`];
/// ```
///
/// # Errors
///
/// The error of writing the file.
pub fn native_commit_configuration(snapshot: Option<&ConfigSnapshot>, scratch: &Path) -> std::io::Result<Vec<OsString>> {
    let Some(found) = snapshot else { return Ok(Vec::new()) };
    let text: Vec<u8> = native_commit_configuration_text(found);
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let file: PathBuf = scratch.join(NATIVE_COMMIT_CONFIGURATION_NAME);
    std::fs::write(file.as_path(), text.as_slice())?;
    // The path's own bytes, so a scratch directory whose name is not text still names the file it was written to.
    let mut setting: Vec<u8> = b"include.path=".to_vec();
    setting.extend_from_slice(file.as_os_str().as_encoded_bytes());
    return Ok(vec![
        OsString::from("-c"),
        os_from_path_bytes(setting.as_slice()),
    ]);
}

/// What:
///  The native-commit entries as a Git configuration file states them,
///  in the order Git resolved them.
/// Why:
///   A key that may repeat keeps every entry,
///  and a key that is read once keeps its last entry when Git reads this file in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.filter((e) => isNativeCommitKey(e.key)).map((e) => statement(e)).join('');
/// ```
fn native_commit_configuration_text(found: &ConfigSnapshot) -> Vec<u8> {
    let mut text: Vec<u8> = Vec::new();
    for entry in found.entries() {
        if !is_native_commit_key(entry.key.as_str()) {
            continue;
        }
        push_entry(&mut text, entry.key.as_str(), entry.value.as_deref());
    }
    return text;
}

/// What:
///  One entry as a configuration file states it:
///  a header naming the section,
///  and the subsection in quotes when the key has one,
///  then the variable with its value.
/// Why:
///  The quoted subsection is what keeps a name such as `review=bot` whole:
///  the dotted spelling of that key stops the variable at the equals sign,
///  exactly as a `-c` setting does.
///  Each entry gets a header of its own,
///  so the order of the file is the order of the listing.
///  A key written without a value stays valueless,
///  which Git reads as a true boolean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // text += `[${section}${subsection === undefined ? '' : ` "${quotedSubsection}"`}]\n\t${name}`
/// // text += value === undefined ? '\n' : ` = "${quotedValue}"\n`
/// ```
fn push_entry(text: &mut Vec<u8>, key: &str, value: Option<&[u8]>) {
    let (section, subsection, name) = key_parts(key);
    text.push(b'[');
    text.extend_from_slice(section.as_bytes());
    if let Some(named) = subsection {
        text.extend_from_slice(b" \"");
        push_quoted_subsection(text, named.as_bytes());
        text.push(b'"');
    }
    text.extend_from_slice(b"]\n\t");
    text.extend_from_slice(name.as_bytes());
    if let Some(bytes) = value {
        text.extend_from_slice(b" = \"");
        push_quoted_value(text, bytes);
        text.push(b'"');
    }
    text.push(b'\n');
}

/// What:
///  The section,
///  the optional subsection,
///  and the variable name of a configuration key.
/// Why:
///   Git writes every key it lists as `section.name` or `section.subsection.name`,
///  and the subsection is written back as its own quoted header.
///  A key of any other shape cannot name a variable,
///  so a listing that held one would be a defect worth failing on rather than dropping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [section, ...rest] = key.split('.'); const name = rest.pop(); return { section, name, subsection: rest.join('.') };
/// ```
fn key_parts(key: &str) -> (&str, Option<&str>, &str) {
    let (section, rest): (&str, &str) = key.split_once('.').expect("a configuration key names a section and a variable");
    assert!(!section.is_empty() && !rest.is_empty(), "a configuration key names a section and a variable: {key}");
    return match rest.rsplit_once('.') {
        Some((subsection, name)) if !subsection.is_empty() && !name.is_empty() => (section, Some(subsection), name),
        Some(_) => (section, None, rest),
        None => (section, None, rest),
    };
}

/// What:
///  A subsection name in the escapes a section header reads back.
/// Why:
///   A section header recognizes `\"` and `\\` and drops the backslash before any other character,
///  so a tab stays literal and only a quote or a backslash is escaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// subsection.replace(/[\"\\]/g, (c) => `\\${c}`);
/// ```
fn push_quoted_subsection(text: &mut Vec<u8>, subsection: &[u8]) {
    for byte in subsection {
        if matches!(byte, b'\\' | b'"') {
            text.push(b'\\');
        }
        text.push(*byte);
    }
}

/// What:
///  A value in the quoted form Git's file parser reads back byte for byte.
/// Why:
///   A quoted value keeps `#`,
///  `;` and surrounding spaces.
///  Each of Git's five value escapes is written as the letter it names,
///  `\\`,
///  `\"`,
///  `\n`,
///  `\t` and `\b`,
///  because a backslash before the character itself is not that escape:
///  a backslash before a raw tab is an invalid escape Git refuses,
///  and a backslash before a raw newline continues the line,
///  which drops the newline from the value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const escapes = { '\\': '\\\\', '"': '\\"', '\n': '\\n', '\t': '\\t', '\b': '\\b' };
/// value.replace(/[\\"\n\t\b]/g, (c) => escapes[c]);
/// ```
fn push_quoted_value(text: &mut Vec<u8>, value: &[u8]) {
    for byte in value {
        match byte {
            b'\\' => text.extend_from_slice(b"\\\\"),
            b'"' => text.extend_from_slice(b"\\\""),
            b'\n' => text.extend_from_slice(b"\\n"),
            b'\t' => text.extend_from_slice(b"\\t"),
            b'\x08' => text.extend_from_slice(b"\\b"),
            other => text.push(*other),
        }
    }
}

/// Configuration controls stay out of the release executable.
#[cfg(test)]
#[path = "native_commit_configuration_tests.rs"]
mod tests;
