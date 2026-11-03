//! What:
//!  Read the real Git that the wrapper's commit service was set up with, from the unit file the wrapper wrote for it.
//! Why:
//!  Nothing the wrapper installs records the real Git it shadows. The one file that names a real Git is the unit of the
//!  commit service (`service_provision.rs`): its `ExecStart` line holds `--git <path>`, the Git the wrapper resolved
//!  when it last provisioned the service. When no real Git follows the wrapper on `PATH`, that path is the only
//!  recorded one, so the resolver (`real_git.rs`) falls back to it, after the same checks a `PATH` candidate passes.
//!  This is the coordinator's pick under the owner's delegation, open to the owner's veto
//!  (`doc/planning/cli-git-no-self-invocation.md`, section "Falling back to the Git the commit service was set up
//!  with"). The file lives in the runtime directory, so it is gone after the account's last logout, and no record then
//!  means no fallback.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const unitText = readFileSync(join(runtimeDir, 'systemd/user/cli-git@.service'), 'utf8');
//! const recordedGit = unitText.match(/^ExecStart=.*\s--git\s+(\S+)/m)?.[1]; // undefined: no record
//! ```

/// What:
///  `use super::...` imports the names of the sibling modules that own the unit file's location and name,
///  and the bound on how much of a record is read.
/// Why:
///   The unit's directory and file name are decided once, where the wrapper writes the unit;
///   this module only reads what that writer produced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { runtimeUnitDir, SERVICE_UNIT_NAME } from './service-provision.ts';
/// ```
use super::real_git_candidate::MAX_SCRIPT_INSPECTION_BYTES;
use super::service_provision::{SERVICE_UNIT_NAME, runtime_unit_dir};
/// `Read` supplies `read_to_end` and `take`, the bounded read below.
use std::io::Read;
/// `Path` is a borrowed filesystem path;
///  `PathBuf` is the owned sibling a function returns.
use std::path::{Path, PathBuf};

/// The option of the service program that names the Git it starts.
const GIT_OPTION: &str = "--git";

/// The key of the unit line that holds the service program and its options.
const EXEC_START_KEY: &str = "ExecStart=";

/// What:
///  The unit file of the commit service below a runtime directory.
/// Why:
///   The wrapper writes the service unit there, and nowhere else is searched:
///   a record found by looking through other places would be a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function serviceUnitFile(runtimeDir: string): string { return join(runtimeDir, 'systemd/user/cli-git@.service'); }
/// ```
pub fn service_unit_file(runtime_dir: &Path) -> PathBuf {
    return runtime_unit_dir(runtime_dir).join(SERVICE_UNIT_NAME);
}

/// What:
///  The absolute path after `--git` on the `ExecStart` line of a unit's text.
///  `Option<PathBuf>` is the path or nothing (`PathBuf | undefined`).
/// Why:
///   The wrapper writes the path unquoted, so whitespace cuts the line into words and the word after `--git` is the
///   path. A relative path is no record: it would mean a different file for each directory the wrapper runs in.
///   A path with a space in it is cut short and then fails the checks of the resolver, which is the safe outcome.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordedGitPath(unitText: string): string | undefined;
/// ```
pub fn recorded_git_path(unit_text: &str) -> Option<PathBuf> {
    for line in unit_text.lines() {
        // `let Some(x) = ... else { continue; }` skips the line when the key is not at its start.
        let Some(command) = line.trim_start().strip_prefix(EXEC_START_KEY) else {
            continue;
        };
        // `.split_whitespace()` cuts the line into words;
        //  `.collect()` gathers them into an owned list so a word can be found and the next one read.
        let words: Vec<&str> = command.split_whitespace().collect();
        let Some(option_at) = words.iter().position(|word| {
            return *word == GIT_OPTION;
        }) else {
            continue;
        };
        // `.get(n)` is a checked index: nothing when the option is the last word.
        let Some(value) = words.get(option_at + 1) else {
            continue;
        };
        let path: PathBuf = PathBuf::from(value);
        if path.is_absolute() {
            // `Some(...)` wraps the found path.
            return Some(path);
        }
    }
    // `None` says there is no record.
    return None;
}

/// What:
///  The Git a unit file records, read with the bound a launcher record is read with.
/// Why:
///   The unit is a few hundred bytes, so a file past the bound is not the wrapper's unit; it names nothing.
///   An unreadable or missing file names nothing either, and the caller keeps its stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readRecordedGit(unitFile: string): string | undefined;
/// ```
pub fn read_recorded_git(unit_file: &Path) -> Option<PathBuf> {
    // `.ok()?` turns a failed open into "nothing" and returns it at once.
    let file: std::fs::File = std::fs::File::open(unit_file).ok()?;
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    // `.take(n)` stops the read after `n` bytes;
    //  the bound is one byte past the limit so that a file over the limit is told from one at it.
    let bound: u64 = MAX_SCRIPT_INSPECTION_BYTES as u64 + 1;
    file.take(bound).read_to_end(&mut bytes).ok()?;
    if bytes.len() > MAX_SCRIPT_INSPECTION_BYTES {
        return None;
    }
    // `String::from_utf8` checks the bytes are text;
    //  `.ok()?` returns nothing for a file that is not.
    let text: String = String::from_utf8(bytes).ok()?;
    return recorded_git_path(text.as_str());
}

/// Controls for the unit text, the file read and the bounds.
#[cfg(test)]
#[path = "real_git_record_tests.rs"]
mod tests;
