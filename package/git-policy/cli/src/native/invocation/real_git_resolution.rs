//! What:
//!  The one resolution of the real Git that every start uses:
//!  the first real Git on `PATH`,
//! otherwise the Git the commit service's unit records,
//! with a notice.
//! Why:
//!  The coordinator picked,
//! under the owner's delegation and open to the owner's veto
//!  (`doc/planning/cli-git-no-self-invocation.md`,
//! section "Falling back to the Git the commit service was set up
//!  with"),
//! that a wrapper which is found but has no real Git after it on `PATH` uses the Git it was set up with instead
//!  of stopping.
//! The search of `PATH` stays in `real_git.rs`;
//! this module adds the fallback after it and the types that
//!  carry its result,
//! so no caller can take the path and drop the notice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { path, notice } = resolveRealGit(inputs); // throws RealGitNotFound
//! ```

/// What:
///  `use super::...` imports the search,
/// the inputs and the failure from the module that owns them,
///  the candidate verdicts,
/// and the reader of the record.
/// Why:
///   The fallback is judged by the same `classify_candidate` that judges a `PATH` candidate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { searchPathForRealGit, type RealGitNotFound } from './real-git.ts';
/// ```
use crate::real_git::{RealGitNotFound, ResolutionInputs, SearchedCandidate, search_path_for_real_git};
use crate::real_git_candidate::{CandidateKind, classify_candidate};
use crate::real_git_record::read_recorded_git;
/// `OsStr` is borrowed operating-system text (siblings:
/// `OsString` owned,
/// `&str` UTF-8 only).
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
///  `&OsStr` and `&Path` borrow operating-system text and a path (siblings:
/// `OsString` and `PathBuf` own them).
///  `String` is owned UTF-8 text (sibling:
/// `&str`,
/// a borrowed view).
/// Why:
///  It says what happened (no real Git after the wrapper on `PATH`,
/// and which `PATH`),
///  what was done (which file,
/// read from which unit),
///  and the one way to make it stop (put the file's directory on the `PATH` of the program that ran the command).
///  An empty `PATH` is said in words,
/// as the failure does.
///  `String` and not `&str` because the line is built here and handed on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fallbackNotice(path: string, git: string, unit: string): string;
/// ```
fn fallback_notice(path: &OsStr, git: &Path, unit_file: &Path) -> String {
    // What:
    //  An `if`/`else` that yields a value, the last expression of each branch.
    //  `String::from("...")` copies a literal into owned text.
    //  `.to_string_lossy()` views operating-system text as UTF-8, replacing bytes that are not valid;
    //  `.into_owned()` makes the result an owned `String`.
    // Why:
    //  The notice shows the `PATH` the person's program had,
    //  and an empty one is more useful said as words than as nothing between parentheses.
    //
    // Gotcha:
    //  `.into_owned()` copies only when the view was borrowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const shownPath = path === '' ? 'empty or unset' : path;
    // ```
    let shown_path: String = if path.is_empty() {
        String::from("empty or unset")
    } else {
        path.to_string_lossy().into_owned()
    };
    // What:
    //  `match` on `git.parent()`, which is `Some(directory)` or `None`.
    //  `Some(found)` binds the directory;
    //  `.display()` makes a path printable and `.to_string()` turns that into owned text.
    // Why:
    //  The notice names the directory to add to `PATH`.
    //  A path with no parent has no such directory, and the words say so instead of printing nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const directory = dirname(git) ?? 'its directory';
    // ```
    let directory: String = match git.parent() {
        Some(found) => found.display().to_string(),
        None => String::from("its directory"),
    };
    // What:
    //  `format!` builds owned text from a template;
    //  `{shown_path}` and `{directory}` read the local variables by name, `{}` takes the next argument in order.
    //  A trailing `\` at the end of a line inside the quotes joins the next line without a break or its indentation.
    // Why:
    //  The notice is one line that ends in a newline, so it can lead standard error without disturbing what follows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return `cli-git: no real Git follows this wrapper on PATH (${shownPath}); used ${git}, ... add ${directory} to the PATH ...\n`;
    // ```
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
///  The coordinator picked,
/// under the owner's delegation and open to the owner's veto,
///  that a wrapper which is found but has no real Git after it on `PATH` uses the Git it was set up with,
///  instead of stopping.
///  The recorded file is judged by `classify_candidate` as a `PATH` candidate is:
///  never this executable,
/// a link or copy of it,
/// a launcher of it,
/// or a script that starts it,
///  and it must be an executable file that exists.
///  A refusal keeps the stop and adds the record to the failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveRealGit(inputs: ResolutionInputs): { path: string; notice?: string }; // throws RealGitNotFound
/// ```
pub fn resolve_real_git(inputs: &ResolutionInputs) -> Result<ResolvedGit, RealGitNotFound> {
    // What:
    //  `match` on the search's `Result`.
    //  `Ok(path)` is the success variant carrying the found path, and `return Ok(...)` leaves with it;
    //  `Err(failure)` is the failure variant, kept for the fallback.
    //  `mut` lets the failure be extended below.
    //  `&` in `search_path_for_real_git(inputs)` lends `inputs` without giving it away.
    // Why:
    //  A `PATH` that offers a real Git is used as it always was, with no notice and no look at the record.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let notFound; try { return { path: searchPathForRealGit(inputs), notice: undefined }; } catch (e) { notFound = e; }
    // ```
    let mut not_found: RealGitNotFound = match search_path_for_real_git(inputs) {
        Ok(path) => {
            return Ok(ResolvedGit { path, notice: None });
        }
        Err(failure) => failure,
    };
    // What:
    //  `let Some(unit_file) = ... else { ... };` takes the value out of an `Option` or runs the `else` block,
    //  which must leave the function.
    //  `.as_deref()` lends the optional owned path as an optional borrowed one.
    //  `Err(not_found)` returns the failure unchanged.
    // Why:
    //  Inputs that name no unit file have no record, and the stop stays as it was.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (inputs.serviceUnit === undefined) throw notFound;
    // ```
    let Some(unit_file) = inputs.service_unit.as_deref() else {
        return Err(not_found);
    };
    // A unit that cannot be read, or that names no absolute Git, is no record either.
    let Some(recorded) = read_recorded_git(unit_file) else {
        return Err(not_found);
    };
    // What:
    //  The recorded path gets the verdict a `PATH` candidate gets.
    //  `.as_path()` lends an owned path as a borrowed one.
    // Why:
    //  A record may name the wrapper, a launcher of it, or a file that is gone;
    //  starting any of them as Git would be wrong, so none is trusted before it is checked.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const kind = classifyCandidate(recorded, inputs.ownExecutable);
    // ```
    let kind: CandidateKind = classify_candidate(recorded.as_path(), inputs.own_executable.as_path());
    if kind == CandidateKind::RealGit {
        let notice: String = fallback_notice(inputs.path.as_os_str(), recorded.as_path(), unit_file);
        // `Some(notice)` wraps the line the caller must print.
        return Ok(ResolvedGit { path: recorded, notice: Some(notice) });
    }
    // What:
    //  `Some(RecordedGit { ... })` stores the refused record in the failure.
    //  `.to_path_buf()` copies the borrowed unit path into an owned one.
    //  `SearchedCandidate { path: recorded, kind }` moves the recorded path in, with the verdict.
    // Why:
    //  The failure then names the unit, the path it recorded and why it was not used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // notFound.recorded = { unitFile, candidate: { path: recorded, kind } };
    // ```
    not_found.recorded = Some(RecordedGit {
        unit_file: unit_file.to_path_buf(),
        candidate: SearchedCandidate { path: recorded, kind },
    });
    return Err(not_found);
}
