//! What:
//!  The one resolution of the real Git that every start uses:
//!  the first real Git on `PATH`, otherwise the Git the commit service's unit records, with a notice.
//! Why:
//!  The coordinator picked, under the owner's delegation and open to the owner's veto
//!  (`doc/planning/cli-git-no-self-invocation.md`, section "Falling back to the Git the commit service was set up
//!  with"), that a wrapper which is found but has no real Git after it on `PATH` uses the Git it was set up with instead
//!  of stopping. The search of `PATH` stays in `real_git.rs`; this module adds the fallback after it and the types that
//!  carry its result, so no caller can take the path and drop the notice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { path, notice } = resolveRealGit(inputs); // throws RealGitNotFound
//! ```

/// What:
///  `use super::...` imports the search, the inputs and the failure from the module that owns them,
///  the candidate verdicts, and the reader of the record.
/// Why:
///   The fallback is judged by the same `classify_candidate` that judges a `PATH` candidate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { searchPathForRealGit, type RealGitNotFound } from './real-git.ts';
/// ```
use super::real_git::{RealGitNotFound, ResolutionInputs, SearchedCandidate, search_path_for_real_git};
use super::real_git_candidate::{CandidateKind, classify_candidate};
use super::real_git_record::read_recorded_git;
/// `OsStr` is borrowed operating-system text (siblings: `OsString` owned, `&str` UTF-8 only).
use std::ffi::OsStr;
/// `Path` is a borrowed filesystem path;
///  `PathBuf` is the owned sibling.
use std::path::{Path, PathBuf};

/// What:
///  The real Git a unit file records,
///  with the verdict the `PATH` candidate checks gave it.
/// Why:
///   When the fallback is refused,
///  the failure names the unit and the recorded file,
///  so the person sees that the record was read and why it was not used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordedGit = { unitFile: string; candidate: SearchedCandidate };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedGit {
    /// The unit file the path was read from.
    pub unit_file: PathBuf,
    /// The recorded path and the verdict about it.
    pub candidate: SearchedCandidate,
}

/// What:
///  The real Git a resolution selected,
///  and the notice owed to the person when it was not found on `PATH`.
/// Why:
///   A caller that starts Git must also print the notice,
///  so the two travel together and a caller cannot take the path alone by accident.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ResolvedGit = { path: string; notice: string | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedGit {
    /// The executable to start as real Git.
    pub path: PathBuf,
    /// One line for standard error,
    ///  ending in a newline,
    ///  when the path came from the unit's record and not from `PATH`.
    pub notice: Option<String>,
}

/// What:
///  The one line a person reads when the recorded Git was used because `PATH` offered none.
/// Why:
///   It says what happened (no real Git after the wrapper on `PATH`, which `PATH`), what was done (which file, read from
///  which unit), and the one way to make it stop (put the file's directory on the `PATH` of the program that ran the
///  command).
///  An empty `PATH` is said in words, as the failure does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fallbackNotice(path: string, git: string, unit: string): string;
/// ```
fn fallback_notice(path: &OsStr, git: &Path, unit_file: &Path) -> String {
    let shown_path: String = if path.is_empty() {
        String::from("empty or unset")
    } else {
        path.to_string_lossy().into_owned()
    };
    // `.parent()` is the directory holding the file, or nothing for a bare root.
    let directory: String = match git.parent() {
        Some(found) => found.display().to_string(),
        None => String::from("its directory"),
    };
    return format!(
        "cli-git: no real Git follows this wrapper on PATH ({shown_path}); used {}, the Git recorded in {} as the one \
         the commit service starts; to silence this notice, add {directory} to the PATH of the program that ran this \
         command.\n",
        git.display(),
        unit_file.display()
    );
}

/// What:
///  Select the real Git for every start:
///  the first `PATH` candidate that is real Git,
///  otherwise the Git the commit service's unit records when that file passes the same checks.
///  `Result<ResolvedGit, RealGitNotFound>` is the selection with its notice,
///  or the typed failure.
/// Why:
///   The coordinator picked, under the owner's delegation and open to the owner's veto,
///  that a wrapper which is found but has no real Git after it on `PATH` uses the Git it was set up with,
///  instead of stopping.
///  The recorded file is judged by `classify_candidate` as a `PATH` candidate is:
///  never this executable, a link or copy of it, a launcher of it, a script that starts it,
///  and it must be an executable file that exists.
///  A refusal keeps the stop and adds the record to the failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveRealGit(inputs: ResolutionInputs): { path: string; notice?: string }; // throws RealGitNotFound
/// ```
pub fn resolve_real_git(inputs: &ResolutionInputs) -> Result<ResolvedGit, RealGitNotFound> {
    // `match` on the search result: keep a found path, or take the failure to the fallback.
    let mut not_found: RealGitNotFound = match search_path_for_real_git(inputs) {
        Ok(path) => {
            return Ok(ResolvedGit { path, notice: None });
        }
        Err(failure) => failure,
    };
    // `let Some(x) = ... else { return ...; }` leaves with the failure when there is no unit to read.
    let Some(unit_file) = inputs.service_unit.as_deref() else {
        return Err(not_found);
    };
    let Some(recorded) = read_recorded_git(unit_file) else {
        return Err(not_found);
    };
    let kind: CandidateKind = classify_candidate(recorded.as_path(), inputs.own_executable.as_path());
    if kind == CandidateKind::RealGit {
        let notice: String = fallback_notice(inputs.path.as_os_str(), recorded.as_path(), unit_file);
        return Ok(ResolvedGit { path: recorded, notice: Some(notice) });
    }
    not_found.recorded = Some(RecordedGit {
        unit_file: unit_file.to_path_buf(),
        candidate: SearchedCandidate { path: recorded, kind },
    });
    return Err(not_found);
}
