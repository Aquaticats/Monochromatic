//! What:
//! What one `install` run did:
//! the tools already at their version,
//! the ones it installed,
//! the ones that failed with why,
//! the declared bins still unresolved,
//! and the warnings it printed,
//! together with the warning for an install that would replace another version.
//! Why:
//! `install` prints the report at the end and the caller decides its exit status from it,
//! and the replacement warning is asked for a tool before it is installed,
//! so both travel through one type and one sentence.
//! A caller imports from `sync` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export type SyncReport = { unchanged: string[]; installed: string[]; reconciled: string[]; failed: [string, string][]; ... };
//! export function replacementWarning(id: string, observed: string | undefined, locked: string): string | undefined;
//! ```

/// Imports what a later install did with the leftovers of a killed one.
use super::super::archive_staging::{
    FinishedCopy,
    Recovered,
};
/// Imports the version comparison the warning uses.
use super::super::parse_output::version_matches;
/// What `install` did:
/// tools already at their version,
/// tools installed,
/// tools that failed (with why),
/// declared bins still
/// unresolved afterwards,
/// and warnings (a shared install being replaced).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SyncReport {
    /// Tool ids already at the locked version.
    pub unchanged: Vec<String>,
    /// Tool ids installed by this run.
    pub installed: Vec<String>,
    /// One sentence per declared Rust component or target this run added to a toolchain it did not install
    /// (one another program put at the locked version,
    /// which `install_tool` never sees).
    pub reconciled: Vec<String>,
    /// One sentence per version a recovery touched:
    /// the publish this run finished,
    /// the copy it dropped because another install had published the version meanwhile,
    /// the finished copy it left hidden because an older repository tool is never kept,
    /// the copy a killed install had moved aside that came back or was left for a live install,
    /// and the hidden directories it removed,
    /// each of which it found gone afterwards.
    pub recovered: Vec<String>,
    /// Tool id and failure text.
    pub failed: Vec<(String, String)>,
    /// Declared bin names without a resolving link.
    pub missing_bins: Vec<String>,
    /// Warnings printed before the install that caused them.
    pub warnings: Vec<String>,
    /// The tools `install` resolved and locked itself before installing,
    /// as tool id and locked version:
    /// the plan entries that were new or whose resolution-affecting fields changed.
    pub resolved: Vec<(String, String)>,
    /// What `install` wrote into the lock from the plan without a lookup:
    /// one sentence per offline-fixable difference,
    /// such as a changed `bins` list.
    pub rewritten: Vec<String>,
    /// The cleanup of old copies of this repository's tools:
    /// one sentence per link re-pointed and per old copy removed.
    pub old_copies: Vec<String>,
    /// One sentence per postinstall link an earlier run made and this run removed
    /// because the lock no longer declares it.
    pub retired_links: Vec<String>,
    /// The environment variable names the generated mise fragment holds after this run
    /// (empty when the lock declares none,
    /// or for a partial sync that leaves the fragment alone).
    pub exported_env: Vec<String>,
}

/// What:
/// `1 hidden directory`,
/// or `3 hidden directories`,
/// for a count of hidden siblings a recovery removed.
/// Why:
/// The report says how many went,
/// and a sentence that says "the 1 hidden directories" reads as a fault in the tool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hiddenDirectories(count: number): string { return count === 1 ? '1 hidden directory' : `${count} hidden directories`; }
/// ```
fn hidden_directories(count: usize) -> String {
    if count == 1 {
        return "1 hidden directory".to_string();
    }
    return format!("{count} hidden directories");
}

/// What:
/// `a killed install`,
/// or `killed installs`,
/// agreeing with the count of hidden directories.
/// Why:
/// The same sentence says one directory or several,
/// and the subject that left them changes with the count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whoLeftThem(count: number): string { return count === 1 ? 'a killed install' : 'killed installs'; }
/// ```
fn who_left_them(count: usize) -> &'static str {
    if count == 1 {
        return "a killed install";
    }
    return "killed installs";
}

/// What:
/// `, and removed 1 hidden directory that a killed install left behind`,
/// for the end of a sentence that already said what became of the finished copy.
/// Why:
/// One wording,
/// so the two sentences about a recovery read the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removedClause(count: number): string {
///   return `, and removed ${hiddenDirectories(count)} that ${whoLeftThem(count)} left behind`;
/// }
/// ```
fn removed_clause(count: usize) -> String {
    return format!(
        ", and removed {} that {} left behind",
        hidden_directories(count),
        who_left_them(count)
    );
}

/// What:
/// The sentence the install report prints for a version a recovery touched,
/// or `None` when the recovery found and did nothing.
/// Why:
/// An install that finished the publish of a version without downloading anything did something a person should see,
/// and so did one that dropped its copy for a publish another install made,
/// and one that only removed hidden directories of an installer known to have ended.
/// "Finished the publish" is printed only for a publish this run made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverySentence(id: string, version: string, recovered: Recovered): string | undefined
/// ```
pub(crate) fn recovery_sentence(
    id: &str,
    version: &str,
    recovered: &Recovered,
) -> Option<String> {
    let removed: usize = recovered.removed.len();
    let head: Option<String> = match &recovered.finished_copy {
        FinishedCopy::PublishedHere => Some(format!(
            "finished the publish of {id} {version} that a killed install left"
        )),
        FinishedCopy::PublishedByAnotherInstall => Some(format!(
            "dropped the copy of {id} {version} that a killed install left, because another install published that version meanwhile"
        )),
        FinishedCopy::AnOlderVersionIsNeverKept {
            newer,
        } => Some(format!(
            "left the finished copy of {id} {version} that a killed install left under its hidden name, because {newer} is installed and an older repository tool is never kept"
        )),
        FinishedCopy::NothingToPublish if recovered.restored => Some(format!(
            "put back the copy of {id} {version} that a killed install had moved aside"
        )),
        FinishedCopy::NothingToPublish if recovered.left_for_a_live_install => Some(format!(
            "left the copy of {id} {version} that a killed install had moved aside where it is, because another install is between its claim and its publish of that version"
        )),
        FinishedCopy::NothingToPublish => None,
    };
    if let Some(mut sentence) = head {
        if removed > 0 {
            sentence.push_str(&removed_clause(removed));
        }
        return Some(sentence);
    }
    if removed > 0 {
        return Some(format!(
            "removed {} that {} of {id} left behind",
            hidden_directories(removed),
            who_left_them(removed)
        ));
    }
    return None;
}

/// What:
/// The sentence the install report prints for one version the newest-only rule removed after a recovery put a copy at a
/// version's name:
/// the version,
/// whether this run is the one that put it there,
/// and the rule that removed it.
/// Why:
/// The rule removes the version this run recovered as well as a version that was installed before this run,
/// so one wording for both told a person the older version was the one that came back in this run;
/// the sentence says what the version was in this run,
/// which is the one fact a reader cannot see from the run's other lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function olderVersionRemovedSentence(id: string, version: string, putThereByThisRun: boolean): string
/// ```
pub(crate) fn older_version_removed_sentence(
    id: &str,
    version: &str,
    put_there_by_this_run: bool,
) -> String {
    // What: `if put_there_by_this_run { ... } else { ... }` picks the clause that says what the version was.
    // Why: a version this run published is one a recovery brought back, and any other version was there before the run
    //  started.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const was = putThereByThisRun ? 'that this run recovered' : 'that was installed before this run';
    // ```
    let was: &str = if put_there_by_this_run {
        "that this run recovered"
    } else {
        "that was installed before this run"
    };
    return format!(
        "removed the older {id} {version} {was}, because a newer one is installed and an older repository tool is never kept"
    );
}

/// What:
/// The warning for an install that would replace a different version,
/// or `None` when the tool is absent or
///           already matches.
/// Why:
/// Several worktrees share one install directory per tool version,
/// and a manager-level install (`cargo
///           binstall`,
/// `uv tool install`) replaces the one version everywhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replacementWarning(id: string, observed: string | undefined, locked: string): string | undefined
/// ```
pub fn replacement_warning(
    id: &str,
    observed: &Option<String>,
    locked: &str,
) -> Option<String> {
    let found: &str = observed.as_deref()?;
    if version_matches(found, locked) {
        return None;
    }
    return Some(format!(
        "{id} is installed at {found} but this tree's lock pins {locked}; installing {locked} replaces it for every tree \
         that shares this install, and a tree whose lock pins the other version replaces it back on its next sync"
    ));
}
