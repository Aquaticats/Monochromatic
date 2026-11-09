//! What:
//!  The words the wrapper prints about reference locks:
//! what it removed,
//! what it left and why,
//! and the exact command that
//!  clears a lock it will not remove.
//! Why:
//!  A lock that is not proven stale must never be removed,
//! so the person is told what was found and given a command that
//!  is safe to paste.
//! Path names come from the file system,
//! so the command quotes every byte a shell or a terminal could
//!  read as something other than part of the name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.stderr.write(outcome.notice()); // "monochromatic-git-policy-cli: left the reference lock ... alone: ...\nmonochromatic-git-policy-cli:   rm -- '...'\n"
//! ```

use crate::iso_time::format_iso_milliseconds;
use crate::stale_ref_lock_found::{Found, Outcome, PidFile};
use crate::stale_ref_lock_judge::{Proof, Verdict};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The start of every line the wrapper prints.
const LINE_PREFIX: &str = "monochromatic-git-policy-cli: ";

/// Bytes a shell reads as part of a plain word.
fn is_plain(byte: &u8) -> bool {
    return byte.is_ascii_alphanumeric() || b"_@%+=:,./-".contains(byte);
}

/// A path as a shell word that stands for exactly that path.
///  Plain names are written as they are;
/// names with other printable characters go in single quotes,
/// with each single
///  quote written `'\''`;
/// names with a control character or a byte that is not UTF-8 use the `$'...'` form,
/// which shells
///  of the Bash family read byte for byte and which keeps a terminal escape sequence from being printed raw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shellQuote(path: string): string;
/// ```
pub fn shell_quote(path: &Path) -> String {
    let bytes: &[u8] = path.as_os_str().as_bytes();
    if !bytes.is_empty() && bytes.iter().all(is_plain) {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let printable: bool = match std::str::from_utf8(bytes) {
        Ok(text) => !text.chars().any(char::is_control),
        Err(_) => false,
    };
    if printable {
        return format!("'{}'", String::from_utf8_lossy(bytes).replace('\'', "'\\''"));
    }
    let mut quoted: String = String::from("$'");
    for byte in bytes {
        match *byte {
            b'\'' => quoted.push_str("\\'"),
            b'\\' => quoted.push_str("\\\\"),
            0x20..=0x7e => quoted.push(char::from(*byte)),
            other => quoted.push_str(format!("\\x{}", crate::hexadecimal_of_bytes::hexadecimal_of_byte(other)).as_str()),
        }
    }
    quoted.push('\'');
    return quoted;
}

/// How long ago,
/// in the largest whole unit.
fn age_text(age: Duration) -> String {
    let seconds: u64 = age.as_secs();
    return match seconds {
        0..=119 => format!("{seconds} seconds"),
        120..=7199 => format!("{} minutes", seconds / 60),
        7200..=172_799 => format!("{} hours", seconds / 3600),
        _ => format!("{} days", seconds / 86_400),
    };
}

/// When a lock was made,
/// if known,
/// and how long ago.
fn born_text(born: Option<SystemTime>) -> String {
    let Some(at) = born else {
        return String::new();
    };
    let milliseconds: i64 = match at.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(elapsed) => i64::try_from(elapsed.as_millis()).unwrap_or(0),
        Err(_) => 0,
    };
    let age: Duration = SystemTime::now().duration_since(at).unwrap_or_default();
    return format!(", made {} ({} ago)", format_iso_milliseconds(milliseconds), age_text(age));
}

/// Why a lock was removed.
fn proof_text(proof: &Proof) -> String {
    return match proof {
        Proof::PidGone(pid) => format!("process {pid}, named by the pid file beside it, has ended"),
        Proof::PidReused { pid, command } => format!("process {pid}, named by the pid file beside it, has ended and the number now belongs to `{command}`"),
        Proof::ClaimedBeforeStop(pid) => format!("process {pid} held it when this wrapper stopped that process (the pid file beside it named {pid} then), and {pid} has ended"),
        Proof::OrphanStaging => String::from("Git makes this staging file only while it holds `packed-refs.lock`, and that lock is gone"),
    };
}

/// The paths a person clears by hand:
/// the lock and what goes with it,
/// quoted.
fn clearing_words(found: &Found) -> String {
    let mut paths: Vec<String> = vec![shell_quote(found.lock.as_path())];
    for path in &found.companions {
        paths.push(shell_quote(path.as_path()));
    }
    return paths.join(" ");
}

impl Outcome {
    /// What the wrapper prints for this pass,
    /// as lines that each start with `monochromatic-git-policy-cli: `;
    /// empty when there is nothing to say.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// outcome.notice(): string
    /// ```
    pub fn notice(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for found in &self.cleared {
            if let Verdict::Stale(proof) = &found.verdict {
                let what: &str = if matches!(proof, Proof::OrphanStaging) { "the packed-references staging file" } else { "the reference lock" };
                lines.push(format!("{LINE_PREFIX}removed {what} {}, left by a Git that is gone: {}.", shell_quote(found.lock.as_path()), proof_text(proof)));
            }
        }
        let mut hint: bool = false;
        for found in &self.left {
            match &found.verdict {
                Verdict::Held(text) => lines.push(format!("{LINE_PREFIX}the reference lock {} is in use: {text}; it was left alone.", shell_quote(found.lock.as_path()))),
                Verdict::Unproven(reason) => {
                    lines.push(format!("{LINE_PREFIX}left the reference lock {} alone: {reason}.", shell_quote(found.lock.as_path())));
                    lines.push(format!("{LINE_PREFIX}it is {} bytes{}. If no Git process is running in this repository, clear it with:", found.size, born_text(found.born)));
                    lines.push(format!("{LINE_PREFIX}  rm -- {}", clearing_words(found)));
                    hint = hint || found.pid_file == PidFile::Absent;
                }
                Verdict::TooNew | Verdict::Stale(_) => {}
            }
        }
        for path in &self.displaced {
            lines.push(format!("{LINE_PREFIX}a lock file was set aside while a new lock took its place, and is left at {}; look at it before removing it.", shell_quote(path.as_path())));
        }
        if hint {
            lines.push(format!("{LINE_PREFIX}Git writes a process number beside its locks only when `core.lockfilePid` is on. The wrapper turns it on for the Git it starts; `git config --global core.lockfilePid true` does it for the rest, so the next lock can be proven."));
        }
        if lines.is_empty() {
            return String::new();
        }
        return format!("{}\n", lines.join("\n"));
    }
}

/// The paths of a list,
/// for a log line.
pub fn path_list(paths: &[PathBuf]) -> String {
    let mut quoted: Vec<String> = Vec::new();
    for path in paths {
        quoted.push(shell_quote(path.as_path()));
    }
    return quoted.join(" ");
}

/// Controls for the notices and the quoting are in the test module.
#[cfg(test)]
#[path = "stale_ref_lock_message_tests.rs"]
mod tests;
