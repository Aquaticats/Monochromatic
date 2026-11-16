//! What:
//!  The PID namespace an owner record names,
//! and the rule for judging an owner that lives in another one.
//! Why:
//!  A PID means something only inside its PID namespace.
//! A wrapper in a container that probes a host owner's PID
//!  finds no process (or an unrelated one),
//! so it would judge a live owner dead and recover or retire what the owner
//!  still holds (issues 1781 and 1784).
//! A record therefore names its namespace,
//! and an owner of another namespace is
//!  judged by the age of its record instead of by the PID.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sameNamespace = readlinkSync('/proc/self/ns/pid') === record.ownerPidNamespace;
//! ```

/// The longest a merge waits for a foreign program's `index.lock`,
/// which the patience is derived from.
use crate::wait_ceilings::MERGE_LOCK_CEILING;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// How long an owner of another PID namespace counts as alive,
/// measured from when its record was written.
///  A commit or a lock hold lasts seconds to minutes.
/// Past this age a record is neither alive nor dead:
/// the command that finds it stops,
///  changes nothing and names it (`RefusedForeignRecord`;
/// owner,
/// 2026-11-04,
/// "Refuse and name it (Recommended)"),
/// because the age proves no
///  death,
/// and a crashed foreign owner's record is removed by the person with the `rm` the error gives.
///  Three kinds of record are judged by this age.
/// A merge's intent record (`merge_isolation_journal.rs`,
/// `write_merge_intent`,
///  written once and synced per merge) and a claim file (`single_owner_claim.rs`,
/// `write_private`,
/// written once) are not
///  rewritten while the owner works,
/// so the age is the owner's whole hold;
/// the item path of a worktree add is judged by its own
///  modification time (`worktree_copy_owner.rs`,
/// `is_alive`),
/// which the add's work can move,
/// so its age is at most the add's
///  length.
/// The patience should outlast the longest such hold;
/// the owner chose a shorter one than the longest add measured here,
/// and "Limit" below says what that costs.
///  Basis:
/// twice `MERGE_LOCK_CEILING` (one minute,
/// `wait_ceilings.rs`),
/// the longest a merge waits for a foreign `index.lock` with its
///  record already written,
/// so two minutes:
/// the owner,
/// 2026-11-10,
/// "Two minutes (Recommended)".
/// It is derived,
/// not written as a number,
/// so the two cannot drift apart again:
/// it was
///  an hour because it was twice the 30 minute live lock wait of `wait_limit.rs`,
/// a file that no longer exists,
/// and then 20 minutes because it was twice the unmeasured ten minutes of a
///  lock ceiling,
/// which no wait took any more and which this release deleted
/// (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "The lock ceiling and the three waits it fed").
/// The owner's
///  rule (2026-11-03,
/// "nothing should ever wait forever or indefinitely") bounded those waits instead.
/// Measured:
/// the longest hold of a lock the commit service's journal records is the `lock_hold_ms` of
///  119,721 ms (4,386 `answered` lines,
/// 2026-11-02T04:01:38-05:00 to 2026-11-08T18:49:55-05:00),
/// a hold of the index,
/// which leaves 239 ms
///  inside this patience.
/// That figure is the closest proxy there is:
/// the journal times the service's own holds,
/// and no log keeps the age of a record of another namespace.
/// The longest whole commands of the same logs are longer
/// (539.7 s for the longest of 79 worktree adds,
///  750.6 s for the longest of 1,838 commits,
/// and 1,228,797 ms in the timing log),
/// and none of them says how long a record stood:
/// the logs time commands of the host's own PID namespace,
///  not records of another one.
/// Limit:
/// a hold of another namespace that does run past two minutes is refused and named
/// (`RefusedForeignRecord`),
///  never taken over,
/// so nothing it holds is lost,
/// and the person who knows the owner is gone removes the record with the `rm` the error gives.
///  That refusal is what a worktree add of another namespace meets when its item path stands still past this age,
/// and the longest
///  add measured here (539.7 s) is longer than it.
/// If `MERGE_LOCK_CEILING` changes,
/// this value follows it;
/// revisit the margin then
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Foreign-owner patience").
pub const FOREIGN_OWNER_PATIENCE: Duration = Duration::from_secs(2 * MERGE_LOCK_CEILING.as_secs());

/// What:
///  The identity of this process's PID namespace,
/// such as `pid:[4026531836]`,
/// or nothing where the system has none.
/// Why:
///   The kernel names a namespace by an inode;
/// two processes share a PID space exactly when the names are equal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readlinkSync('/proc/self/ns/pid')
/// ```
pub fn current_pid_namespace() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        // `?` in an `Option` function returns `None` early, where the old `.ok().and_then(..)` chain produced `None` too.
        let target: std::path::PathBuf = std::fs::read_link("/proc/self/ns/pid").ok()?;
        return target.into_os_string().into_string().ok();
    }
    #[cfg(not(target_os = "linux"))]
    {
        return None;
    }
}

/// What:
///  The value a PID namespace names:
///  the number inside the `/proc/self/ns/pid` link's `pid:[...]` text,
///  or a bare number,
///  with no leading zero,
///  or nothing when the text is no number at all.
/// Why:
///  A namespace is named by one number,
/// and one number has one value however many zeros lead it:
/// two texts that name one value compared as text name two namespaces,
/// and a record's own namespace read as another one is judged by its age and can refuse the command of a live owner.
/// A text that is no number names no namespace this reader can compare.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pidNamespaceValue('pid:[04026531836]') // '4026531836'
/// pidNamespaceValue('4026531836') // '4026531836'
/// pidNamespaceValue('pid:[x]') // undefined
/// ```
pub fn pid_namespace_value(text: &str) -> Option<String> {
    let number: &str = text
        .strip_prefix("pid:[")
        .and_then(|rest| { return rest.strip_suffix(']'); })
        .unwrap_or(text);
    return crate::process_identity::kernel_decimal(number);
}

/// What:
///  Whether a recorded namespace and the reader's namespace are both known and differ.
/// Why:
///   Without both names nothing proves the PID space differs,
/// so the PID probe keeps its old meaning;
/// a record
///  written before this field existed carries no name.
/// The two are compared by the number each names, so a namespace written with a leading zero is the namespace it names;
/// a text that names no number is read as one this reader cannot show to be its own, which judges the record by its age.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recorded !== undefined && current !== undefined && recorded !== current
/// ```
pub fn is_other_namespace(recorded: Option<&str>, current: Option<&str>) -> bool {
    return match (recorded, current) {
        (Some(recorded_name), Some(current_name)) if recorded_name == current_name => false,
        (Some(recorded_name), Some(current_name)) => match (pid_namespace_value(recorded_name), pid_namespace_value(current_name)) {
            (Some(recorded_value), Some(current_value)) => recorded_value != current_value,
            // A text that names no namespace number is no name this reader can show to be its own.
            _ => true,
        },
        _ => false,
    };
}

/// What:
///  How long a record file has stood,
///  as its own file time says it.
/// Why:
///   The record file's modification time stands for when the owner started;
/// a time that is later than this reader's clock has no age at all,
/// and a time that cannot be read has none either:
/// a clock that was set back here,
/// a record copied with its time,
/// or another machine's clock produces the first,
/// and neither proves the owner alive or dead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordAge = { kind: 'measured'; age: number } | { kind: 'ahead' } | { kind: 'unreadable'; reason: string } | { kind: 'gone' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordAge {
    /// The file's time is at or before this reader's clock:
    /// the record has stood this long.
    Measured(Duration),
    /// The file's time is later than this reader's clock,
    /// so no age exists.
    Ahead,
    /// The file's time cannot be read,
    /// with the reason the platform gave.
    Unreadable(String),
    /// The record is not there any more:
    /// its owner finished and removed it,
    /// so nothing stands in the way and nothing is unreadable.
    Gone,
}

/// What:
///  The age of a record file,
/// or why there is none.
/// Why:
///   The record file's modification time stands for when the owner started;
/// a time that is later than this reader's clock cannot be turned into an age,
/// and taking it for a live owner's keeps a dead owner's record in the way for ever
/// (audit of 2026-11-09,
/// finding 2),
///  so the two cases are told apart here instead of being answered together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Date.now() - statSync(recordPath).mtimeMs // throws when the time is ahead of the clock
/// ```
pub fn foreign_record_age(record_path: &Path) -> RecordAge {
    // A record that is gone was finished by the owner that wrote it, and it is not an unreadable one: the owner that
    // completes its merge removes its intent between a reader's two reads, and a refusal there stopped the command with
    // an instruction to remove a file that no longer exists (cross-vendor review of 2026-11-09, finding 3).
    let metadata: std::fs::Metadata = match std::fs::metadata(record_path) {
        Ok(found) => found,
        Err(error) if is_missing(&error) => return RecordAge::Gone,
        Err(error) => return RecordAge::Unreadable(error.to_string()),
    };
    let modified: SystemTime = match metadata.modified() {
        Ok(found) => found,
        Err(error) => return RecordAge::Unreadable(error.to_string()),
    };
    return match SystemTime::now().duration_since(modified) {
        Ok(age) => RecordAge::Measured(age),
        Err(_) => RecordAge::Ahead,
    };
}

/// What:
///  Whether an error says the path is not there.
/// Why:
///   A record that is gone and a path that names no file are both absent records,
///  and neither is an unreadable one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.code === 'ENOENT' || error.code === 'ENOTDIR'
/// ```
fn is_missing(error: &std::io::Error) -> bool {
    return matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory);
}

/// What:
///  Whether a record of this age has outlived the patience;
/// exactly the patience still counts as alive.
/// Why:
///   The boundary is a fact of the rule,
/// so it is separated from the clock to be checked exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// age > FOREIGN_OWNER_PATIENCE_MS
/// ```
pub fn age_outlives_patience(age: Duration) -> bool {
    return age > FOREIGN_OWNER_PATIENCE;
}

/// What:
///  What a record of another PID namespace says about its owner now.
/// Why:
///   An owner of another namespace is judged by its record and not by its PID,
///  so a record that stands,
///  one that is gone and one that is refused are told apart:
///  a gone record is a finished owner's,
///  which holds nothing and in whose way nothing stands
///  (cross-vendor review of 2026-11-09,
///  finding 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForeignStanding = { kind: 'standing' } | { kind: 'finished' } | { kind: 'refused'; refusal: RefusedForeignRecord };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForeignStanding {
    /// The record stands and is inside the patience:
    /// its owner may still run.
    Standing,
    /// The record is gone:
    /// its owner finished and removed it,
    /// so nothing holds and nothing is in the way.
    Finished,
    /// The record is neither alive nor dead,
    /// and the command stops with it named.
    Refused(RefusedForeignRecord),
}

/// What:
///  Why a record of another PID namespace is refused:
/// its measured age past the patience,
/// or a file time that cannot be trusted at all.
/// Why:
///   A measured age and an untrustworthy time are different facts,
/// and the person deciding what to do needs the one that applies:
/// an age says how long the record has stood (by this host's clock),
/// and a time later than the clock says nothing about the hold at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForeignRefusal = { kind: 'past-the-patience'; age: number } | { kind: 'time-ahead' } | { kind: 'time-unreadable'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForeignRefusal {
    /// The record's age is measured and past the patience;
    /// the age proves no death.
    PastThePatience(Duration),
    /// The record's own time is later than this reader's clock,
    /// so no age exists to weigh.
    TimeAhead,
    /// The record's file time cannot be read,
    /// with the reason the platform gave.
    TimeUnreadable(String),
}

/// What:
///  A record of another PID namespace that is neither alive nor dead:
/// one past the patience,
/// and one whose own time cannot be trusted.
/// Why:
///   Owner,
/// 2026-11-04,
/// "Refuse and name it (Recommended)":
/// expiry no longer authorizes a command to treat a foreign owner as dead
///  and to recover,
/// take over or remove what it holds.
/// The age is a length that nothing bounds (a write that cannot be run twice may
///  run past any number,
/// and a step the wrapper left running outlives the wrapper),
/// so it proves no death.
/// A file time later than this reader's clock is no age either:
/// it is what a clock that was set back,
/// a copied time or another machine's clock produces,
/// and it says nothing about the owner (audit of 2026-11-09,
/// finding 2).
/// The command stops,
/// changes
///  nothing,
/// and says which record it found and what a person checks before removing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefusedForeignRecord = { record: string; namespace: string; because: ForeignRefusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefusedForeignRecord {
    /// The record file.
    pub record: std::path::PathBuf,
    /// The PID namespace the record names.
    pub namespace: String,
    /// Why the record is refused.
    pub because: ForeignRefusal,
}

impl RefusedForeignRecord {
    /// What:
    ///  The error a command stops with:
    /// the record file,
    /// the owner's PID and namespace,
    /// why the record is refused,
    /// that the owner may be live in
    ///  another container or a Git process left running,
    /// and the exact `rm`.
    /// Why:
    ///   The person must be able to decide,
    /// and the condition for removing the record covers the Git processes the wrapper leaves
    ///  running (a step that cannot be run twice survives the wrapper that started it),
    /// not only a running monochromatic-git-policy-cli command.
    /// A measured age is said to be this host's reading of the record,
    /// because a clock that moved makes it read longer or shorter than the hold.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// `monochromatic-git-policy-cli: the record ${record} was written by process ${pid} ... rm -- ${quoted(record)}`
    /// ```
    pub fn describe(&self, owner_pid: i64) -> String {
        let state: String = refusal_state(&self.because);
        return format!(
            "monochromatic-git-policy-cli: the record {record} was written by process {owner_pid} in the PID namespace {namespace}, which is not this one, {state}. Nothing was \
             changed. The owner may still be alive in another container, and a Git process that monochromatic-git-policy-cli left running (a write it could not \
             safely end, such as a merge, a reset or a branch update) may still be working for it. Look in every container and on this host \
             that use this repository for a monochromatic-git-policy-cli command and for a Git process monochromatic-git-policy-cli left running (`ps` shows them); when none runs, \
             remove the record with `rm -- {path}` and run the command again.",
            record = self.record.display(),
            namespace = self.namespace,
            path = crate::stale_ref_lock_message::shell_quote(self.record.as_path())
        );
    }
}

/// What:
///  How a record is judged when it names another PID namespace:
/// alive while its age is measured and younger than the patience,
/// finished once it is gone,
/// refused once that age is past it,
/// and refused as soon as its own time cannot be trusted,
/// and
///  nothing for a record of this namespace or of none,
/// which the PID decides.
/// A record that is gone is finished:
/// its owner removed it,
/// so nothing holds and no command stops over it.
/// Why:
///   Callers share one rule:
/// an owner of another namespace is alive only while its record's age is measured and inside the patience;
///  past it,
/// or without a time that can be measured,
/// the command stops (`RefusedForeignRecord`) and changes nothing.
/// An age that cannot be measured is never taken for a live owner's:
/// that answer would keep the record in the way for as long as its time stays ahead of the clock
/// (audit of 2026-11-09,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isOtherNamespace(recorded, current) ? standingOf(ageOf(recordPath)) : undefined
/// ```
pub fn foreign_owner_standing(recorded: Option<&str>, record_path: &Path) -> Option<ForeignStanding> {
    if !is_other_namespace(recorded, current_pid_namespace().as_deref()) {
        return None;
    }
    let namespace: String = recorded.map_or_else(String::new, String::from);
    return Some(match aged(record_path) {
        Aged::Standing => ForeignStanding::Standing,
        Aged::Finished => ForeignStanding::Finished,
        Aged::Refused(because) => ForeignStanding::Refused(RefusedForeignRecord { record: record_path.to_path_buf(), namespace, because }),
    });
}

/// What:
///  What a record's own age says about a holder that its process number cannot decide.
/// Why:
///   Two owners are judged by the age of their record rather than by the number it names:
///  one of another PID namespace,
///  whose number means nothing here,
///  and one whose birth identity is no value this command can read,
///  whose number names a process that may be the owner or may have taken its number.
///  Both share one reading of the age,
///  so they cannot drift apart,
///  and a record that is gone is finished for both:
///  its owner removed it,
///  so nothing holds and no command stops over a file that is not there
///  (cross-vendor review of 2026-11-09,
///  finding 3).
enum Aged {
    /// The record stands and is inside the patience: its owner may still run.
    Standing,
    /// The record is gone: its owner finished and removed it.
    Finished,
    /// The record is neither, with the reason its age cannot decide it.
    Refused(ForeignRefusal),
}

/// `Aged` for a record file,
/// from one reading of its own time.
fn aged(record_path: &Path) -> Aged {
    #[cfg(test)]
    STANDING_LOOKUPS.with(|count| { count.set(count.get() + 1); });
    return match foreign_record_age(record_path) {
        RecordAge::Measured(age) if age_outlives_patience(age) => Aged::Refused(ForeignRefusal::PastThePatience(age)),
        RecordAge::Measured(_) => Aged::Standing,
        RecordAge::Gone => Aged::Finished,
        RecordAge::Ahead => Aged::Refused(ForeignRefusal::TimeAhead),
        RecordAge::Unreadable(reason) => Aged::Refused(ForeignRefusal::TimeUnreadable(reason)),
    };
}

/// What:
///  The part of a record whose holder this command could not read.
/// Why:
///  A record says who holds it in two parts, and either can be no value this command can read:
///  the birth identity it names,
///  or the process record the platform keeps for the number it names.
///  The words a person reads say which of the two it was,
///  because what they check before removing the record differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnreadablePart = { birth: string } | { process: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnreadablePart {
    /// The birth identity the record names, as it is written: no spelling of a value this command can read.
    Birth(String),
    /// The process the record names: the platform would not report it, with the reason it gave.
    Process(String),
    /// The record's own text names no process at all: the owner line is missing, has no process number, carries text that
    ///  is no number, or the record is empty, with the text it holds.
    Owner(String),
}

/// What:
///  A record whose holder this command could not read, and which has stood past the patience:
///  neither a live owner's nor a gone one's.
/// Why:
///   The wrapper writes one spelling of a birth,
///  so a text that is no spelling of a value was written by something that is not this release (an older release,
///  another tool,
///  a damaged file),
///  and a process the platform will not report on can be a live owner's as easily as a reused number's.
///  Either way the record may be a live owner's, and a process that has the number says nothing,
///  because the part that would tell the two apart cannot be read.
///  Owner,
///  2026-11-04,
///  "Refuse and name it (Recommended)" is the rule for the same state of a record of another PID namespace,
///  and this is that rule applied to either part:
///  inside the patience the record stands for an owner that may run,
///  and past it the command stops,
///  changes nothing,
///  and says which record it found,
///  which part it cannot read,
///  and what a person does about it.
///  Owner,
///  2026-11-16,
///  "A: stays, command says how (Recommended)" is that decision made,
///  and it holds for every reader of such a record on this branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefusedUntellableRecord = { record: string; part: UnreadablePart; because: ForeignRefusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefusedUntellableRecord {
    /// The record file, or the directory a record of a copy stands in.
    pub record: std::path::PathBuf,
    /// The part of the record this command could not read.
    pub part: UnreadablePart,
    /// Why the record's age cannot decide it either.
    pub because: ForeignRefusal,
}

impl RefusedUntellableRecord {
    /// What:
    ///  The sentence a command stops with, or says:
    ///  the record file,
    ///  the owner's PID,
    ///  the part that cannot be read,
    ///  how long it has stood,
    ///  and the exact command that removes it.
    /// Why:
    ///   The person must be able to decide.
    ///  Nothing in the record says whether its owner still runs,
    ///  so the words say that the record may be a live owner's and that a command of this repository may be working;
    ///  the condition for removing it covers the Git processes the wrapper leaves running,
    ///  not only a running monochromatic-git-policy-cli command.
    ///  A copy keeps its record in a directory of its own,
    ///  so the removal the words give is `rm --recursive` for a directory and `rm` for a file:
    ///  the one the person runs from the words must remove what the words name.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// `monochromatic-git-policy-cli: the record ${record} was written by process ${pid}, ${part}, ... rm ${recursive}-- ${quoted(record)}`
    /// ```
    pub fn describe(&self, owner_pid: i64) -> String {
        let writer: String = match &self.part {
            UnreadablePart::Owner(_) => String::from("was written by a process it does not name in a form this command can read"),
            UnreadablePart::Birth(_) | UnreadablePart::Process(_) => format!("was written by process {owner_pid}"),
        };
        let part: String = match &self.part {
            UnreadablePart::Birth(birth) => format!(
                "and the birth identity it names ({birth}) is no spelling of a value this command can read"
            ),
            UnreadablePart::Process(reason) => format!("and the process record of that process cannot be read ({reason})"),
            UnreadablePart::Owner(text) => format!(
                "and the owner it names ({text}) is no process and birth identity this command can read"
            ),
        };
        let removal: String = match self.record.is_dir() {
            true => format!("remove the copy with `rm --recursive -- {}`", crate::stale_ref_lock_message::shell_quote(self.record.as_path())),
            false => format!("remove the record with `rm -- {}`", crate::stale_ref_lock_message::shell_quote(self.record.as_path())),
        };
        return format!(
            "monochromatic-git-policy-cli: the record {record} {writer}, {part}, so whether that process still runs cannot be told from \
             the record, {state}. Nothing was changed. A record like that was written by a release or a program that is not this one, or names a process this host does \
             not report; its owner may still be running and a Git process monochromatic-git-policy-cli left running (a write it could not safely end, such as a merge, a \
             reset or a branch update) may still be working for it; `ps` shows them. When no monochromatic-git-policy-cli command and no such Git process runs for this \
             repository, {removal} and run the command again.",
            record = self.record.display(),
            writer = writer,
            part = part,
            state = refusal_state(&self.because),
            removal = removal,
        );
    }
}

/// What:
///  Why a record's own age cannot decide its holder,
///  as the sentence that follows the rest of a refusal.
/// Why:
///   Both refusals (another namespace, and a birth of no value) are excused by the same three facts,
///  so the sentences are written once.
fn refusal_state(because: &ForeignRefusal) -> String {
    return match because {
        ForeignRefusal::PastThePatience(age) => format!(
            "and it is {minutes} minutes old, past the {patience} minutes after which monochromatic-git-policy-cli no longer takes such a record for a live owner's; \
             its age is read from this host's clock, so a clock that was set forward here, or one that is behind where the record was written, makes it read older than \
             the hold",
            minutes = age.as_secs() / 60,
            patience = FOREIGN_OWNER_PATIENCE.as_secs() / 60
        ),
        ForeignRefusal::TimeAhead => String::from(
            "and its own time is later than this reader's clock, so how long it has stood cannot be measured: a clock that was set back here, or one that is ahead \
             where the record was written, explains that, and nothing says whether its owner still runs",
        ),
        ForeignRefusal::TimeUnreadable(reason) => format!(
            "and its time cannot be read ({reason}), so how long it has stood cannot be measured and nothing says whether its owner still runs"
        ),
    };
}

/// What:
///  What a record whose birth identity is no value this command can read says about the owner its process number names.
/// Why:
///   Three answers, and callers give them their own words:
///  inside the patience the record stands for an owner that may run and what it holds stays,
///  a record that is gone held nothing,
///  and past the patience it is neither,
///  so the command stops or says so with the record named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnreadableOwnerStanding = 'may-run' | 'ended' | { refused: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnreadableOwnerStanding {
    /// Inside the patience: the owner may still run, and what it holds stays.
    MayRun,
    /// The record is gone, so its owner removed it and nothing holds.
    Ended,
    /// Neither, past the patience, with the words a command stops with or says.
    Refused(String),
}

/// What:
///  What a record whose part `part` could not be read says about the owner `owner_pid`,
///  from one reading of the record's own age.
/// Why:
///   This is the one answer every reader of such a record gives,
///  so the bound and the words cannot differ between the mark of a lock,
///  the record of a worktree copy,
///  the record of a commit transaction and the records of a worktree removal.
///  Owner,
///  2026-11-16,
///  "A: stays, command says how (Recommended)": the record stays,
///  nothing is said inside the patience,
///  and past it the words name the record,
///  the owner,
///  its age and the exact command that removes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unreadableOwnerStanding(recordPath, part, pid) // 'may-run' | 'ended' | words
/// ```
pub fn unreadable_owner_standing(record_path: &Path, part: UnreadablePart, owner_pid: i64) -> UnreadableOwnerStanding {
    return match aged(record_path) {
        Aged::Standing => UnreadableOwnerStanding::MayRun,
        Aged::Finished => UnreadableOwnerStanding::Ended,
        Aged::Refused(because) => UnreadableOwnerStanding::Refused(
            RefusedUntellableRecord { record: record_path.to_path_buf(), part, because }.describe(owner_pid),
        ),
    };
}

#[cfg(test)]
thread_local! {
    /// How many times this thread asked for a standing,
    /// so a control can see that a decision asked once.
    pub(crate) static STANDING_LOOKUPS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Namespace controls stay out of the release executable.
#[cfg(test)]
#[path = "pid_namespace_tests.rs"]
mod tests;
