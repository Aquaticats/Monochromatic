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
//!  so the last entry of a key is the value Git would have used.
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
/// Ordered map of the effective value of each key.
use std::collections::BTreeMap;

/// What:
///  The configuration keys native `git commit` reads that change what it writes.
/// Why:
///  A commit takes the native route because one of these is set,
///  so each one's effective value must reach the shadow's Git:
///  signing,
///  its format and programs,
///  and the message-shaping keys.
///  `trailer.*` keys are of this kind too and are matched by prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NATIVE_COMMIT_KEYS = ['commit.gpgsign', 'gpg.format', ...];
/// ```
const NATIVE_COMMIT_KEYS: [&str; 14] = [
    "commit.gpgsign",
    "commit.signoff",
    "commit.template",
    "commit.cleanup",
    "commit.verbose",
    "gpg.format",
    "gpg.program",
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
///  The `-c` arguments that give the shadow's Git every native-commit key at the value the invocation resolved.
/// Why:
///  `-c` is read after every file,
///  so the value here is the one the person's command used,
///  and it cannot be overridden by a condition that no longer matches in the shadow.
///  A key written without a value stays valueless,
///  which Git reads as a true boolean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...effective].flatMap(([key, value]) => ['-c', value === undefined ? key : `${key}=${value.toString()}`]);
/// ```
pub fn native_commit_configuration(snapshot: Option<&ConfigSnapshot>) -> Vec<OsString> {
    let Some(found) = snapshot else { return Vec::new() };
    // `mut` lets a later entry of a key replace an earlier one, which is Git's precedence order.
    let mut effective: BTreeMap<&str, &Option<Vec<u8>>> = BTreeMap::new();
    for entry in found.entries() {
        if is_native_commit_key(entry.key.as_str()) {
            effective.insert(entry.key.as_str(), &entry.value);
        }
    }
    let mut arguments: Vec<OsString> = Vec::new();
    for (key, value) in effective {
        let mut setting: Vec<u8> = key.as_bytes().to_vec();
        if let Some(bytes) = value {
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
