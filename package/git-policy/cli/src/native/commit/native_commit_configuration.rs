//! What:
//!  The configuration native `git commit` reads that decides what it writes,
//!  and the effective values of those keys as the invocation resolved them.
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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // effective = snapshot.entries.filter(e => isNativeCommitKey(e.key)).reduce((at, e) => at.set(e.key, e.value), new Map())
//! ```

/// The configuration as the invocation read it,
/// and the object that a command line setting is built from.
use crate::config_snapshot::ConfigSnapshot;
/// Owned argument and environment text.
use std::ffi::OsString;
/// The bytes of a path,
/// for a value that is not text.
use crate::path_bytes::os_from_path_bytes;

/// What:
///  The configuration keys native `git commit` reads that change what it writes.
/// Why:
///  A commit takes the native route because one of these is set,
///  so each one's effective value must reach the shadow's Git:
///  signing,
///  its format and programs,
///  and the message-shaping keys.
///  `trailer.*` keys are of this kind too and are matched by prefix.
///  `gpg.program` is the legacy name of `gpg.openpgp.program`,
///  so both are carried: the listing keeps the order Git resolved them in,
///  and the later entry is the program Git signs with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NATIVE_COMMIT_KEYS = ['commit.gpgsign', 'gpg.format', ...];
/// ```
const NATIVE_COMMIT_KEYS: [&str; 15] = [
    "commit.gpgsign",
    "commit.signoff",
    "commit.template",
    "commit.cleanup",
    "commit.verbose",
    "gpg.format",
    "gpg.program",
    "gpg.openpgp.program",
    "gpg.ssh.program",
    "gpg.ssh.defaultkeycommand",
    "gpg.ssh.allowedsignersfile",
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
/// const isNativeCommitKey = (key: string) => NATIVE_COMMIT_KEYS.includes(key) || key.startsWith('trailer.');
/// ```
pub fn is_native_commit_key(key: &str) -> bool {
    return NATIVE_COMMIT_KEYS.contains(&key) || key.starts_with("trailer.");
}

/// What:
///  The `-c` arguments that give the shadow's Git every native-commit entry the invocation resolved.
/// Why:
///  `-c` is read after every file,
///  so these are the values the person's command used,
///  and they cannot be overridden by a condition that no longer matches in the shadow.
///  The entries keep Git's precedence order,
///  so a key read once keeps its last entry and a key that may repeat keeps every one.
///  A key written without a value stays valueless,
///  which Git reads as a true boolean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = snapshot.entries.filter(isNativeCommitKey).flatMap((e) => ['-c', e.value === undefined ? e.key : `${e.key}=${e.value}`]);
/// ```
pub fn native_commit_configuration(snapshot: Option<&ConfigSnapshot>) -> Vec<OsString> {
    let Some(found) = snapshot else { return Vec::new() };
    let mut arguments: Vec<OsString> = Vec::new();
    for entry in found.entries() {
        if !is_native_commit_key(entry.key.as_str()) {
            continue;
        }
        let mut setting: Vec<u8> = entry.key.as_bytes().to_vec();
        if let Some(bytes) = &entry.value {
            setting.push(b'=');
            setting.extend_from_slice(bytes.as_slice());
        }
        arguments.push(OsString::from("-c"));
        arguments.push(os_from_path_bytes(setting.as_slice()));
    }
    return arguments;
}

/// Configuration controls stay out of the release executable.
#[cfg(test)]
#[path = "native_commit_configuration_tests.rs"]
mod tests;
