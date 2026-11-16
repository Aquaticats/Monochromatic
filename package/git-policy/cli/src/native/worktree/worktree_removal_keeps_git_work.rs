//! What:
//!  The Git work a removal would leave named by nothing:
//!  a commit only the worktree's own state names.
//! Why:
//!   A removal deletes the worktree's own Git directory,
//!  and with it everything that can name a commit there:
//!  a detached `HEAD`,
//!  the references Git keeps per worktree
//!  (`refs/worktree/`,
//!  `refs/bisect/`,
//!  `refs/rewritten/`),
//!  the logs under `logs` (the `HEAD` reflog,
//!  which names the commit after every amend,
//!  reset and rebase made in the worktree),
//!  and the records of an operation that stopped:
//!  `ORIG_HEAD`, `MERGE_HEAD`, `CHERRY_PICK_HEAD`, `REVERT_HEAD`, `REBASE_HEAD`, `MERGE_AUTOSTASH`,
//!  `BISECT_LOG`, `BISECT_EXPECTED_REV`,
//!  and `rebase-merge/`, `rebase-apply/` and `sequencer/`, whose `onto`, `orig-head`, `stopped-sha`,
//!  `git-rebase-todo`, `done`, `rewritten-list`, `autostash`, `dropped`, `rewritten`, `head`, `todo` and
//!  `abort-safety` name the commits of the operation.
//!  A commit no name that survives the removal reaches is one `git gc --prune=now` takes,
//!  so the removal would destroy it,
//!  and the owner's rule is that a removal never destroys Git work:
//!  "a commit that no reference holds" is never destroyed by a removal
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "How far the protection of a worktree being removed reaches", 2026-11-12T13:04:14-05:00).
//!  A commit an ancestor of a surviving name holds is not stranded:
//!  nothing that names it dies with the worktree.
//!  A surviving name is a reference of the common directory,
//!  and a log under the common `logs` directory,
//!  which a removal leaves alone;
//!  the reflog of a branch therefore keeps the commits an amend or a rebase took off it,
//!  and a detached `HEAD` in a linked worktree is the state where nothing else names them.
//!  The same question is asked of every worktree this repository registers inside the one being removed,
//!  because a removal takes those too
//!  (a Sonnet audit of 0.6.1-alpha.14,
//!  finding F2).
//!  Nothing here keeps a commit and nothing here removes one:
//!  the caller keeps the worktree and says what stands in the way,
//!  and the person names the commit with a branch of their own before removing it
//!  (a review of `e615fb599b`, finding 1, for the reflog this was missing).
//!
//! ## Refusing the removal, or keeping the commit on a name of this program's own
//!
//!  A keep is the other way to lose nothing:
//!  before the worktree's own Git directory goes,
//!  every commit named only there is given a reference under `refs/cli-git/kept/`
//!  (`worktree_removal_held_commits.rs` does exactly that for a commit the commit service holds).
//!  This program refuses instead, for the reasons the record gives.
//!  A keep where a person is at the keyboard has no owner:
//!  every reference written that way stays in the repository until someone deletes it,
//!  and nothing in this program's records says who finds `refs/cli-git/kept/reflog/<id>/<n>` again,
//!  or when it is cleared,
//!  and a keep nobody can find is a loss while one that never ends is a leak.
//!  The keeps this program does write are for state no person can be asked about:
//!  the commit the commit service holds, whose batch no command of a live worktree settles once the worktree is gone.
//!  Where a person is present,
//!  every guard of a removal answers the same way instead:
//!  it stops, names the commit, and prints the one command that names it for good
//!  (`SPEC.md`,
//!  "a command that could destroy a person's data is refused, and the person who means it runs it with the guard's opt-out").
//!  Measured on this repository's own worktrees, 2026-11-14:
//!  179 linked worktrees, of which 4 hold a commit that only a per-worktree name holds,
//!  all 4 the same fetched commit named by `FETCH_HEAD`
//!  (`keep/measurement-worktrees-holding-a-reflog-only-commit.log` of the run that answered the review),
//!  and none held a commit named only by a reflog.
//!  So the refusal this adds fires on no worktree of this repository today,
//!  and a keep would have fired on four.
//!  `FETCH_HEAD` is deliberately not read:
//!  it names an object Git fetched from somewhere else,
//!  which is still there where it came from and comes back with the same fetch,
//!  and it is no commit a person made;
//!  keeping four worktrees of other agents' standing over such an object
//!  would cost more than the object is worth.
//!  What a keep would change:
//!  the four worktrees above would go,
//!  each removal would write references nobody owns into the common directory,
//!  and one more namespace,
//!  one more record and one more clearing path would need their own design and review.
//!
//! ## An object name is compared as Git compares it
//!
//!  Git prints every object name it prints in small hexadecimal letters,
//!  and it reads a name in either case as the same object:
//!  a record may hold `HEAD` in capitals, so every name read from a file is kept as Git prints names
//!  (`as_git_prints_an_object_name`) and every comparison of one with a printed name is Git's comparison
//!  (`the_same_object_name`).
//!  A text comparison dropped a commit whose detached `HEAD` was written in capitals, the removal went ahead, and a
//!  collection took it
//!  (a review of `8858f843ee`, finding 3).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stranded = gitWorkARemovalWouldStrand(context, commonDir, taken); // string[] reasons, or the reason nothing was proven
//! ```

use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_ladder_target::checkout_environment;
use crate::worktree_removal_worktrees_it_takes::Taken;
use crate::worktree_registration_proof::read_record_here;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The start of every name Git keeps in one worktree rather than in the repository.
/// Why:
///   Git's own list (`REFS_PER_WORKTREE` in `refs.c`) is these three,
///  and a removal deletes all of them with the worktree's own Git directory.
const PER_WORKTREE_PREFIXES: [&str; 3] = ["refs/worktree/", "refs/bisect/", "refs/rewritten/"];

/// How many digits of a commit a sentence shows.
/// Why:
///   A person matches a commit by the prefix Git prints beside it,
///  and twelve digits are that prefix with room to be unambiguous in a repository of any size this program meets.
const SHOWN_DIGITS: usize = 12;

/// How many stranded objects one answer names before it counts the rest.
const SHOWN_CONTAINERS: usize = 3;

/// How many digits a SHA-1 object name has,
/// which is one length a `HEAD` record can hold.
const SHA1_DIGITS: usize = 40;

/// How many digits a SHA-256 object name has.
const SHA256_DIGITS: usize = 64;

/// How many bytes of one record or log this program reads before it treats the file as one it cannot read.
/// Why:
///   A reflog of a busy worktree is hundreds of kilobytes and a `git-rebase-todo` a few,
///  so this bound holds every record Git writes
///  while keeping a runaway file from being read into memory
///  (`SPEC.md`,
///  "Every wait has a bound").
const NAME_RECORD_LIMIT: u64 = 8 << 20;

/// The records a worktree's own Git directory keeps directly that name a commit,
/// and the list of the ones it does not, each with its reason.
/// Why:
///   A removal deletes the worktree's own Git directory,
///  so every object name Git wrote there is a name the removal takes away.
///  The list is taken from Git's own documentation of the repository layout and of the commands that write these
///  records, not from memory:
///  `gitrepository-layout(5)` for `HEAD`, `index`, `logs`, `refs`, `packed-refs`, `commondir`, `gitdir`, `config`,
///  `hooks`, `info`, `shallow` and `worktrees/<id>`,
///  `git-fetch(1)` for `FETCH_HEAD` ("the names of refs that are fetched, together with the object names they point at,
///  are written to `.git/FETCH_HEAD`"),
///  `git-merge(1)` for `MERGE_HEAD`, `MERGE_AUTOSTASH` and `AUTO_MERGE`, and `git-rebase(1)` for `ORIG_HEAD` and
///  `REBASE_HEAD`;
///  `rebase-merge/`, `rebase-apply/` and `sequencer/` are the state of an operation that stopped, and the files named
///  in `NAMING_RECORD_FILES` are the ones that hold the commits of that operation.
///  In (read for an object name):
///  `HEAD`, `logs` (the whole tree), `refs` (through the reference query), `packed-refs` (the same query),
///  `ORIG_HEAD`, `MERGE_HEAD`, `CHERRY_PICK_HEAD`, `REVERT_HEAD`, `REBASE_HEAD`, `MERGE_AUTOSTASH`,
///  `BISECT_LOG`, `BISECT_EXPECTED_REV`, `BISECT_START`,
///  `FETCH_HEAD`,
///  and the operation records listed below.
///  Out, with the reason:
///  `index` names the blobs of what is staged; a clean worktree's removal needs none of them, and a worktree whose
///  staged bytes would be lost is not clean, so Git refuses it before the question is asked;
///  `shallow` names commits their own refs also reach;
///  `AUTO_MERGE` names a tree, and the commits it was made from are named by `MERGE_HEAD`;
///  `MERGE_MSG`, `SQUASH_MSG`, `COMMIT_EDITMSG` and `TAG_EDITMSG` hold messages a person wrote, and a name read out of
///  one would keep a worktree for a commit nothing names;
///  `commondir`, `gitdir`, `config`, `config.worktree`, `hooks`, `info`, `locked` and `objects` hold no object name.
///  `FETCH_HEAD` is in by the project's rule, not by the reading this module had first:
///  a commit made here and fetched by its object name is named by nothing else, so a refusal that left `FETCH_HEAD` out
///  let the collection take it
///  (a review of `dfd6fab13e`, finding 3).
///  A plain `git fetch` and a `git pull` leave refs naming what they fetched, so the refusal does not reach them;
///  a fetch of a bare URL or of a branch into nothing leaves commits only `FETCH_HEAD` names, and that refusal is the
///  one a person meets (measured in the run that answered that review).
const NAMING_RECORDS: [&str; 10] = [
    "ORIG_HEAD",
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "REBASE_HEAD",
    "MERGE_AUTOSTASH",
    "BISECT_LOG",
    "BISECT_EXPECTED_REV",
    "BISECT_START",
    "FETCH_HEAD",
];

/// The records inside the directories an operation in progress keeps that name a commit,
/// with the directory each stands in.
/// Why:
///   A rebase, an `am`, a cherry-pick of several commits and a bisect keep their place in a directory,
///  and the files that say which commits are involved are these;
///  the messages and patch bodies beside them are text a person wrote,
///  and a name read out of those would keep a worktree for a commit that nothing names.
const NAMING_RECORD_FILES: [(&str, &str); 16] = [
    ("rebase-merge", "onto"),
    ("rebase-merge", "orig-head"),
    ("rebase-merge", "stopped-sha"),
    ("rebase-merge", "git-rebase-todo"),
    ("rebase-merge", "git-rebase-todo.backup"),
    ("rebase-merge", "done"),
    ("rebase-merge", "dropped"),
    ("rebase-merge", "rewritten-list"),
    ("rebase-merge", "autostash"),
    ("rebase-apply", "onto"),
    ("rebase-apply", "orig-head"),
    ("rebase-apply", "rewritten"),
    ("rebase-apply", "autostash"),
    ("sequencer", "head"),
    ("sequencer", "todo"),
    ("sequencer", "abort-safety"),
];

/// What a name of a worktree being removed reaches.
/// Why:
///   A reference in a worktree can name a commit,
///  a tag of a commit,
///  or an object that is no commit at all,
///  and every one of those dies with the worktree:
///  a tag it named is peeled by Git, and a tree or a blob it named is as nameless after the removal as a commit would be.
///  Reachability answers the question for a commit alone,
///  so an object that is no commit is kept as it stands rather than dropped
///  (a review of `5a2bd4bb6c`, finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reached = { kind: 'commit' } | { kind: 'other'; objectType: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
enum Reached {
    /// A commit,
    /// which survival is a question of reachability for.
    Commit,
    /// A tree, a blob or a tag this program could not see through,
    /// with the type Git gave it.
    Other(String),
}

/// One object the worktree's own state names and no surviving name does.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Stranded {
    /// The object,
    /// as its full name.
    object: String,
    /// What it is,
    /// which decides the sentence and whether survival is asked at all.
    reached: Reached,
    /// What names it here,
    /// as the person reads it.
    holder: String,
}

/// What:
///  Why a removal of `worktree` would leave Git work named by nothing,
///  as the sentences a person reads,
///  or the reason that could not be proven.
/// Why:
///   The one question a removal asks before it deletes a worktree's own Git directory,
///  and the same answer serves the sweep,
///  the service's removal and the person's own command.
///  The worktrees the removal takes are asked,
///  all of them,
///  from the one list the caller made (`worktree_removal_worktrees_it_takes`):
///  a question that walked its own list would keep or miss a worktree the other questions do not know about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reasons = gitWorkARemovalWouldStrand(context, commonDir, worktree, admin);
/// ```
///
/// # Errors
///
/// The words for a record,
/// a reference or a Git answer this cannot read,
/// which leaves the worktree kept rather than read as clear.
pub(crate) fn git_work_a_removal_would_strand(context: &GitContext, common_dir: &Path, places: &[Taken]) -> Result<Vec<String>, String> {
    let mut said: Vec<String> = Vec::new();
    let mut counted: usize = 0;
    for one in places {
        let (root, own) = (one.root.as_path(), one.admin.as_path());
        let stranded: Vec<Stranded> = stranded_objects(context, common_dir, root, own)?;
        counted += stranded.len();
        for held in stranded {
            if said.len() < SHOWN_CONTAINERS {
                said.push(sentence(root, &held));
            }
        }
    }
    if counted > said.len() {
        said.push(format!("and {} more objects that only a worktree being removed names", counted - said.len()));
    }
    return Ok(said);
}

/// What:
///  The commits of one worktree that only its own state names.
/// Why:
///   A `HEAD` that names a branch survives the removal,
///  and one that names a commit does not;
///  a reference Git keeps in the worktree dies with it,
///  and so does every log and record under the worktree's own Git directory.
///  A commit any name that survives the removal reaches is not stranded however many dying names hold it.
///  The worktree's own reference namespaces are read only when the Git directory holds any reference at all,
///  and the logs only when the directory has them,
///  so a worktree that has neither costs two file looks and no Git start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stranded = strandedCommits(context, common, root, admin);
/// ```
fn stranded_objects(context: &GitContext, common_dir: &Path, root: &Path, admin: &Path) -> Result<Vec<Stranded>, String> {
    let mut holders: Vec<Stranded> = Vec::new();
    // The detached `HEAD` first, so the sentence about the object it names is the one a person reads for it.
    if let Some(commit) = detached_head(admin)? {
        holders.push(Stranded { object: commit, reached: Reached::Commit, holder: String::from("its detached HEAD") });
    }
    // Each reference's own object, of whatever kind it holds: the tag object itself is the name a person sees,
    // and the walk from it reaches what it tags.
    let references: Vec<(String, String)> = per_worktree_references(context, root, admin)?;
    let objects: Vec<String> = references.iter().map(|(_, object)| return object.clone()).collect();
    for (object, reached) in objects_among(context, common_dir, root, objects.as_slice())? {
        let holder: String = match references.iter().find(|(_, named)| return *named == as_git_prints_an_object_name(object.as_str())) {
            Some((name, _)) => format!("the reference `{name}`"),
            None => String::from("a reference of this worktree"),
        };
        holders.push(Stranded { object, reached, holder });
    }
    // Every object a log or a record of the worktree names, of whatever kind it is: a reflog line can name a tag object,
    // and the walk from it reaches the commit.
    let dying: Vec<(String, String)> = names_that_die_with_the_worktree(admin)?;
    let kinds: Vec<(String, Reached)> = objects_among(context, common_dir, root, dying.iter().map(|(object, _)| return object.clone()).collect::<Vec<String>>().as_slice())?;
    for (object, reached) in kinds {
        let holder: String = match dying.iter().find(|(named, _)| return *named == as_git_prints_an_object_name(object.as_str())) {
            Some((_, holder)) => holder.clone(),
            None => String::from("a record of this worktree"),
        };
        holders.push(Stranded { object, reached, holder });
    }
    // One holder per object, the first name it has here.
    let mut named: Vec<Stranded> = Vec::new();
    for held in holders {
        if named.iter().any(|one| return the_same_object_name(one.object.as_str(), held.object.as_str())) {
            continue;
        }
        named.push(held);
    }
    let dying_names: Vec<String> = named.iter().map(|held| return held.object.clone()).collect();
    let stranded: Vec<String> = match dying_names.is_empty() {
        true => Vec::new(),
        false => objects_only_the_dying_names_reach(context, common_dir, root, dying_names.as_slice())?,
    };
    let answer: Vec<Stranded> = named
        .into_iter()
        .filter(|held| return stranded.iter().any(|one| return the_same_object_name(one.as_str(), held.object.as_str())))
        .collect();
    // One sentence per stranded name is enough: every other object the answer holds is reached from one of these
    // (a commit implies the trees and blobs below it, a tag implies what it tags), so naming the name names it too.
    return Ok(answer);
}

/// What:
///  The commit a worktree's `HEAD` names when it names a commit,
///  nothing when it names a branch,
///  and the reason nothing may be removed when it names neither.
/// Why:
///   A detached `HEAD` is the one record that says which commit the worktree is on,
///  and the registration reader already opens that record without waiting for a writer.
///  Two states are read: a line that names a reference, which outlives the removal,
///  and a full object name, which is a commit only this directory names.
///  Anything else is a record a killed writer can leave torn,
///  and it may name a commit this program cannot see:
///  answering "not detached" there would report that commit unstranded and delete the only thing that names it,
///  so an unreadable record is the reason the removal keeps the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// detachedHead(admin) // '0f1af2e...' | undefined
/// ```
fn detached_head(admin: &Path) -> Result<Option<String>, String> {
    let bytes: Vec<u8> = read_record_here(admin.join("HEAD").as_path(), "the record that names the branch of a worktree being removed")
        .map_err(|refusal| return refusal.into_words())?;
    let text: String = String::from_utf8(bytes).map_err(|_| return String::from("a HEAD this program cannot read may name a commit of its own"))?;
    let line: &str = text.trim();
    if line.starts_with("ref: ") {
        return Ok(None);
    }
    let digits: usize = line.len();
    if (digits == SHA1_DIGITS || digits == SHA256_DIGITS) && line.bytes().all(|byte| return byte.is_ascii_hexdigit()) {
        // A record may hold the name in capital letters, and Git reads it as the same object: the name is kept as Git
        // prints names, so every comparison below is Git's comparison.
        return Ok(Some(as_git_prints_an_object_name(line)));
    }
    return Err(format!(
        "the record HEAD at {} names neither a branch nor a commit, so nothing proves which commit it holds and a removal could delete the only name of one",
        admin.display()
    ));
}

/// What:
///  The references Git keeps in one worktree,
///  with the object each of them reaches.
/// Why:
///   Those references live in the worktree's own Git directory and the removal deletes them,
///  so an object they alone name dies with it,
///  and a person's `refs/worktree/person-keep` is the ordinary way to be in that state.
///  A reference can name a tag rather than the object a person made the tag for,
///  and the field `%(*objectname)` is the object the reference reaches:
///  Git peels the tag for it, however many tag objects stand in between
///  (measured: `keep/probe-git-peeling.log` of the run that answered the review of `5a2bd4bb6c`).
///  Reading `%(objectname)` alone named a tag object, which is no commit and was dropped, so the commit the tag held
///  went unasked
///  (that review, finding 1).
///  A Git directory holding no reference has none,
///  and asking Git there would cost a process for nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// perWorktreeReferences(context, root, admin) // [{ name, reached }]
/// ```
fn per_worktree_references(context: &GitContext, root: &Path, admin: &Path) -> Result<Vec<(String, String)>, String> {
    if !admin.join("refs").is_dir() && !admin.join("packed-refs").exists() {
        return Ok(Vec::new());
    }
    let arguments: Vec<OsString> = [
        OsString::from("for-each-ref"),
        // The reference's own object, of whatever kind it is: what the question below asks about is the object the name
        // itself holds, and a tag's target is reached by the walk from that tag.
        OsString::from("--format=%(refname)%09%(objectname)"),
        // Git's own list of the namespaces it keeps per worktree.
        OsString::from("refs/worktree"),
        OsString::from("refs/bisect"),
        OsString::from("refs/rewritten"),
    ]
    .to_vec();
    let request: GitRequest = GitRequest {
        cwd: root.to_path_buf(),
        arguments,
        without_prefix: true,
        environment: checkout_environment(admin, root),
        ..GitRequest::default()
    };
    let output: GitOutput = run_git(context, &request).map_err(|error| format!("the references this worktree keeps cannot be listed ({error})"))?;
    if output.code != Some(0) {
        return Err(format!(
            "the references this worktree keeps cannot be listed ({})",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    let mut found: Vec<(String, String)> = Vec::new();
    for line in String::from_utf8_lossy(output.stdout.as_slice()).lines() {
        let Some((name, object)) = line.split_once('\t') else { continue };
        if object.is_empty() {
            continue;
        }
        found.push((name.to_owned(), object.to_owned()));
    }
    return Ok(found);
}

/// What:
///  The context of the questions below,
///  naming the repository they run in.
/// Why:
///   A request that carries its input on standard input is fed from a private file under the repository's own scratch
///  directory,
///  and the context a typed removal is asked with knows the repository's location but not its common directory yet:
///  a request without it ends in a defect rather than an answer, which a guard would read as a refusal of every worktree
///  it was asked about
///  (measured by the binary controls of `e615fb599b`'s review run).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withTheRepository(context, commonDir): GitContext;
/// ```
fn with_the_repository(context: &GitContext, common_dir: &Path) -> GitContext {
    let mut owned: GitContext = context.clone();
    owned.common_dir = Some(common_dir.to_path_buf());
    return owned;
}

/// What:
///  The objects, of those the dying names reach, that no name surviving the removal reaches,
///  of every kind: a commit, a tag object, a tree, a blob.
/// Why:
///  Git's own collection keeps an object reachable from any reference or log it reads,
///  and from every ancestor of what those name,
///  so the question is reachability and not the bytes of a record:
///  a commit an amend took off a branch is still named by that branch's reflog,
///  and one a person reset away in a detached worktree is named by nothing.
///  `--objects` is what makes this a question of objects and not only of commits:
///  a tip of any kind is a valid revision to Git's walk (`rev-list --objects <tree>` prints the tree and what it holds,
///  `<annotated tag>` prints the tag object and then what it tags),
///  so a tag object and the commit it tags are both in the answer,
///  where a walk without `--objects` prints commits alone and dropped the tag
///  (a review of `177112aac1`, findings 5 and 6; measured in `keep/probe-object-listing.log`).
///  One `rev-list` answers all of them at once,
///  asked from the dying names to the names that survive and never the other way round:
///  the walk it then does is bounded by the dying names,
///  while the surviving names are tips the answer excludes.
///  The surviving logs are read only when the surviving references leave something unanswered:
///  an ordinary removal of a finished worktree is answered by the references alone,
///  so the reading of the common `logs` tree is skipped for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function objectsOnlyTheDyingNamesReach(context, commonDir, root, dying): string[]; // throws the reason
/// ```
fn objects_only_the_dying_names_reach(context: &GitContext, common_dir: &Path, root: &Path, dying: &[String]) -> Result<Vec<String>, String> {
    // The requests below carry their input on standard input, which this program feeds from a private file under the
    // repository's own scratch directory, so the context has to name the repository it was started for.
    let owned: GitContext = with_the_repository(context, common_dir);
    let surviving_references: Vec<String> = surviving_reference_names(&owned, root)?;
    let first: Vec<String> = objects_no_named_revision_reaches(&owned, root, dying, surviving_references.clone())?;
    if first.is_empty() {
        return Ok(Vec::new());
    }
    // The second stage asks about what the first left, and keeps both exclusions: a walk from an object lists the
    // commits that object was built on as well, so a second stage carrying the logs alone would bring back a commit
    // that a surviving reference had already excluded. The surviving logs are read only here, which is the saving.
    let mut surviving: Vec<String> = surviving_references;
    surviving.extend(surviving_log_names(common_dir)?);
    return objects_no_named_revision_reaches(&owned, root, first.as_slice(), surviving);
}

/// What:
///  The objects the tips in `dying` reach that no tip in `surviving` reaches,
///  and so no ancestor of one either.
/// Why:
///  One `rev-list --objects` with the dying names as tips and the surviving names as exclusions answers the reachability
///  question for all of them at once, and its output is bounded by the dying names rather than by the repository's
///  history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function objectsNoNamedRevisionReaches(context, root, dying, surviving): string[]; // throws the reason
/// ```
fn objects_no_named_revision_reaches(context: &GitContext, root: &Path, dying: &[String], surviving: Vec<String>) -> Result<Vec<String>, String> {
    let mut input: String = String::new();
    for object in dying {
        input.push_str(object);
        input.push('\n');
    }
    for name in surviving {
        input.push('^');
        input.push_str(name.as_str());
        input.push('\n');
    }
    let arguments: Vec<OsString> = [OsString::from("rev-list"), OsString::from("--objects"), OsString::from("--no-object-names"), OsString::from("--stdin")].to_vec();
    let request: GitRequest = GitRequest {
        cwd: root.to_path_buf(),
        arguments,
        without_prefix: true,
        input: Some(input.into_bytes()),
        ..GitRequest::default()
    };
    let output: GitOutput = run_git(context, &request).map_err(|error| return format!("nothing proves which objects no surviving name reaches ({error})"))?;
    if output.code != Some(0) {
        return Err(format!(
            "nothing proves which objects no surviving name reaches ({})",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    // Git prints its own answer, and these are the names a name read from a file is compared with: the comparison is
    // Git's, whatever case each was written in.
    let mut objects: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(output.stdout.as_slice()).lines() {
        let object: String = as_git_prints_an_object_name(line);
        if !objects.contains(&object) {
            objects.push(object);
        }
    }
    return Ok(objects);
}

/// What:
///  Those of `candidates` this repository still holds,
///  each with what it is.
/// Why:
///   A record can name an object a collection already took,
///  and feeding a missing object to `rev-list` ends the whole question in an error a person reads as a refusal.
///  An object that is a commit is asked the survival question,
///  while one that is a tree, a blob or a tag nothing peeled is kept as it stands:
///  no reachability test answers for those, and they die with the worktree just as a commit would
///  (a review of `5a2bd4bb6c`, finding 1, for the tag that was dropped here).
///  One `cat-file --batch-check` answers every candidate at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function objectsAmong(context, commonDir, root, candidates): [object, Reached][]; // throws the reason
/// ```
fn objects_among(context: &GitContext, common_dir: &Path, root: &Path, candidates: &[String]) -> Result<Vec<(String, Reached)>, String> {
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let mut input: String = String::new();
    for commit in candidates {
        input.push_str(commit);
        input.push('\n');
    }
    let arguments: Vec<OsString> = [OsString::from("cat-file"), OsString::from("--batch-check")].to_vec();
    let request: GitRequest = GitRequest {
        cwd: root.to_path_buf(),
        arguments,
        without_prefix: true,
        input: Some(input.into_bytes()),
        ..GitRequest::default()
    };
    let owned: GitContext = with_the_repository(context, common_dir);
    let output: GitOutput = run_git(&owned, &request).map_err(|error| return format!("nothing proves which of the objects a record names this repository still holds ({error})"))?;
    if output.code != Some(0) {
        return Err(format!(
            "nothing proves which of the objects a record names this repository still holds ({})",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    let mut objects: Vec<(String, Reached)> = Vec::new();
    for line in String::from_utf8_lossy(output.stdout.as_slice()).lines() {
        // `<name> <type> <size>`, and `<name> missing` for one that is gone.
        let fields: Vec<&str> = line.split_ascii_whitespace().collect();
        let [name, kind, ..] = fields.as_slice() else { continue };
        match *kind {
            "commit" => objects.push((as_git_prints_an_object_name(name), Reached::Commit)),
            // Nothing is left to lose of an object this repository no longer holds.
            "missing" => continue,
            other => objects.push((as_git_prints_an_object_name(name), Reached::Other(other.to_owned()))),
        }
    }
    return Ok(objects);
}

/// What:
///  The names of the references a removal leaves alone.
/// Why:
///   Every reference outside the namespaces Git keeps per worktree lives in the common directory and survives,
///  while `refs/worktree`, `refs/bisect` and `refs/rewritten` die with the worktree.
///  A name is given rather than an object,
///  so a tag is peeled by Git itself,
///  and a name that is no commit or tag is left out because it names no history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function survivingReferenceNames(context, root): string[]; // throws the reason
/// ```
fn surviving_reference_names(context: &GitContext, root: &Path) -> Result<Vec<String>, String> {
    // Every reference, whatever kind of object it holds: a name of any kind that survives keeps what it holds,
    // and the walk below excludes it by the name Git itself peels.
    let arguments: Vec<OsString> = [OsString::from("for-each-ref"), OsString::from("--format=%(refname)")].to_vec();
    let request: GitRequest = GitRequest {
        cwd: root.to_path_buf(),
        arguments,
        without_prefix: true,
        ..GitRequest::default()
    };
    let output: GitOutput = run_git(context, &request).map_err(|error| return format!("nothing proves which references outlive the worktree ({error})"))?;
    if output.code != Some(0) {
        return Err(format!(
            "nothing proves which references outlive the worktree ({})",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    let mut names: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(output.stdout.as_slice()).lines() {
        let name: &str = line.trim();
        if name.is_empty() || PER_WORKTREE_PREFIXES.iter().any(|prefix| return name.starts_with(prefix)) {
            continue;
        }
        names.push(name.to_owned());
    }
    return Ok(names);
}

/// What:
///  Every commit named by a reflog under the common Git directory,
///  which a removal leaves alone.
/// Why:
///   Git's own collection keeps what a reflog names,
///  and the reflog of a branch is where the commits an amend or a rebase took off it stay named:
///  a question that read only references would report those stranded and keep a worktree nothing stands in the way of.
///  Only the files Git itself reads as reflogs are read here,
///  and only the lines of them Git reads,
///  because a name the collection does not keep proves nothing
///  (`is_a_reflog_file_name`, `reflog_line_names`).
///  The worktree's own logs are not read here:
///  they are in its own Git directory, and they die with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function survivingLogNames(commonDir): string[]; // throws the reason
/// ```
fn surviving_log_names(common_dir: &Path) -> Result<Vec<String>, String> {
    let mut names: Vec<String> = Vec::new();
    let logs: PathBuf = common_dir.join("logs");
    for path in files_beneath(logs.as_path()).map_err(|error| return format!("nothing proves which logs outlive the worktree ({error})"))? {
        // A file Git does not read as a reflog names nothing, and reading it anyway authorized a removal: a lock file is
        // an update that was never renamed into place, and a killed Git leaves one behind (a review of `09fd2138d7`).
        let Some(name) = path.file_name() else { continue };
        if !is_a_reflog_file_name(name.as_encoded_bytes()) {
            continue;
        }
        // A log another command is writing may end in a torn line, and Git reads the lines before it: the line itself is
        // skipped, and the names of the lines Git reads are kept.
        let Ok(text) = crate::private_storage::read_regular_file(path.as_path(), NAME_RECORD_LIMIT) else { continue };
        for line in complete_lines(text.as_slice()) {
            let Some((old, new)) = reflog_line_names(line) else { continue };
            for held in [old, new] {
                if held.bytes().any(|byte| return byte != b'0') {
                    names.push(held);
                }
            }
        }
    }
    return Ok(names);
}

/// What:
///  Whether Git reads a file of this name as a reflog.
/// Why:
///   Git's reflog iterator walks the files below `logs` and skips every one whose own name is not a ref name
///  (`files_reflog_iterator_advance` in Git's `refs/files-backend.c` asks `check_refname_format(basename,
///  REFNAME_ALLOW_ONELEVEL)`),
///  which is what leaves a lock file, a file whose name holds a space and a name ending in a dot no reflog at all:
///  a reflog is written through a lock and renamed into place, so a killed Git leaves `logs/HEAD.lock` behind,
///  and reading that file's lines as names authorized a removal that took the only name of a commit
///  (a review of `09fd2138d7`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAReflogFileName('HEAD') // true; isAReflogFileName('HEAD.lock') // false
/// ```
fn is_a_reflog_file_name(name: &[u8]) -> bool {
    // `check_refname_format` with `REFNAME_ALLOW_ONELEVEL`, for the one part of a path that a file's name holds.
    if name.is_empty() || name[0] == b'.' || name.ends_with(b".") || name.ends_with(b".lock") || name == b"@" {
        return false;
    }
    let mut previous: u8 = 0;
    for byte in name {
        if byte.is_ascii_control() || *byte == b' ' || b"~^:?*[\\".contains(byte) {
            return false;
        }
        if (previous == b'.' && *byte == b'.') || (previous == b'@' && *byte == b'{') {
            return false;
        }
        previous = *byte;
    }
    return true;
}

/// What:
///  The lines of a text that end in a newline,
///  which are the ones Git reads.
/// Why:
///   Git's reflog reader takes one line at a time and skips a last line that has no newline
///  (`show_one_reflog_ent`, `refs/files-backend.c`: `sb->buf[sb->len - 1] != '\n'`),
///  so a file writing its last line names nothing yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// completeLines('a\nb') // ['a\n']
/// ```
fn complete_lines(text: &[u8]) -> Vec<&[u8]> {
    let mut lines: Vec<&[u8]> = Vec::new();
    let mut rest: &[u8] = text;
    while let Some(end) = rest.iter().position(|byte| return *byte == b'\n') {
        lines.push(&rest[..=end]);
        rest = &rest[end + 1..];
    }
    return lines;
}

/// What:
///  The two object names of one reflog line, when Git reads that line as a reflog entry.
/// Why:
///   Git writes `old new name <email> time zone<TAB>message` and reads only a line of that shape
///  (`show_one_reflog_ent`, which checks the two names, the closing `>` of the ident with the space after it,
///  the time, and the sign and four digits of the zone),
///  so a line of any other shape names nothing:
///  a file whose lines Git cannot read is no proof that a commit outlives the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reflogLineNames('aaa... bbb... Fixture <f@e> 1700000000 +0000\tx\n') // [old, new]
/// ```
fn reflog_line_names(line: &[u8]) -> Option<(String, String)> {
    let body: &[u8] = line.strip_suffix(b"\n")?;
    let (old, after_old) = take_a_reflog_name(body)?;
    let (new, rest) = take_a_reflog_name(after_old)?;
    // The ident ends at its first `>`, which the space before the time follows.
    let at: usize = rest.iter().position(|byte| return *byte == b'>')?;
    let after_ident: &[u8] = rest.get(at + 1..)?.strip_prefix(b" ")?;
    // The time is a decimal number, and the zone a sign and four digits.
    let digits: usize = after_ident.iter().position(|byte| return !byte.is_ascii_digit())?;
    if digits == 0 {
        return None;
    }
    let after_time: &[u8] = after_ident.get(digits..)?.strip_prefix(b" ")?;
    let sign: &[u8] = after_time.get(..1)?;
    if sign != b"+" && sign != b"-" {
        return None;
    }
    let zone: &[u8] = after_time.get(1..4 + 1)?;
    if !zone.iter().all(|byte| return byte.is_ascii_digit()) {
        return None;
    }
    return Some((old, new));
}

/// What:
///  One object name at the start of these bytes and the space after it,
///  or nothing.
/// Why:
///   A reflog line begins with two object names of the repository's kind,
///  and each is followed by one space;
///  the longer kind is tried first so that a name of the shorter kind is never read as one of the longer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// takeAReflogName(`${'a'.repeat(40)} rest`) // ['aaa...', ' rest']
/// ```
fn take_a_reflog_name(bytes: &[u8]) -> Option<(String, &[u8])> {
    for digits in [SHA256_DIGITS, SHA1_DIGITS] {
        if bytes.len() <= digits || bytes[digits] != b' ' {
            continue;
        }
        let name: &[u8] = &bytes[..digits];
        if !name.iter().all(|byte| return byte.is_ascii_hexdigit()) {
            continue;
        }
        let text: &str = std::str::from_utf8(name).ok()?;
        return Some((as_git_prints_an_object_name(text), &bytes[digits + 1..]));
    }
    return None;
}

/// What:
///  Every commit a worktree's own Git directory names in a log or a record that a removal deletes with it,
///  with the record that names it.
/// Why:
///   These are the names a reflog is: the `HEAD` reflog after an amend, a reset or a rebase in the worktree,
///  a reference's own log, and the records of an operation that stopped
///  (`NAMING_RECORDS`, `NAMING_RECORD_FILES`).
///  A line of a log that is neither of the two object names Git writes there is the reason the question cannot be
///  answered:
///  it may name a commit this program cannot see, and answering "nothing there" would delete the only thing that names
///  it.
///  A record's own text is read for object names alone,
///  because a `git-rebase-todo` line and a bisect log line carry the commit beside a word and a subject.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namesThatDieWithTheWorktree(admin): [commit, holder][]; // throws the reason
/// ```
fn names_that_die_with_the_worktree(admin: &Path) -> Result<Vec<(String, String)>, String> {
    let mut held: Vec<(String, String)> = Vec::new();
    for path in files_beneath(admin.join("logs").as_path()).map_err(|error| return format!("nothing proves which logs of the worktree would go with it ({error})"))? {
        let holder: String = log_holder(admin, path.as_path());
        let text: Vec<u8> = crate::private_storage::read_regular_file(path.as_path(), NAME_RECORD_LIMIT)
            .map_err(|refusal| return format!("{holder} cannot be read ({}), so nothing proves which commit it holds", refusal_words(&refusal)))?;
        for (number, line) in String::from_utf8_lossy(text.as_slice()).lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            // A line Git writes names the commit it left and the commit it moved to, in that order.
            let fields: Vec<&str> = line.split_ascii_whitespace().take(2).collect();
            let unreadable = || return format!("{holder} holds a line this program cannot read ({line}) at line {}, so nothing proves which commit it holds", number + 1);
            let [old, new] = fields.as_slice() else { return Err(unreadable()); };
            if !is_an_object_name(old) || !is_an_object_name(new) {
                return Err(unreadable());
            }
            for field in [*old, *new] {
                if field.bytes().all(|byte| return byte == b'0') {
                    continue;
                }
                held.push((as_git_prints_an_object_name(field), holder.clone()));
            }
        }
    }
    for name in NAMING_RECORDS {
        let holder: String = format!("the record `{name}`");
        let text: Vec<u8> = read_a_record_or_nothing(admin.join(name).as_path(), holder.as_str())?;
        push_object_names(&mut held, text.as_slice(), holder);
    }
    for (directory, name) in NAMING_RECORD_FILES {
        let holder: String = format!("the record `{directory}/{name}`");
        let text: Vec<u8> = read_a_record_or_nothing(admin.join(directory).join(name).as_path(), holder.as_str())?;
        push_object_names(&mut held, text.as_slice(), holder);
    }
    return Ok(held);
}

/// What:
///  The bytes of one record of an operation that may be in progress,
///  or nothing when the record is not there,
///  and the reason nothing may be removed when it is there and cannot be read.
/// Why:
///  A record that cannot be read may hold the only surviving name of a commit,
///  and answering "no record" for it deleted that name with the worktree
///  (a review of `5a2bd4bb6c`, finding 2).
///  Only a name that is not there is no name:
///  every other refusal, from a permission to a named pipe to a record too large to read, is the reason the removal stops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readARecordOrNothing(path: string, holder: string): Uint8Array; // empty when missing; throws otherwise
/// ```
fn read_a_record_or_nothing(path: &Path, holder: &str) -> Result<Vec<u8>, String> {
    return match crate::private_storage::read_regular_file(path, NAME_RECORD_LIMIT) {
        Ok(text) => Ok(text),
        Err(crate::private_storage::ReadRefusal::Missing) => Ok(Vec::new()),
        Err(refusal) => Err(format!("{holder} cannot be read ({}), so nothing proves which commit it holds", refusal_words(&refusal))),
    };
}

/// What:
///  What a read that refused means to a person,
///  for a log or a record of a worktree being removed.
/// Why:
///  The sentences of this question say why nothing was proven,
///  and a refusal that names its cause is what lets the person fix it:
///  a name that is no plain file is a planted pipe or link,
///  while an input-output error is the file system's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refusalWords(new ReadRefusal.TooLarge()) // 'it holds more than 8388608 bytes'
/// ```
fn refusal_words(refusal: &crate::private_storage::ReadRefusal) -> String {
    return match refusal {
        crate::private_storage::ReadRefusal::Missing => String::from("nothing exists at that name"),
        crate::private_storage::ReadRefusal::NotRegular => String::from("it is not a plain file (a named pipe, a device or a directory)"),
        crate::private_storage::ReadRefusal::TooLarge => format!("it holds more than {NAME_RECORD_LIMIT} bytes"),
        crate::private_storage::ReadRefusal::Io(error) => format!("the file system refused the read ({error})"),
    };
}

/// What:
///  The words for the log at `path`,
///  as the person reads them in a sentence.
/// Why:
///   The `HEAD` reflog is the one whose loss the review of `e615fb599b` reproduced,
///  and it reads best as "its HEAD reflog";
///  every other log is named by the reference it belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function logHolder(admin, path): string;
/// ```
fn log_holder(admin: &Path, path: &Path) -> String {
    let shown: &Path = path.strip_prefix(admin).unwrap_or(path);
    let text: String = shown.display().to_string();
    return match text.as_str() {
        "logs/HEAD" => String::from("its HEAD reflog"),
        _ => format!("the log `{text}`"),
    };
}

/// What:
///  Every object name in `text`, with `holder` as what names it.
/// Why:
///   A record's text is a word beside a commit at most,
///  so the names in it are read by shape alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pushObjectNames(held, text, holder): void;
/// ```
fn push_object_names(held: &mut Vec<(String, String)>, text: &[u8], holder: String) {
    for word in String::from_utf8_lossy(text).split_ascii_whitespace() {
        if is_an_object_name(word) && word.bytes().any(|byte| return byte != b'0') {
            held.push((as_git_prints_an_object_name(word), holder.clone()));
        }
    }
}

/// What:
///  An object name written the way Git writes every name it prints,
///  for every comparison this question makes.
/// Why:
///  Git prints object names in small hexadecimal letters,
///  while a record Git wrote may hold the same name in capital letters,
///  and Git itself reads both as the same object:
///  a detached `HEAD` written in capitals was dropped from the stranded set because the comparison was text,
///  the removal went ahead, and a collection took the commit
///  (a review of `8858f843ee`, finding 3).
///  Names written by this program's own sentences keep the small form Git prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asGitPrintsAnObjectName = (name: string) => name.toLowerCase();
/// ```
fn as_git_prints_an_object_name(name: &str) -> String {
    return name.to_ascii_lowercase();
}

/// What:
///  Whether two object names are the same object,
///  whatever case each was written in.
/// Why:
///  This is the comparison Git makes of two names,
///  and it is used wherever a name read from a file meets a name Git printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theSameObjectName(one, other) // one.toLowerCase() === other.toLowerCase()
/// ```
fn the_same_object_name(one: &str, other: &str) -> bool {
    return one.eq_ignore_ascii_case(other);
}

/// What:
///  Whether `word` has the shape of an object name in this kind of repository.
/// Why:
///   Git writes SHA-1 names of forty digits and SHA-256 names of sixty-four,
///  and any other word in a record is text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAnObjectName = (word: string) => word.length === 40 && word.every(isHex);
/// ```
fn is_an_object_name(word: &str) -> bool {
    let digits: usize = word.len();
    return (digits == SHA1_DIGITS || digits == SHA256_DIGITS) && word.bytes().all(|byte| return byte.is_ascii_hexdigit());
}

/// What:
///  Every file below `directory`, in a stable order,
///  or nothing when the directory is not there.
/// Why:
///   The logs and the records of an operation sit at a depth Git chose,
///  and a worktree that never made one has no directory at all,
///  which is no reason to refuse anything.
///  A name that went between the listing and its look is no file, while any other failure of the look may hide a log
///  that names a commit, so it is the reason the removal stops rather than a name quietly passed over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function filesBeneath(directory): string[]; // throws the reason a directory could not be read
/// ```
fn files_beneath(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut found: Vec<PathBuf> = Vec::new();
    let listing = match std::fs::read_dir(directory) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(found),
        // A path that is no directory holds no file, and is no reason to refuse.
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => return Ok(found),
        Err(error) => return Err(error),
    };
    let mut entries: Vec<(std::ffi::OsString, PathBuf)> = Vec::new();
    for listed in listing {
        let entry: std::fs::DirEntry = listed?;
        let path: PathBuf = entry.path();
        entries.push((entry.file_name(), path));
    }
    // A stable order, so one record's sentence comes before another's however the directory is listed.
    entries.sort();
    for (_, path) in entries {
        match std::fs::symlink_metadata(path.as_path()) {
            Ok(metadata) if metadata.is_dir() => found.append(&mut files_beneath(path.as_path())?),
            Ok(_) => found.push(path),
            // A name that went between the listing and this look names no file, and any other failure may hide a log
            // that names a commit, which is the reason the removal stops.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        }
    }
    return Ok(found);
}

/// What:
///  The sentence for one commit a removal would leave named by nothing.
/// Why:
///   A person decides about their own commit,
///  so the sentence names the commit,
///  what names it now,
///  why that dies with the removal,
///  and the one command that gives it a name that outlives the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sentence(root, stranded) // 'its detached HEAD names the commit 0f1af2e0f295, ...'
/// ```
fn sentence(root: &Path, stranded: &Stranded) -> String {
    let short: &str = &stranded.object[..SHOWN_DIGITS.min(stranded.object.len())];
    // The path is quoted where it enters the command the person copies: a path with a space, a quote, a dollar sign
    // or a line break otherwise turns that command into another one.
    let quoted: String = crate::shell_quote::shell_quote(root.as_os_str().as_encoded_bytes());
    // A commit is given a branch, and a tree or a blob a tag, which is the name Git can hold for an object that is no commit.
    return match &stranded.reached {
        // A fetch record is the one holder a person will not expect: the sentence says where the name came from.
        Reached::Commit if stranded.holder == "the record `FETCH_HEAD`" => format!(
            "{} names the commit {short}, which is what a fetch brought here and which nothing else of this repository \
             names, so a removal would leave nothing that names it and `git gc --prune=now` would take it. Give it a name \
             that outlives the worktree first, with `git -C {quoted} branch <a name you choose> {short}`, then remove the \
             worktree again",
            stranded.holder
        ),
        Reached::Commit => format!(
            "{} names the commit {short}, and nothing else of this repository does, so a removal would leave nothing that names it and \
             `git gc --prune=now` would take it. Give it a name that outlives the worktree first, with `git -C {quoted} branch <a name you \
             choose> {short}`, then remove the worktree again",
            stranded.holder
        ),
        Reached::Other(kind) => format!(
            "{} names the {kind} {short}, which is no commit, and nothing here proves an object of that kind outlives the removal, so \
             nothing was removed. Give it a name that outlives the worktree first, with `git -C {quoted} tag <a name you choose> {short}` \
             (a tag may name an object of that kind), then remove the worktree again",
            stranded.holder
        ),
    };
}

/// The controls of this question stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_keeps_git_work_tests.rs"]
mod tests;
