//! What:
//!  The two candidate sources that need a private index:
//!  what `git add` would stage,
//!       and the worktree files a direct `git monochromatic-git-policy-cli check` or `fix` selects.
//! Why:
//!  Both are answered before Git touches the real index.
//!  `git add` is replayed on a
//!      private copy of the index,
//!  and the candidates are the entries the replay changed.
//!  A direct command stages its scope with `git add --all` on a private copy,
//!  and every selected entry is a candidate.
//!  Either
//!      way the bytes are blobs Git hashed from the worktree at that moment,
//!  read through
//!      the one object reader,
//!  so the number of Git processes does not grow with the
//!      number of files.
//!  The listings each source makes are `crate::candidate_prediction_listings`'s;
//!  this file holds the request,
//!  what a caller gets back and the one preparation that answers it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await using prepared = await prepareCandidates(git, { kind: 'add', region: ['--', 'a.txt'] });
//! // for (const candidate of prepared.version.candidates) await prepared.store.bytes(candidate);
//! ```

/// The candidate's text attribute and where its candidates were listed from.
use crate::candidate_attribute::{
    AttributeSource,
    CandidateAttributes,
};
/// Import the layer's failure type.
use crate::candidate_error::CandidateError;
/// The landed commit object name a landed-delta request reads.
use crate::candidate_object::ObjectId;
/// Running Git for a prediction,
/// and the path of the real index.
use crate::candidate_prediction_git_runs::real_index_path;
/// The listings each source makes,
/// and the prefix no caller pathspec mode reaches.
use crate::candidate_prediction_listings::{
    magic_enabled_prefix,
    note_listing,
    predicted_add,
    projected_scope,
    staged_scope,
};
/// Import the private index copy every prediction stages into.
use crate::candidate_private_index::PrivateIndex;
/// Import the candidate record a version is built from.
use crate::candidate_record::CandidateRecord;
/// Import the per-invocation owner of versions and bytes.
use crate::candidate_store::CandidateStore;
/// Import the immutable version type.
use crate::candidate_version::{
    CandidateSource,
    CandidateVersion,
};
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
use std::path::{
    Path,
    PathBuf,
};
/// `Rc<T>` is a shared,
///  read-only handle;
///  versions are shared by every policy of a pass.
use std::rc::Rc;

/// The environment variable that points Git at a different index file.
pub const INDEX_FILE_VARIABLE: &str = "GIT_INDEX_FILE";

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
    /// The entries the index already holds for the pathspecs,
    /// read without staging the worktree first.
    ///  The commit path uses it,
    /// so corrections already written to the private index are the candidates.
    Staged(
        /// The pathspecs of the scope.
        Vec<OsString>,
    ),
    /// The entries the private index holds for the pathspecs,
    /// with a recorded base commit (the empty tree when the
    ///  branch was unborn) as the baseline that marks additions,
    /// read without staging the worktree.
    /// The commit
    ///  transaction uses it:
    /// its baseline is the base it recorded at invocation,
    /// never the `HEAD` another landing may
    ///  have moved since.
    /// No pathspec means no candidate.
    StagedAtBase {
        /// The recorded base commit,
        /// or nothing for an unborn branch.
        base: Option<ObjectId>,
        /// The pathspecs of the scope,
        /// each literal and rooted at the top level.
        pathspecs: Vec<OsString>,
    },
    /// The changes a manual push publishes:
    /// each newly published commit's own delta and the whole tree
    ///  of each tip with no known published base.
    Pushed(
        /// The commits and trees the scan reads.
        PushedScope,
    ),
    /// The paths one landed commit changed,
    /// each against every parent,
    ///  as the post-commit gate reads them:
    /// the committed bytes and no worktree or index state.
    Landed(
        /// The landed commit whose tree changes are listed.
        ObjectId,
    ),
}

/// What:
///  The commits and trees a manual push scans.
/// Why:
///   A pushed commit contributes its own delta,
/// and a tip with no known base contributes its tree;
/// both are
///  listed without staging anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushedScope = { commits: ObjectId[]; trees: ObjectId[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushedScope {
    /// Newly published commits,
    /// each contributing its own delta.
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
    /// A request that only lists the index needs no copy of it,
    /// and this is then nothing.
    _private: Option<PrivateIndex>,
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
///  Every listing and the staging of a direct scope read their pathspecs through
///       `magic_enabled_prefix`,
///  so no global pathspec mode can empty them;
///  only the replay of a
///       `git add` keeps the caller's mode.
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
        CandidateRequest::Staged(_)
        | CandidateRequest::StagedAtBase {
            ..
        } => "read the staged index entries",
        CandidateRequest::Landed(_) => "read the landed commit changes",
        CandidateRequest::Pushed(_) => "read the pushed commit changes",
    };
    let real_index: PathBuf = real_index_path(real_git, global_prefix, overlay, purpose)?;
    // What: A private copy of the index is made only for a request that writes it: predicting `git add` and reading a
    //       selected worktree stage into it, and everything else only lists it.
    // Why:  The copy is three megabytes of a large worktree, and a commit makes it after it already has its own private
    //       index: a traced commit waited 2,288 ms and 4,813 ms in this copy under the host's input and output pressure.
    //       A request that lists the index reads the one the caller named, which is already private to that command.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const private = request.stages ? PrivateIndex.create(realIndex) : undefined;
    // ```
    let private: Option<PrivateIndex> = match request {
        CandidateRequest::Add(_) | CandidateRequest::Direct(_) => {
            Some(PrivateIndex::create(real_index.as_path())?)
        }
        CandidateRequest::Staged(_)
        | CandidateRequest::StagedAtBase {
            ..
        }
        | CandidateRequest::Landed(_)
        | CandidateRequest::Pushed(_) => None,
    };
    // The caller's environment, with every command of the prediction on the private index where there is one.
    let mut private_overlay: Vec<(OsString, OsString)> = overlay.to_vec();
    if let Some(copy) = private.as_ref() {
        private_overlay.push((
            OsString::from(INDEX_FILE_VARIABLE),
            copy.path().as_os_str().to_os_string(),
        ));
    }
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
            let listing_prefix: Vec<OsString> = magic_enabled_prefix(global_prefix);
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
            let scope_prefix: Vec<OsString> = magic_enabled_prefix(global_prefix);
            let mut scope_store: CandidateStore = CandidateStore::new(
                real_git,
                scope_prefix.as_slice(),
                private_overlay.as_slice(),
            );
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
            let scope_prefix: Vec<OsString> = magic_enabled_prefix(global_prefix);
            let mut scope_store: CandidateStore = CandidateStore::new(
                real_git,
                scope_prefix.as_slice(),
                private_overlay.as_slice(),
            );
            let selected: Vec<CandidateRecord> =
                staged_scope(&mut scope_store, pathspecs.as_slice())?;
            (scope_store, selected)
        }
        CandidateRequest::StagedAtBase {
            base,
            pathspecs,
        } => {
            let scope_prefix: Vec<OsString> = magic_enabled_prefix(global_prefix);
            let mut scope_store: CandidateStore = CandidateStore::new(
                real_git,
                scope_prefix.as_slice(),
                private_overlay.as_slice(),
            )
            .with_baseline(base.clone());
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
