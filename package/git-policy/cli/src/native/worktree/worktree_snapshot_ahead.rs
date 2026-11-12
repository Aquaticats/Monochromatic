//! What:
//!  The snapshots the wrapper makes ahead of an add,
//! and the add's claim of one instead of a snapshot of its own.
//! Why:
//!  One `BTRFS_IOC_SNAP_CREATE_V2` forces a btrfs transaction commit on the caller's thread,
//! and waits for the
//!  source's writeback first (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
//! "A worktree add"):
//! on this volume it took 7,443.8 ms at the median,
//! 18,006.2 ms at the ninetieth percentile and 39,828.5 ms at its
//!  worst,
//! while an add that finds a snapshot ready only renames one directory there ("Step 2:
//! snapshots made
//!  ahead").
//! The commit service of the repository makes them on its own time,
//! from a work item the add writes
//! (`worktree_snapshot_ahead_job.rs`),
//! so it is the service's thread that waits for the commit and not the
//!  person's.
//! The owner's bound for an add is 60 s ("60s is fine for adding worktrees") and nothing may wait indefinitely,
//! so an add waited no longer than `SNAPSHOT_AHEAD_CEILING` for a copy.
//! The owner answered "Fresh snapshot every add" on 2026-11-11 when he was asked to keep that copy
//!  or to take one on every add,
//! so no add claims a copy from this pool and no add waits for one:
//! the modules stay for a record an earlier release queued and a removal clears a folder it filled.
//!
//! A snapshot made ahead is a copy of the source as it was when the service made it,
//! so an add may not trust it:
//! the
//!  files the new worktree receives must equal the source's index,
//! and an entry that fell behind it would give a
//!  worktree whose tracked files carry an older state beside freshly recorded stat data,
//! which Git would then report
//!  as clean.
//! Every candidate is therefore proven by the same walk the add makes over its own snapshot
//! (`worktree_snapshot_index_step.rs`,
//! `prove_the_copy`) before it is renamed into the stage name;
//! one that fails the
//!  proof is handed to the service for deletion and the next candidate is tried.
//! A source with uncommitted changes is not served this way at all:
//! its snapshot's own index is one instant with its
//!  files,
//! so that snapshot has to be taken on the add's own path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const look = lookForASnapshotAhead({ source, plan, stage, environment, service });
//! // 'claimed' with the proof, 'none' (take one here), or 'timedOut' with the words for the person
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::{ObjectId, handle_path, open_root};
use crate::hexadecimal_of_bytes::hexadecimal_of_bytes;
use crate::iso_time::now_iso;
use crate::path_bytes::path_bytes_of;
use crate::random_id::random_uuid;
use crate::service_client::{Activation, ClientConfig, START_WAIT, manager_watch_for, wake};
use crate::service_messages::render;
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::service_provision::provision;
use crate::service_provision_host_and_manager::units_record_dir_of;
use crate::service_route::{Route, route_from_environment};
use crate::service_work::work_directory;
use crate::subvolume_sha::sha256;
use crate::transaction_git::GitContext;
use crate::wait_ceilings::{ANSWER_CEILING, SHORT_EXTRA_WAIT, START_CEILING, WITHDRAW_CEILING, waiting_notice_after};
use crate::work_item::{Enqueued, WorkItem, WorkKind, enqueue, requeue_after_giving_up};
use crate::worktree_remove_hand_over::DELETE_REFERENCE;
use crate::worktree_snapshot_index_step::Proof;
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeSnapshotAhead";

/// What:
///  The start of the name of a snapshot the wrapper made ahead,
/// in the folder an add makes its stage in.
/// Why:
///   The name says what the directory is to a person reading the worktrees folder,
/// and it is the same prefix on every filesystem,
///  so one listing finds a source's entries wherever they are.
/// It has neither the stage prefix nor the shape a removal renames a
///  subvolume to,
/// so nothing that reads those names takes an entry for its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const AHEAD_PREFIX = '.cli-git-worktree-ahead-';
/// ```
pub const AHEAD_PREFIX: &str = ".cli-git-worktree-ahead-";

/// What:
///  How many snapshots ahead one source keeps.
/// Why:
///  Three is what the measured cost and the measured arrivals need at the median:
/// one snapshot took 7,443.8 ms at
///  the median while adds started 2.8 s apart at the tenth percentile of the gap between two
/// (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
/// "Step 2"),
/// so 2.7 snapshots' worth of adds can arrive
///  inside one snapshot's cost.
/// The fourteen the worst measured call would need are not kept,
/// because
///  every entry pins the blocks of each file the source rewrites after it,
/// on a volume that is 95 percent full;
/// the
///  ceiling and the rung after the snapshot are what covers that case instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const AHEAD_KEPT = 3;
/// ```
pub const AHEAD_KEPT: usize = 3;

/// What:
///  How old a snapshot ahead may be and still be kept for an add that has not arrived.
/// Why:
///   A snapshot pins the blocks of every file the source rewrites after it,
/// and nothing else would ever remove one,
/// so a
///  repository whose adds stopped must not keep them for good.
/// Ten minutes is longer than the measured gap between two
///  adds except an idle night ("Step 2"),
/// and every add removes what it finds older here,
/// so an active
///  repository never reaches it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const AHEAD_OLDEST = 10 * 60 * 1000;
/// ```
pub const AHEAD_OLDEST: Duration = Duration::from_secs(10 * 60);

/// The reference an item of this kind names,
/// which is no branch:
/// the work queue keeps one reference-shaped field on every record,
/// and no push is ever made for this kind.
pub const AHEAD_REFERENCE: &str = "refs/worktree/cli-git-service/snapshot-ahead";

/// What:
///  How often the wait looks in the folder again.
/// Why:
///   Short enough that an add takes a snapshot the moment the service publishes one,
/// and long enough that the look
///  costs one listing rather than the processor.
/// It is also the delay the wait adds to an entry that arrived between two looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const POLL = 25;
/// ```
pub(crate) const POLL: Duration = Duration::from_millis(25);

/// What:
///  How the add reaches the service that makes the snapshots ahead,
/// and how long it waits for one.
/// Why:
///   A control must prove the claim with no service and no session bus,
/// so the enqueue-and-wake is injected,
/// and the
///  ceiling is a value a control can shorten without changing what an add does.
/// The ask answers the id of the record it wrote,
/// when
///  it wrote one,
/// so the wait can see the service give that request up for good and stop waiting for a
///  copy that can never come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AheadService = { handOver: (stale: string[]) => Result<void>; ask: () => Result<string | undefined>; ceiling: Duration };
/// ```
pub struct AheadService<'a> {
    /// Ask the repository's commit service for one snapshot ahead,
    /// answering the id of the record it wrote;
    /// the error is why it could not be told.
    pub ask: &'a dyn Fn() -> Result<Option<String>, String>,
    /// Hand the service every name this add proved stale or keeps no longer,
    /// and wake it;
    /// the error is why it
    ///  could not be told.
    pub hand_over: &'a dyn Fn(&[OsString]) -> Result<(), String>,
    /// The longest the add waits for one to arrive.
    pub ceiling: Duration,
}

/// What:
///  The reason a seam that reaches no service answers with.
/// Why:
///   A caller with no service to ask is told so in words that name what it would have asked,
/// and the add then takes
///  its own snapshot instead of waiting for one that cannot come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cannotAsk = () => err('this add asks no commit service for a snapshot ahead');
/// ```
fn cannot_ask() -> Result<Option<String>, String> {
    return Err(String::from("this add asks no commit service for a snapshot ahead"));
}

/// What:
///  The reason a seam that reaches no service answers with,
/// for the names a control hands it.
/// Why:
///   A control still records what an add wanted deleted,
/// and the words say why nothing acted on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cannotHandOver = (stale) => err(`this add hands ${stale.length} name(s) to no commit service`);
/// ```
fn cannot_hand_over(stale: &[OsString]) -> Result<(), String> {
    return Err(format!("this add hands {} name(s) to no commit service", stale.len()));
}

impl AheadService<'_> {
    /// What:
    ///  A seam that reaches nothing and waits for nothing:
    /// the add takes its own snapshot.
    /// Why:
    ///   A control must take a snapshot with no service and no session bus,
    /// and that is also what an add does where no
    ///  service can be started (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
    /// "Step 3").
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// AheadService.nothing() // { handOver: cannotAsk, ceiling: 0 }
    /// ```
    pub fn nothing() -> AheadService<'static> {
        return AheadService { ask: &cannot_ask, hand_over: &cannot_hand_over, ceiling: Duration::ZERO };
    }
}

/// What one add's look for a snapshot made ahead came to.
/// Why:
///   The three answers decide three different things:
/// the proof travels on to the index step,
/// no answer means the
///  add takes its own snapshot,
/// and a ceiling ends the snapshot rung and lets the rung after it make the worktree.
#[derive(Debug)]
pub enum AheadLook {
    /// A snapshot was claimed,
    /// with the proof that its files equal the source's index.
    Claimed {
        /// The identity of the object now at the stage name.
        made: ObjectId,
        /// The walk of the claimed snapshot against the source's index.
        proof: Proof,
        /// The millisecond its name says the copy was made in,
        /// which is the age of the ignored state the new worktree receives.
        copied_at_millis: u128,
    },
    /// There was nothing to claim and nothing was waited for:
    /// the add takes its own snapshot.
    None,
    /// Nothing came within the ceiling;
    /// the words say why,
    /// and this add takes no snapshot of its own.
    TimedOut(String),
}

/// What:
///  The first sixteen hexadecimal digits of the digest of a source root's path,
/// which every name of that source's snapshots ahead carries.
/// Why:
///   A snapshot ahead copies one source root,
/// and one worktrees folder can hold the snapshots of an add run from the main
///  checkout beside those of an add run from a linked worktree.
/// An add must never claim a copy of another tree,
/// and the
///  name is the whole test for that,
/// so no record has to be read.
/// Sixteen digits are 64 bits of the digest,
/// which two
///  paths of one machine do not reach,
/// and they keep the name short enough to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const key = (root: string) => createHash('sha256').update(root).digest('hex').slice(0, 16);
/// ```
pub(crate) fn ahead_key(root: &Path) -> String {
    let digest: String = hexadecimal_of_bytes(&sha256(path_bytes_of(root)));
    return digest.chars().take(16).collect();
}

/// What:
///  The name a snapshot ahead of `source_root` takes when the service makes it,
/// `at` milliseconds after the epoch.
/// Why:
///   The time is in the name so the newest entry is found by sorting the names,
/// and the random part keeps two
///  services of one repository from choosing one name.
/// The key is what makes the name one source's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = (root: string, at: number) => `${AHEAD_PREFIX}${key(root)}-${at.toString(16)}-${randomUUID().replaceAll('-', '')}`;
/// ```
pub fn ahead_name(source_root: &Path, at: SystemTime) -> Result<OsString, String> {
    let unique: String = match random_uuid() {
        Ok(made) => made.replace('-', ""),
        Err(error) => return Err(format!("no name can be made for a snapshot ahead ({})", error.0)),
    };
    let milliseconds: u128 = at.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis();
    return Ok(OsString::from(format!("{AHEAD_PREFIX}{}-{milliseconds:016x}-{unique}", ahead_key(source_root))));
}

/// What:
///  The millisecond a name says its snapshot was made in,
/// or nothing when the name is no snapshot ahead.
/// Why:
///   The age of an entry decides whether it is kept,
/// and the newest is the one an add proves first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const madeAt = (name: string) => BigInt('0x' + name.slice(AHEAD_PREFIX.length).split('-')[1]);
/// ```
pub(crate) fn made_at(name: &OsStr) -> Option<u128> {
    let text: &str = name.to_str()?;
    let after_prefix: &str = text.strip_prefix(AHEAD_PREFIX)?;
    let (_key, after_key) = after_prefix.split_once('-')?;
    let (milliseconds, unique) = after_key.split_once('-')?;
    if unique.is_empty() {
        return None;
    }
    return u128::from_str_radix(milliseconds, 16).ok();
}

/// What:
///  Whether a name is one of `key`'s snapshots ahead.
/// Why:
///   One folder can hold the snapshots of several source roots,
/// and only this source's own are candidates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isOfThisSource = (name: string, key: string) => name.startsWith(`${AHEAD_PREFIX}${key}-`) && madeAt(name) !== undefined;
/// ```
fn is_of_this_source(name: &OsStr, key: &str) -> bool {
    let Some(text) = name.to_str() else { return false };
    let Some(rest) = text.strip_prefix(AHEAD_PREFIX) else { return false };
    let Some((found, _rest)) = rest.split_once('-') else { return false };
    return found == key && made_at(name).is_some();
}

/// What:
///  The order of two entries by the millisecond each was made in,
/// newest first.
/// Why:
///   The newest candidate is the one most likely to still equal the source's index,
/// so it is proven first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const newestFirst = (left, right) => right.made - left.made;
/// ```
fn newest_first(left: &(u128, OsString), right: &(u128, OsString)) -> std::cmp::Ordering {
    return right.0.cmp(&left.0);
}

/// What:
///  The names of `key`'s snapshots ahead in the held folder,
/// newest first.
/// Why:
///   The newest is the one most likely to still equal the source's index,
/// so it is proven first;
/// an older one is
///  proven only when the newer ones were claimed or failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const candidates = (dir, key) => readdirSync(handlePath(dir)).filter((name) => isOfThisSource(name, key)).sort((left, right) => madeAt(right) - madeAt(left));
/// ```
pub(crate) fn candidates(folder: &File, key: &str) -> Vec<OsString> {
    let Ok(listing) = std::fs::read_dir(handle_path(folder)) else { return Vec::new() };
    let mut found: Vec<(u128, OsString)> = Vec::new();
    for entry in listing.flatten() {
        let name: OsString = entry.file_name();
        if !is_of_this_source(name.as_os_str(), key) {
            continue;
        }
        let Some(at) = made_at(name.as_os_str()) else { continue };
        found.push((at, name));
    }
    found.sort_by(newest_first);
    let mut names: Vec<OsString> = Vec::with_capacity(found.len());
    for (_at, name) in found {
        names.push(name);
    }
    return names;
}

/// What:
///  The folder the snapshots the wrapper makes ahead of an add stand in:
/// `<git-dir>/monochromatic-git-policy-cli/ahead`,
/// beside the work
///  queue of the same repository.
/// Why:
///   Two things must hold of a snapshot that waits for an add,
/// and this folder is what makes both true:
/// the
///  destination's parent is inside the source worktree and may be anything Git does not ignore,
/// so a copy of the whole
///  source standing there would list as untracked until an add took it;
/// and a name inside the Git directory is never untracked,
///  in any worktree of the repository,
/// whatever the person's ignore rules say.
/// It is also one btrfs filesystem with the
///  source root,
/// because it is inside it,
/// so the ioctl that makes an entry there can share every data block with the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = (gitDir: string) => join(gitDir, 'monochromatic-git-policy-cli', 'ahead');
/// ```
pub fn pool_folder(git_dir: &Path) -> PathBuf {
    return git_dir.join("monochromatic-git-policy-cli").join("ahead");
}

/// What:
///  The names of the snapshots ahead in `folder` this add keeps no longer:
/// everything after the newest `AHEAD_KEPT`,
/// and everything older than
///  `AHEAD_OLDEST`.
/// Why:
///   Every add leaves the pool no larger,
/// and one that claimed nothing must still not keep a growing pile,
/// while an
///  entry no add claimed for ten minutes is one nothing needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const overKept = (names, now) => names.filter((name, index) => index >= AHEAD_KEPT || now - madeAt(name) > AHEAD_OLDEST);
/// ```
pub(crate) fn over_kept(names: &[OsString], now: u128) -> Vec<OsString> {
    let mut over: Vec<OsString> = Vec::new();
    for (index, name) in names.iter().enumerate() {
        let too_old: bool = match made_at(name.as_os_str()) {
            Some(made) => older_than_kept(made, now),
            None => true,
        };
        if index >= AHEAD_KEPT || too_old {
            over.push(name.clone());
        }
    }
    return over;
}

/// What:
///  Whether a copy made `made` milliseconds after the epoch is older than an add keeps.
/// Why:
///   The age is the one number that decides whether a copy a repository is not adding to stays,
/// and one named
///  relation keeps the rule in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const olderThanKept = (made, now) => now - made > AHEAD_OLDEST;
/// ```
fn older_than_kept(made: u128, now: u128) -> bool {
    return now.saturating_sub(made) > AHEAD_OLDEST.as_millis();
}

/// What:
///  The names of the snapshots ahead in `folder` that belong to `source_root`,
/// newest first.
/// Why:
///   The age sweep and the diagnostics ask the same question the claim asks,
/// so one function answers it for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = (folder, root) => candidates(folder, key(root));
/// ```
pub fn kept_in(folder: &Path, source_root: &Path) -> Vec<OsString> {
    let Ok(held) = open_root(folder) else { return Vec::new() };
    return candidates(&held, ahead_key(source_root).as_str());
}

/// What:
///  The item that asks the commit service for one snapshot ahead of `source_root`,
/// made in `pool`.
/// Why:
///   The record outlives the command,
/// so an add that finds nothing,
/// and a service that was not running,
///  still leave the request for the repository's next round,
/// and the add after this one finds the pool filled.
/// The identity
///  is random,
/// so no add's request hides behind another's in the queue's idempotency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = (commonDir, root, pool) => ({ kind: 'snapshot-ahead', repository: root, gitDir: commonDir, ref: AHEAD_REFERENCE, oid: sha256(random()), parent: pool, created: now, tries: 0 });
/// ```
pub fn request_item(git_dir: &Path, source_root: &Path, pool: &Path) -> Result<WorkItem, String> {
    let unique: String = match random_uuid() {
        Ok(made) => made.replace('-', ""),
        Err(error) => return Err(format!("no identity can be made for a request of a snapshot ahead ({})", error.0)),
    };
    let digest: String = hexadecimal_of_bytes(&sha256(unique.as_bytes()));
    return Ok(WorkItem {
        kind: WorkKind::SnapshotAhead,
        repository: source_root.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from(AHEAD_REFERENCE),
        oid: digest,
        parent: pool.display().to_string(),
        created: now_iso(),
        tries: 0,
    });
}

/// What:
///  The item that has the service delete one snapshot ahead this add proved stale,
/// or has kept too long.
/// Why:
///   The destroy is a btrfs call that takes the filesystem's
/// `subvol_sem` write lock against every snapshot in flight,
/// so it is the service's work and never the person's,
/// as a removed worktree's subvolume already is
/// (`worktree_remove_hand_over.rs`).
/// The name is the identity,
/// so two adds that proved one entry stale write one record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deletion = (commonDir, pool, name) => ({ kind: 'subvolume-delete', repository: pool, gitDir: commonDir, ref: DELETE_REFERENCE, oid: sha256(name), parent: join(pool, name), created: now, tries: 0 });
/// ```
pub fn deletion_item(git_dir: &Path, pool: &Path, name: &OsStr) -> WorkItem {
    let entry: PathBuf = pool.join(name);
    return WorkItem {
        kind: WorkKind::SubvolumeDelete,
        repository: pool.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from(DELETE_REFERENCE),
        oid: hexadecimal_of_bytes(&sha256(path_bytes_of(entry.as_path()))),
        parent: entry.display().to_string(),
        created: now_iso(),
        tries: 0,
    };
}

/// What:
///  Wake the commit service of the worktree whose own Git directory is `git_dir`,
/// saying nothing of its own,
/// as an add wakes it for its trash.
/// Why:
///   The route and the records must name one worktree:
/// the service that answers a worktree drains that worktree's own
///  work directory,
/// so a record written under another directory is refused there.
/// Why:
///   A wake that cannot be made leaves the records for the next command of the repository,
/// which is every commit
///  and every add,
/// so a snapshot that arrives later is never this add's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wakeService({ context, commonDir });
/// ```
fn wake_the_service(context: &GitContext, git_dir: &Path) -> Result<(), String> {
    let Some(Route { address, provision: host }) = route_from_environment(context.real_git.as_path(), git_dir) else {
        return Err(String::from("no commit service can be started from this session"));
    };
    if let Some(setup) = host.as_ref()
        && let Err(failure) = provision(setup)
    {
        return Err(render(&failure));
    }
    let config: ClientConfig = ClientConfig {
        address,
        activation: Activation::Managed,
        start_wait: START_WAIT,
        heartbeat_silence: HEARTBEAT_SILENCE,
        start_watch: host.as_ref().map(manager_watch_for),
        keep_fresh: None,
        answer_ceiling: ANSWER_CEILING,
        withdraw_ceiling: WITHDRAW_CEILING,
        waiting_notice_after: waiting_notice_after(),
        extra_wait: SHORT_EXTRA_WAIT,
        start_ceiling: START_CEILING,
        units_record_dir: host.as_ref().map(units_record_dir_of),
    };
    let mut silent: std::io::Sink = std::io::sink();
    return match wake(&config, git_dir, None, &mut silent) {
        Ok(()) => Ok(()),
        Err(failure) => Err(render(&failure)),
    };
}

/// What:
///  Ask the commit service of this repository for one snapshot ahead of this add,
/// and wake it.
/// Why:
///   The request is what fills the pool for the add after this one,
/// and its record outlives the command.
/// A record that cannot be written is a reason,
/// never the add's failure:
/// the add takes its
///  own snapshot and says why.
/// The id it answers is the record's,
/// so the add that waits for it can see the service give it up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// askForOne({ context, source }) // enqueue(request); wake(commonDir); the record's id
/// ```
pub fn ask_for_one(context: &GitContext, source: &SnapshotSource) -> Result<Option<String>, String> {
    let pool: PathBuf = pool_folder(source.git_dir.as_path());
    let asked: WorkItem = ask_with(source.git_dir.as_path(), source.root.as_path(), pool.as_path())?;
    wake_the_service(context, source.git_dir.as_path())?;
    return Ok(Some(asked.id()));
}

/// What:
///  The record of one request for a snapshot ahead,
/// which the caller wakes the service for.
/// Why:
///   The record is written before the service is told,
/// so a control asserts the record without a service,
/// and
///  `ask_for_one` wakes the service after it and answers its id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// askWith(gitDir, root, pool) // the record only; the caller wakes the service
/// ```
pub fn ask_with(git_dir: &Path, source_root: &Path, pool: &Path) -> Result<WorkItem, String> {
    let request: WorkItem = request_item(git_dir, source_root, pool)?;
    if let Err(reason) = enqueue(work_directory(git_dir).as_path(), &request) {
        return Err(format!("the request of a snapshot ahead could not be written: {reason}"));
    }
    debug(TAG, format!("{}: asked for a snapshot ahead in {}", source_root.display(), pool.display()).as_str());
    return Ok(request);
}

/// What:
///  Hand this add's stale names,
/// and the names it keeps no longer,
/// to the commit service of the repository,
/// and wake it.
/// Why:
///   The destroy each name needs is a btrfs call that takes the filesystem's write lock against every
///  snapshot in flight,
/// so it is the service's work and never the person's,
/// as a removed worktree's subvolume already is.
/// A
///  reason here is never the add's failure:
/// the names stay in the folder and a later add hands them over again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverToTheService({ context, source, stale }) // enqueue(deletion) for each; wake(commonDir)
/// ```
pub fn hand_over_to_the_service(context: &GitContext, source: &SnapshotSource, stale: &[OsString]) -> Result<(), String> {
    let pool: PathBuf = pool_folder(source.git_dir.as_path());
    hand_over_with(source.git_dir.as_path(), pool.as_path(), stale)?;
    return wake_the_service(context, source.git_dir.as_path());
}

/// What:
///  One record per stale name,
/// and nothing else.
/// Why:
///   The records are written before the service is told,
/// so a control asserts them without a service,
/// and one
///  record that cannot be written is said and does not stop the others.
/// A record the service already gave up on is queued again rather than left in `failed`:
/// the id of the
///  delete of one path is that path,
/// so every later add names the same record,
/// and a queue that only answers
///  "already there" would never delete that copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverWith(gitDir, pool, stale) // the records only; the caller wakes the service
/// ```
pub fn hand_over_with(git_dir: &Path, pool: &Path, stale: &[OsString]) -> Result<(), String> {
    let work: PathBuf = work_directory(git_dir);
    for name in stale {
        let record: WorkItem = deletion_item(git_dir, pool, name.as_os_str());
        match enqueue(work.as_path(), &record) {
            Ok(Enqueued::Added) => {}
            Ok(Enqueued::Known) => {
                if requeue_after_giving_up(work.as_path(), record.id().as_str()) {
                    debug(TAG, format!("the delete of {} was given up on and is queued again", pool.join(name).display()).as_str());
                }
            }
            Err(reason) => debug(TAG, format!("the delete of {} could not be recorded: {reason}", pool.join(name).display()).as_str()),
        }
    }
    debug(TAG, format!("{} name(s) of snapshots ahead are handed to the service in {}", stale.len(), pool.display()).as_str());
    return Ok(());
}

/// The add's own look for a snapshot the service made ahead of it:
/// the proof,
/// the claim,
/// the bounded wait and the sweep.
pub use crate::worktree_snapshot_ahead_claim::look_for_a_snapshot_ahead;

/// Controls of the snapshots made ahead stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_ahead_tests.rs"]
mod tests;
