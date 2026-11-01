//! What:
//!  The two candidate sources that need a private index:
//!  what `git add` would stage,
//!       and the worktree files a direct `git cli-git check` or `fix` selects.
//! Why:
//!  Both are answered before Git touches the real index.
//!  `git add` is replayed on a
//!      private copy of the index,
//!  and the candidates are the entries the replay changed,
//!      as the installed wrapper's `add-policy-facts.ts` and `add-staged-delta.ts` define
//!      them.
//!  A direct command stages its scope with `git add --all` on a private copy,
//!      as `direct-check-facts.ts` does,
//!  and every selected entry is a candidate.
//!  Either
//!      way the bytes are blobs Git hashed from the worktree at that moment,
//!  read through
//!      the one object reader,
//!  so the number of Git processes does not grow with the
//!      number of files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await using prepared = await prepareCandidates(git, { kind: 'add', region: ['--', 'a.txt'] });
//! // for (const candidate of prepared.version.candidates) await prepared.store.bytes(candidate);
//! ```

/// Import the layer's failure type and its closed list of causes.
use super::candidate_attribute::{AttributeSource, CandidateAttributes};
use super::candidate_error::{CandidateError, CandidateFailure};
/// The landed commit object name a landed-delta request reads.
use super::candidate_object::ObjectId;
/// Import the private index copy every prediction stages into.
use super::candidate_private_index::PrivateIndex;
/// Import the candidate record a version is built from.
use super::candidate_record::CandidateRecord;
/// Import the stage-record comparison and the two ways of turning entries into candidates.
use super::candidate_stage::{
    ChangedPath, StageRecord, delta_candidates, scope_candidates, staged_delta,
};
/// Import the per-invocation owner of versions and bytes.
use super::candidate_store::CandidateStore;
/// Import the immutable version type.
use super::candidate_version::{CandidateSource, CandidateVersion};
/// Import the captured-query runner and its byte helpers.
use super::git_metadata::{MetadataOutput, path_from_git_bytes, run_metadata_git, strip_git_line};
/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Arguments,
///  pathspecs and environment values reach Git byte for byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::collections::HashMap;
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};
/// `Rc<T>` is a shared,
///  read-only handle;
///  versions are shared by every policy of a pass.
use std::rc::Rc;

/// The environment variable that points Git at a different index file.
pub const INDEX_FILE_VARIABLE: &str = "GIT_INDEX_FILE";

/// What:
///  The most pathspecs one `git diff-index` receives.
///  `usize` is the type of counts.
/// Why:
///   The installed wrapper passes candidate paths 2,048 at a time
///       (`commit-transaction-candidate-batch.ts`),
///  far below any platform's argument
///       limit;
///  one process per 2,048 changed paths keeps the count bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PATHSPEC_CHUNK_SIZE = 2_048;
/// ```
pub const PATHSPEC_CHUNK_SIZE: usize = 2_048;

/// What:
///  What a lifecycle asks for.
///  An `enum` is a closed set of named alternatives;
///  each
///       carries the arguments it needs.
///  `#[derive(...)]` generates cloning,
///  debug
///       printing and `==`.
/// Why:
///   The two sources differ only in what is staged on the private copy and in which
///       entries become candidates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CandidateRequest = { kind: 'add'; region: string[] } | { kind: 'direct'; pathspecs: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CandidateRequest {
    /// What `git add` with these arguments would stage.
    Add(
        /// The arguments after `add`,
        ///  wrapper controls removed.
        Vec<OsString>,
    ),
    /// The worktree files a direct command selects.
    Direct(
        /// The pathspecs of the scope;
        ///  `--all` is `:/`.
        Vec<OsString>,
    ),
    /// The entries the index already holds for the pathspecs, read without staging the worktree first.
    ///  The commit path uses it, so corrections already written to the private index are the candidates.
    Staged(
        /// The pathspecs of the scope.
        Vec<OsString>,
    ),
    /// The entries the private index holds for the pathspecs, with a recorded base commit (the empty tree when the
    ///  branch was unborn) as the baseline that marks additions, read without staging the worktree. The commit
    ///  transaction uses it: its baseline is the base it recorded at invocation, never the `HEAD` another landing may
    ///  have moved since. No pathspec means no candidate.
    StagedAtBase {
        /// The recorded base commit, or nothing for an unborn branch.
        base: Option<ObjectId>,
        /// The pathspecs of the scope, each literal and rooted at the top level.
        pathspecs: Vec<OsString>,
    },
    /// The changes a manual push publishes: each newly published commit's own delta and the whole tree
    ///  of each tip with no known published base.
    Pushed(
        /// The commits and trees the scan reads.
        PushedScope,
    ),
    /// The paths one landed commit changed, each against every parent,
    ///  as the post-commit gate reads them: the committed bytes and no worktree or index state.
    Landed(
        /// The landed commit whose tree changes are listed.
        ObjectId,
    ),
}

/// What:
///  The commits and trees a manual push scans.
/// Why:
///   A pushed commit contributes its own delta, and a tip with no known base contributes its tree; both are
///  listed without staging anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushedScope = { commits: ObjectId[]; trees: ObjectId[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushedScope {
    /// Newly published commits, each contributing its own delta.
    pub commits: Vec<ObjectId>,
    /// Pushed tips whose whole tree is new.
    pub trees: Vec<ObjectId>,
}

/// What:
///  Prepared candidates:
///  the store that reads their bytes,
///  the version that lists
///       them,
///  and the private index the store is bound to.
///  Fields are dropped in the
///       order written,
///  so the store's object reader ends before the private directory
///       is removed.
/// Why:
///   The store keeps reading blobs after preparation;
///  the private index must outlive it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparedCandidates = { store: CandidateStore; version: CandidateVersion; [Symbol.dispose](): void };
/// ```
pub struct PreparedCandidates {
    /// Reads candidate bytes through one long-lived object reader.
    pub store: CandidateStore,
    /// The candidates,
    ///  in the order the installed wrapper reports them.
    pub version: Rc<CandidateVersion>,
    /// The real index the private copy was made from,
    ///  which a direct fix proves unchanged.
    pub real_index: PathBuf,
    /// Reads the `text` attribute of the candidates from the state they were listed from.
    pub attributes: CandidateAttributes,
    /// Kept only so the private directory lives as long as the store;
    ///  never read.
    _private: PrivateIndex,
}

/// What:
///  Build the failure of a Git command a prediction ran.
///       `&MetadataOutput` borrows the captured run;
///  `None` means Git could not be started.
/// Why:
///   A failed replay is reported with Git's own message,
///  so the person sees why the
///       command they typed could not be staged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function predictionFailure(purpose: string, operation: string, output?: GitOutput, error?: Error): CandidateError;
/// ```
fn command_failure(purpose: &str, operation: &str, output: &MetadataOutput) -> CandidateError {
    // `String::from_utf8_lossy` decodes Git's message for display, replacing invalid bytes.
    return CandidateError::new(
        CandidateFailure::GitFailed,
        format!(
            "cli-git could not {purpose}: git {operation} failed: {}",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        )
        .as_str(),
    );
}

/// What:
///  Build the failure of a Git command that could not be started.
/// Why:
///   Without Git there is nothing to predict,
///  and the person needs the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function notStarted(purpose: string, operation: string, error: Error): CandidateError;
/// ```
fn not_started(purpose: &str, operation: &str, error: &std::io::Error) -> CandidateError {
    return CandidateError::new(
        CandidateFailure::GitNotStarted,
        format!("cli-git could not {purpose}: git {operation} could not be started: {error}.")
            .as_str(),
    );
}

/// What:
///  Run one Git command of a prediction and return its output when it succeeded.
///       `&[OsString]` borrows the global prefix and the command separately.
/// Why:
///   The prefix selects the same repository the caller's command selects;
///  the
///       command follows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runGit(git, prefix, command, overlay, purpose): Promise<GitOutput>;
/// ```
fn run_git(
    real_git: &Path,
    prefix: &[OsString],
    command: &[OsString],
    overlay: &[(OsString, OsString)],
    purpose: &str,
) -> Result<MetadataOutput, CandidateError> {
    // `.to_vec()` copies the borrowed prefix; `.extend_from_slice(..)` appends the command.
    let mut arguments: Vec<OsString> = prefix.to_vec();
    arguments.extend_from_slice(command);
    // The operation named in messages is the command's first word, `add` or `rev-parse`.
    let operation: String = match command.first() {
        Some(word) => word.to_string_lossy().into_owned(),
        None => String::from("(no command)"),
    };
    // `match` on the start result: `Err(error)` means Git could not be started at all.
    let output: MetadataOutput = match run_metadata_git(real_git, arguments.as_slice(), overlay) {
        Ok(captured) => captured,
        Err(error) => return Err(not_started(purpose, operation.as_str(), &error)),
    };
    if !output.success {
        return Err(command_failure(purpose, operation.as_str(), &output));
    }
    return Ok(output);
}

/// What:
///  The absolute path of the index the caller's command would use.
/// Why:
///   Git answers for `GIT_INDEX_FILE`,
///  a linked worktree's own index and
///       `--git-dir`,
///  so the copy is of exactly the index `git add` would change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function realIndexPath(git, prefix, overlay): Promise<string>;
/// ```
fn real_index_path(
    real_git: &Path,
    prefix: &[OsString],
    overlay: &[(OsString, OsString)],
    purpose: &str,
) -> Result<PathBuf, CandidateError> {
    let output: MetadataOutput = run_git(
        real_git,
        prefix,
        &[
            OsString::from("rev-parse"),
            OsString::from("--path-format=absolute"),
            OsString::from("--git-path"),
            OsString::from("index"),
        ],
        overlay,
        purpose,
    )?;
    // `match` unpacks "a path or nothing": empty output names no index.
    match path_from_git_bytes(strip_git_line(output.stdout.as_slice())) {
        Some(path) => return Ok(path),
        None => {
            return Err(CandidateError::new(
                CandidateFailure::ListingMalformed,
                format!(
                    "cli-git could not {purpose}: git rev-parse --git-path index printed no path."
                )
                .as_str(),
            ));
        }
    }
}

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
fn baseline_records(
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
                    "cli-git could not predict what git add stages: Git listed an entry whose \
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
fn predicted_add(
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
fn projected_scope(
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
    run_git(
        real_git,
        prefix,
        command.as_slice(),
        overlay,
        "read the selected worktree files",
    )?;
    return staged_scope(store, pathspecs);
}

/// What:
///  The candidates of the entries the index already holds for these pathspecs, against `HEAD`.
/// Why:
///   The direct and the staged requests both end here; only the direct one stages the worktree first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scopeCandidates(await stageRecords(pathspecs), await headRecords(pathspecs));
/// ```
fn staged_scope(
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
///  its candidate comes from the first, so its attributes do too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const record of listed) if (!listedFrom.has(record.path)) listedFrom.set(record.path, source);
/// ```
fn note_listing(
    listed_from: &mut HashMap<Vec<u8>, ObjectId>,
    listed: &[CandidateRecord],
    source: &ObjectId,
) {
    for record in listed {
        listed_from
            .entry(record.path.clone())
            .or_insert_with(|| { return source.clone(); });
    }
}

/// What:
///  Prepare the candidates of one request.
///  `real_git`,
///  `global_prefix` and `overlay`
///       are the Git executable,
///  the caller's arguments before the subcommand and the
///       child environment additions,
///  exactly as every other Git query receives them.
/// Why:
///   The real index is copied first,
///  so nothing below can change it.
///  For `git add`
///       the store's own listings add `--no-literal-pathspecs` after the caller's prefix:
///       they rely on pathspec magic (`:/`,
///  `:(top,literal)`),
///  which literal pathspec
///       mode,
///  from `GIT_LITERAL_PATHSPECS` or `--literal-pathspecs`,
///  would turn into
///       names that match nothing,
///  leaving every change unchecked.
///  The replay keeps the
///       caller's mode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareCandidates(git, prefix, overlay, request): Promise<PreparedCandidates>;
/// ```
pub fn prepare_candidates(
    real_git: &Path,
    global_prefix: &[OsString],
    overlay: &[(OsString, OsString)],
    request: &CandidateRequest,
) -> Result<PreparedCandidates, CandidateError> {
    let purpose: &str = match request {
        CandidateRequest::Add(_) => "predict what git add stages",
        CandidateRequest::Direct(_) => "read the selected worktree files",
        CandidateRequest::Staged(_) | CandidateRequest::StagedAtBase { .. } => "read the staged index entries",
        CandidateRequest::Landed(_) => "read the landed commit changes",
        CandidateRequest::Pushed(_) => "read the pushed commit changes",
    };
    let real_index: PathBuf = real_index_path(real_git, global_prefix, overlay, purpose)?;
    let private: PrivateIndex = PrivateIndex::create(real_index.as_path())?;
    // The caller's environment, with every command of the prediction on the private index.
    let mut private_overlay: Vec<(OsString, OsString)> = overlay.to_vec();
    private_overlay.push((
        OsString::from(INDEX_FILE_VARIABLE),
        private.path().as_os_str().to_os_string(),
    ));
    // A pushed scope lists committed records directly; nothing is staged or predicted for it.
    if let CandidateRequest::Pushed(scope) = request {
        let mut pushed_store: CandidateStore =
            CandidateStore::new(real_git, global_prefix, private_overlay.as_slice());
        let mut records: Vec<CandidateRecord> = Vec::new();
        // The commit or tree that first lists a pathname answers for its attributes, as it answers for the
        // pathname's candidate.
        let mut listed_from: HashMap<Vec<u8>, ObjectId> = HashMap::new();
        for commit in &scope.commits {
            let listed: Vec<CandidateRecord> = pushed_store.committed_records(commit)?;
            note_listing(&mut listed_from, listed.as_slice(), commit);
            records.extend(listed);
        }
        for tree in &scope.trees {
            let listed: Vec<CandidateRecord> = pushed_store.tree_records(tree)?;
            note_listing(&mut listed_from, listed.as_slice(), tree);
            records.extend(listed);
        }
        let version: Rc<CandidateVersion> = pushed_store.derived_version(records);
        return Ok(PreparedCandidates {
            store: pushed_store,
            version,
            real_index,
            attributes: CandidateAttributes::new(
                real_git,
                global_prefix,
                private_overlay.as_slice(),
                AttributeSource::PerPath(listed_from),
            ),
            _private: private,
        });
    }
    // A landed commit lists its committed records directly; nothing is staged or predicted for it.
    if let CandidateRequest::Landed(commit) = request {
        let mut landed_store: CandidateStore =
            CandidateStore::new(real_git, global_prefix, private_overlay.as_slice());
        let version: Rc<CandidateVersion> =
            landed_store.version(&CandidateSource::Committed(commit.clone()))?;
        return Ok(PreparedCandidates {
            store: landed_store,
            version,
            real_index,
            attributes: CandidateAttributes::new(
                real_git,
                global_prefix,
                private_overlay.as_slice(),
                AttributeSource::Tree(commit.clone()),
            ),
            _private: private,
        });
    }
    // `match` picks the source; each arm builds its store and its candidate records.
    // `mut store` because building the version below reads the store's generation.
    let (mut store, records): (CandidateStore, Vec<CandidateRecord>) = match request {
        CandidateRequest::Add(region) => {
            let mut listing_prefix: Vec<OsString> = global_prefix.to_vec();
            listing_prefix.push(OsString::from("--no-literal-pathspecs"));
            let mut add_store: CandidateStore = CandidateStore::new(
                real_git,
                listing_prefix.as_slice(),
                private_overlay.as_slice(),
            );
            let staged: Vec<CandidateRecord> = predicted_add(
                &mut add_store,
                real_git,
                global_prefix,
                private_overlay.as_slice(),
                region.as_slice(),
            )?;
            (add_store, staged)
        }
        CandidateRequest::Landed(_) | CandidateRequest::Pushed(_) => {
            unreachable!("landed and pushed requests return before the staged match");
        }
        CandidateRequest::Direct(pathspecs) => {
            // The same magic-enabled prefix as the add arm: the staging and the listings must read
            // `:/` and the selected pathspecs alike, so literal mode cannot split them.
            let mut scope_prefix: Vec<OsString> = global_prefix.to_vec();
            scope_prefix.push(OsString::from("--no-literal-pathspecs"));
            let mut scope_store: CandidateStore =
                CandidateStore::new(real_git, scope_prefix.as_slice(), private_overlay.as_slice());
            let selected: Vec<CandidateRecord> = projected_scope(
                &mut scope_store,
                real_git,
                scope_prefix.as_slice(),
                private_overlay.as_slice(),
                pathspecs.as_slice(),
            )?;
            (scope_store, selected)
        }
        CandidateRequest::Staged(pathspecs) => {
            let mut scope_store: CandidateStore =
                CandidateStore::new(real_git, global_prefix, private_overlay.as_slice());
            let selected: Vec<CandidateRecord> = staged_scope(&mut scope_store, pathspecs.as_slice())?;
            (scope_store, selected)
        }
        CandidateRequest::StagedAtBase { base, pathspecs } => {
            let mut scope_store: CandidateStore =
                CandidateStore::new(real_git, global_prefix, private_overlay.as_slice()).with_baseline(base.clone());
            // An empty scope would list the whole index, so it is answered without asking Git.
            let selected: Vec<CandidateRecord> = if pathspecs.is_empty() {
                Vec::new()
            } else {
                staged_scope(&mut scope_store, pathspecs.as_slice())?
            };
            (scope_store, selected)
        }
    };
    let version: Rc<CandidateVersion> = store.derived_version(records);
    return Ok(PreparedCandidates {
        store,
        version,
        real_index,
        // Staged and projected candidates are judged by the private index they were listed from.
        attributes: CandidateAttributes::new(
            real_git,
            global_prefix,
            private_overlay.as_slice(),
            AttributeSource::Index,
        ),
        _private: private,
    });
}

/// Prediction and projection controls against real Git stay out of the release executable.
#[cfg(test)]
#[path = "candidate_prediction_tests.rs"]
mod tests;

/// The process-count bound of both sources and its per-file positive control.
#[cfg(test)]
#[path = "candidate_prediction_count_tests.rs"]
mod count_tests;
