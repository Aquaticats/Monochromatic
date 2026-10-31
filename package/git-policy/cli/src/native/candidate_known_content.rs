//! What:
//!  Which candidates hold content that a parent commit's tree already holds,
//!  found by exact blob object name.
//! Why:
//!  A pure rename or a copy re-lists a file as an addition,
//!  and a merge lists every file one parent lacks as an addition,
//!  although the bytes are already in history.
//!  Scanning them again refuses moves of
//!      files that were accepted when they entered history,
//!  while an object name is the hash of the bytes,
//!  so a
//!      candidate whose object a parent holds carries no byte that is new.
//!  The comparison is by object
//!      name only:
//!  never by pathname,
//!  never by similarity,
//!  so one changed byte makes the content new.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const known = await findKnownContent(store, version, ContentBasis.of(request), rulesPath, corrected);
//! // known.entry(candidate.identity.index)?.parent
//! ```

/// Import the layer's failure type.
use super::candidate_error::CandidateError;
/// Import the candidate modes and the validated object name.
use super::candidate_object::{CandidateMode, ObjectId};
/// Import what a lifecycle asks for.
use super::candidate_prediction::CandidateRequest;
/// Import the store that lists commits and trees.
use super::candidate_store::CandidateStore;
/// Import the candidate and the immutable version type.
use super::candidate_version::{Candidate, CandidateVersion};
/// Import the change a converged fix installs,
///  which marks a candidate whose bytes are no longer its object's.
use super::direct_fix_install::InstallChange;
/// Import which candidates a scan reads at all.
use super::scanner_selection::is_scannable;
/// What:
///  `BTreeMap<K, V>` is a map kept sorted by key (sibling `HashMap` is unordered);
///  `HashSet<T>` is an
///       unordered set with constant-time membership tests.
/// Why:
///   Known candidates are listed in candidate order,
///  and each parent's blobs are looked up by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Map<number, KnownEntry> iterated in key order; Set<ObjectId>.
/// ```
use std::collections::{BTreeMap, HashSet};

/// What:
///  The commits whose trees count as already holding content for one lifecycle.
///       `#[derive(...)]` generates cloning,
///  debug printing and `==`.
/// Why:
///   Each lifecycle compares with a different history:
///  the commit being made with its base,
///  a landed
///       commit with each of its parents,
///  a push with the parents that were published before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ContentBasis = { kind: 'nothing' } | { kind: 'baseline' } | { kind: 'parents-outside'; commits: ObjectId[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentBasis {
    /// No tree counts:
    ///  a direct check reads the selected files for what they hold now,
    ///  and a push with a tip whose published history is unknown has no commit to vouch for anything.
    Nothing,
    /// The commit the candidates are compared with:
    ///  the recorded base of a commit,
    ///  else the commit `HEAD` names;
    ///  an unborn branch has none.
    Baseline,
    /// Every parent of these commits that is not one of them:
    ///  a landed commit's parents,
    ///  or a push's already published parents.
    ParentsOutside(
        /// The commits whose parents are listed.
        Vec<ObjectId>,
    ),
}

/// What:
///  `impl ContentBasis { ... }` attaches the choice for a request.
/// Why:
///   The choice follows the request's kind alone,
///  so no policy decides it separately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ContentBasis { static of(request: CandidateRequest): ContentBasis }
/// ```
impl ContentBasis {
    /// What:
    ///  The basis of one request.
    ///  `&CandidateRequest` borrows what the lifecycle asked for.
    /// Why:
    ///   A parent commit of a pushed commit that is itself pushed is new to the remote,
    ///  so a push counts only the parents outside the pushed commits.
    ///  A pushed tip tree has no published
    ///       base to compare with,
    ///  and its candidates share one version with the commits' candidates,
    ///  so a push with such a tip skips nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static of(request: CandidateRequest): ContentBasis
    /// ```
    pub fn of(request: &CandidateRequest) -> ContentBasis {
        match request {
            CandidateRequest::Direct(_) => return ContentBasis::Nothing,
            CandidateRequest::Add(_)
            | CandidateRequest::Staged(_)
            | CandidateRequest::StagedAtBase { .. } => return ContentBasis::Baseline,
            CandidateRequest::Landed(commit) => {
                return ContentBasis::ParentsOutside(vec![commit.clone()]);
            }
            CandidateRequest::Pushed(scope) => {
                if !scope.trees.is_empty() {
                    return ContentBasis::Nothing;
                }
                return ContentBasis::ParentsOutside(scope.commits.clone());
            }
        }
    }
}

/// What:
///  One candidate whose content a parent already holds:
///  the object and the first parent that holds it.
///       `#[derive(...)]` generates cloning,
///  debug printing and `==`.
/// Why:
///   The skip is a decision,
///  and a decision is only checkable when its inputs are kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KnownEntry = { object: ObjectId; parent: ObjectId };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnownEntry {
    /// The candidate's blob.
    pub object: ObjectId,
    /// The commit whose tree holds the blob.
    pub parent: ObjectId,
}

/// What:
///  `impl KnownEntry { ... }` attaches the record of the decision.
/// Why:
///   One wording for the decision,
///  so a log line always carries the same inputs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class KnownEntry { describe(displayPath: string): string }
/// ```
impl KnownEntry {
    /// What:
    ///  The decision as one line of text.
    ///  `&str` is the pathname as the scanner masks it.
    /// Why:
    ///   The pathname is shown masked because a pathname may hold a secret;
    ///  the blob and the parent are
    ///       object names.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// describe(displayPath: string): string
    /// ```
    pub fn describe(&self, display_path: &str) -> String {
        return format!(
            "skipped the content of {display_path}: blob {} is already in the tree of commit {}; only the pathname was checked.",
            self.object.as_str(),
            self.parent.as_str()
        );
    }
}

/// What:
///  The known candidates of one version,
///  by the candidate's position in the version.
///  `#[derive(...)]` generates cloning,
///  debug printing and `==`;
///  `Default` is the empty value.
/// Why:
///   The scan pass asks by position,
///  and the log lists them in candidate order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class KnownContent { entry(index: number): KnownEntry | undefined }
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KnownContent {
    /// The known candidates,
    ///  sorted by position.
    entries: BTreeMap<usize, KnownEntry>,
}

/// What:
///  `impl KnownContent { ... }` attaches the read-only accessors.
/// Why:
///   A scan can read the decision and cannot change it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class KnownContent { entry(index) {} len() {} }
/// ```
impl KnownContent {
    /// What:
    ///  The entry of the candidate at `index`,
    ///  if its content is known.
    /// Why:
    ///   The scan pass asks once per candidate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// entry(index: number): KnownEntry | undefined
    /// ```
    pub fn entry(&self, index: usize) -> Option<&KnownEntry> {
        return self.entries.get(&index);
    }

    /// What:
    ///  How many candidates are known.
    /// Why:
    ///   A pass with none reads exactly the bytes it read before the rule existed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get length(): number
    /// ```
    pub fn len(&self) -> usize {
        return self.entries.len();
    }

    /// What:
    ///  Whether no candidate is known.
    /// Why:
    ///   Clippy asks a type with a `len()` to provide `is_empty()` too.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get isEmpty(): boolean
    /// ```
    pub fn is_empty(&self) -> bool {
        return self.entries.is_empty();
    }
}

/// What:
///  Whether a candidate's content can be known from a parent's tree at all.
///       `&[Vec<u8>]` is the rules files' own pathnames.
/// Why:
///   Only a scanned blob has bytes that a parent could already hold.
///  A submodule entry is a commit name,
///  and a file a fix corrected no longer holds its object's bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const comparable = (candidate, rulesPath, corrected) => isScannable(candidate, rulesPath) && candidate.object !== undefined && candidate.mode !== 'gitlink' && !corrected.has(candidate.path);
/// ```
fn is_comparable(
    candidate: &Candidate,
    rules_paths: &[Vec<u8>],
    corrected: &BTreeMap<Vec<u8>, InstallChange>,
) -> bool {
    return is_scannable(candidate, rules_paths)
        && candidate.object.is_some()
        && candidate.mode != CandidateMode::Gitlink
        && !corrected.contains_key(&candidate.path);
}

/// What:
///  Find every comparable candidate whose blob a parent's tree holds.
///  `&mut CandidateStore` lends the listing;
///  `&[Vec<u8>]` is the rules files' pathnames;
///  `&BTreeMap<..>` are the files a fix corrected.
/// Why:
///   Parents are listed one after another and listing stops when every candidate is placed,
///  so a
///       version that is all known costs one listing.
///  No candidate to compare and no parent to compare with
///       each cost no Git process beyond the basis question.
///  The first parent that holds a blob is the one reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function findKnownContent(store, version, basis, rulesPath, corrected): Promise<KnownContent>;
/// ```
pub fn find_known_content(
    store: &mut CandidateStore,
    version: &CandidateVersion,
    basis: &ContentBasis,
    rules_paths: &[Vec<u8>],
    corrected: &BTreeMap<Vec<u8>, InstallChange>,
) -> Result<KnownContent, CandidateError> {
    let mut pending: Vec<&Candidate> = Vec::new();
    for candidate in version.candidates() {
        if is_comparable(candidate, rules_paths, corrected) {
            pending.push(candidate);
        }
    }
    let mut entries: BTreeMap<usize, KnownEntry> = BTreeMap::new();
    if pending.is_empty() {
        return Ok(KnownContent { entries });
    }
    let parents: Vec<ObjectId> = match basis {
        ContentBasis::Nothing => Vec::new(),
        // `.into_iter().collect()` turns "a commit or nothing" into a list of zero or one.
        ContentBasis::Baseline => store.baseline_reference()?.into_iter().collect(),
        ContentBasis::ParentsOutside(commits) => store.parents_outside(commits.as_slice())?,
    };
    for parent in parents {
        if pending.is_empty() {
            break;
        }
        let mut blobs: HashSet<ObjectId> = HashSet::new();
        for record in store.commit_tree_records(&parent)? {
            if record.mode != CandidateMode::Gitlink {
                blobs.insert(record.object);
            }
        }
        let mut unplaced: Vec<&Candidate> = Vec::new();
        for candidate in pending {
            match candidate.object.as_ref() {
                Some(object) if blobs.contains(object) => {
                    entries.insert(
                        candidate.identity.index,
                        KnownEntry {
                            object: object.clone(),
                            parent: parent.clone(),
                        },
                    );
                }
                Some(_) | None => unplaced.push(candidate),
            }
        }
        pending = unplaced;
    }
    return Ok(KnownContent { entries });
}

/// Basis choice,
///  lookup by object name and the stop once every candidate is placed stay out of the release executable.
#[cfg(test)]
#[path = "candidate_known_content_tests.rs"]
mod tests;
