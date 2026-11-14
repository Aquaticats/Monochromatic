//! What:
//!  The Git work a removal would leave named by nothing:
//!  a commit only the worktree's own state names.
//! Why:
//!   A removal deletes the worktree's own Git directory,
//!  and with it the two things that can name a commit there:
//!  a detached `HEAD` and the references Git keeps per worktree
//!  (`refs/worktree/`,
//!  `refs/bisect/`,
//!  `refs/rewritten/`).
//!  A commit no reference of the repository names is one `git gc --prune=now` takes,
//!  so the removal would destroy it,
//!  and the owner's rule is that a removal never destroys Git work
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "How far the protection of a worktree being removed reaches").
//!  A worktree a person detached and committed in is the ordinary case:
//!  the commit exists there and nowhere else.
//!  A commit an ancestor of a surviving reference names is not stranded:
//!  nothing that names it dies with the worktree.
//!  The same question is asked of every worktree this repository registers inside the one being removed,
//!  because a removal takes those too,
//!  and a nested worktree whose own `HEAD` is detached is a state Git records no branch for
//!  (a Sonnet audit of 0.6.1-alpha.14,
//!  finding F2).
//!  Nothing here keeps a commit:
//!  the caller keeps the worktree and says what stands in the way,
//!  and the person names the commit with a branch of their own before removing it.
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
/// The words for a record, a reference or a Git answer this cannot read,
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
        for stranded in stranded_commits(context, root.as_path(), own.as_path())? {
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
///  and a commit a reference of the repository names is not stranded however many worktree references also name it.
///  The worktree's own reference namespaces are read only when the Git directory holds any reference at all,
///  so a worktree that has none costs two file looks and no Git start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stranded = strandedCommits(context, root, admin);
/// ```
fn stranded_commits(context: &GitContext, root: &Path, admin: &Path) -> Result<Vec<Stranded>, String> {
    let mut holders: Vec<Stranded> = Vec::new();
    if let Some(commit) = detached_head(admin)? {
        holders.push(Stranded { commit, holder: String::from("its detached HEAD") });
    }
    for (name, commit) in per_worktree_references(context, root, admin)? {
        holders.push(Stranded { commit, holder: format!("the reference `{name}`") });
    }
    let mut stranded: Vec<Stranded> = Vec::new();
    for held in holders {
        if stranded.iter().any(|one| return one.commit == held.commit) {
            continue;
        }
        if !named_by_a_surviving_reference(context, root, held.commit.as_str())? {
            stranded.push(held);
        }
    }
    return Ok(stranded);
}

/// What:
///  The commit a worktree's `HEAD` names when it names a commit,
///  or nothing when it names a branch.
/// Why:
///   A detached `HEAD` is the one record that says which commit the worktree is on,
///  and the registration reader already opens that record without waiting for a writer.
///  A `HEAD` that starts with anything but a reference is not a state this program acts on:
///  a record it cannot read is the reason the removal keeps the worktree.
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
    return Ok(None);
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
///  Whether a reference of the repository that outlives the removal names `commit`.
/// Why:
///   Only a name that survives keeps a commit out of reach of the next collection,
///  and a worktree's own references are exactly the names that do not.
///  Ask it of `for-each-ref --contains`,
///  so a commit an older commit of a branch holds counts as named,
///  which is what Git's own collection reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namedByASurvivingReference(context, root, commit) // boolean
/// ```
fn named_by_a_surviving_reference(context: &GitContext, root: &Path, commit: &str) -> Result<bool, String> {
    let arguments: Vec<OsString> = [format!("--contains={commit}"), String::from("--format=%(refname)")].into_iter().map(OsString::from).collect();
    let mut all: Vec<OsString> = vec![OsString::from("for-each-ref")];
    all.extend(arguments);
    let output: GitOutput = run_git(context, &GitRequest::new(root, all.as_slice())).map_err(|error| format!("nothing proves which references name {commit} ({error})"))?;
    if output.code != Some(0) {
        return Err(format!(
            "nothing proves which references name {commit} ({})",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    for name in String::from_utf8_lossy(output.stdout.as_slice()).lines() {
        if !PER_WORKTREE_PREFIXES.iter().any(|prefix| return name.starts_with(prefix)) {
            return Ok(true);
        }
    }
    return Ok(false);
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
    return format!(
        "{} names the commit {short}, and nothing else of this repository does, so a removal would leave nothing that names it and \
         `git gc --prune=now` would take it. Give it a name that outlives the worktree first, with `git -C {} branch <a name you \
         choose> {short}`, then remove the worktree again",
        stranded.holder,
        root.display()
    );
}

/// The controls of this question stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_keeps_git_work_tests.rs"]
mod tests;
