//! What:
//!  The fsmonitor section of `git monochromatic-git-policy-cli doctor`:
//! whether Git's own file-system monitor (`core.fsmonitor`) is off,
//! and
//!  where it is on,
//! the file and scope that turn it on and the command that turns it off.
//! Why:
//!  The owner decided on 2026-11-03:
//! "If git's fsmonitor is fundamentally broken and cannot be easily or meaningfully
//!  worked around,
//! disable git's fsmonitor globally entirely and make doctor say so."
//! It is off in the person's global
//!  configuration,
//! and the Git the wrapper starts for its own work passes `-c core.fsmonitor=false`
//! (`hooks_off.rs`),
//! so what doctor reports is about the person's own Git:
//! one line when it is off,
//! and a finding with the exact command when some scope turns it on.
//!  Doctor reports and never writes configuration.
//! The exit status does not depend on it,
//! like the notes about retired keys.
//!  The origin of a value (the file) is not in the configuration snapshot the commit path reads,
//! which lists scopes only,
//!  so this section reads one more listing with `--show-origin`;
//! doctor runs on request,
//! never on the commit path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = renderFsmonitor(parseFsmonitorEntries(await git(['config', '--list', '--show-origin', '--show-scope', '-z'])));
//! ```

use crate::config_snapshot::canonical_bool;
use crate::diagnostic_log::debug;
use crate::transaction_git::{GitContext, GitRequest};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::path::Path;

/// The key this section reads,
/// as `git config --list` prints it (section and variable in lower case).
const FSMONITOR_KEY: &str = "core.fsmonitor";

/// The scopes that `git config` can write with a flag of the same name (`--system`,
/// `--global`,
/// `--local`,
///  `--worktree`);
/// the `command` scope (`-c` and the environment) has no file.
const WRITABLE_SCOPES: [&str; 4] = ["system", "global", "local", "worktree"];

/// The repository-relative path of the document that records why the owner turned Git's fsmonitor off.
pub const FSMONITOR_DOCUMENT: &str = "doc/troubleshooting/git-fsmonitor-daemon-start-wait.md";

/// The date of the owner's decision to keep Git's fsmonitor off.
pub const DECISION_DATE: &str = "2026-11-03";

/// The one-sentence reason,
/// from measurements of 2026-11-03 on the owner's main checkout (261 daemons held 4,591,335
///  watches;
/// one daemon watched 2,269,377 directories and spent 117.7 s of CPU and 734 MB to start;
/// a running daemon
///  saved about 7 ms per `git status`) and from Git 2.56.0 (`compat/fsmonitor/fsm-listen-linux.c`,
///  `builtin/fsmonitor--daemon.c`);
/// the full account is in the document named by `FSMONITOR_DOCUMENT`.
pub const FSMONITOR_REASON: &str = "Git's fsmonitor daemon registers a watch on every directory of the work tree, \
linked worktrees inside it included, before it answers, so a Git command that finds no running daemon starts one and \
waits for it for up to `fsmonitor.startTimeout` (60 s by default), while a running daemon saved about 7 ms per \
`git status` on the owner's main checkout.";

/// One `core.fsmonitor` entry of the configuration listing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FsmonitorEntry {
    /// `system`,
    /// `global`,
    /// `local`,
    /// `worktree` or `command`.
    pub scope: String,
    /// Where Git read the entry,
    /// as it prints it:
    /// `file:<path>` or `command line:`.
    pub origin: String,
    /// The value bytes,
    /// or `None` for a key written without `=`.
    pub value: Option<Vec<u8>>,
}

/// What:
///  Whether one byte is the NUL byte that ends every field of the listing.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///   `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNulByte = (byte: number) => byte === 0;
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether one byte is the line feed that separates a key from its value.
///  `&u8` borrows the byte,
/// as `position` hands it over.
/// Why:
///   `position` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLineFeedByte = (byte: number) => byte === '\n'.charCodeAt(0);
/// ```
fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  The `core.fsmonitor` entries of a `git config --list --show-origin --show-scope -z` listing,
/// in precedence order.
///  `Option<Vec<...>>` is `None` when the listing is not in the expected shape.
/// Why:
///   The listing is `scope NUL origin NUL key [LF value] NUL` repeated,
/// and the last entry of a key is the one Git
///  uses;
/// a listing in any other shape is refused whole,
/// so a half-understood configuration is never reported on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseFsmonitorEntries(bytes); // undefined when the shape is wrong
/// ```
pub fn parse_fsmonitor_entries(listing: &[u8]) -> Option<Vec<FsmonitorEntry>> {
    let mut entries: Vec<FsmonitorEntry> = Vec::new();
    if listing.is_empty() {
        return Some(entries);
    }
    let body: &[u8] = listing.strip_suffix(&[0])?;
    let mut fields = body.split(is_nul_byte);
    while let Some(scope) = fields.next() {
        let origin: &[u8] = fields.next()?;
        let record: &[u8] = fields.next()?;
        // Only the first line feed separates key from value; the value may hold more of them.
        let (key, value): (&[u8], Option<&[u8]>) = match record.iter().position(is_line_feed_byte) {
            Some(newline) => (&record[..newline], Some(&record[newline + 1..])),
            None => (record, None),
        };
        if key != FSMONITOR_KEY.as_bytes() {
            continue;
        }
        entries.push(FsmonitorEntry {
            scope: String::from(std::str::from_utf8(scope).ok()?),
            origin: String::from(std::str::from_utf8(origin).ok()?),
            value: value.map(<[u8]>::to_vec),
        });
    }
    return Some(entries);
}

/// What:
///  The command that turns `core.fsmonitor` off for the scope of an entry,
/// or the instruction to remove a value that
///  no file holds.
/// Why:
///   `git config --<scope> core.fsmonitor false` rewrites the file of that scope;
/// a `command`-scope value comes from `-c`
///  or the environment,
/// which no configuration command can change.
/// When the entry came from a file the scope includes,
///  the command changes the including file and doctor then names the included file,
/// which a second run shows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// WRITABLE_SCOPES.includes(scope) ? `Turn it off with: git config --${scope} core.fsmonitor false` : '...';
/// ```
fn remedy(entry: &FsmonitorEntry) -> String {
    if WRITABLE_SCOPES.contains(&entry.scope.as_str()) {
        return format!("Turn it off with: git config --{} core.fsmonitor false", entry.scope);
    }
    return String::from(
        "No configuration file sets it: remove the `-c core.fsmonitor=...` option or the GIT_CONFIG_COUNT or \
GIT_CONFIG_PARAMETERS environment variable that passes it.",
    );
}

/// What:
///  The standard error text for the entries of `core.fsmonitor`:
/// the off line,
/// or the finding.
/// Why:
///   The last entry decides,
/// as for `git config --get`.
/// Off (`false`,
/// `no`,
/// `off`,
/// `0`,
/// empty,
/// or no entry) gets one line
///  that names the entry that decided;
/// on (a true word,
/// a non-zero number,
/// a key without a value,
/// or a hook path) gets a
///  finding with the file,
/// the scope,
/// the reason,
/// the document and the command for that scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renderFsmonitor(entries); // one line ending in a line feed
/// ```
pub fn render_fsmonitor(entries: &[FsmonitorEntry]) -> String {
    let Some(deciding) = entries.last() else {
        return format!(
            "git monochromatic-git-policy-cli doctor: Git's fsmonitor is off by the owner's decision of {DECISION_DATE} (core.fsmonitor is not set). \
{FSMONITOR_REASON} See {FSMONITOR_DOCUMENT}.\n"
        );
    };
    let place: String = format!("{}, {} scope", deciding.origin, deciding.scope);
    let state: Option<bool> = canonical_bool(deciding.value.as_deref());
    if state == Some(false) {
        // The value is named as written; a spelling other than `false` also says how Git reads it.
        let written: String = String::from_utf8_lossy(deciding.value.as_deref().unwrap_or_default()).into_owned();
        let value: String = if written == "false" {
            String::from("false")
        } else if written.is_empty() {
            String::from("empty (Git reads it as false)")
        } else {
            format!("{written} (Git reads it as false)")
        };
        return format!(
            "git monochromatic-git-policy-cli doctor: Git's fsmonitor is off by the owner's decision of {DECISION_DATE} (core.fsmonitor is {value} in \
{place}). {FSMONITOR_REASON} See {FSMONITOR_DOCUMENT}.\n"
        );
    }
    // A value that is not a boolean word or a number is the path of a hook program, which also turns fsmonitor on.
    let setting: String = match (state, deciding.value.as_deref()) {
        (Some(_), None) => format!("core.fsmonitor is true (the key has no value) in {place}"),
        (Some(_), Some(value)) => format!("core.fsmonitor is {} in {place}", String::from_utf8_lossy(value)),
        (None, Some(value)) => format!("core.fsmonitor names the hook {} in {place}", String::from_utf8_lossy(value)),
        (None, None) => unreachable!("a key without a value always reads as true"),
    };
    return format!(
        "git monochromatic-git-policy-cli doctor: Git's fsmonitor is on, against the owner's decision of {DECISION_DATE} to keep it off: {setting}. \
{FSMONITOR_REASON} See {FSMONITOR_DOCUMENT}. {}\n",
        remedy(deciding)
    );
}

/// What:
///  Read the listing for `cwd` and render the fsmonitor text.
/// Why:
///   Doctor reports what it could not read rather than staying silent,
/// because silence would read as "off";
/// the exit
///  status is unaffected either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await fsmonitorReport({ context, cwd });
/// ```
pub async fn fsmonitor_report(context: &GitContext, cwd: &Path) -> String {
    let request: GitRequest = GitRequest::new(cwd, &["config", "--list", "--show-origin", "--show-scope", "-z"]);
    let output = match run_git_awaiting(context, &request).await {
        Ok(finished) => finished,
        Err(error) => {
            debug("fsmonitorReport", format!("git config --list could not start: {error}").as_str());
            return format!("git monochromatic-git-policy-cli doctor: could not read Git's fsmonitor setting: git config --list could not start: {error}\n");
        }
    };
    if !output.succeeded() {
        debug("fsmonitorReport", format!("git config --list failed: {}", output.error_text()).as_str());
        return format!("git monochromatic-git-policy-cli doctor: could not read Git's fsmonitor setting: git config --list failed: {}\n", output.error_text());
    }
    let Some(entries) = parse_fsmonitor_entries(output.stdout.as_slice()) else {
        debug("fsmonitorReport", "the configuration listing has an unexpected shape");
        return String::from("git monochromatic-git-policy-cli doctor: could not read Git's fsmonitor setting: git config --list printed an unexpected shape\n");
    };
    return render_fsmonitor(entries.as_slice());
}

/// Fsmonitor controls are in the test module.
#[cfg(test)]
#[path = "doctor_fsmonitor_tests.rs"]
mod tests;
