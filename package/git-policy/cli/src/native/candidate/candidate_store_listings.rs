//! What:
//!  The listings a caller asks one candidate store for,
//!  and the Git commands behind them:
//!  the version of one source,
//!  the index entries a pathspec selects,
//!  the changed paths of the index against `HEAD`,
//!  and the `HEAD` records,
//!  the staged entries and the empty tree those three compare with.
//! Why:
//!  Every one of them is one run of the store's own listing command followed by
//!  one parse,
//!  so they live apart from the store that runs it
//!  (`crate::candidate_store`) and each file says one thing;
//!  the store keeps the generations these listings belong to,
//!  because a fix or replay retires them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const version = await store.version({ kind: 'staged-against-head' });
//! const records = await store.stageRecords(['--', 'src/a.rs']);
//! ```

/// Import the layer's failure type and its closed list of causes.
use crate::candidate_error::{
    CandidateError,
    CandidateFailure,
};
/// The validated object name and its parser.
use crate::candidate_object::{
    ObjectId,
    parse_object_id,
};
/// Pure parsing of raw NUL-delimited changed-path records.
use crate::candidate_record::{
    CandidateRecord,
    parse_raw_records,
};
/// Pure parsing of `git ls-files --stage` records.
use crate::candidate_stage::{
    StageRecord,
    parse_stage_records,
};
/// The store these listings belong to,
/// and the failure of a listing command it runs.
use crate::candidate_store::{
    CandidateStore,
    listing_failure,
};
/// The immutable version these listings answer with,
/// its source and its builder.
use crate::candidate_version::{
    CandidateSource,
    CandidateVersion,
    build_version,
};
/// The captured-query runner's line helper,
/// which trims the newline Git prints.
use crate::git_metadata::strip_git_line;
/// `OsString` is owned operating-system text of raw OS bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `Rc<T>` is a shared,
///  read-only handle to one heap value,
///  counted so the value lives until its last handle is gone (siblings:
///  `Box<T>`,
///  a single owner,
///  and
///       `Arc<T>`,
///  the thread-safe form).
/// Why:
///   Every policy of a pass holds the same version,
///  and nothing here crosses threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Every object reference in TS already behaves like this.
/// ```
use std::rc::Rc;

impl CandidateStore {
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
    pub(crate) fn list_committed(
        &self,
        commit: &ObjectId,
    ) -> Result<Vec<u8>, CandidateError> {
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
}
