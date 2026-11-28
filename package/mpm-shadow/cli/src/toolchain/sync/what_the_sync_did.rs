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
/// the hidden name that holds the wait,
/// and what it read about the finished copy that went with a directory it removed.
use super::super::archive_staging::{
    FinishedCopy,
    HoldingInstaller,
    Recovered,
    RemovedFinishedCopy,
    RemovedLeftover,
    WhyNotPublished,
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
    /// the copy an earlier install had moved aside that came back or was left for a live install,
    /// and the hidden directories it removed,
    /// each of which it found gone afterwards,
    /// with what went with one when a finished copy did and whether that copy was a second one.
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
    /// One sentence per symbolic link this run removed from the exposure directory
    /// because no row of the lock declares its name.
    pub removed_exposure_links: Vec<String>,
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
/// `an install of mpm-shadow that has ended`,
/// or `installs of mpm-shadow that have ended`,
/// agreeing with the count of hidden directories.
/// Why:
/// A hidden directory is removed only when the system answered that its installer ended,
/// so the sentence names that answer and never a kill no run read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whoLeftThem(id: string, count: number): string {
///   return count === 1 ? `an install of ${id} that has ended` : `installs of ${id} that have ended`;
/// }
/// ```
fn who_left_them(
    id: &str,
    count: usize,
) -> String {
    if count == 1 {
        return format!("an install of {id} that has ended");
    }
    return format!("installs of {id} that have ended");
}

/// What:
/// `removed 1 hidden directory left behind by an install of mpm-shadow that has ended`,
/// together with one clause for each removed directory that held a finished copy.
/// Why:
/// One wording,
/// so the sentence that stands on its own and the clause a recovery appends to another sentence read the same way,
/// and a person learns what each removed directory took with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removedPart(id: string, removed: RemovedLeftover[]): string {
///   return `removed ${hiddenDirectories(removed.length)} left behind by ${whoLeftThem(id, removed.length)}`
///     + removed.map(whatWentWithIt).join('');
/// }
/// ```
fn removed_part(
    id: &str,
    removed: &[&RemovedLeftover],
) -> String {
    let mut part: String = format!(
        "removed {} left behind by {}",
        hidden_directories(removed.len()),
        who_left_them(id, removed.len())
    );
    // What: `.iter()` walks the removed directories, and each one that held a finished copy adds its own clause.
    // Why: the version and the reason differ per copy, so each clause is built from the directory it is about.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const leftover of removed) part += whatWentWithIt(leftover);
    // ```
    for leftover in removed {
        part.push_str(&what_went_with_it(leftover));
    }
    return part;
}

/// What:
/// `, 1 of which held a second finished copy of 0.2.1, because 0.2.1 holds its completion marker and its program at its name`,
/// `, 1 of which held a finished copy of 0.2.1 that this run did not publish, because ...; no finished copy of 0.2.1 is at its name, so that copy is gone`,
/// or nothing when the directory held no finished copy.
/// Why:
/// A removed finished copy is either a second one or the one the version had here,
/// and only the second case is the copy a person loses;
/// the first says which reads made the run call the version finished,
/// because `is_finished` asks for a completion marker and a program found under the name,
/// and nothing of the bytes either one holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatWentWithIt(leftover: RemovedLeftover): string {
///   if (leftover.finishedCopy === 'no-finished-copy') return '';
///   if ('secondCopy' in leftover.finishedCopy) {
///     const { secondCopy: version, programFound } = leftover.finishedCopy;
///     const read = programFound ? 'holds its completion marker and its program' : 'holds its completion marker';
///     return `, 1 of which held a second finished copy of ${version}, because ${version} ${read} at its name`;
///   }
///   const { version, whyNotPublished } = leftover.finishedCopy.notAtItsName;
///   return `, 1 of which held a finished copy of ${version} that this run did not publish, because ${whyNotPublishedText(whyNotPublished)}; no finished copy of ${version} is at its name, so that copy is gone`;
/// }
/// ```
fn what_went_with_it(leftover: &RemovedLeftover) -> String {
    // What: `match` on the three answers, which is a discriminated union in TS, each of which reads differently.
    // Why: the reason a copy was not published is a different fact in each arm, so no one wording covers them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (leftover.finishedCopy === 'no-finished-copy') return '';
    // if ('secondCopy' in leftover.finishedCopy) return secondCopyClause(leftover.finishedCopy.secondCopy);
    // return notAtItsNameClause(leftover.finishedCopy.notAtItsName);
    // ```
    return match &leftover.finished_copy {
        RemovedFinishedCopy::NoFinishedCopy => String::new(),
        // What: the clause names the two reads the run made at the version's own name, and nothing about its bytes.
        // Why: the marker and a program found are what `is_finished` asks, so a program cut short under its marker
        //  reads as finished here, and a sentence that said nothing was lost, or that the copy at the name is the one
        //  a person keeps, would claim more than the run read.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const read = programFound ? 'holds its completion marker and its program' : 'holds its completion marker';
        // return `, 1 of which held a second finished copy of ${version}, because ${version} ${read} at its name`;
        // ```
        RemovedFinishedCopy::SecondCopy {
            version,
            program_found,
        } => {
            // What: which of the two reads the sentence may name, since a program is looked for only when its name is known.
            // Why: `is_finished` reads the marker alone for a tool that declares no bin, and the sentence has to match.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const read = programFound ? 'holds its completion marker and its program' : 'holds its completion marker';
            // ```
            let read: &str = if *program_found {
                "holds its completion marker and its program"
            } else {
                "holds its completion marker"
            };
            format!(
                ", 1 of which held a second finished copy of {version}, because {version} {read} at its name"
            )
        }
        RemovedFinishedCopy::NotAtItsName {
            version,
            why_not_published,
        } => format!(
            ", 1 of which held a finished copy of {version} that this run did not publish, because {}; no finished copy of {version} is at its name, so that copy is gone",
            why_not_published_text(why_not_published)
        ),
    };
}

/// What:
/// The clause that says why this run did not publish a finished copy it removed.
/// Why:
/// Both reasons are facts the run read,
/// and naming one it did not read is the fault this wording exists to avoid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whyNotPublishedText(why: WhyNotPublished): string {
///   return why === 'the-lock-does-not-record-its-archive'
///     ? "that copy's completion marker does not record the archive this lock names"
///     : `${why.anOlderRepositoryToolIsNeverKept} is installed and an older repository tool is never kept`;
/// }
/// ```
fn why_not_published_text(why: &WhyNotPublished) -> String {
    return match why {
        WhyNotPublished::TheLockDoesNotRecordItsArchive => {
            "that copy's completion marker does not record the archive this lock names".to_string()
        }
        WhyNotPublished::AnOlderRepositoryToolIsNeverKept {
            newer,
        } => format!("{newer} is installed and an older repository tool is never kept"),
    };
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
/// and one that only removed hidden directories whose installer the system reports ended.
/// "Finished the publish" is printed only for a publish this run made,
/// and it names the hidden name the copy was read at rather than a kill no run can see.
/// A removed directory that held a finished copy says whether that copy was a second one or the version's only one
/// here,
/// which is the one fact that decides whether the person lost something.
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
    // What: the removed directories this run can name in a sentence, in the order it found them.
    // Why: `removed_part` takes the directories themselves, because what went with each one decides its clause.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const removedDirectories = recovered.removed;
    // ```
    let removed_directories: Vec<&RemovedLeftover> = recovered.removed.iter().collect();
    let head: Option<String> = match &recovered.finished_copy {
        FinishedCopy::PublishedHere {
            copy,
        } => Some(format!(
            "finished the publish of {id} {version} from the finished copy an earlier install left at {}",
            copy.display()
        )),
        FinishedCopy::PublishedByAnotherInstall => Some(format!(
            "dropped the copy of {id} {version} that an earlier install left, because another install published that version meanwhile"
        )),
        FinishedCopy::AnOlderVersionIsNeverKept {
            newer,
        } => Some(format!(
            "left the finished copy of {id} {version} under the hidden name an earlier install left it at, because {newer} is installed and an older repository tool is never kept"
        )),
        FinishedCopy::NothingToPublish if recovered.restored => Some(format!(
            "put back the copy of {id} {version} that an earlier install had moved aside"
        )),
        FinishedCopy::NothingToPublish => recovered
            .left_for_a_live_install
            .as_ref()
            .map(|holder| return left_where_it_is(id, version, holder)),
    };
    if let Some(mut sentence) = head {
        if removed > 0 {
            // What: `, and ` joins this sentence to the removal clause, which is the same one a run with no head prints.
            // Why: a person reads one sentence per version, whatever else the recovery did.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // sentence += `, and ${removedPart(id, removedDirectories)}`;
            // ```
            sentence.push_str(", and ");
            sentence.push_str(&removed_part(id, &removed_directories));
        }
        return Some(sentence);
    }
    if removed > 0 {
        return Some(removed_part(id, &removed_directories));
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
