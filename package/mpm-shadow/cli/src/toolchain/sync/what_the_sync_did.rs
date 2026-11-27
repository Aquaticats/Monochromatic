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

/// Imports what a later install did with the leftovers of a killed one,
/// and the hidden name that holds the wait.
use super::super::archive_staging::{
    FinishedCopy,
    HoldingInstaller,
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
/// together with what went with a removed directory when a finished copy did,
/// for the end of a sentence that already said what became of the finished copy.
/// Why:
/// One wording,
/// so the two sentences about a recovery read the same way,
/// and a person learns that a removed directory took the only finished copy of its version with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removedClause(count: number, finishedCopies: number): string {
///   return `, and removed ${hiddenDirectories(count)} that ${whoLeftThem(count)} left behind${finishedCopiesClause(finishedCopies)}`;
/// }
/// ```
fn removed_clause(
    count: usize,
    finished_copies: usize,
) -> String {
    let mut clause: String = format!(
        ", and removed {} that {} left behind",
        hidden_directories(count),
        who_left_them(count)
    );
    if finished_copies > 0 {
        clause.push_str(&finished_copies_clause(finished_copies));
    }
    return clause;
}

/// What:
/// `, 1 of which held a finished copy that this run could not publish, so that copy is gone`,
/// or the same words for several copies.
/// Why:
/// A removed hidden directory may be the only finished copy of its version,
/// and a person has to know that the removal took it,
/// because the version then comes back only from a download or another copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishedCopiesClause(count: number): string {
///   return count === 1
///     ? ', 1 of which held a finished copy that this run could not publish, so that copy is gone'
///     : `, ${count} of which held a finished copy that this run could not publish, so those copies are gone`;
/// }
/// ```
fn finished_copies_clause(count: usize) -> String {
    if count == 1 {
        return ", 1 of which held a finished copy that this run could not publish, so that copy is gone"
            .to_string();
    }
    return format!(
        ", {count} of which held a finished copy that this run could not publish, so those copies are gone"
    );
}

/// What:
/// The sentence for a copy a killed install moved aside and this run left where it is,
/// because nothing it read shows that the install holding that name has ended.
/// Why:
/// A person looks for the install that holds the wait,
/// and the run knows only which hidden name it is and whether that name holds a finished copy:
/// a name whose installer the system reports running ends by itself,
/// while one no reader can judge may belong to an install that died where nothing can see it,
/// and then nothing of this build publishes that copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leftWhereItIs(id: string, version: string, holder: HoldingInstaller): string {
///   const moved = `left the copy of ${id} ${version} that an earlier install had moved aside where it is`;
///   if (holder.running) return `${moved}, because an install that is still running holds the hidden name ${holder.path}`;
///   const unjudged = `${moved}, because the hidden name ${holder.path} may belong to an install that is still running, and nothing here can judge that install`;
///   return holder.finishedCopy ? `${unjudged}; that name holds a finished copy of ${version}, which nothing here publishes while that install cannot be judged` : unjudged;
/// }
/// ```
fn left_where_it_is(
    id: &str,
    version: &str,
    holder: &HoldingInstaller,
) -> String {
    let moved_aside: String = format!(
        "left the copy of {id} {version} that an earlier install had moved aside where it is"
    );
    if holder.running {
        return format!(
            "{moved_aside}, because an install that is still running holds the hidden name {}",
            holder.path.display()
        );
    }
    let unjudged: String = format!(
        "{moved_aside}, because the hidden name {} may belong to an install that is still running, and nothing here can judge that install",
        holder.path.display()
    );
    if !holder.finished_copy {
        return unjudged;
    }
    return format!(
        "{unjudged}; that name holds a finished copy of {version}, which nothing here publishes while that install cannot be judged"
    );
}

/// What:
/// The sentence the install report prints for a version a recovery touched,
/// or `None` when the recovery found and did nothing.
/// Why:
/// An install that finished the publish of a version without downloading anything did something a person should see,
/// and so did one that dropped its copy for a publish another install made,
/// and one that only removed hidden directories of an installer known to have ended.
/// "Finished the publish" is printed only for a publish this run made,
/// and it names the hidden name the copy was read at rather than a kill no run can see.
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
    // What: how many of the removed directories held a finished copy, which the sentence has to say.
    // Why: a finished copy that went cannot be published by any later run.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const finishedCopiesRemoved = recovered.removed.filter((leftover) => leftover.heldAFinishedCopy).length;
    // ```
    let finished_copies_removed: usize = recovered
        .removed
        .iter()
        .filter(|leftover| return leftover.held_a_finished_copy)
        .count();
    let head: Option<String> = match &recovered.finished_copy {
        FinishedCopy::PublishedHere {
            copy,
        } => Some(format!(
            "finished the publish of {id} {version} from the finished copy an earlier install left at {}",
            copy.display()
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
        FinishedCopy::NothingToPublish => recovered
            .left_for_a_live_install
            .as_ref()
            .map(|holder| return left_where_it_is(id, version, holder)),
    };
    if let Some(mut sentence) = head {
        if removed > 0 {
            sentence.push_str(&removed_clause(removed, finished_copies_removed));
        }
        return Some(sentence);
    }
    if removed > 0 {
        let mut sentence: String = format!(
            "removed {} that {} of {id} left behind",
            hidden_directories(removed),
            who_left_them(removed)
        );
        if finished_copies_removed > 0 {
            sentence.push_str(&finished_copies_clause(finished_copies_removed));
        }
        return Some(sentence);
    }
    return None;
}

/// What:
/// The sentence the install report prints when a recovery put an older version at its name and the newest-only rule then
/// removed it.
/// Why:
/// The same run warns that older repository tools are never kept,
/// so the removal is a second thing a person should see rather than a silent finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function olderVersionsRemovedSentence(id: string, versions: string[]): string
/// ```
pub(crate) fn older_versions_removed_sentence(
    id: &str,
    versions: &[String],
) -> String {
    if versions.len() == 1 {
        return format!(
            "removed the older {id} {} that this run recovered, because a newer one is installed and an older repository tool is never kept",
            versions[0]
        );
    }
    return format!(
        "removed the older {id} versions {}, because a newer one is installed and an older repository tool is never kept",
        versions.join(", ")
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
