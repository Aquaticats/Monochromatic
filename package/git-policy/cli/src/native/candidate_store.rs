//! What:
//!  The per-invocation owner of candidate versions and candidate bytes.
//! Why:
//!  Policies read committed or staged content,
//!  never whatever is in the live
//!      worktree.
//!  A version costs one listing process,
//!  its bytes come from one
//!      long-lived object reader,
//!  and neither count grows with the number of files.
//!      After a fix or replay changed the index,
//!  `invalidate` retires every version.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const store = new CandidateStore(git, prefix, overlay);
//! // const version = await store.version({ kind: 'staged-against-head' });
//! // const bytes = await store.bytes(version.candidates[0]);
//! ```

/// Import the layer's failure type and its closed list of causes.
use super::candidate_error::{CandidateError, CandidateFailure};
/// Import the candidate modes,
///  the validated object name and its parser.
use super::candidate_object::{CandidateMode, ObjectId, parse_object_id};
/// Import the long-lived object reader.
use super::candidate_reader::{ObjectReader, start_object_reader};
/// Import the listing parser and its records.
use super::candidate_record::{CandidateRecord, parse_raw_records};
/// Import the stage-record parser of `git ls-files --stage`.
use super::candidate_stage::{StageRecord, parse_stage_records};
/// Import the tree-record parser of `git ls-tree -r`.
use super::candidate_tree::{TreeRecord, parse_tree_records};
/// Import the version types and builder.
use super::candidate_version::{Candidate, CandidateSource, CandidateVersion, build_version};
/// Import the captured-query runner and its line helper.
use super::git_metadata::{MetadataOutput, run_metadata_git, strip_git_line};
/// `HashSet<T>` is an unordered collection of distinct values with constant-time membership tests.
use std::collections::HashSet;
/// `OsString` is owned operating-system text of raw OS bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};
/// What:
///  `Rc<T>` is a shared,
///  read-only handle to one heap value,
///  counted so the value
///       lives until its last handle is gone (siblings:
///  `Box<T>`,
///  a single owner,
///  and
///       `Arc<T>`,
///  the thread-safe form).
/// Why:
///   Several policies hold the same version and the same bytes at once.
///  Nothing
///       here crosses threads,
///  so `Arc` would add cost for no benefit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Every object reference in TS already behaves like this.
/// ```
use std::rc::Rc;

/// What:
///  Everything one invocation needs to list and read candidates.
///       `Option<ObjectReader>` is "a running reader or nothing";
///  `u64` is an unsigned
///       64-bit counter (siblings `u32`,
///  `usize`).
/// Why:
///   The reader is started only when something needs it,
///  so an invocation that
///       reads no object starts no reader.
///  `u64` cannot run out within a process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateStore { #reader?: ObjectReader; #generation = 0n; #versions: [CandidateSource, CandidateVersion][] = [] }
/// ```
pub struct CandidateStore {
    /// The real Git executable every listing and the reader start.
    real_git: PathBuf,
    /// The caller's arguments before the subcommand,
    ///  replayed so Git selects the same repository.
    global_prefix: Vec<OsString>,
    /// Added environment pairs,
    ///  such as a private index file or object directory.
    overlay: Vec<(OsString, OsString)>,
    /// The long-lived object reader,
    ///  once started.
    reader: Option<ObjectReader>,
    /// Counts invalidations;
    ///  candidates of an earlier generation are refused.
    generation: u64,
    /// Versions already listed in this generation,
    ///  by source.
    versions: Vec<(CandidateSource, Rc<CandidateVersion>)>,
    /// The commit every "against `HEAD`" listing compares with instead of the live `HEAD`:
    ///  `Some(Some(commit))` for a recorded base, `Some(None)` for a recorded unborn branch, `None` for live `HEAD`.
    baseline: Option<Option<ObjectId>>,
}

/// The most commits one `git rev-list` receives.
///  `usize` is the type of counts; 2,048 object names stay far below any argument limit.
const COMMIT_CHUNK_SIZE: usize = 2_048;

/// What:
///  Build the failure of a listing command that could not be started or exited unsuccessfully.
/// Why:
///   The message names the Git operation,
///  so the person can rerun it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listingFailure(failure: CandidateFailure, operation: string, detail: string): CandidateError;
/// ```
fn listing_failure(failure: CandidateFailure, operation: &str, detail: &str) -> CandidateError {
    return CandidateError::new(
        failure,
        format!("cli-git could not list candidate files: git {operation} {detail}").as_str(),
    );
}

/// What:
///  `impl CandidateStore { ... }` attaches the store's operations,
///  like class methods.
/// Why:
///   The store is the only place that starts Git for candidates,
///  which is what
///       keeps the process count bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CandidateStore { version(source) {} bytes(candidate) {} invalidate() {} }
/// ```
impl CandidateStore {
    /// What:
    ///  Create a store without starting any process.
    ///       `&[OsString]` borrows the global arguments;
    ///  `&[(OsString, OsString)]` borrows
    ///       environment pairs;
    ///  `.to_vec()` and `.to_path_buf()` copy them into owned storage.
    /// Why:
    ///   The store outlives the caller's borrowed arguments,
    ///  and commands that
    ///       never ask for candidates pay nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(realGit: string, globalPrefix: string[], overlay: [string, string][])
    /// ```
    pub fn new(
        real_git: &Path,
        global_prefix: &[OsString],
        overlay: &[(OsString, OsString)],
    ) -> CandidateStore {
        return CandidateStore {
            real_git: real_git.to_path_buf(),
            global_prefix: global_prefix.to_vec(),
            overlay: overlay.to_vec(),
            // `None` is the "absent" variant: no reader yet.
            reader: None,
            generation: 0,
            versions: Vec::new(),
            baseline: None,
        };
    }

    /// What:
    ///  Compare every "against `HEAD`" listing with a recorded base instead of the live `HEAD`.
    /// Why:
    ///   A commit transaction reads its candidates against the base it recorded at invocation,
    ///  because another landing may have moved `HEAD` since.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// listChangedIndexPaths({ baseRevision }) // never live HEAD
    /// ```
    #[must_use]
    pub fn with_baseline(mut self, baseline: Option<ObjectId>) -> CandidateStore {
        self.baseline = Some(baseline);
        return self;
    }

    /// What:
    ///  The commit the "against `HEAD`" listings compare with:
    ///  the recorded baseline when one was set, otherwise the commit `HEAD` names now.
    /// Why:
    ///   One place decides the baseline, so the listings and the `HEAD` tree always agree.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const baseline = recorded ?? await headCommit();
    /// ```
    fn baseline_commit(&mut self) -> Result<Option<ObjectId>, CandidateError> {
        if let Some(recorded) = &self.baseline {
            return Ok(recorded.clone());
        }
        return self.reader()?.head_commit();
    }

    /// What:
    ///  The running reader,
    ///  started on first use.
    ///  `&mut ObjectReader` lends it for changing.
    /// Why:
    ///   One reader serves every object read of the invocation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// #reader(): ObjectReader { return (this.#reader ??= startObjectReader(...)); }
    /// ```
    fn reader(&mut self) -> Result<&mut ObjectReader, CandidateError> {
        if self.reader.is_none() {
            // A trailing `?` returns a start failure to our caller, or unwraps the reader.
            let started: ObjectReader = start_object_reader(
                self.real_git.as_path(),
                self.global_prefix.as_slice(),
                self.overlay.as_slice(),
            )?;
            // `Some(...)` is the "present" variant.
            self.reader = Some(started);
        }
        // `.as_mut()` lends the stored reader; `let Some(..) = .. else` unwraps it.
        let Some(reader) = self.reader.as_mut() else {
            // `Err(...)` is the failure variant. The reader was stored just above, so this is not expected.
            return Err(CandidateError::new(
                CandidateFailure::ReaderEnded,
                "cli-git could not read a Git object: the object reader was not started.",
            ));
        };
        // `Ok(...)` is the success variant.
        return Ok(reader);
    }

    /// What:
    ///  Run one listing command and return its standard output.
    ///       `Vec<u8>` is an owned byte list (sibling `String` would require UTF-8).
    /// Why:
    ///   Listings print raw pathname bytes.
    ///  A command that fails lists nothing
    ///       trustworthy,
    ///  so its output is discarded and Git's own message is reported.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async #runListing(operation: string, args: string[]): Promise<Buffer>
    /// ```
    fn run_listing(
        &self,
        operation: &str,
        command: &[OsString],
    ) -> Result<Vec<u8>, CandidateError> {
        // `.clone()` copies the stored prefix; `.extend_from_slice(..)` appends the command after it.
        let mut arguments: Vec<OsString> = self.global_prefix.clone();
        arguments.extend_from_slice(command);
        // `match` on the run's `Result`: `Err(error)` means Git could not be started.
        let output: MetadataOutput = match run_metadata_git(
            self.real_git.as_path(),
            arguments.as_slice(),
            self.overlay.as_slice(),
        ) {
            Ok(captured) => captured,
            Err(error) => {
                return Err(listing_failure(
                    CandidateFailure::GitNotStarted,
                    operation,
                    format!("could not be started: {error}.").as_str(),
                ));
            }
        };
        if !output.success {
            // `String::from_utf8_lossy` decodes Git's message for display, replacing invalid bytes.
            return Err(listing_failure(
                CandidateFailure::GitFailed,
                operation,
                format!(
                    "failed: {}",
                    String::from_utf8_lossy(output.stderr.as_slice()).trim()
                )
                .as_str(),
            ));
        }
        return Ok(output.stdout);
    }

    /// What:
    ///  The name of the empty tree in this repository's hash format.
    /// Why:
    ///   A repository without commits has no `HEAD` to compare the index with;
    ///  the
    ///       empty tree makes every staged entry an addition.
    ///  Git computes the name
    ///       from empty input,
    ///  so SHA-1 and SHA-256 repositories both work.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async #emptyTree(): Promise<ObjectId>
    /// ```
    fn empty_tree(&self) -> Result<ObjectId, CandidateError> {
        // The query runner gives Git no standard input, which is the empty tree's content.
        let output: Vec<u8> = self.run_listing(
            "hash-object",
            &[
                OsString::from("hash-object"),
                OsString::from("-t"),
                OsString::from("tree"),
                OsString::from("--stdin"),
            ],
        )?;
        let Some(tree) = parse_object_id(strip_git_line(output.as_slice())) else {
            return Err(listing_failure(
                CandidateFailure::ListingMalformed,
                "hash-object",
                "did not print the empty tree's object name.",
            ));
        };
        return Ok(tree);
    }

    /// What:
    ///  List the index against a commit,
    ///  or against the empty tree when there is none,
    ///       limited to `pathspecs`.
    ///  `Option<&ObjectId>` is "a borrowed commit name or
    ///       nothing";
    ///  an empty `pathspecs` lists the whole index.
    /// Why:
    ///   `diff-index --cached` reads only the index and the tree,
    ///  never the
    ///       worktree.
    ///  Rename detection is off,
    ///  so a renamed file is one deleted path
    ///       and one added path,
    ///  each judged on its own.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async #listStaged(baseline: ObjectId | undefined, pathspecs: string[]): Promise<Buffer>
    /// ```
    fn list_staged(
        &self,
        baseline: Option<&ObjectId>,
        pathspecs: &[OsString],
    ) -> Result<Vec<u8>, CandidateError> {
        let tree: ObjectId = match baseline {
            Some(commit) => commit.clone(),
            None => self.empty_tree()?,
        };
        // `vec![...]` builds the fixed part of the command; the pathspecs follow `--`.
        let mut command: Vec<OsString> = vec![
            OsString::from("diff-index"),
            OsString::from("--cached"),
            OsString::from("--raw"),
            OsString::from("-z"),
            OsString::from("--no-renames"),
            OsString::from("--no-abbrev"),
            OsString::from(tree.as_str()),
            OsString::from("--"),
        ];
        command.extend_from_slice(pathspecs);
        return self.run_listing("diff-index", command.as_slice());
    }

    /// What:
    ///  List what one commit changed against each of its parents.
    /// Why:
    ///   `-m` lists a merge once per parent and `--root` lists a first commit
    ///       against nothing,
    ///  so every commit shape yields its changed paths.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async #listCommitted(commit: ObjectId): Promise<Buffer>
    /// ```
    fn list_committed(&self, commit: &ObjectId) -> Result<Vec<u8>, CandidateError> {
        return self.run_listing(
            "diff-tree",
            &[
                OsString::from("diff-tree"),
                OsString::from("--root"),
                OsString::from("--no-commit-id"),
                OsString::from("-r"),
                OsString::from("-z"),
                OsString::from("-m"),
                OsString::from("--no-renames"),
                OsString::from("--no-abbrev"),
                OsString::from(commit.as_str()),
            ],
        );
    }

    /// What:
    ///  The version of one source,
    ///  listed once per generation and shared afterwards.
    ///       `Rc::clone(..)` makes another handle to the same version without copying it.
    /// Why:
    ///   Every policy of a pass sees the same paths,
    ///  and asking again costs no process.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async version(source: CandidateSource): Promise<CandidateVersion>
    /// ```
    pub fn version(
        &mut self,
        source: &CandidateSource,
    ) -> Result<Rc<CandidateVersion>, CandidateError> {
        // `for (known, version) in &self.versions` destructures each borrowed pair.
        for (known, version) in &self.versions {
            if known == source {
                return Ok(Rc::clone(version));
            }
        }
        let output: Vec<u8> = match source {
            CandidateSource::StagedAgainstHead => {
                let head: Option<ObjectId> = self.baseline_commit()?;
                // `.as_ref()` turns `&Option<ObjectId>` into `Option<&ObjectId>`; `&[]` is no pathspec.
                self.list_staged(head.as_ref(), &[])?
            }
            CandidateSource::StagedAgainstCommit(commit) => self.list_staged(Some(commit), &[])?,
            CandidateSource::Committed(commit) => self.list_committed(commit)?,
        };
        let records: Vec<CandidateRecord> = parse_raw_records(output.as_slice())?;
        // `Rc::new(..)` moves the built version into shared storage.
        let version: Rc<CandidateVersion> = Rc::new(build_version(self.generation, records));
        self.versions.push((source.clone(), Rc::clone(&version)));
        return Ok(version);
    }

    /// What:
    ///  The index entries `pathspecs` select,
    ///  every conflict stage included,
    ///  with
    ///       repository-relative pathnames.
    /// Why:
    ///   `ls-files --stage` reads only the index,
    ///  so comparing two of these listings
    ///       shows exactly which entries an operation on that index changed.
    ///  `--full-name`
    ///       keeps pathnames repository-relative from any directory.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async stageRecords(pathspecs: string[]): Promise<StageRecord[]>
    /// ```
    pub fn stage_records(
        &mut self,
        pathspecs: &[OsString],
    ) -> Result<Vec<StageRecord>, CandidateError> {
        let mut command: Vec<OsString> = vec![
            OsString::from("ls-files"),
            OsString::from("--stage"),
            OsString::from("-z"),
            OsString::from("--full-name"),
            OsString::from("--"),
        ];
        command.extend_from_slice(pathspecs);
        let output: Vec<u8> = self.run_listing("ls-files", command.as_slice())?;
        return parse_stage_records(output.as_slice());
    }

    /// What:
    ///  The changed-path records of the index against `HEAD`,
    ///  limited to `pathspecs`;
    ///       against the empty tree when no commit exists yet.
    /// Why:
    ///   Which selected paths `HEAD` lacks makes them additions,
    ///  and which removed
    ///       paths it has makes them deletions.
    ///  The limit keeps a conflict elsewhere in the
    ///       index out of the listing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async headRecords(pathspecs: string[]): Promise<CandidateRecord[]>
    /// ```
    pub fn head_records(
        &mut self,
        pathspecs: &[OsString],
    ) -> Result<Vec<CandidateRecord>, CandidateError> {
        let head: Option<ObjectId> = self.baseline_commit()?;
        let output: Vec<u8> = self.list_staged(head.as_ref(), pathspecs)?;
        return parse_raw_records(output.as_slice());
    }

    /// What: Every file of the `HEAD` commit with its mode and object; nothing when the
    ///       branch has no commit yet.
    /// Why:  A policy that compares the candidate state with `HEAD` reads `HEAD`'s copy of
    ///       a file without naming the path in an object request, and learns its mode.
    ///       `--full-tree` lists from the top level whatever directory Git runs in.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async headTreeRecords(): Promise<TreeRecord[]>
    /// ```
    pub fn head_tree_records(&mut self) -> Result<Vec<TreeRecord>, CandidateError> {
        let Some(head) = self.baseline_commit()? else {
            return Ok(Vec::new());
        };
        return self.commit_tree_records(&head);
    }

    /// What: Every file of one commit with its mode and object.
    /// Why:  Policies that compare candidates with the parents of a commit read each parent's files
    ///       the way they read `HEAD`'s. `--full-tree` lists from the top level whatever directory
    ///       Git runs in.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async commitTreeRecords(commit: ObjectId): Promise<TreeRecord[]>
    /// ```
    pub fn commit_tree_records(
        &mut self,
        commit: &ObjectId,
    ) -> Result<Vec<TreeRecord>, CandidateError> {
        let output: Vec<u8> = self.run_listing(
            "ls-tree",
            &[
                OsString::from("ls-tree"),
                OsString::from("-r"),
                OsString::from("-z"),
                OsString::from("--full-tree"),
                OsString::from(commit.as_str()),
            ],
        )?;
        return parse_tree_records(output.as_slice());
    }

    /// What: The commit the "against `HEAD`" listings compare with: the recorded base or the commit
    ///       `HEAD` names now; nothing when the branch has no commit yet.
    /// Why:  A policy that skips content the baseline already holds names the commit in its log
    ///       record and lists its files through `commit_tree_records`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async baselineReference(): Promise<ObjectId | undefined>
    /// ```
    pub fn baseline_reference(&mut self) -> Result<Option<ObjectId>, CandidateError> {
        return self.baseline_commit();
    }

    /// What: The parents of these commits that are not among the commits themselves, each once, in
    ///       the order of the commits and then of their parents.
    /// Why:  A pushed commit's parent that is also pushed is new to the remote, so only the parents
    ///       outside the given commits count as already published. One `rev-list` answers 2,048
    ///       commits, so the process count does not grow with the commits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async parentsOutside(commits: ObjectId[]): Promise<ObjectId[]>
    /// ```
    pub fn parents_outside(
        &mut self,
        commits: &[ObjectId],
    ) -> Result<Vec<ObjectId>, CandidateError> {
        // Membership tests stay constant-time however many commits a push carries.
        let given: HashSet<&ObjectId> = commits.iter().collect();
        let mut seen: HashSet<ObjectId> = HashSet::new();
        let mut outside: Vec<ObjectId> = Vec::new();
        for chunk in commits.chunks(COMMIT_CHUNK_SIZE) {
            let mut command: Vec<OsString> = vec![
                OsString::from("rev-list"),
                OsString::from("--no-walk=unsorted"),
                OsString::from("--parents"),
            ];
            for commit in chunk {
                command.push(OsString::from(commit.as_str()));
            }
            let output: Vec<u8> = self.run_listing("rev-list", command.as_slice())?;
            for line in output.split(|byte| { return *byte == b'\n'; }) {
                // The first name of a line is the commit itself; the rest are its parents.
                for name in line.split(|byte| { return *byte == b' '; }).skip(1) {
                    let Some(parent) = parse_object_id(name) else {
                        return Err(listing_failure(
                            CandidateFailure::ListingMalformed,
                            "rev-list",
                            "printed a parent that is not an object name.",
                        ));
                    };
                    if !given.contains(&parent) && seen.insert(parent.clone()) {
                        outside.push(parent);
                    }
                }
            }
        }
        return Ok(outside);
    }

    /// What: A version of the current generation built from records the caller derived.
    ///       `Vec<CandidateRecord>` is taken by value: the records move into the version.
    /// Why:  A predicted staging operation has no single listing command; its candidates
    ///       come from comparing index states. The version is not remembered by source,
    ///       because no source names it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// derivedVersion(records: CandidateRecord[]): CandidateVersion
    /// ```
    pub fn derived_version(&mut self, records: Vec<CandidateRecord>) -> Rc<CandidateVersion> {
        return Rc::new(build_version(self.generation, records));
    }

    /// What:
    ///  The records of one commit's own delta, each against every parent (or the empty tree for a root commit).
    /// Why:
    ///   A pushed commit's changes are the content a manual-push scan reads; the listing is the one the
    ///  landed-delta source already uses.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// committedRecords(commit: ObjectId): CandidateRecord[]
    /// ```
    pub fn committed_records(
        &mut self,
        commit: &ObjectId,
    ) -> Result<Vec<CandidateRecord>, CandidateError> {
        let output: Vec<u8> = self.list_committed(commit)?;
        return parse_raw_records(output.as_slice());
    }

    /// What:
    ///  Every blob entry of one tree, as records that add each path.
    /// Why:
    ///   A push with no known published base scans the pushed tip's whole tree.
    ///   Submodule entries are not listed: their content is a commit name, which no content policy reads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// treeRecords(tree: ObjectId): CandidateRecord[] // ls-tree -r, every blob as added
    /// ```
    pub fn tree_records(
        &mut self,
        tree: &ObjectId,
    ) -> Result<Vec<CandidateRecord>, CandidateError> {
        let listing: Vec<u8> = self.run_listing(
            "ls-tree",
            &[
                OsString::from("ls-tree"),
                OsString::from("--full-tree"),
                OsString::from("-r"),
                OsString::from("-z"),
                OsString::from(tree.as_str()),
            ],
        )?;
        let raw: Vec<u8> = added_raw_from_ls_tree(listing.as_slice())?;
        return parse_raw_records(raw.as_slice());
    }

    /// What:
    ///  The exact bytes a candidate holds in its version.
    ///       `Rc<[u8]>` is a shared read-only byte list.
    /// Why:
    ///   Bytes come from the object the listing named,
    ///  so an edit to the worktree
    ///       file after staging is invisible here.
    ///  A deleted path has no bytes.
    ///  A
    ///       gitlink has no blob in this repository,
    ///  so its bytes are the submodule
    ///       commit's name,
    ///  as in the TypeScript wrapper.
    ///  A candidate of a retired
    ///       version is refused instead of answered with bytes that may be outdated.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async bytes(candidate: Candidate): Promise<Uint8Array>
    /// ```
    pub fn bytes(&mut self, candidate: &Candidate) -> Result<Rc<[u8]>, CandidateError> {
        if candidate.identity.generation != self.generation {
            return Err(CandidateError::new(
                CandidateFailure::StaleCandidate,
                "cli-git refused to read a candidate from a version that was invalidated by a fix or replay; \
                 list the version again and use its candidates.",
            ));
        }
        let Some(object) = candidate.object.as_ref() else {
            // `Rc::from(Vec::new())` is a shared empty byte list.
            return Ok(Rc::from(Vec::new()));
        };
        if candidate.mode == CandidateMode::Gitlink {
            // `.as_bytes()` views the name's text as bytes; `Rc::from(..)` copies them into shared storage.
            return Ok(Rc::from(object.as_str().as_bytes()));
        }
        return self.reader()?.blob(object);
    }

    /// What:
    ///  Retire every version listed so far.
    ///  `+= 1` advances the generation.
    /// Why:
    ///   A fix or replay rewrites index entries,
    ///  so earlier listings no longer
    ///       describe what would be committed.
    ///  Blob bytes stay remembered in the
    ///       reader:
    ///  an object name always denotes the same bytes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// invalidate(): void { this.#versions = []; this.#generation += 1n; }
    /// ```
    pub fn invalidate(&mut self) {
        self.versions.clear();
        self.generation += 1;
    }
}

/// Repository controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_store_tests.rs"]
mod tests;

/// Committed versions,
///  invalidation and listing failures.
#[cfg(test)]
#[path = "candidate_store_lifecycle_tests.rs"]
mod lifecycle_tests;

/// The process-count bound and its per-file positive control.
#[cfg(test)]
#[path = "candidate_process_count_tests.rs"]
mod process_count_tests;

/// Unborn,
///  SHA-256 and private-index listings.
#[cfg(test)]
#[path = "candidate_store_baseline_tests.rs"]
mod baseline_tests;

/// Listings that must be refused.
#[cfg(test)]
#[path = "candidate_store_failure_tests.rs"]
mod failure_tests;

/// What:
///  Turn NUL-separated `ls-tree -r -z` records into raw diff records that add each blob.
/// Why:
///   The record parser is the one the diff listings use, so tree entries follow the same path and mode rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addedRawFromLsTree(output: Uint8Array): Uint8Array;
/// ```
fn added_raw_from_ls_tree(output: &[u8]) -> Result<Vec<u8>, CandidateError> {
    let mut raw: Vec<u8> = Vec::new();
    for record in output
        .split(|byte| {
            return *byte == 0;
        })
        .filter(|record| {
            return !record.is_empty();
        })
    {
        let Some(tab) = record.iter().position(|byte| { return *byte == b'\t'; }) else {
            return Err(CandidateError::new(
                super::candidate_error::CandidateFailure::ListingMalformed,
                "cli-git could not list the pushed tree: an entry lacks its path separator.",
            ));
        };
        // The mode, type and object name are ASCII; the path stays the exact bytes Git printed.
        let metadata = String::from_utf8_lossy(&record[..tab]);
        let path: &[u8] = &record[tab + 1..];
        let fields: Vec<&str> = metadata.split(' ').collect();
        if fields.len() != 3 {
            return Err(CandidateError::new(
                super::candidate_error::CandidateFailure::ListingMalformed,
                "cli-git could not list the pushed tree: an entry lacks its mode, type or object name.",
            ));
        }
        if fields[1] != "blob" {
            continue;
        }
        let zeros = "0".repeat(fields[2].len());
        raw.extend_from_slice(format!(":000000 {} {zeros} {} A", fields[0], fields[2]).as_bytes());
        raw.push(0);
        raw.extend_from_slice(path);
        raw.push(0);
    }
    return Ok(raw);
}
/// The `HEAD` tree listing.
#[cfg(test)]
#[path = "candidate_store_tree_tests.rs"]
mod tree_tests;
