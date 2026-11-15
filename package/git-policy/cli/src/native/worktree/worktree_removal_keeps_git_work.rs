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
//! In TS you'd write (pseudocode):
//! ```ts
//! const stranded = gitWorkARemovalWouldStrand(context, commonDir, worktree, admin); // string[] reasons, or the reason nothing was proven
//! ```

use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_ladder_target::checkout_environment;
use crate::worktree_landed_registrations::registered_worktrees;
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

/// How many stranded commits one answer names before it counts the rest.
const SHOWN_COMMITS: usize = 3;

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

/// The records a worktree's own Git directory keeps directly that name a commit.
/// Why:
///   Each is a name a removal deletes with the directory,
///  and each was seen holding a commit when the operation was made in a linked worktree
///  (`keep/probe-admin-state.log` of the run that answered the review):
///  a reset's `ORIG_HEAD`,
///  a merge's `MERGE_HEAD`,
///  a cherry-pick's and a revert's,
///  a rebase's `REBASE_HEAD`,
///  the stash a merge or a rebase put aside,
///  and a bisect's log and the revision it was about to try.
const NAMING_RECORDS: [&str; 9] = ["ORIG_HEAD", "MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "REBASE_HEAD", "MERGE_AUTOSTASH", "BISECT_LOG", "BISECT_EXPECTED_REV", "BISECT_START"];

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

/// One commit the worktree's own state names and no surviving reference does.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Stranded {
    /// The commit.
    commit: String,
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
///  The worktrees this repository registers inside `worktree` are asked too:
///  the removal takes their files and their own Git directories with the outer one.
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
pub(crate) fn git_work_a_removal_would_strand(context: &GitContext, common_dir: &Path, worktree: &Path, admin: &Path) -> Result<Vec<String>, String> {
    let mut places: Vec<(PathBuf, PathBuf)> = vec![(worktree.to_path_buf(), admin.to_path_buf())];
    for one in registered_worktrees(common_dir).map_err(|reason| format!("nothing proves no worktree inside this one holds a commit of its own, so nothing is removed: {reason}"))? {
        if one.worktree != worktree && one.worktree.starts_with(worktree) {
            places.push((one.worktree, one.admin));
        }
    }
    let mut said: Vec<String> = Vec::new();
    let mut counted: usize = 0;
    for (root, own) in places {
        for stranded in stranded_commits(context, common_dir, root.as_path(), own.as_path())? {
            counted += 1;
            if said.len() < SHOWN_COMMITS {
                said.push(sentence(root.as_path(), &stranded));
            }
        }
    }
    if counted > said.len() {
        said.push(format!("and {} more commits that only a worktree being removed names", counted - said.len()));
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
fn stranded_commits(context: &GitContext, common_dir: &Path, root: &Path, admin: &Path) -> Result<Vec<Stranded>, String> {
    let mut holders: Vec<Stranded> = Vec::new();
    // The detached `HEAD` first, so the sentence about the commit it names is the one a person reads for it.
    if let Some(commit) = detached_head(admin)? {
        holders.push(Stranded { commit, holder: String::from("its detached HEAD") });
    }
    for (name, commit) in per_worktree_references(context, root, admin)? {
        holders.push(Stranded { commit, holder: format!("the reference `{name}`") });
    }
    for (commit, holder) in names_that_die_with_the_worktree(admin)? {
        holders.push(Stranded { commit, holder });
    }
    if holders.is_empty() {
        return Ok(Vec::new());
    }
    let mut candidates: Vec<String> = Vec::new();
    for held in &holders {
        if !candidates.iter().any(|one| return *one == held.commit) {
            candidates.push(held.commit.clone());
        }
    }
    let stranded: Vec<String> = commits_nothing_survives(context, common_dir, root, candidates.as_slice())?;
    let mut answer: Vec<Stranded> = Vec::new();
    for held in holders {
        if answer.iter().any(|one| return one.commit == held.commit) {
            continue;
        }
        if stranded.iter().any(|one| return *one == held.commit) {
            answer.push(held);
        }
    }
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
        return Ok(Some(line.to_owned()));
    }
    return Err(format!(
        "the record HEAD at {} names neither a branch nor a commit, so nothing proves which commit it holds and a removal could delete the only name of one",
        admin.display()
    ));
}

/// What:
///  The references Git keeps in one worktree,
///  with the commit each names.
/// Why:
///   Those references live in the worktree's own Git directory and the removal deletes them,
///  so a commit they alone name dies with it,
///  and a person's `refs/worktree/person-keep` is the ordinary way to be in that state.
///  A Git directory holding no reference has none,
///  and asking Git there would cost a process for nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// perWorktreeReferences(context, root, admin) // [{ name, commit }]
/// ```
fn per_worktree_references(context: &GitContext, root: &Path, admin: &Path) -> Result<Vec<(String, String)>, String> {
    if !admin.join("refs").is_dir() && !admin.join("packed-refs").exists() {
        return Ok(Vec::new());
    }
    let arguments: Vec<OsString> = [
        OsString::from("for-each-ref"),
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
        let Some((name, commit)) = line.split_once('\t') else { continue };
        if commit.is_empty() {
            continue;
        }
        found.push((name.to_owned(), commit.to_owned()));
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
///  The commits, of those given, that no name surviving the removal reaches.
/// Why:
///  Git's own collection keeps an object reachable from any reference or log it reads,
///  and from every ancestor of what those name,
///  so the question is reachability and not the bytes of a record:
///  a commit an amend took off a branch is still named by that branch's reflog,
///  and one a person reset away in a detached worktree is named by nothing.
///  One `rev-list` answers all of them at once,
///  asked from the candidates to the names that survive and never the other way round:
///  the walk it then does is bounded by the candidates,
///  while the surviving names are tips the answer excludes,
///  and walking the survivors first costs 6.9 s here where this costs 30 to 98 ms
///  (`keep/measurement-cost-of-the-reflog-question.log` of the run that answered the review).
///  A name that is missing or is no commit is no candidate:
///  nothing can be lost that is not there, and a tree or a blob is no commit a removal strands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commitsNothingSurvives(context, commonDir, root, candidates): string[]; // throws the reason
/// ```
fn commits_nothing_survives(context: &GitContext, common_dir: &Path, root: &Path, candidates: &[String]) -> Result<Vec<String>, String> {
    // The two requests below carry their input on standard input, which this program feeds from a private file under the
    // repository's own scratch directory, so the context has to name the repository it was started for.
    let owned: GitContext = with_the_repository(context, common_dir);
    let existing: Vec<String> = commits_among(&owned, root, candidates)?;
    if existing.is_empty() {
        return Ok(Vec::new());
    }
    let mut input: String = String::new();
    for commit in &existing {
        input.push_str(commit);
        input.push('\n');
    }
    // Every name the removal leaves alone: a reference of the common directory, and a log under it.
    for name in surviving_reference_names(&owned, root)? {
        input.push('^');
        input.push_str(name.as_str());
        input.push('\n');
    }
    for name in surviving_log_names(common_dir)? {
        input.push('^');
        input.push_str(name.as_str());
        input.push('\n');
    }
    let arguments: Vec<OsString> = [OsString::from("rev-list"), OsString::from("--stdin")].to_vec();
    let request: GitRequest = GitRequest {
        cwd: root.to_path_buf(),
        arguments,
        without_prefix: true,
        input: Some(input.into_bytes()),
        ..GitRequest::default()
    };
    let output: GitOutput = run_git(&owned, &request).map_err(|error| return format!("nothing proves which commits no surviving name reaches ({error})"))?;
    if output.code != Some(0) {
        return Err(format!(
            "nothing proves which commits no surviving name reaches ({})",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    let mut stranded: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(output.stdout.as_slice()).lines() {
        if existing.iter().any(|one| return one == line) && !stranded.iter().any(|one| return one == line) {
            stranded.push(line.to_owned());
        }
    }
    return Ok(stranded);
}

/// What:
///  Those of `candidates` that are commits this repository still holds.
/// Why:
///   A record can name an object a collection already took,
///  and feeding a missing or non-commit object to `rev-list` ends the whole question in an error a person reads as a refusal.
///  One `cat-file --batch-check` answers every candidate at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commitsAmong(context, root, candidates): string[]; // throws the reason
/// ```
fn commits_among(context: &GitContext, root: &Path, candidates: &[String]) -> Result<Vec<String>, String> {
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
    let output: GitOutput = run_git(context, &request).map_err(|error| return format!("nothing proves which of the commits a record names this repository still holds ({error})"))?;
    if output.code != Some(0) {
        return Err(format!(
            "nothing proves which of the commits a record names this repository still holds ({})",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    let mut commits: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(output.stdout.as_slice()).lines() {
        // `<name> <type> <size>`, and `<name> missing` for one that is gone.
        let fields: Vec<&str> = line.split_ascii_whitespace().collect();
        let [name, kind, ..] = fields.as_slice() else { continue };
        if *kind == "commit" {
            commits.push((*name).to_owned());
        }
    }
    return Ok(commits);
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
    let arguments: Vec<OsString> = [OsString::from("for-each-ref"), OsString::from("--format=%(refname)%09%(objecttype)")].to_vec();
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
        let Some((name, kind)) = line.split_once('\t') else { continue };
        if PER_WORKTREE_PREFIXES.iter().any(|prefix| return name.starts_with(prefix)) {
            continue;
        }
        if kind == "commit" || kind == "tag" {
            names.push(name.to_owned());
        }
    }
    return Ok(names);
}

/// What:
///  Every commit named by a log under the common Git directory,
///  which a removal leaves alone.
/// Why:
///   Git's own collection keeps what a reflog names,
///  and the reflog of a branch is where the commits an amend or a rebase took off it stay named:
///  a question that read only references would report those stranded and keep a worktree nothing stands in the way of.
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
        // A log another command is writing may end in a torn line, and a torn line of a surviving log can only make a
        // commit look stranded, which keeps the worktree rather than losing anything: it is skipped.
        let Ok(text) = crate::private_storage::read_regular_file(path.as_path(), NAME_RECORD_LIMIT) else { continue };
        for line in String::from_utf8_lossy(text.as_slice()).lines() {
            for field in line.split_ascii_whitespace().take(2) {
                if is_an_object_name(field) && field.bytes().any(|byte| return byte != b'0') {
                    names.push(field.to_owned());
                }
            }
        }
    }
    return Ok(names);
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
                held.push((field.to_owned(), holder.clone()));
            }
        }
    }
    for name in NAMING_RECORDS {
        let path: PathBuf = admin.join(name);
        let Ok(text) = crate::private_storage::read_regular_file(path.as_path(), NAME_RECORD_LIMIT) else { continue };
        push_object_names(&mut held, text.as_slice(), format!("the record `{name}`"));
    }
    for (directory, name) in NAMING_RECORD_FILES {
        let path: PathBuf = admin.join(directory).join(name);
        let Ok(text) = crate::private_storage::read_regular_file(path.as_path(), NAME_RECORD_LIMIT) else { continue };
        push_object_names(&mut held, text.as_slice(), format!("the record `{directory}/{name}`"));
    }
    return Ok(held);
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
            held.push((word.to_owned(), holder.clone()));
        }
    }
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
            Err(_) => continue,
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
    let short: &str = &stranded.commit[..SHOWN_DIGITS.min(stranded.commit.len())];
    // The path is quoted where it enters the command the person copies: a path with a space, a quote, a dollar sign
    // or a line break otherwise turns that command into another one.
    let quoted: String = crate::shell_quote::shell_quote(root.as_os_str().as_encoded_bytes());
    return format!(
        "{} names the commit {short}, and nothing else of this repository does, so a removal would leave nothing that names it and \
         `git gc --prune=now` would take it. Give it a name that outlives the worktree first, with `git -C {quoted} branch <a name you \
         choose> {short}`, then remove the worktree again",
        stranded.holder
    );
}

/// The controls of this question stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_keeps_git_work_tests.rs"]
mod tests;
