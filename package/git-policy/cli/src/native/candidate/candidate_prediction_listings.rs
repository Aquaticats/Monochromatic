//! What:
//!  The listings one prediction makes before any candidate is judged:
//!  what `git add` with the caller's arguments would stage on a private index,
//!  the worktree files a direct command selects,
//!  the index entries the staged requests read,
//!  the `HEAD` records the changed paths are compared with,
//!  and the prefix no caller pathspec mode reaches.
//! Why:
//!  Every source answers the same question with a different pair of listings,
//!  and each one is a Git run followed by one parse;
//!  they live apart from the request and its preparation
//!  (`crate::candidate_prediction`) so each file says one thing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const staged = predictedAdd(store, git, prefix, overlay, ['--', 'src/a.rs']);
//! ```

/// Import the layer's failure type and its closed list of causes.
use crate::candidate_error::{
    CandidateError,
    CandidateFailure,
};
/// The landed commit object name a pushed or landed listing names.
use crate::candidate_object::ObjectId;
/// The size of a pathspec chunk,
/// kept in a file of its own so that sibling files share it without importing each other.
use crate::candidate_pathspec_chunk_size::PATHSPEC_CHUNK_SIZE;
/// Running Git for a prediction.
use crate::candidate_prediction_git_runs::run_git;
/// Import the candidate record a version is built from.
use crate::candidate_record::CandidateRecord;
/// Import the stage-record comparison and the two ways of turning entries into candidates.
use crate::candidate_stage::{
    ChangedPath,
    StageRecord,
    delta_candidates,
    scope_candidates,
    staged_delta,
};
/// Import the per-invocation owner of versions and bytes.
use crate::candidate_store::CandidateStore;
/// Import the captured-query runner and its byte helpers.
use crate::git_metadata::path_from_git_bytes;
/// Which pathspec names a pushed scope has already listed,
///  so each pathname answers with the commit or tree that listed it first.
use std::collections::HashMap;
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  One literal,
///  top-level pathspec naming exactly `path`,
///  or nothing when the bytes
///       cannot be a path on this platform.
/// Why:
///   `:(top,literal)` makes Git match the repository-relative name byte for byte from
///       any directory,
///  whatever wildcard or magic characters it contains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const literalPathspec = (path: Buffer) => Buffer.concat([Buffer.from(':(top,literal)'), path]);
/// ```
fn literal_pathspec(path: &[u8]) -> Option<OsString> {
    // `?` returns `None` when the bytes are not a path here.
    let native: PathBuf = path_from_git_bytes(path)?;
    let mut pathspec: OsString = OsString::from(":(top,literal)");
    pathspec.push(native.as_os_str());
    return Some(pathspec);
}

/// What:
///  The records of `HEAD` for the changed paths,
///  asked 2,048 paths at a time.
/// Why:
///   Which changed paths `HEAD` lacks or still has decides additions and deletions;
///       nothing outside the changed paths is listed,
///  so a conflict elsewhere in the
///       index cannot stop the prediction.
///  No change asks nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function baselineRecords(store: CandidateStore, changed: ChangedPath[]): Promise<CandidateRecord[]>;
/// ```
pub(crate) fn baseline_records(
    store: &mut CandidateStore,
    changed: &[ChangedPath],
) -> Result<Vec<CandidateRecord>, CandidateError> {
    let mut records: Vec<CandidateRecord> = Vec::new();
    // `.chunks(n)` yields consecutive slices of at most `n` items; an empty list yields none.
    for chunk in changed.chunks(PATHSPEC_CHUNK_SIZE) {
        let mut pathspecs: Vec<OsString> = Vec::with_capacity(chunk.len());
        for path in chunk {
            let Some(pathspec) = literal_pathspec(path.path.as_slice()) else {
                return Err(CandidateError::new(
                    CandidateFailure::ListingMalformed,
                    "monochromatic-git-policy-cli could not predict what git add stages: Git listed an entry whose \
                     name is not a path on this platform.",
                ));
            };
            pathspecs.push(pathspec);
        }
        records.extend(store.head_records(pathspecs.as_slice())?);
    }
    return Ok(records);
}

/// What:
///  The candidates `git add` with `region` would stage,
///  against `store`'s private index.
/// Why:
///   The whole index is listed before and after the replay,
///  so a path the replay
///       touched outside the caller's directory is still found.
///  The replay uses the
///       caller's own prefix and environment,
///  so it reads its pathspecs exactly as the
///       real command will.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function predictAdd(store, git, prefix, overlay, region): Promise<CandidateRecord[]>;
/// ```
pub(crate) fn predicted_add(
    store: &mut CandidateStore,
    real_git: &Path,
    prefix: &[OsString],
    overlay: &[(OsString, OsString)],
    region: &[OsString],
) -> Result<Vec<CandidateRecord>, CandidateError> {
    let whole_index: [OsString; 1] = [OsString::from(":/")];
    let before: Vec<StageRecord> = store.stage_records(&whole_index)?;
    let mut command: Vec<OsString> = vec![OsString::from("add")];
    command.extend_from_slice(region);
    run_git(
        real_git,
        prefix,
        command.as_slice(),
        overlay,
        &[],
        "predict what git add stages",
    )?;
    let after: Vec<StageRecord> = store.stage_records(&whole_index)?;
    let changed: Vec<ChangedPath> = staged_delta(before.as_slice(), after.as_slice());
    let baseline: Vec<CandidateRecord> = baseline_records(store, changed.as_slice())?;
    return delta_candidates(changed.as_slice(), baseline.as_slice());
}

/// What:
///  Every entry `pathspecs` select after staging them with `git add --all` on the
///       private index.
/// Why:
///   The private stage hashes the selected worktree files,
///  so their bytes are what
///       the worktree holds,
///  tracked and new files alike,
///  and the real index is untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function projectScope(store, git, prefix, overlay, pathspecs): Promise<CandidateRecord[]>;
/// ```
pub(crate) fn projected_scope(
    store: &mut CandidateStore,
    real_git: &Path,
    prefix: &[OsString],
    overlay: &[(OsString, OsString)],
    pathspecs: &[OsString],
) -> Result<Vec<CandidateRecord>, CandidateError> {
    let mut command: Vec<OsString> = vec![
        OsString::from("add"),
        OsString::from("--all"),
        OsString::from("--"),
    ];
    command.extend_from_slice(pathspecs);
    // Every name here is a pathspec the wrapper wrote (`:/` or `:(top,literal)<path>`), so no pathspec mode the caller
    //  set may reach this start: literal mode would read `:/` as a file name and the staging would match nothing.
    run_git(
        real_git,
        prefix,
        command.as_slice(),
        overlay,
        crate::conflict_side_modes::MODE_VARIABLES,
        "read the selected worktree files",
    )?;
    return staged_scope(store, pathspecs);
}

/// What:
///  The candidates of the entries the index already holds for these pathspecs,
/// against `HEAD`.
/// Why:
///   The direct and the staged requests both end here;
/// only the direct one stages the worktree first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scopeCandidates(await stageRecords(pathspecs), await headRecords(pathspecs));
/// ```
pub(crate) fn staged_scope(
    store: &mut CandidateStore,
    pathspecs: &[OsString],
) -> Result<Vec<CandidateRecord>, CandidateError> {
    let entries: Vec<StageRecord> = store.stage_records(pathspecs)?;
    if entries.is_empty() {
        // Nothing selected: no baseline is needed and no object reader starts.
        return Ok(Vec::new());
    }
    let baseline: Vec<CandidateRecord> = store.head_records(pathspecs)?;
    return scope_candidates(entries.as_slice(), baseline.as_slice());
}

/// What:
///  Record `source` as the commit or tree that first lists each pathname of `listed`.
///  `HashMap::entry(..).or_insert_with(..)` keeps the first source of a pathname.
/// Why:
///   A pushed pathname can be listed by several commits;
///  its candidate comes from the first,
/// so its attributes do too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const record of listed) if (!listedFrom.has(record.path)) listedFrom.set(record.path, source);
/// ```
pub(crate) fn note_listing(
    listed_from: &mut HashMap<Vec<u8>, ObjectId>,
    listed: &[CandidateRecord],
    source: &ObjectId,
) {
    for record in listed {
        // The source is cloned only for a path not yet listed, as `or_insert_with` ran its closure.
        if let std::collections::hash_map::Entry::Vacant(slot) =
            listed_from.entry(record.path.clone())
        {
            slot.insert(source.clone());
        }
    }
}

/// What:
///  Prepare the candidates of one request.
///  `real_git`,
/// What:
///  The caller's global arguments without the options that set a pathspec mode.
/// Why:
///  Every listing the candidate store makes relies on pathspec magic (`:/`,
///  `:(top,literal)`),
///  which literal pathspec mode turns into names that match nothing and so lists no candidate at all,
///  and case-insensitive matching widens to a name the commit never selected;
///  the store also removes the mode variables,
/// so both halves of a caller's mode are gone here.
///  Only the replay of the caller's own command keeps the caller's mode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const magicEnabledPrefix = (prefix: string[]) => withoutPathspecModeOptions(prefix);
/// ```
pub(crate) fn magic_enabled_prefix(global_prefix: &[OsString]) -> Vec<OsString> {
    return crate::global_arguments_pathspec_modes::without_pathspec_mode_options(global_prefix);
}
