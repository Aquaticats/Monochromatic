//! What:
//!  The check that the removal's command path runs the read-only reading and no
//!  entry that writes.
//! Why:
//!  This release has no snapshot batch,
//!  head destroy,
//!  origin cleanup or Git settle,
//!  so a removal that filed the intent,
//!  unmounted the clone and wrote the destruction plan would leave a person
//!  with an unmounted clone and a plan no command of this release could finish
//!  or cancel
//!  (plan steps 14 to 17;
//!  `worktree_clone_removal_proof_and_plan::removal_reading`).
//!  The command's own controls drive the reading and the scripted pool,
//!  so none of them would notice the wiring handed over to the writing entry:
//!  that is a fact about which function the release names,
//!  and this check reads the release source for it.
//!  Plan step 16's wiring is the change that names the entry,
//!  and it removes this check in the same commit,
//!  listing it in its round's report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // source_guard/removal_command_path_writes_nothing_tests.ts: one name check over the release tree.
//! ```

use std::path::Path;

use super::blocking_code_off_the_runtime_tests::read_the_code_of;
use crate::source_scan_tests::{CrateRoot, classified_sources, real_roots, source_root};

/// The folder the removal's own steps live in,
/// whose files are the ones allowed to name the writing entry.
const THE_REMOVAL_FOLDER: &str = "worktree_clone_removal_proof_and_plan/";

/// The one call that files the intent,
/// unmounts the clone and writes the destruction plan.
const THE_WRITING_ENTRY: &str = "enter_the_removal";

/// The call this release's removal command makes instead.
/// The opening parenthesis is part of it, because an import names it too:
/// the check is about the call the command path makes, not the name in a `use` line.
const THE_READING_CALL: &str = "read_the_removal(&";

/// The removal's command path.
const THE_COMMAND_PATH: &str = "worktree_copying/worktree_removal_of_a_bound_clone.rs";

/// The file that declares the writing entry,
/// which the check reads to prove the name it bans is still a name in the tree.
const THE_ENTRY_FILE: &str = "worktree_clone_removal_proof_and_plan/removal_entry.rs";

/// What:
///  One release file's code,
///  comments and literal characters masked,
///  so a doc line about the entry is not a call to it.
/// Why:
///  Both the command path's doc and the module's doc name the entry in prose to
///  say where it belongs,
///  and a check that read prose could only be satisfied by deleting the
///  explanation.
///
/// # Throws
/// The file whose literals or comments do not close.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const code = mask(readFileSync(path, 'utf8'));
/// ```
fn code_of(root: &Path, path: &str) -> Result<String, String> {
    let text: String = std::fs::read_to_string(root.join(path)).map_err(|error| return format!("{path} cannot be read: {error}"))?;
    let read = read_the_code_of(text.as_str()).map_err(|error| return format!("{path} cannot be read as code: {error}"))?;
    return Ok(read.masked.join("\n"));
}

/// What:
///  Every release file outside the removal's own folder that names the writing
///  entry,
///  and whether the command path names the reading instead.
/// Why:
///  The scan answers the one question the check asks,
///  so the test reads as the rule does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const callers = releaseFiles.filter((file) => !file.startsWith(folder) && file.code.includes('enter_the_removal'));
/// ```
fn callers_of_the_writing_entry(root: &Path, roots: &[CrateRoot]) -> Result<(Vec<String>, bool, bool), String> {
    let mut callers: Vec<String> = Vec::new();
    let mut the_command_path_reads: bool = false;
    let mut the_entry_is_declared: bool = false;
    for file in classified_sources(root, roots)? {
        if file.test_only {
            continue;
        }
        let code: String = code_of(root, file.path.as_str())?;
        if file.path == THE_ENTRY_FILE && code.contains("fn enter_the_removal(") {
            the_entry_is_declared = true;
        }
        if file.path == THE_COMMAND_PATH && code.contains(THE_READING_CALL) {
            the_command_path_reads = true;
        }
        if !file.path.starts_with(THE_REMOVAL_FOLDER) && code.contains(THE_WRITING_ENTRY) {
            callers.push(file.path);
        }
    }
    return Ok((callers, the_command_path_reads, the_entry_is_declared));
}

/// What:
///  No release file outside the removal's own folder names the entry that
///  writes,
///  the command path names the reading,
///  and the entry is still declared where the check expects it.
/// Why:
///  The first two are the rule:
///  the writing entry waits for the steps that can finish or cancel what it
///  starts.
///  The third keeps the check from passing on a tree where the entry was simply
///  deleted,
///  which would leave plan steps 15 to 17 nothing to wire.
#[test]
fn the_removal_command_path_runs_the_reading_and_no_writing_entry() {
    let root = source_root();
    let (callers, the_command_path_reads, the_entry_is_declared) = callers_of_the_writing_entry(root.as_path(), real_roots().as_slice()).expect("the crate's release source tree reads");
    assert!(the_entry_is_declared, "positive control: `{THE_ENTRY_FILE}` still declares `{THE_WRITING_ENTRY}`, so the name this check bans is a name in the tree");
    assert!(the_command_path_reads, "`{THE_COMMAND_PATH}` must call `{THE_READING_CALL}…)`: this release's removal proves its lineage read-only and writes nothing");
    assert!(
        callers.is_empty(),
        "no release file outside `{THE_REMOVAL_FOLDER}` may name `{THE_WRITING_ENTRY}` until plan step 16 wires it, and these do: {callers:?}"
    );
}
