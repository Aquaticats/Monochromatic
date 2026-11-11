//! What:
//!  The question whether an index still records the stat data its worktree has,
//! answered in process.
//! Why:
//!  The scratch `git commit` reads and hashes every file whose index entry records stat data the worktree no longer
//!  has,
//!  and that read is what the scratch step's ceiling ends
//!  (`commit_index_stat_refresh.rs`).
//!  A refresh of the real index pays those reads where the ceiling does not apply and keeps them,
//!  but every `git update-index --refresh` takes Git's `index.lock` on the real index,
//!  even the refresh of an index that already describes the worktree:
//!  measured on 2026-11-08 in throwaway repositories,
//!  `update-index --refresh` creates `index.lock` on every run and renames it over the index only when it writes.
//!  So the refresh is asked for only when an entry's recorded stat data differs from the file's.
//!  An index Git calls racily clean is not asked for:
//!  its entries record the stat data their files have,
//!  and three refreshes of an index written in the same second as its entry left the index's bytes identical (the same
//!  SHA-1),
//!  so such a refresh writes nothing the private copies could carry,
//!  and the scratch commit reads those entries just as native `git commit` does.
//!  The entries of paths the commit stages from the worktree are skipped as well,
//!  because `git add` writes their stat data into the private index afresh.
//!  Anything this reading does not fully understand answers "stale",
//!  and Git's own refresh decides.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stale: boolean = indexStatDataNeedsRefresh({ context, cwd, index, staging });
//! ```

/// The Git context,
/// which carries the overlay a directory is read with.
use crate::transaction_git::GitContext;
/// One entry with its stat data and flags,
/// and the whole index file read into entries.
#[cfg(unix)]
use crate::index_full::{ASSUME_VALID, FullEntry, FullIndex, SKIP_WORKTREE, parse_full_index};
/// The nine 32-bit stat fields an entry stores.
#[cfg(unix)]
use crate::index_stat::StatData;
/// A refusal to read a file,
/// and the bounded read that reports it.
#[cfg(unix)]
use crate::private_storage::{ReadRefusal, read_regular_file};
/// The worktree's top level,
/// the start directory below it and the object hash a directory is read with.
#[cfg(unix)]
use crate::repo_read_layout::{RepoLayout, read_layout_in};
/// The threads an independent set of items may use,
/// as the neighbouring file-tree jobs derive them.
#[cfg(unix)]
use crate::parallel_budget::{BudgetRequest, HostResources, SLICE_MEMORY, worker_budget};
/// One job per thread inside one thread scope,
/// and the join of every job.
#[cfg(unix)]
use crate::side_by_side_threads::run_all_side_by_side;
/// How many bytes one object name of the repository's hash has.
#[cfg(unix)]
use crate::repo_read_object::name_length;
/// `OsStr` borrows operating-system text without copying it.
#[cfg(unix)]
use std::ffi::OsStr;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
#[cfg(unix)]
use std::path::PathBuf;
/// Index paths are bytes that need not be UTF-8,
/// and this turns them into an operating-system file name.
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
/// One file's stat fields,
/// each 32 bits wide on this platform as the index stores them.
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// The most bytes of an index file this reads:
/// the bound the in-process query layer already puts on the same file
/// (`repo_read_query_request_answer_and_context.rs`),
/// not a new measurement.
#[cfg(unix)]
const INDEX_LIMIT: u64 = 1 << 30;

/// What:
///  The fewest entries at which walking them on several threads beat walking them on one:
/// 32 entries.
/// Why:
///  A stat walk of one entry costs a few microseconds,
/// so a short walk pays less than the threads it would start.
///  Measured on a throwaway repository of 22,200 committed files on this host
/// (8 CPUs,
/// other sessions running,
/// the release build of 2026-11-08,
/// 25 samples of each walk in one run,
/// medians,
/// warm cache):
/// 16 entries took 0.125 ms on one thread and 0.145 ms on seven,
/// so the threads lost there;
/// 32 took 0.218 ms against 0.158 ms,
/// 64 0.391 ms against 0.193 ms,
/// 128 0.786 ms against 0.289 ms,
/// 256 1.651 ms against 0.479 ms,
/// 512 3.223 ms against 0.858 ms,
/// 2,048 13.335 ms against 2.912 ms,
/// and all 22,200 took 160.12 ms against 30.56 ms on the seven threads the host's budget gives.
/// The neighbouring file-tree jobs measured their own break-even at 16 entries
/// (`parallel_budget.rs`,
/// `PARALLEL_BREAK_EVEN_ENTRIES`);
/// a stat walk's entry is cheaper than theirs,
/// so its break-even is higher.
#[cfg(unix)]
const STAT_WALK_BREAK_EVEN_ENTRIES: usize = 32;

/// The file-type bits of an entry's mode.
#[cfg(unix)]
const TYPE_BITS: u32 = 0o170_000;

/// The file-type bits of an entry that names another repository:
/// a gitlink holds no file to stat.
#[cfg(unix)]
const GITLINK: u32 = 0o160_000;

/// The file-type bits of a symbolic link.
#[cfg(unix)]
const SYMBOLIC_LINK: u32 = 0o120_000;

/// The file-type bits of a regular file.
#[cfg(unix)]
const REGULAR_FILE: u32 = 0o100_000;

/// The one permission bit of a file's mode that an index entry records.
#[cfg(unix)]
const EXECUTABLE_BIT: u32 = 0o100;

/// The mode an index entry has for a regular file,
/// as Git's `create_ce_mode` makes it.
#[cfg(unix)]
const MODE_REGULAR_FILE: u32 = 0o100_644;

/// The mode an index entry has for an executable regular file.
#[cfg(unix)]
const MODE_EXECUTABLE_FILE: u32 = 0o100_755;

/// What a commit stages from the worktree,
/// so its recorded stat data cannot cost the scratch commit a read.
pub enum WorktreeStaging<'a> {
    /// The positional pathspecs of an explicit-path commit,
    /// exactly as the command line gave them.
    Named(&'a [std::ffi::OsString]),
    /// Nothing:
    /// the commit takes its content from the index or from the merged tree.
    None,
}

/// The stat data one worktree file has,
/// each field cut to the 32 bits an index entry stores.
/// Why not the full 64-bit fields:
/// Git compares the index's cut fields with the file's own,
/// so a comparison here has to cut them the same way,
/// or every entry would look changed.
#[cfg(unix)]
struct WorktreeFile {
    /// The device.
    dev: u32,
    /// The inode.
    ino: u32,
    /// The mode Git would record for the file.
    mode: u32,
    /// The owner.
    uid: u32,
    /// The group.
    gid: u32,
    /// The size.
    size: u32,
    /// The change time,
    /// seconds.
    ctime_sec: u32,
    /// The change time,
    /// nanoseconds.
    ctime_nsec: u32,
    /// The modification time,
    /// seconds.
    mtime_sec: u32,
    /// The modification time,
    /// nanoseconds.
    mtime_nsec: u32,
}

/// What:
///  Whether one entry is one Git's own refresh never re-reads.
/// Why:
///  An unmerged entry,
///  one marked assume-unchanged or skip-worktree,
///  and a gitlink have no file this reading could compare,
///  and Git's `refresh_index` skips exactly those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leftToGit = (entry) => entry.stage !== 0 || entry.assumeValid || entry.skipWorktree || entry.gitlink;
/// ```
#[cfg(unix)]
fn is_left_to_git(entry: &FullEntry) -> bool {
    if entry.stage() != 0 {
        return true;
    }
    if entry.flags & ASSUME_VALID != 0 {
        return true;
    }
    if entry.extended & SKIP_WORKTREE != 0 {
        return true;
    }
    return entry.mode & TYPE_BITS == GITLINK;
}

/// What:
///  Whether one byte of a pathspec is one of the characters that make a pathspec name more than the one path it spells.
/// Why:
///  A pathspec with magic,
///  an escape or a wildcard names paths this reading cannot work out,
///  and exempting the wrong entry would leave the scratch commit to read a file it need not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotMagicByte = (byte) => !':*?[]\\\0'.includes(String.fromCharCode(byte));
/// ```
#[cfg(unix)]
fn is_not_a_magic_byte(byte: &u8) -> bool {
    return !matches!(*byte, b':' | b'*' | b'?' | b'[' | b']' | b'\\' | 0);
}

/// What:
///  Whether a pathspec names exactly the one path it spells.
/// Why:
///  Only such a pathspec can be matched against an index path without guessing what a directory or a glob names,
/// and a pathspec this refuses costs one Git start,
/// never a wrong answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLiteralPath = (spec) => spec.length > 0 && !spec.endsWith('/') && [...spec].every(isNotMagicByte);
/// ```
#[cfg(unix)]
fn is_literal_path(spec: &[u8]) -> bool {
    if spec.is_empty() || spec.ends_with(b"/") {
        return false;
    }
    return spec.iter().all(is_not_a_magic_byte);
}

/// What:
///  The top-relative paths the selection stages from the worktree.
/// Why:
///  `git add --all` writes their stat data into the private index afresh,
/// so what the real index records for them cannot cost the scratch commit a read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const restaged = staging.named.filter(isLiteralPath).map((spec) => prefix + spec);
/// ```
#[cfg(unix)]
fn restaged_paths(staging: &WorktreeStaging<'_>, prefix: &[u8]) -> Vec<Vec<u8>> {
    let mut paths: Vec<Vec<u8>> = Vec::new();
    let WorktreeStaging::Named(named) = staging else {
        return paths;
    };
    for spec in named.iter() {
        let spelled: &[u8] = spec.as_bytes();
        if !is_literal_path(spelled) {
            continue;
        }
        let mut top_relative: Vec<u8> = Vec::with_capacity(prefix.len() + spelled.len());
        top_relative.extend_from_slice(prefix);
        top_relative.extend_from_slice(spelled);
        paths.push(top_relative);
    }
    return paths;
}

/// What:
///  Whether one index path is one the selection stages from the worktree.
/// Why:
///  The lookup says whether the entry's recorded stat data still matters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isRestaged = (restaged, path) => restaged.some((named) => named === path);
/// ```
#[cfg(unix)]
fn is_restaged(restaged: &[Vec<u8>], path: &[u8]) -> bool {
    for named in restaged.iter() {
        if named.as_slice() == path {
            return true;
        }
    }
    return false;
}

/// What:
///  The mode an index entry has for a file with `file_mode`,
/// or nothing when no entry can describe the file.
/// Why:
///  Git records a regular file as 100644 or 100755 and a link as 120000,
/// whatever the rest of the permission bits say,
/// so comparing the raw mode would report every file as changed.
/// A directory,
/// socket or device in a tracked path is a change only Git can judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordedMode = (fileMode) =>
///   fileMode & TYPE_BITS === SYMBOLIC_LINK ? 0o120000
///   : fileMode & TYPE_BITS === REGULAR_FILE ? (fileMode & EXECUTABLE_BIT ? 0o100755 : 0o100644)
///   : undefined;
/// ```
#[cfg(unix)]
fn recorded_mode(file_mode: u32) -> Option<u32> {
    if file_mode & TYPE_BITS == SYMBOLIC_LINK {
        return Some(SYMBOLIC_LINK);
    }
    if file_mode & TYPE_BITS != REGULAR_FILE {
        return None;
    }
    return if file_mode & EXECUTABLE_BIT != 0 { Some(MODE_EXECUTABLE_FILE) } else { Some(MODE_REGULAR_FILE) };
}

/// What:
///  The stat data the file at `path` has.
/// Why:
///  A file this cannot read is one Git has to judge,
///  and so is a link where the worktree expects none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worktreeFile = (path) => {
///   const stat = lstatSync(path); // the link itself, not what it points at
///   const mode = recordedMode(stat.mode);
///   return mode === undefined ? undefined : { dev: stat.dev, ino: stat.ino, mode, ... };
/// };
/// ```
#[cfg(unix)]
fn worktree_file(path: &Path) -> Option<WorktreeFile> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).ok()?;
    let mode: u32 = recorded_mode(metadata.mode())?;
    return Some(WorktreeFile {
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        mode,
        uid: metadata.uid(),
        gid: metadata.gid(),
        size: metadata.size() as u32,
        ctime_sec: metadata.ctime() as u32,
        ctime_nsec: metadata.ctime_nsec() as u32,
        mtime_sec: metadata.mtime() as u32,
        mtime_nsec: metadata.mtime_nsec() as u32,
    });
}

/// What:
///  Whether the stat data an entry records is the stat data its file has.
/// Why:
///  This is the question `git update-index --refresh` asks,
///  field for field,
///  and only an entry that answers it needs no refresh.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const matchesRecorded = (entry, file) => entry.mode === file.mode && entry.dev === file.dev && entry.ino === file.ino &&
///   entry.uid === file.uid && entry.gid === file.gid && entry.size === file.size &&
///   entry.ctimeSec === file.ctimeSec && entry.ctimeNsec === file.ctimeNsec &&
///   entry.mtimeSec === file.mtimeSec && entry.mtimeNsec === file.mtimeNsec;
/// ```
#[cfg(unix)]
fn matches_recorded(entry: &FullEntry, file: &WorktreeFile) -> bool {
    let recorded: StatData = entry.stat;
    return entry.mode == file.mode
        && recorded.dev == file.dev
        && recorded.ino == file.ino
        && recorded.uid == file.uid
        && recorded.gid == file.gid
        && recorded.size == file.size
        && recorded.ctime_sec == file.ctime_sec
        && recorded.ctime_nsec == file.ctime_nsec
        && recorded.mtime_sec == file.mtime_sec
        && recorded.mtime_nsec == file.mtime_nsec;
}

/// What:
///  The reading itself:
/// every entry the commit leaves alone is compared with its file,
/// and every doubt answers "stale".
/// Why:
///  `index_stat_data_needs_refresh` is the one caller,
/// and separating the reading keeps that answer readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readInProcess = ({ context, cwd, index, staging }) => {
///   const bytes = readIndex(index);          // absent: nothing to refresh
///   const layout = readLayout(cwd, overlay(context));
///   const parsed = parseFullIndex(bytes, layout.objectFormat);
///   const restaged = restagedPaths(staging, layout.prefix);
///   const stale = walkInSlices(parsed.entries, layout.worktreeRoot, restaged, workersFor(parsed.entries.length));
///   return stale === undefined ? UNKNOWN : stale.length === 0 ? CURRENT : STALE(stale, parsed.entries.length);
/// };
/// ```
#[cfg(unix)]
fn read_in_process(context: &GitContext, cwd: &Path, index: &Path, staging: WorktreeStaging<'_>) -> StaleStatData {
    // What:
    //  The index file's bytes.
    // Why:
    //  Every entry and its stat data is in them,
    //  so no Git is needed to ask the question.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const bytes = readRegularFile(index, INDEX_LIMIT);
    // ```
    let bytes: Vec<u8> = match read_regular_file(index, INDEX_LIMIT) {
        Ok(read) => read,
        // A repository nothing was ever staged in has no index, and so no stat data to refresh.
        Err(ReadRefusal::Missing) => return StaleStatData::Current,
        // An index this reading cannot open is one Git answers for.
        Err(_) => return StaleStatData::Unknown,
    };
    // What:
    //  The environment the directory is read with,
    //  including the real Git a child would run.
    // Why:
    //  The in-process reader declines some configurations,
    //  and the answer then has to be "stale".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const overlay = withRealGit(context.overlay, context.realGit);
    // ```
    let overlay: Vec<(std::ffi::OsString, std::ffi::OsString)> = crate::git_system_config::with_real_git(context.overlay.as_slice(), context.real_git.as_path());
    // What:
    //  The worktree's top level, the start directory below it and the repository's object hash.
    // Why:
    //  Index paths are relative to the top level, which need not be the directory the command runs in,
    //  and the hash length says where the index's entries end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const layout = readLayoutIn(cwd, overlay);
    // ```
    let layout: RepoLayout = match read_layout_in(cwd, overlay.as_slice()) {
        Some(read) => read,
        None => return StaleStatData::Unknown,
    };
    // What:
    //  Every entry of the index with its stat data.
    // Why:
    //  A form this reader does not understand is one Git has to answer for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parsed = parseFullIndex(bytes, nameLength(layout.objectFormat));
    // ```
    let parsed: FullIndex = match parse_full_index(bytes.as_slice(), name_length(layout.object_format)) {
        Some(read) => read,
        None => return StaleStatData::Unknown,
    };
    // What:
    //  The top-relative paths this commit stages from the worktree.
    // Why:
    //  Their recorded stat data is replaced by `git add`,
    //  so a stale record of theirs costs the scratch commit nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const restaged = restagedPaths(staging, layout.prefix);
    // ```
    let restaged: Vec<Vec<u8>> = restaged_paths(&staging, layout.prefix.as_slice());
    // What:
    //  The paths whose entries record stat data their files no longer have,
    // walked in slices on several threads
    //  where the index is large enough for the walk to pay for them (`stale_paths_in_slice`).
    // Why:
    //  Git's own stat pass preloads the same work across threads (`core.preloadIndex`),
    // and one thread of this walk took 47 to
    //  58 ms on a checkout of 22,161 entries against Git's 10 ms;
    // over 22,200 files a walk of one thread took 160.12 ms and seven
    //  took 30.56 ms (`STAT_WALK_BREAK_EVEN_ENTRIES`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stale = walkInSlices(parsed.entries, layout.worktreeRoot, restaged, workersFor(parsed.entries.length));
    // ```
    let workers: usize = workers_for(parsed.entries.len());
    let slice: EntrySlice<'_> = EntrySlice { entries: parsed.entries.as_slice(), worktree_root: layout.worktree_root.as_path(), restaged: restaged.as_slice() };
    let stale: Option<Vec<Vec<u8>>> = match workers {
        1 => Some(stale_paths_in_slice(slice)),
        _ => walk_in_slices(slice, workers),
    };
    let Some(stale) = stale else {
        return StaleStatData::Unknown;
    };
    if stale.is_empty() {
        return StaleStatData::Current;
    }
    return StaleStatData::Stale { paths: stale, entries: parsed.entries.len() };
}

/// What:
///  One contiguous slice of the entries the question walks,
/// with what the walk of every entry needs.
/// Why:
///   A worker borrows its slice of the caller's entries without copying them,
/// and the slices together are the one linear walk
///  over the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntrySlice = { entries: FullEntry[]; worktreeRoot: string; restaged: Uint8Array[] };
/// ```
#[cfg(unix)]
struct EntrySlice<'a> {
    /// The entries this walk judges,
    /// in index order.
    entries: &'a [FullEntry],
    /// The worktree's top level,
    /// which index paths are relative to.
    worktree_root: &'a Path,
    /// The top-relative paths the commit stages from the worktree afresh.
    restaged: &'a [Vec<u8>],
}

/// What:
///  What the walk of an index found about the stat data its entries record.
/// Why:
///   The answer is the reason a refresh of the private index is started, or is not,
/// and the paths it names are the ones a refresh has to look at:
/// a refresh that names them costs a question per stale path instead of one per entry of the index
/// (`commit_index_stat_refresh.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StaleStatData = { kind: 'current' } | { kind: 'stale'; paths: Uint8Array[]; entries: number } | { kind: 'unknown' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaleStatData {
    /// Every entry the commit leaves alone records the stat data its file has.
    Current,
    /// These entries, by their top-relative index path, record stat data their files no longer have.
    Stale {
        /// The paths the walk could not prove,
        /// in index order.
        paths: Vec<Vec<u8>>,
        /// How many entries the index holds,
        /// which says whether naming the paths is cheaper than naming the index
        /// (`REFRESH_NAMES_ONE_PATH_IN` in `commit_index_stat_refresh.rs`).
        entries: usize,
    },
    /// This reading cannot answer,
    /// so Git's own refresh of the whole index has to.
    Unknown,
}

/// What:
///  Why one entry is one the walk judges,
/// or what keeps it out of the walk altogether.
/// Why:
///   The walk asks about every entry the commit leaves alone,
/// because an entry it skips cannot starve the refresh of a path Git would have needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const judged = (entry) => !isLeftToGit(entry) && !isRestaged(restaged, entry.path);
/// ```
#[cfg(unix)]
fn judged(entry: &FullEntry, restaged: &[Vec<u8>]) -> bool {
    return !is_left_to_git(entry) && !is_restaged(restaged, entry.path.as_slice());
}

/// What:
///  The paths of `slice` whose entries record stat data their files no longer have.
/// Why:
///   Every entry the walk judges is asked about,
/// so the paths it answers with are the whole of what a refresh has to look at,
///  and the answer for a clean slice costs the same walk the yes-or-no question cost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stalePaths = (slice) => slice.entries.filter((entry) => judged(entry) && !matchesRecorded(entry, worktreeFile(join(slice.worktreeRoot, entry.path)) ?? stale)).map((entry) => entry.path);
/// ```
#[cfg(unix)]
fn stale_paths_in_slice(slice: EntrySlice<'_>) -> Vec<Vec<u8>> {
    let mut stale: Vec<Vec<u8>> = Vec::new();
    for entry in slice.entries.iter() {
        // What:
        //  Whether the commit leaves this entry's recorded stat data to this walk at all.
        // Why:
        //  An entry left to Git or staged afresh costs the scratch commit nothing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!judged(entry, slice.restaged)) continue;
        // ```
        if !judged(entry, slice.restaged) {
            continue;
        }
        // What:
        //  The file the entry names, inside the worktree.
        // Why:
        //  Its stat data is the answer to the question for this entry.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const path = join(slice.worktreeRoot, entry.path);
        // ```
        let path: PathBuf = slice.worktree_root.join(OsStr::from_bytes(entry.path.as_slice()));
        // What:
        //  The stat data that file has now.
        // Why:
        //  A file this cannot read is one Git has to judge.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const file = worktreeFile(path);
        // ```
        let Some(file) = worktree_file(path.as_path()) else {
            stale.push(entry.path.clone());
            continue;
        };
        if !matches_recorded(entry, &file) {
            stale.push(entry.path.clone());
        }
    }
    return stale;
}

/// What:
///  The threads the walk uses for `entries` entries:
/// one below `STAT_WALK_BREAK_EVEN_ENTRIES`,
/// the host's budget above it.
/// Why:
///  Below the break-even measured for this walk,
/// creating the threads costs more than the walk they share,
/// and the budget
///  leaves the CPU margin of the host free and honours a container's quota.
/// No ceiling bounds it:
/// no measurement found the stat calls flattening at a number of threads the
///  budget gives.
///  Each worker copies nothing,
/// so `SLICE_MEMORY`,
/// the conservative per-worker bound of the file-tree jobs,
/// is an upper bound here too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const workers = entries < STAT_WALK_BREAK_EVEN_ENTRIES ? 1 : workerBudget({ items: entries, perItemMemory: SLICE_MEMORY, ceiling: undefined, cap: undefined, host: HostResources.read() });
/// ```
#[cfg(unix)]
fn workers_for(entries: usize) -> usize {
    if entries < STAT_WALK_BREAK_EVEN_ENTRIES {
        return 1;
    }
    return worker_budget(&BudgetRequest { items: entries, per_item_memory: SLICE_MEMORY, ceiling: None, cap: None, host: &HostResources::read() });
}

/// What:
///  The paths `slice`'s entries hold that need the refresh,
/// the slices walked side by side on `workers` threads,
/// or nothing when a worker could not answer.
/// Why:
///   A worker that panicked judged no entry,
/// and an entry nothing judged is one Git's refresh decides,
/// so the answer is
///  the whole index there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stale = (await Promise.allSettled(slices.map((slice) => walk(slice)))).every(ok) ? merged : undefined;
/// ```
#[cfg(unix)]
fn walk_in_slices(slice: EntrySlice<'_>, workers: usize) -> Option<Vec<Vec<u8>>> {
    let size: usize = slice.entries.len().div_ceil(workers.max(1)).max(1);
    let mut slices: Vec<EntrySlice<'_>> = Vec::new();
    for entries in slice.entries.chunks(size) {
        slices.push(EntrySlice { entries, worktree_root: slice.worktree_root, restaged: slice.restaged });
    }
    let walked: Vec<std::thread::Result<Vec<Vec<u8>>>> = run_all_side_by_side(slices, stale_paths_in_slice);
    let mut stale: Vec<Vec<u8>> = Vec::new();
    for outcome in walked {
        match outcome {
            Ok(paths) => stale.extend(paths),
            Err(_) => return None,
        }
    }
    return Some(stale);
}

/// What:
///  What `index` records about the stat data of the worktree,
/// answered in process for every entry `staging` does not write afresh.
/// Why:
///  A commit the worktree has not moved needs no refresh,
/// and asking Git would cost it a start and the real index's lock;
/// a commit whose worktree moved gets the paths a refresh has to look at,
///  which costs it a question per stale path instead of one per entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = staleStatData({ context, cwd, index, staging }); // unknown: ask Git
/// ```
#[cfg(unix)]
pub fn stale_stat_data(context: &GitContext, cwd: &Path, index: &Path, staging: WorktreeStaging<'_>) -> StaleStatData {
    // The reading is the preparation's own work: it parses the whole index and stats every entry the commit leaves alone,
    // so a record that names it is the only thing that can say whether a slow preparation was this question.
    let began: std::time::Instant = std::time::Instant::now();
    let answer: StaleStatData = read_in_process(context, cwd, index, staging);
    crate::prepare_work_spans::note_span(crate::prepare_work_spans::INDEX_STAT_CHECK, began.elapsed());
    return answer;
}

/// Where there is no `stat` this reading knows,
/// Git's own refresh always decides.
#[cfg(not(unix))]
pub fn stale_stat_data(_context: &GitContext, _cwd: &Path, _index: &Path, _staging: WorktreeStaging<'_>) -> StaleStatData {
    return StaleStatData::Unknown;
}

/// Controls for the question itself,
/// kept in a file of their own so the reading above stays readable.
#[cfg(test)]
#[path = "commit_index_stat_staleness_tests.rs"]
mod tests;
